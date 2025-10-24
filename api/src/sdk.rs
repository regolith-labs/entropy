use steel::*;

use crate::prelude::*;

pub fn open(
    signer: Pubkey,
    id: u64,
    last_commit_at: u64,
    last_reveal_at: u64,
    close_at: u64,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(var_pda(signer, id).0, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Open {
            id: id.to_le_bytes(),
            last_commit_at,
            last_reveal_at,
            close_at,
        }
        .to_bytes(),
    }
}

pub fn commit(signer: Pubkey, var: Pubkey, hash: [u8; 32]) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(commitment_pda(signer, var).0, false),
            AccountMeta::new(var, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Commit { hash }.to_bytes(),
    }
}

pub fn reveal(signer: Pubkey, var: Pubkey, seed: [u8; 32]) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(commitment_pda(signer, var).0, false),
            AccountMeta::new(var, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Reveal { seed }.to_bytes(),
    }
}
