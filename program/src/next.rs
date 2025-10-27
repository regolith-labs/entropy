use entropy_api::prelude::*;
use steel::*;

pub fn process_next(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Next::try_from_bytes(data)?;
    let end_at = args.end_at;

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, var_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let var = var_info
        .as_account_mut::<Var>(&entropy_api::ID)?
        .assert_mut_msg(|v| v.authority == *signer_info.key, "Invalid var authority")?
        .assert_mut_msg(|v| clock.slot >= v.end_at, "Not ready to next")?
        .assert_mut_msg(|v| v.slot_hash == [0; 32], "Slot hash not sampled")?
        .assert_mut_msg(|v| v.seed != [0; 32], "Seed not revealed")?
        .assert_mut_msg(|v| v.value != [0; 32], "Value is not finalized")?
        .assert_mut_msg(|v| v.samples > 0, "No samples remaining")?;

    // Validate the end at slot.
    assert!(
        end_at > clock.slot,
        "End at must be greater than current slot"
    );

    // Update the var for the next value.
    var.commit = var.seed;
    var.seed = [0; 32];
    var.slot_hash = [0; 32];
    var.value = [0; 32];
    var.samples -= 1;
    var.start_at = clock.slot;
    var.end_at = end_at;

    Ok(())
}
