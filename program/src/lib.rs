mod close;
mod next;
mod open;
mod reveal;
mod sample;
mod update;

use close::*;
use next::*;
use open::*;
use reveal::*;
use sample::*;
use update::*;

use entropy_api::prelude::*;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&entropy_api::ID, program_id, data)?;

    match ix {
        EntropyInstruction::Open => process_open(accounts, data)?,
        EntropyInstruction::Close => process_close(accounts, data)?,
        EntropyInstruction::Next => process_next(accounts, data)?,
        EntropyInstruction::Reveal => process_reveal(accounts, data)?,
        EntropyInstruction::Sample => process_sample(accounts, data)?,
        EntropyInstruction::Update => process_update(accounts, data)?,
        // _ => {
        //     return Err(trace(
        //         "Invalid instruction",
        //         ProgramError::InvalidInstructionData,
        //     ))
        // }
    }

    Ok(())
}

entrypoint!(process_instruction);
