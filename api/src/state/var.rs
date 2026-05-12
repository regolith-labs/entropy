use steel::*;

use super::EntropyAccount;
use crate::consts::NUM_FEEDS;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Var {
    /// The current random hash value.
    pub value: [u8; 32],

    /// The slot the sample was taken at.
    pub sample_at: u64,

    /// Current bit state (1 bit per feed).
    pub bits: u32,

    /// Padding for alignment.
    pub _padding: [u8; 4],

    /// Prices from the last sample (normalized to 8 decimals).
    pub prices: [i64; NUM_FEEDS],

    /// EWMA variance-per-slot for each feed.
    pub variances: [Numeric; NUM_FEEDS],
}

account!(EntropyAccount, Var);
