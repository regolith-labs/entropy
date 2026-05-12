use steel::*;

use super::EntropyAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Var {
    /// The id of the variable.
    pub value: [u8; 32],

    /// The slot the sample was taken at.
    pub sample_at: u64,
}

account!(EntropyAccount, Var);
