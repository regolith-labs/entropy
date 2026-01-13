use entropy_api::prelude::*;
use steel::*;

pub fn process_reveal(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Reveal::try_from_bytes(data)?;
    let seed = args.seed;

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, var_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let var = var_info
        .as_account_mut::<Var>(&entropy_api::ID)?
        .assert_mut_msg(|v| clock.slot >= v.end_at, "Not ready to reveal")?
        .assert_mut_msg(|v| v.slot_hash != [0; 32], "Slot hash not sampled")?;

    // Finalize the variable.
    var.finalize(seed)?;

    Ok(())
}
