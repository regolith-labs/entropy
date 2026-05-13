use std::sync::{Arc, Mutex};

use crossterm::{cursor, execute, terminal};
use entropy_api::prelude::*;
use fixed::types::I80F48;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use solana_sdk::pubkey;
use solana_sdk::{
    address_lookup_table::{
        instruction::{create_lookup_table, extend_lookup_table},
        state::AddressLookupTable,
        AddressLookupTableAccount,
    },
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    message::{v0::Message, VersionedMessage},
    signature::{read_keypair_file, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use steel::AccountDeserialize;
use tokio_stream::StreamExt;

const PYTH_PRICE_OFFSET: usize = 73;
const PYTH_EXPONENT_OFFSET: usize = 89;
const PYTH_PUBLISH_TIME_OFFSET: usize = 93;

// TODO: Create new LUT after deploying with 32 feeds and update this address.
const LUT_ADDRESS: solana_sdk::pubkey::Pubkey =
    pubkey!("38fAv3Tmwumkufk9JZu6pVU5DgdAnnNoVp831CqCaX4i");

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args
        .get(1)
        .expect("Usage: entropy-cli <init|close|sample|var|lut|watch>");

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
        "watch" => {
            watch(&rpc).await;
        }
        _ => {
            eprintln!("Unknown command: {command}");
            eprintln!("Usage: entropy-cli <init|close|sample|var|lut|watch>");
            std::process::exit(1);
        }
    }
}

// --- Watch dashboard ---

#[derive(Clone)]
struct WatchState {
    var_data: Option<Vec<u8>>,
    feed_prices: [i64; NUM_FEEDS],
    feed_publish_times: [i64; NUM_FEEDS],
}

async fn watch(rpc: &RpcClient) {
    // Convert HTTP URL to WSS
    let rpc_url = std::env::var("RPC").unwrap();
    let wss_url = rpc_url
        .replace("https://", "wss://")
        .replace("http://", "ws://");

    // Fetch initial state
    let var_address = var_pda().0;
    let state = Arc::new(Mutex::new(WatchState {
        var_data: None,
        feed_prices: [0i64; NUM_FEEDS],
        feed_publish_times: [0i64; NUM_FEEDS],
    }));

    // Load initial var account
    if let Ok(account) = rpc.get_account(&var_address).await {
        state.lock().unwrap().var_data = Some(account.data);
    }

    // Load initial feed prices
    for i in 0..NUM_FEEDS {
        if let Ok(account) = rpc.get_account(&FEED_ADDRESSES[i]).await {
            let mut s = state.lock().unwrap();
            if let Some(price) = parse_pyth_price_from_bytes(&account.data) {
                s.feed_prices[i] = price;
            }
            if let Some(ts) = parse_pyth_publish_time(&account.data) {
                s.feed_publish_times[i] = ts;
            }
        }
    }

    // Initial render
    render_dashboard(&state.lock().unwrap());

    // Subscribe to var account
    let state_var = state.clone();
    let wss_var = wss_url.clone();
    tokio::spawn(async move {
        loop {
            match PubsubClient::new(&wss_var).await {
                Ok(pubsub) => {
                    let config = RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        commitment: Some(CommitmentConfig::confirmed()),
                        ..Default::default()
                    };
                    match pubsub.account_subscribe(&var_address, Some(config)).await {
                        Ok((mut stream, _unsub)) => {
                            while let Some(response) = stream.next().await {
                                if let Some(ui_account) = response.value.data.decode() {
                                    state_var.lock().unwrap().var_data = Some(ui_account);
                                    render_dashboard(&state_var.lock().unwrap());
                                }
                            }
                        }
                        Err(e) => eprintln!("Var subscribe error: {e}"),
                    }
                }
                Err(e) => eprintln!("WSS connect error: {e}"),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    });

    // Subscribe to each feed account
    for i in 0..NUM_FEEDS {
        let state_feed = state.clone();
        let wss_feed = wss_url.clone();
        let feed_address = FEED_ADDRESSES[i];
        tokio::spawn(async move {
            loop {
                match PubsubClient::new(&wss_feed).await {
                    Ok(pubsub) => {
                        let config = RpcAccountInfoConfig {
                            encoding: Some(UiAccountEncoding::Base64),
                            commitment: Some(CommitmentConfig::confirmed()),
                            ..Default::default()
                        };
                        match pubsub.account_subscribe(&feed_address, Some(config)).await {
                            Ok((mut stream, _unsub)) => {
                                while let Some(response) = stream.next().await {
                                    if let Some(data) = response.value.data.decode() {
                                        let mut s = state_feed.lock().unwrap();
                                        if let Some(price) = parse_pyth_price_from_bytes(&data) {
                                            s.feed_prices[i] = price;
                                        }
                                        if let Some(ts) = parse_pyth_publish_time(&data) {
                                            s.feed_publish_times[i] = ts;
                                        }
                                        render_dashboard(&s);
                                        drop(s);
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    Err(_) => {}
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });
    }

    // Keep main alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}

fn render_dashboard(state: &WatchState) {
    let mut out = std::io::stdout();
    let _ = execute!(
        out,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    );

    println!("=== ENTROPY DASHBOARD ===\n");

    let var = match &state.var_data {
        Some(data) => match Var::try_from_bytes(data) {
            Ok(v) => Some(*v),
            Err(_) => None,
        },
        None => None,
    };

    let Some(var) = var else {
        println!("Var account not loaded yet...");
        return;
    };

    // Current on-chain state
    println!(
        "On-chain value:  {}",
        solana_sdk::keccak::Hash::new_from_array(var.value)
    );
    println!("Sample slot:     {}", var.sample_at);
    println!("Current bits:    0b{:032b}", var.bits);
    println!();

    // Simulate what sample would produce right now
    let mut projected_bits = var.bits;
    let mut flips: u32 = 0;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    println!(
        "{:<8} {:>14} {:>14} {:>10} {:>10}  {} {:>6}",
        "FEED", "LAST SAMPLE", "LIVE PRICE", "STD DEV", "THRESHOLD", "BIT", "AGE"
    );
    println!("{}", "-".repeat(82));

    for i in 0..NUM_FEEDS {
        let live_price = state.feed_prices[i];
        let prev_price = var.prices[i];
        let old_var_f = var.variances[i].to_i80f48();
        let std_dev = old_var_f.sqrt();

        let bit = (projected_bits >> i) & 1;

        let age = format_age(now, state.feed_publish_times[i]);

        if live_price <= 0 {
            println!(
                "{:<8} {:>14} {:>14} {:>10} {:>10}  {} {:>6}",
                FEED_TICKERS[i],
                format_price_short(prev_price),
                "---",
                "---",
                "---",
                bit,
                age
            );
            continue;
        }

        // Simulate threshold (use dt=150 as estimate since we don't know exact slot)
        let dt = 150u64; // approximate
        let dt_f = I80F48::from_num(dt);
        let sqrt_dt = dt_f.sqrt();
        let vol_threshold = std_dev * sqrt_dt;
        let prev_abs = I80F48::from_num(prev_price.unsigned_abs());
        let min_threshold =
            prev_abs * I80F48::from_num(MIN_BPS) / I80F48::from_num(10_000u64);
        let threshold = if vol_threshold > min_threshold {
            vol_threshold
        } else {
            min_threshold
        };

        let dp = live_price - prev_price;
        let dp_abs = I80F48::from_num(dp.unsigned_abs());
        let would_flip = dp_abs > threshold;

        if would_flip {
            flips += 1;
            let new_bit: u32 = if live_price > prev_price { 1 } else { 0 };
            if new_bit == 1 {
                projected_bits |= 1 << i;
            } else {
                projected_bits &= !(1 << i);
            }
        }

        let bit_display = (projected_bits >> i) & 1;
        let flip_marker = if would_flip { " FLIP" } else { "" };

        println!(
            "{:<8} {:>14} {:>14} {:>10} {:>10}  {} {:>6}{}",
            FEED_TICKERS[i],
            format_price_short(prev_price),
            format_price_short(live_price),
            format_num(std_dev),
            format_num(threshold),
            bit_display,
            age,
            flip_marker,
        );
    }

    let projected_hash = solana_sdk::keccak::hash(&projected_bits.to_le_bytes());

    println!();
    println!("Projected flips: {}", flips);
    println!("Projected bits:  0b{:032b}", projected_bits);
    println!(
        "Projected value: {}",
        projected_hash
    );
    println!(
        "Projected 1/25:  {}",
        u32::from_le_bytes(projected_hash.0[0..4].try_into().unwrap()) % 25
    );
    println!(
        "Projected 1/625: {}",
        u32::from_le_bytes(projected_hash.0[4..8].try_into().unwrap()) % 625
    );
}

fn format_price_short(price: i64) -> String {
    let whole = price / 100_000_000;
    let frac = (price % 100_000_000).unsigned_abs();
    format!("${}.{:04}", whole, frac / 10_000)
}

fn format_num(v: I80F48) -> String {
    let n = v.to_num::<f64>();
    if n > 1_000_000.0 {
        format!("{:.0}", n)
    } else if n > 1000.0 {
        format!("{:.0}", n)
    } else if n > 1.0 {
        format!("{:.2}", n)
    } else {
        format!("{:.4}", n)
    }
}

fn format_age(now: i64, publish_time: i64) -> String {
    if publish_time == 0 {
        return "---".to_string();
    }
    let age = now - publish_time;
    if age < 0 {
        "0s".to_string()
    } else if age < 60 {
        format!("{}s", age)
    } else if age < 3600 {
        format!("{}m", age / 60)
    } else if age < 86400 {
        format!("{}h", age / 3600)
    } else {
        format!("{}d", age / 86400)
    }
}

fn parse_pyth_publish_time(data: &[u8]) -> Option<i64> {
    if data.len() < PYTH_PUBLISH_TIME_OFFSET + 8 {
        return None;
    }
    let ts = i64::from_le_bytes(
        data[PYTH_PUBLISH_TIME_OFFSET..PYTH_PUBLISH_TIME_OFFSET + 8]
            .try_into()
            .ok()?,
    );
    if ts > 1_000_000_000 && ts < 2_000_000_000 {
        Some(ts)
    } else {
        None
    }
}

fn parse_pyth_price_from_bytes(data: &[u8]) -> Option<i64> {
    if data.len() < PYTH_EXPONENT_OFFSET + 4 {
        return None;
    }
    let price = i64::from_le_bytes(data[PYTH_PRICE_OFFSET..PYTH_PRICE_OFFSET + 8].try_into().ok()?);
    let exponent =
        i32::from_le_bytes(data[PYTH_EXPONENT_OFFSET..PYTH_EXPONENT_OFFSET + 4].try_into().ok()?);
    normalize_price(price, exponent)
}

fn normalize_price(price: i64, exponent: i32) -> Option<i64> {
    const PRICE_DECIMALS: i32 = 8;
    let shift = PRICE_DECIMALS + exponent;
    if shift.unsigned_abs() > 18 {
        return None; // exponent out of reasonable range
    }
    let result = if shift == 0 {
        price as i128
    } else if shift > 0 {
        (price as i128).checked_mul(10i128.pow(shift as u32))?
    } else {
        (price as i128) / 10i128.pow((-shift) as u32)
    };
    i64::try_from(result).ok()
}

// --- Existing commands ---

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
