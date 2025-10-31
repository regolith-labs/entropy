use steel::*;

use super::EntropyAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Var {
    /// The creator of the variable.
    pub authority: Pubkey,

    /// The provider of the entropy data.
    pub provider: Pubkey,

    /// The commit provided by Entropy provider.
    pub commit: [u8; 32],

    /// The revealed seed.
    pub seed: [u8; 32],

    /// The slot hash
    pub slot_hash: [u8; 32],

    /// The current value of the variable.
    pub value: [u8; 32],

    /// The number of random variables remaining to be sampled.
    pub samples: u64,

    /// Whether or not the Entropy provider should automatically sample the slot hash.
    pub is_auto: u64,

    /// The slot at which the variable was opened.
    pub start_at: u64,

    /// The slot at which the variable should sample the slothash.
    pub end_at: u64,
}

impl Var {
    pub fn is_valid(&self, seed: [u8; 32]) -> bool {
        if self.slot_hash == [0; 32] {
            return false;
        }
        if self.value != [0; 32] {
            return false;
        }
        if self.samples == 0 {
            return false;
        }
        let expected_commit = solana_program::keccak::hash(&seed).to_bytes();
        expected_commit == self.commit
    }
}

account!(EntropyAccount, Var);
