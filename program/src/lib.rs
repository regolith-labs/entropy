mod init;
mod sample;

use init::*;
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
        EntropyInstruction::Init => process_init(accounts, data)?,
        EntropyInstruction::Sample => process_sample(accounts, data)?,
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
}
