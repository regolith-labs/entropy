mod commitment;
mod var;

pub use commitment::*;
pub use var::*;

use steel::*;

use crate::consts::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum EntropyAccount {
    Commitment = 0,
    Var = 1,
}

/// Fetch PDA of the counter account.
pub fn commitment_pda(authority: Pubkey, var: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[COMMITMENT, &authority.to_bytes(), &var.to_bytes()],
        &crate::id(),
    )
}

/// Fetch PDA of the var account.
pub fn var_pda(authority: Pubkey, id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[VAR, &authority.to_bytes(), &id.to_le_bytes()],
        &crate::id(),
    )
}
