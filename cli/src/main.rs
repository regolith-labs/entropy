use std::str::FromStr;

use entropy_api::prelude::*;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    client_error::{
        reqwest::{self, StatusCode},
        ClientErrorKind,
    },
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    compute_budget::ComputeBudgetInstruction,
    message::{v0::Message, VersionedMessage},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signature, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use solana_sdk::{keccak, pubkey};
use steel::{AccountDeserialize, Clock, Discriminator, Instruction};

const ENTROPY_PROVIDER: Pubkey = pubkey!("AKBXJ7jQ2DiqLQKzgPn791r1ZVNvLchTFH6kpesPAAWF");

#[tokio::main]
async fn main() {
    // Read keypair from file
    let payer =
        read_keypair_file(&std::env::var("KEYPAIR").expect("Missing KEYPAIR env var")).unwrap();

    // Build transaction
    let rpc = RpcClient::new(std::env::var("RPC").expect("Missing RPC env var"));
    match std::env::var("COMMAND")
        .expect("Missing COMMAND env var")
        .as_str()
    {
        "open" => {
            open(&rpc, &payer).await.unwrap();
        }
        "crank" => {
            crank(&rpc, &payer).await.unwrap();
        }
        "close" => {
            close(&rpc, &payer).await.unwrap();
        }
        "var" => {
            log_var(&rpc).await.unwrap();
        }
        _ => panic!("Invalid command"),
    };
}

async fn close(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let address = std::env::var("ADDRESS").unwrap();
    let address = Pubkey::from_str(&address).expect("Invalid ADDRESS");
    let ix = entropy_api::sdk::close(payer.pubkey(), address);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn open(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let id = std::env::var("ID").unwrap();
    let id = u64::from_str(&id).expect("Invalid ID");
    let var_address = var_pda(payer.pubkey(), id).0;
    println!("Var address: {:?}", var_address);
    let commit = keccak::Hash::from_str("9YWL8MgUwgWLF32dHbUwTdTqPWGQjj8mt2A6atayrJC2").unwrap();
    let clock = get_clock(rpc).await?;
    let ix = entropy_api::sdk::open(
        payer.pubkey(),
        payer.pubkey(),
        id,
        ENTROPY_PROVIDER,
        commit.to_bytes(),
        false,
        999_998,
        clock.slot + 150,
    );
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn crank(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let address = std::env::var("ADDRESS").unwrap();
    let address = Pubkey::from_str(&address).expect("Invalid ADDRESS");

    // Get var.
    let var = get_var(rpc, address).await?;

    // Get the clock
    let clock = get_clock(rpc).await?;

    // Check if the var is ready to next.
    let buffer_slots = 4;
    if clock.slot < var.end_at + buffer_slots {
        println!(
            "Var seed is not revealed yet. Waiting for {} slots.",
            buffer_slots + (var.end_at - clock.slot)
        );
        return Ok(());
    }

    // Get the seed from the API
    let url = format!("https://entropy-api.onrender.com/var/{}/seed", address);
    let response = reqwest::get(&url).await?;
    let seed_response: entropy_types::response::GetSeedResponse = response.json().await?;
    println!("Seed: {:?}", seed_response.seed);

    // Build the instructions
    let sample_ix = entropy_api::sdk::sample(payer.pubkey(), address);
    let reveal_ix = entropy_api::sdk::reveal(payer.pubkey(), address, seed_response.seed);
    // let next_ix = entropy_api::sdk::next(payer.pubkey(), address, clock.slot + 150);
    // submit_transaction(rpc, payer, &[sample_ix, reveal_ix, next_ix]).await?;
    submit_transaction(rpc, payer, &[sample_ix, reveal_ix]).await?;
    Ok(())
}

async fn log_var(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let address = std::env::var("ADDRESS").unwrap();
    let address = Pubkey::from_str(&address).expect("Invalid ADDRESS");
    let var = get_var(rpc, address).await?;
    print_var(&var);
    Ok(())
}

fn print_var(var: &Var) {
    println!("Var: {:?}", var);
    println!("  Authority: {:?}", var.authority);
    println!("  Provider: {:?}", var.provider);
    println!(
        "  Commit: {:?}",
        keccak::Hash::new_from_array(var.commit).to_string()
    );
    println!(
        "  Seed: {:?}",
        keccak::Hash::new_from_array(var.seed).to_string()
    );
    println!(
        "  Slot hash: {:?}",
        keccak::Hash::new_from_array(var.slot_hash).to_string()
    );
    println!(
        "  Value: {:?}",
        keccak::Hash::new_from_array(var.value).to_string()
    );
    println!("  Samples: {:?}", var.samples);
    println!("  Is auto: {:?}", var.is_auto);
}

async fn get_clock(rpc: &RpcClient) -> Result<Clock, anyhow::Error> {
    let data = rpc.get_account_data(&solana_sdk::sysvar::clock::ID).await?;
    let clock = bincode::deserialize::<Clock>(&data)?;
    Ok(clock)
}

async fn get_var(rpc: &RpcClient, address: Pubkey) -> Result<Var, anyhow::Error> {
    let account = rpc.get_account(&address).await?;
    let var = Var::try_from_bytes(&account.data)?;
    Ok(*var)
}

#[allow(dead_code)]
async fn simulate_transaction(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) {
    let blockhash = rpc.get_latest_blockhash().await.unwrap();
    let x = rpc
        .simulate_transaction(&Transaction::new_signed_with_payer(
            instructions,
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ))
        .await;
    println!("Simulation result: {:?}", x);
}

#[allow(dead_code)]
async fn simulate_transaction_with_address_lookup_tables(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
    address_lookup_table_accounts: Vec<AddressLookupTableAccount>,
) {
    let blockhash = rpc.get_latest_blockhash().await.unwrap();
    let tx = VersionedTransaction {
        signatures: vec![Signature::default()],
        message: VersionedMessage::V0(
            Message::try_compile(
                &payer.pubkey(),
                instructions,
                &address_lookup_table_accounts,
                blockhash,
            )
            .unwrap(),
        ),
    };
    let s = tx.sanitize();
    println!("Sanitize result: {:?}", s);
    s.unwrap();
    let x = rpc.simulate_transaction(&tx).await;
    println!("Simulation result: {:?}", x);
}

#[allow(unused)]
async fn submit_transaction_batches(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    mut ixs: Vec<solana_sdk::instruction::Instruction>,
    batch_size: usize,
) -> Result<(), anyhow::Error> {
    // Batch and submit the instructions.
    while !ixs.is_empty() {
        let batch = ixs
            .drain(..std::cmp::min(batch_size, ixs.len()))
            .collect::<Vec<Instruction>>();
        submit_transaction_no_confirm(rpc, payer, &batch).await?;
    }
    Ok(())
}

#[allow(unused)]
async fn simulate_transaction_batches(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    mut ixs: Vec<solana_sdk::instruction::Instruction>,
    batch_size: usize,
) -> Result<(), anyhow::Error> {
    // Batch and submit the instructions.
    while !ixs.is_empty() {
        let batch = ixs
            .drain(..std::cmp::min(batch_size, ixs.len()))
            .collect::<Vec<Instruction>>();
        simulate_transaction(rpc, payer, &batch).await;
    }
    Ok(())
}

async fn submit_transaction(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) -> Result<solana_sdk::signature::Signature, anyhow::Error> {
    let blockhash = rpc.get_latest_blockhash().await?;
    let mut all_instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
        ComputeBudgetInstruction::set_compute_unit_price(1_000_000),
    ];
    all_instructions.extend_from_slice(instructions);
    let transaction = Transaction::new_signed_with_payer(
        &all_instructions,
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    match rpc.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => {
            println!("Transaction submitted: {:?}", signature);
            Ok(signature)
        }
        Err(e) => {
            println!("Error submitting transaction: {:?}", e);
            Err(e.into())
        }
    }
}

async fn submit_transaction_no_confirm(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) -> Result<solana_sdk::signature::Signature, anyhow::Error> {
    let blockhash = rpc.get_latest_blockhash().await?;
    let mut all_instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
        ComputeBudgetInstruction::set_compute_unit_price(1_000_000),
    ];
    all_instructions.extend_from_slice(instructions);
    let transaction = Transaction::new_signed_with_payer(
        &all_instructions,
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    match rpc.send_transaction(&transaction).await {
        Ok(signature) => {
            println!("Transaction submitted: {:?}", signature);
            Ok(signature)
        }
        Err(e) => {
            println!("Error submitting transaction: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn get_program_accounts<T>(
    client: &RpcClient,
    program_id: Pubkey,
    filters: Vec<RpcFilterType>,
) -> Result<Vec<(Pubkey, T)>, anyhow::Error>
where
    T: AccountDeserialize + Discriminator + Clone,
{
    let mut all_filters = vec![RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        0,
        &T::discriminator().to_le_bytes(),
    ))];
    all_filters.extend(filters);
    let result = client
        .get_program_accounts_with_config(
            &program_id,
            RpcProgramAccountsConfig {
                filters: Some(all_filters),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await;

    match result {
        Ok(accounts) => {
            let accounts = accounts
                .into_iter()
                .filter_map(|(pubkey, account)| {
                    if let Ok(account) = T::try_from_bytes(&account.data) {
                        Some((pubkey, account.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            Ok(accounts)
        }
        Err(err) => match err.kind {
            ClientErrorKind::Reqwest(err) => {
                if let Some(status_code) = err.status() {
                    if status_code == StatusCode::GONE {
                        panic!(
                                "\n{} Your RPC provider does not support the getProgramAccounts endpoint, needed to execute this command. Please use a different RPC provider.\n",
                                "ERROR"
                            );
                    }
                }
                return Err(anyhow::anyhow!("Failed to get program accounts: {}", err));
            }
            _ => return Err(anyhow::anyhow!("Failed to get program accounts: {}", err)),
        },
    }
}
