use entropy_api::prelude::*;
use fixed::types::I80F48;
use solana_program::{keccak, log::sol_log};
use steel::*;

/// Pyth pull oracle price account field offsets.
const PYTH_PRICE_OFFSET: usize = 73;
const PYTH_EXPONENT_OFFSET: usize = 89;

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

    let clock = Clock::get()?;
    let slot = clock.slot;
    let first_sample = var.sample_at == 0;

    let dt = slot.saturating_sub(var.sample_at).max(1);
    let dt_f = I80F48::from_num(dt);

    // Alpha = min(dt / HALFLIFE, 1.0)
    let alpha = if dt >= HALFLIFE {
        I80F48::ONE
    } else {
        I80F48::from_num(dt) / I80F48::from_num(HALFLIFE)
    };

    let mut bits = var.bits;
    let mut flips: u32 = 0;

    for i in 0..NUM_FEEDS {
        let feed_info = &feed_accounts[i];
        feed_info.has_address(&FEED_ADDRESSES[i])?;

        let data = feed_info.try_borrow_data()?;
        let price = match parse_pyth_price(&data) {
            Ok(p) if p > 0 => p,
            _ => {
                sol_log(&format!("{}: skipped", FEED_TICKERS[i]));
                continue;
            }
        };

        if first_sample {
            // First sample: store price, set initial bit, skip variance.
            var.prices[i] = price;
            var.variances[i] = Numeric::ZERO;
            if price & 1 == 1 {
                bits |= 1 << i;
            } else {
                bits &= !(1 << i);
            }
            flips += 1;
            sol_log(&format!(
                "{}: {} bit={}",
                FEED_TICKERS[i],
                format_price(price),
                (bits >> i) & 1
            ));
            continue;
        }

        let prev = var.prices[i];
        let dp = price - prev;
        let dp_f = I80F48::from_num(dp);

        // EWMA variance update: variance = lerp(old, observed, alpha)
        let dp_sq = dp_f.saturating_mul(dp_f);
        let observed_var = dp_sq / dt_f;
        let old_var = var.variances[i].to_i80f48();
        let mut new_var = old_var + alpha * (observed_var - old_var);
        if new_var.is_negative() {
            new_var = I80F48::ZERO;
        }
        var.variances[i] = Numeric::from_i80f48(new_var);

        // Threshold = max(MULT * sqrt(variance) * sqrt(dt), |prev| * MIN_BPS / 10_000)
        let std_dev = new_var.sqrt();
        let sqrt_dt = dt_f.sqrt();
        let sensitivity = I80F48::from_num(SENSITIVITY_NUM) / I80F48::from_num(SENSITIVITY_DENOM);
        let vol_threshold = sensitivity * std_dev * sqrt_dt;

        let prev_abs = I80F48::from_num(prev.unsigned_abs());
        let min_threshold = prev_abs * I80F48::from_num(MIN_BPS) / I80F48::from_num(10_000u64);

        let threshold = if vol_threshold > min_threshold {
            vol_threshold
        } else {
            min_threshold
        };

        // Flip bit if |dp| > threshold.
        // Count all threshold crossings, not just bit-value changes,
        // because the initial bit assignment is arbitrary.
        let dp_abs = I80F48::from_num(dp.unsigned_abs());
        let flipped = dp_abs > threshold;
        if flipped {
            flips += 1;
            let new_bit: u32 = if price > prev { 1 } else { 0 };
            if new_bit == 1 {
                bits |= 1 << i;
            } else {
                bits &= !(1 << i);
            }
        }

        sol_log(&format!(
            "{}: {} var={} thr={} {}",
            FEED_TICKERS[i],
            format_price(price),
            format_fixed(std_dev),
            format_fixed(threshold),
            if flipped { "FLIP" } else { "-" }
        ));

        var.prices[i] = price;
    }

    // Hash bits only — slot is excluded so leaders can't gain advantage by
    // choosing which slot to land the transaction in.
    let hash = keccak::hash(&bits.to_le_bytes());

    var.bits = bits;
    var.value = hash.to_bytes();
    var.sample_at = slot;

    sol_log(&format!("flips: {}", flips));
    sol_log(&format!("bits: 0b{:032b}", bits));
    sol_log(&format!("entropy: {}", hash));

    Ok(())
}

/// Formats a fixed-point i64 (8 decimals) as a USD string.
fn format_price(price: i64) -> String {
    let whole = price / 100_000_000;
    let frac = (price % 100_000_000).unsigned_abs();
    format!("${}.{:08}", whole, frac)
}

/// Formats an I80F48 as a decimal string (truncated to 2 places).
fn format_fixed(v: I80F48) -> String {
    let int_part = v.to_num::<i64>();
    let frac_part = ((v - I80F48::from_num(int_part)).abs() * I80F48::from_num(100))
        .to_num::<u64>();
    format!("{}.{:02}", int_part, frac_part)
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

/// Normalizes a price with a given exponent to 8 decimal places.
/// Uses i128 internally to avoid overflow on large prices or large shifts.
fn normalize_price(price: i64, exponent: i32) -> Result<i64, ProgramError> {
    const PRICE_DECIMALS: i32 = 8;
    let shift = PRICE_DECIMALS + exponent;
    let result = if shift == 0 {
        price as i128
    } else if shift > 0 {
        let factor = 10i128.pow(shift as u32);
        (price as i128) * factor
    } else {
        let factor = 10i128.pow((-shift) as u32);
        (price as i128) / factor
    };
    i64::try_from(result).map_err(|_| ProgramError::ArithmeticOverflow)
}
