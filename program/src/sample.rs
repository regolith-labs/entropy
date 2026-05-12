use entropy_api::prelude::*;
use solana_program::{keccak, log::sol_log};
use steel::*;

/// Pyth pull oracle price account field offsets.
const PYTH_PRICE_OFFSET: usize = 73;
const PYTH_EXPONENT_OFFSET: usize = 89;

/// Reads raw prices from 28 Pyth feeds, logs each with its ticker,
/// and hashes all prices together to produce a random variable.
pub fn process_sample(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    let (core_accounts, feed_accounts) = accounts.split_at(2);
    let [signer_info, var_info] = core_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let var = var_info.as_account_mut::<Var>(&entropy_api::ID)?;

    if feed_accounts.len() < NUM_FEEDS {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Read raw price bytes from each feed and build hash input.
    let mut hash_input = [0u8; NUM_FEEDS * 8];
    for i in 0..NUM_FEEDS {
        let feed_info = &feed_accounts[i];
        feed_info.has_address(&FEED_ADDRESSES[i])?;

        let data = feed_info.try_borrow_data()?;
        let price = parse_pyth_price(&data)?;

        // Log the price with its ticker.
        sol_log(&format!("{}: {}", FEED_TICKERS[i], format_price(price)));

        // Copy raw price bytes (i64 LE) into hash input.
        hash_input[i * 8..(i + 1) * 8].copy_from_slice(&price.to_le_bytes());
    }

    // Hash all prices together.
    let hash = keccak::hash(&hash_input);
    sol_log(&format!("hash: {}", hash));

    // Update var.
    var.value = hash.to_bytes();
    var.sample_at = Clock::get()?.slot;

    Ok(())
}

/// Parses the price from a Pyth pull oracle price account.
/// Returns the price normalized to 8 decimal places as an i64.
fn parse_pyth_price(data: &[u8]) -> Result<i64, ProgramError> {
    if data.len() < PYTH_EXPONENT_OFFSET + 4 {
        return Err(ProgramError::InvalidAccountData);
    }
    let price = i64::from_le_bytes(
        data[PYTH_PRICE_OFFSET..PYTH_PRICE_OFFSET + 8]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    );
    let exponent = i32::from_le_bytes(
        data[PYTH_EXPONENT_OFFSET..PYTH_EXPONENT_OFFSET + 4]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    );
    normalize_price(price, exponent)
}

/// Normalizes a price with a given exponent to PRICE_DECIMALS (8) precision.
fn normalize_price(price: i64, exponent: i32) -> Result<i64, ProgramError> {
    const PRICE_DECIMALS: i32 = 8;
    let shift = PRICE_DECIMALS + exponent;
    if shift == 0 {
        Ok(price)
    } else if shift > 0 {
        let factor = 10i64
            .checked_pow(shift as u32)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        price
            .checked_mul(factor)
            .ok_or(ProgramError::ArithmeticOverflow)
    } else {
        let factor = 10i64
            .checked_pow((-shift) as u32)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        Ok(price / factor)
    }
}

/// Formats a fixed-point i64 (8 decimals) as a human-readable USD string.
fn format_price(price: i64) -> String {
    let whole = price / 100_000_000;
    let frac = (price % 100_000_000).unsigned_abs();
    format!("${}.{:08}", whole, frac)
}
