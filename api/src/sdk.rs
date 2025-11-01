use steel::*;

use crate::prelude::*;

pub fn open(
    signer: Pubkey,
    payer: Pubkey,
    id: u64,
    provider: Pubkey,
    commit: [u8; 32],
    is_auto: bool,
    samples: u64,
    end_at: u64,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(provider, false),
            AccountMeta::new(var_pda(signer, id).0, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Open {
            id: id.to_le_bytes(),
            is_auto: (is_auto as u64).to_le_bytes(),
            commit,
            samples: samples.to_le_bytes(),
            end_at: end_at.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn next(signer: Pubkey, var: Pubkey, end_at: u64) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![AccountMeta::new(signer, true), AccountMeta::new(var, false)],
        data: Next {
            end_at: end_at.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn update(signer: Pubkey, var: Pubkey, end_at: u64) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![AccountMeta::new(signer, true), AccountMeta::new(var, false)],
        data: Update {
            end_at: end_at.to_le_bytes(),
        }
        .to_bytes(),
    }
}

pub fn reveal(signer: Pubkey, var: Pubkey, seed: [u8; 32]) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![AccountMeta::new(signer, true), AccountMeta::new(var, false)],
        data: Reveal { seed }.to_bytes(),
    }
}

pub fn sample(signer: Pubkey, var: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(var, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
        ],
        data: Sample {}.to_bytes(),
    }
}
