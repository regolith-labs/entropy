use entropy_api::prelude::*;
use steel::*;

pub fn process_update(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Next::try_from_bytes(data)?;
    let end_at = u64::from_le_bytes(args.end_at);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, var_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let var = var_info
        .as_account_mut::<Var>(&entropy_api::ID)?
        .assert_mut_msg(|v| v.authority == *signer_info.key, "Invalid var authority")?
        .assert_mut_msg(|v| v.slot_hash == [0; 32], "Slot hash already sampled")?
        .assert_mut_msg(|v| v.seed == [0; 32], "Seed already revealed")?
        .assert_mut_msg(|v| v.value == [0; 32], "Value already finalized")?;

    // Validate the end at slot.
    assert!(
        end_at > clock.slot,
        "End at must be greater than current slot"
    );

    // Update the end at slot.
    var.end_at = end_at;

    Ok(())
}
