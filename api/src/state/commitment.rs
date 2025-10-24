use steel::*;

use super::EntropyAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Commitment {
    /// The creator of the variable.
    pub authority: Pubkey,

    /// The committed hash.
    pub hash: [u8; 32],

    /// The var this commitment is for.
    pub var: Pubkey,
}

impl Commitment {
    pub fn is_valid(&self, seed: [u8; 32]) -> bool {
        solana_program::keccak::hash(&seed).to_bytes() == self.hash
    }
}

account!(EntropyAccount, Commitment);
