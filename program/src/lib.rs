mod close;
mod commit;
mod open;
mod reveal;

use close::*;
use commit::*;
use open::*;
use reveal::*;

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
        EntropyInstruction::Commit => process_commit(accounts, data)?,
        EntropyInstruction::Reveal => process_reveal(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
