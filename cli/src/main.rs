use entropy_api::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey;
use solana_sdk::{
    address_lookup_table::{
        instruction::{create_lookup_table, extend_lookup_table},
        state::AddressLookupTable,
        AddressLookupTableAccount,
    },
    compute_budget::ComputeBudgetInstruction,
    message::{v0::Message, VersionedMessage},
    signature::{read_keypair_file, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use steel::AccountDeserialize;

// TODO: Create new LUT after deploying with 32 feeds and update this address.
const LUT_ADDRESS: solana_sdk::pubkey::Pubkey =
    pubkey!("38fAv3Tmwumkufk9JZu6pVU5DgdAnnNoVp831CqCaX4i");

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args
        .get(1)
        .expect("Usage: entropy-cli <init|close|sample|var|lut>");

    let rpc_url = std::env::var("RPC").expect("Missing RPC env var");
    let rpc = RpcClient::new(rpc_url);

    match command.as_str() {
        "init" => {
            let payer = load_keypair();
            let ix = entropy_api::sdk::init(payer.pubkey());
            let sig = send_and_confirm(&rpc, &payer, &[ix]).await;
            println!("Initialized var account: {}", var_pda().0);
            println!("Signature: {sig}");
        }
        "close" => {
            let payer = load_keypair();
            let ix = entropy_api::sdk::close(payer.pubkey());
            let sig = send_and_confirm(&rpc, &payer, &[ix]).await;
            println!("Closed var account. Signature: {sig}");
        }
        "lut" => {
            let payer = load_keypair();
            let lut_address = create_lut(&rpc, &payer).await;
            println!("LUT address: {lut_address}");
        }
        "sample" => {
            let payer = load_keypair();
            sample_v0(&rpc, &payer, LUT_ADDRESS).await;
        }
        "var" => {
            let address = var_pda().0;
            let account = rpc
                .get_account(&address)
                .await
                .expect("Failed to fetch var account");
            let var = Var::try_from_bytes(&account.data).expect("Failed to deserialize var");
            println!("Var account: {address}");
            println!(
                "  value:     {}",
                solana_sdk::keccak::Hash::new_from_array(var.value)
            );
            println!("  sample_at: {}", var.sample_at);
            println!("  bits:      0b{:032b}", var.bits);
            println!("  prices:");
            for i in 0..NUM_FEEDS {
                let price = var.prices[i];
                let whole = price / 100_000_000;
                let frac = (price % 100_000_000).unsigned_abs();
                let bit = (var.bits >> i) & 1;
                println!(
                    "    {:<6} ${}.{:08}  bit={}",
                    FEED_TICKERS[i], whole, frac, bit
                );
            }
        }
        _ => {
            eprintln!("Unknown command: {command}");
            eprintln!("Usage: entropy-cli <init|close|sample|var|lut>");
            std::process::exit(1);
        }
    }
}

fn load_keypair() -> solana_sdk::signer::keypair::Keypair {
    let path = std::env::var("KEYPAIR").expect("Missing KEYPAIR env var");
    read_keypair_file(&path).expect("Failed to read keypair file")
}

async fn create_lut(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> solana_sdk::pubkey::Pubkey {
    let recent_slot = rpc.get_slot().await.expect("Failed to get slot") - 4;
    let (create_ix, lut_address) = create_lookup_table(payer.pubkey(), payer.pubkey(), recent_slot);

    let mut addresses: Vec<solana_sdk::pubkey::Pubkey> = Vec::new();
    addresses.push(var_pda().0);
    addresses.extend_from_slice(&FEED_ADDRESSES);

    let batch_size = 20;
    let batches: Vec<&[solana_sdk::pubkey::Pubkey]> = addresses.chunks(batch_size).collect();

    let extend_ix_0 = extend_lookup_table(
        lut_address,
        payer.pubkey(),
        Some(payer.pubkey()),
        batches[0].to_vec(),
    );
    let sig = send_and_confirm(rpc, payer, &[create_ix, extend_ix_0]).await;
    println!("Created LUT + batch 1: {sig}");

    for (i, batch) in batches[1..].iter().enumerate() {
        let extend_ix = extend_lookup_table(
            lut_address,
            payer.pubkey(),
            Some(payer.pubkey()),
            batch.to_vec(),
        );
        let sig = send_and_confirm(rpc, payer, &[extend_ix]).await;
        println!("Extended LUT batch {}: {sig}", i + 2);
    }

    lut_address
}

async fn sample_v0(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    lut_address: solana_sdk::pubkey::Pubkey,
) {
    let lut_account = rpc
        .get_account(&lut_address)
        .await
        .expect("Failed to fetch LUT account");
    let lut =
        AddressLookupTable::deserialize(&lut_account.data).expect("Failed to deserialize LUT");
    let lut_account = AddressLookupTableAccount {
        key: lut_address,
        addresses: lut.addresses.to_vec(),
    };

    let sample_ix = entropy_api::sdk::sample(payer.pubkey());

    let ixs = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
        ComputeBudgetInstruction::set_compute_unit_price(100_000),
        sample_ix,
    ];

    let blockhash = rpc
        .get_latest_blockhash()
        .await
        .expect("Failed to get blockhash");
    let msg = Message::try_compile(&payer.pubkey(), &ixs, &[lut_account], blockhash)
        .expect("Failed to compile v0 message");
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[payer])
        .expect("Failed to sign transaction");

    let sig = rpc
        .send_and_confirm_transaction(&tx)
        .await
        .expect("Transaction failed");
    println!("Sampled. Signature: {sig}");
}

async fn send_and_confirm(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) -> solana_sdk::signature::Signature {
    let blockhash = rpc
        .get_latest_blockhash()
        .await
        .expect("Failed to get blockhash");
    let mut all_ixs = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
        ComputeBudgetInstruction::set_compute_unit_price(100_000),
    ];
    all_ixs.extend_from_slice(instructions);
    let tx =
        Transaction::new_signed_with_payer(&all_ixs, Some(&payer.pubkey()), &[payer], blockhash);
    rpc.send_and_confirm_transaction(&tx)
        .await
        .expect("Transaction failed")
}
