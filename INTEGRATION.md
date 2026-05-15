# Client Integration Guide

This guide shows how to integrate with the Entropy program from a Rust client. It covers reading on-chain state, subscribing to real-time updates, and projecting the entropy value before `sample` is called.

## Dependencies

```toml
[dependencies]
entropy-api = { git = "https://github.com/regolith-labs/entropy" }
fixed = "1.28"
solana-client = "2.1"
solana-pubsub-client = "2.1"
solana-sdk = "2.1"
solana-account-decoder = "2.1"
steel = "4.0.2"
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
```

## Reading the Var Account

The `Var` account is a PDA at seeds `[b"var"]` under the Entropy program. It contains the current random value, the bit state, stored prices, and EWMA variances.

```rust
use entropy_api::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use steel::AccountDeserialize;

let rpc = RpcClient::new("https://your-rpc-url".to_string());
let var_address = var_pda().0;
let account = rpc.get_account(&var_address).await?;
let var = Var::try_from_bytes(&account.data)?;

// The random value (256-bit hash)
let value: [u8; 32] = var.value;

// The slot it was sampled at
let sample_slot: u64 = var.sample_at;

// Derive outcomes
let winning_square = u32::from_le_bytes(var.value[0..4].try_into().unwrap()) % 25;
let motherlode = u32::from_le_bytes(var.value[4..8].try_into().unwrap()) % 625;
```

## Parsing Pyth Price Feeds

Each Pyth pull oracle account stores the price at byte offset 73 (i64 LE), the exponent at offset 89 (i32 LE), and the publish timestamp at offset 93 (i64 LE).

```rust
const PYTH_PRICE_OFFSET: usize = 73;
const PYTH_EXPONENT_OFFSET: usize = 89;
const PYTH_PUBLISH_TIME_OFFSET: usize = 93;

fn parse_pyth_price(data: &[u8]) -> Option<i64> {
    if data.len() < PYTH_PUBLISH_TIME_OFFSET + 8 {
        return None;
    }
    let raw_price = i64::from_le_bytes(
        data[PYTH_PRICE_OFFSET..PYTH_PRICE_OFFSET + 8].try_into().ok()?
    );
    let exponent = i32::from_le_bytes(
        data[PYTH_EXPONENT_OFFSET..PYTH_EXPONENT_OFFSET + 4].try_into().ok()?
    );
    // Normalize to 8 decimal places
    let shift = 8 + exponent;
    if shift.unsigned_abs() > 18 { return None; }
    let result = if shift == 0 {
        raw_price as i128
    } else if shift > 0 {
        (raw_price as i128).checked_mul(10i128.pow(shift as u32))?
    } else {
        (raw_price as i128) / 10i128.pow((-shift) as u32)
    };
    i64::try_from(result).ok()
}

fn parse_publish_time(data: &[u8]) -> Option<i64> {
    if data.len() < PYTH_PUBLISH_TIME_OFFSET + 8 { return None; }
    let ts = i64::from_le_bytes(
        data[PYTH_PUBLISH_TIME_OFFSET..PYTH_PUBLISH_TIME_OFFSET + 8].try_into().ok()?
    );
    if ts > 1_000_000_000 && ts < 2_000_000_000 { Some(ts) } else { None }
}
```

## Projecting the Entropy Value

Given the current Var state and live Pyth prices, you can compute what the entropy value *would be* if `sample` were called right now.

```rust
use entropy_api::prelude::*;
use fixed::types::I80F48;

fn project_entropy(var: &Var, live_prices: &[i64; NUM_FEEDS]) -> [u8; 32] {
    let mut bits = var.bits;

    for i in 0..NUM_FEEDS {
        let live_price = live_prices[i];
        let prev_price = var.prices[i];

        // Skip invalid feeds
        if live_price <= 0 || prev_price <= 0 {
            continue;
        }

        // Read stored EWMA variance and compute std dev
        let variance = var.variances[i].to_i80f48();
        let std_dev = variance.sqrt();

        // Estimate dt (use HALFLIFE as approximation)
        let dt = I80F48::from_num(HALFLIFE);
        let sqrt_dt = dt.sqrt();

        // Compute threshold = max(sensitivity * std_dev * sqrt_dt, price * MIN_BPS / 10_000)
        let sensitivity = I80F48::from_num(SENSITIVITY_NUM)
            / I80F48::from_num(SENSITIVITY_DENOM);
        let vol_threshold = sensitivity * std_dev * sqrt_dt;
        let min_threshold = I80F48::from_num(prev_price.unsigned_abs())
            * I80F48::from_num(MIN_BPS)
            / I80F48::from_num(10_000u64);
        let threshold = if vol_threshold > min_threshold {
            vol_threshold
        } else {
            min_threshold
        };

        // Check if price moved beyond threshold
        let dp = (live_price - prev_price).unsigned_abs();
        if I80F48::from_num(dp) > threshold {
            let new_bit: u32 = if live_price > prev_price { 1 } else { 0 };
            if new_bit == 1 {
                bits |= 1 << i;
            } else {
                bits &= !(1 << i);
            }
        }
    }

    // Hash bits only (no slot included)
    solana_sdk::keccak::hash(&bits.to_le_bytes()).0
}
```

## Real-Time Subscriptions

Subscribe to the Var account and all 32 Pyth feeds via WebSocket. On every update, recompute the projected entropy value.

```rust
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::{Arc, Mutex};
use tokio_stream::StreamExt;

struct EntropyState {
    var: Option<Var>,
    live_prices: [i64; NUM_FEEDS],
}

async fn subscribe_entropy(wss_url: &str) -> Arc<Mutex<EntropyState>> {
    let state = Arc::new(Mutex::new(EntropyState {
        var: None,
        live_prices: [0i64; NUM_FEEDS],
    }));

    let config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        commitment: Some(CommitmentConfig::confirmed()),
        ..Default::default()
    };

    // Subscribe to Var account
    let var_address = var_pda().0;
    let state_clone = state.clone();
    let wss = wss_url.to_string();
    tokio::spawn(async move {
        loop {
            if let Ok(pubsub) = PubsubClient::new(&wss).await {
                let config = RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    commitment: Some(CommitmentConfig::confirmed()),
                    ..Default::default()
                };
                if let Ok((mut stream, _unsub)) =
                    pubsub.account_subscribe(&var_address, Some(config)).await
                {
                    while let Some(response) = stream.next().await {
                        if let Some(data) = response.value.data.decode() {
                            if let Ok(var) = Var::try_from_bytes(&data) {
                                state_clone.lock().unwrap().var = Some(*var);
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    });

    // Subscribe to each Pyth feed
    for i in 0..NUM_FEEDS {
        let state_clone = state.clone();
        let wss = wss_url.to_string();
        let feed_address = FEED_ADDRESSES[i];
        tokio::spawn(async move {
            loop {
                if let Ok(pubsub) = PubsubClient::new(&wss).await {
                    let config = RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        commitment: Some(CommitmentConfig::confirmed()),
                        ..Default::default()
                    };
                    if let Ok((mut stream, _unsub)) =
                        pubsub.account_subscribe(&feed_address, Some(config)).await
                    {
                        while let Some(response) = stream.next().await {
                            if let Some(data) = response.value.data.decode() {
                                if let Some(price) = parse_pyth_price(&data) {
                                    state_clone.lock().unwrap().live_prices[i] = price;
                                }
                            }
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });
    }

    state
}
```

## Using the Projected Value

Once subscribed, poll the state to get the current projection:

```rust
let state = subscribe_entropy("wss://your-rpc-url").await;

loop {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let s = state.lock().unwrap();
    let Some(var) = &s.var else { continue };

    let projected = project_entropy(var, &s.live_prices);
    let projected_hash = solana_sdk::keccak::Hash::new_from_array(projected);

    let winning_square = u32::from_le_bytes(projected[0..4].try_into().unwrap()) % 25;
    let motherlode = u32::from_le_bytes(projected[4..8].try_into().unwrap()) % 625;

    println!("Projected entropy: {}", projected_hash);
    println!("Winning square: {}", winning_square);
    println!("Motherlode: {} (hit={})", motherlode, motherlode == 0);
}
```

## Submitting a Sample

To actually update the on-chain entropy value, submit a `sample` instruction. This requires a v0 transaction with an address lookup table (LUT) because the instruction references 34 accounts.

```rust
use solana_sdk::{
    address_lookup_table::{state::AddressLookupTable, AddressLookupTableAccount},
    compute_budget::ComputeBudgetInstruction,
    message::{v0::Message, VersionedMessage},
    transaction::VersionedTransaction,
};

async fn submit_sample(
    rpc: &RpcClient,
    payer: &Keypair,
    lut_address: Pubkey,
) -> Signature {
    // Fetch the LUT
    let lut_account = rpc.get_account(&lut_address).await.unwrap();
    let lut = AddressLookupTable::deserialize(&lut_account.data).unwrap();
    let lut = AddressLookupTableAccount {
        key: lut_address,
        addresses: lut.addresses.to_vec(),
    };

    // Build v0 transaction
    let ixs = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
        ComputeBudgetInstruction::set_compute_unit_price(100_000),
        entropy_api::sdk::sample(payer.pubkey()),
    ];
    let blockhash = rpc.get_latest_blockhash().await.unwrap();
    let msg = Message::try_compile(&payer.pubkey(), &ixs, &[lut], blockhash).unwrap();
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[payer]).unwrap();

    rpc.send_and_confirm_transaction(&tx).await.unwrap()
}
```

## Account Layout Reference

```
Var account (820 bytes + 8 discriminator):
  offset  8: value        [u8; 32]     — keccak hash (the random value)
  offset 40: sample_at    u64          — slot of last sample
  offset 48: bits         u32          — current bit state (1 per feed)
  offset 52: _padding     [u8; 4]
  offset 56: prices       [i64; 32]    — prices from last sample (8 decimals)
  offset 312: variances   [Numeric; 32] — EWMA variance per feed (I80F48)

Pyth feed account (134 bytes):
  offset 73: price        i64          — raw price
  offset 89: exponent     i32          — decimal exponent
  offset 93: publish_time i64          — unix timestamp of last update
```
