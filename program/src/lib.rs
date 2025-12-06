mod close;
mod next;
mod open;
mod reveal;
mod sample;

use close::*;
use next::*;
use open::*;
use reveal::*;
use sample::*;

use entropy_api::prelude::*;
use solana_security_txt::security_txt;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&entropy_api::ID, program_id, data)?;

    match ix {
        // EntropyInstruction::Open => process_open(accounts, data)?,
        EntropyInstruction::Close => process_close(accounts, data)?,
        EntropyInstruction::Next => process_next(accounts, data)?,
        EntropyInstruction::Reveal => process_reveal(accounts, data)?,
        EntropyInstruction::Sample => process_sample(accounts, data)?,
        _ => {
            return Err(trace(
                "Invalid instruction",
                ProgramError::InvalidInstructionData,
            ))
        }
    }

    Ok(())
}

entrypoint!(process_instruction);

security_txt! {
    name: "Entropy",
    project_url: "https://ore.supply",
    contacts: "email:hardhatchad@gmail.com,discord:hardhatchad",
    policy: "https://github.com/regolith-labs/entropy/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/regolith-labs/entropy"
    // source_revision: default_env!("GITHUB_SHA", ""),
    // source_release: default_env!("GITHUB_REF_NAME", ""),
    // auditors: "None"
}
