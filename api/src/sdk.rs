use steel::*;

use crate::prelude::*;

pub fn init(payer: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(var_pda().0, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Init {}.to_bytes(),
    }
}

pub fn close(signer: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(var_pda().0, false),
        ],
        data: Close {}.to_bytes(),
    }
}

pub fn sample(signer: Pubkey) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(signer, true),
        AccountMeta::new(var_pda().0, false),
    ];
    for feed in &FEED_ADDRESSES {
        accounts.push(AccountMeta::new_readonly(*feed, false));
    }
    Instruction {
        program_id: crate::ID,
        accounts,
        data: Sample {}.to_bytes(),
    }
}
