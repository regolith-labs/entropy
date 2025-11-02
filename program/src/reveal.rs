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

    // Silent return.
    if var.seed != [0; 32] {
        return Ok(());
    }

    // Validate the seed.
    if !var.is_valid(seed) {
        return Err(trace("Invalid seed", ProgramError::InvalidInstructionData));
    }

    // Record the revealed seed.
    var.seed = seed;

    // Finalize the value.
    var.value =
        solana_program::keccak::hashv(&[&var.slot_hash, &var.seed, &var.samples.to_le_bytes()])
            .to_bytes();

    Ok(())
}
