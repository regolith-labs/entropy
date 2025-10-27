use steel::*;

use crate::prelude::*;

pub fn open(
    signer: Pubkey,
    commit: [u8; 32],
    is_auto: bool,
    samples: u64,
    end_at: u64,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(ENTROPY_PROVIDER, true),
            AccountMeta::new(var_pda(signer).0, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Open {
            is_auto: is_auto as u64,
            commit,
            samples,
            end_at,
        }
        .to_bytes(),
    }
}

pub fn next(signer: Pubkey, var: Pubkey, end_at: u64) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![AccountMeta::new(signer, true), AccountMeta::new(var, false)],
        data: Next { end_at }.to_bytes(),
    }
}

pub fn reveal(signer: Pubkey, var: Pubkey, seed: [u8; 32]) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![AccountMeta::new(signer, true), AccountMeta::new(var, false)],
        data: Reveal { seed }.to_bytes(),
    }
}
