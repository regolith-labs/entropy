mod var;

pub use var::*;

use steel::*;

use crate::consts::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum EntropyAccount {
    Var = 0,
}

/// Fetch PDA of the var account.
pub fn var_pda(authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VAR, &authority.to_bytes()], &crate::id())
}
