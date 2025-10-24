use steel::*;

use crate::error::EntropyError;

use super::EntropyAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Var {
    /// The creator of the variable.
    pub authority: Pubkey,

    /// The amount of SOL that must be deposited per commitment, returned only if revealed.
    pub deposit: u64,

    /// The digest of the variable.
    pub digest: [u8; 32],

    /// The account which should receive the deposits of unrevealed commits.
    pub fee_collector: Pubkey,

    /// A unique identifier for the variable.
    pub id: u64,

    /// The timestamp after which commits are no longer accepted.
    pub last_commit_at: u64,

    /// The timestamp after which reveals are no longer accepted.
    pub last_reveal_at: u64,

    /// The timestamp after which the variable account may be closed.
    pub close_at: u64,

    /// The number of commits submitted.
    pub commit_count: u64,

    /// The number of reveals submitted.
    pub reveal_count: u64,
}

impl Var {
    pub fn finalize(&self, clock: &Clock) -> Result<[u8; 32], EntropyError> {
        if self.reveal_count < self.commit_count && clock.slot < self.last_reveal_at {
            return Err(EntropyError::IncompleteDigest);
        }
        Ok(solana_program::keccak::hash(self.digest.as_ref()).to_bytes())
    }
}

account!(EntropyAccount, Var);
