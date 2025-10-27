use entropy_api::prelude::*;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

pub fn process_sample(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, var_info, slot_hashes_sysvar] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let var = var_info
        .as_account_mut::<Var>(&entropy_api::ID)?
        .assert_mut(|v| clock.slot >= v.end_at)?
        .assert_mut(|v| v.slot_hash == [0; 32])?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Sample the slot hash.
    let slot_hashes =
        bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
    let Some(slot_hash) = slot_hashes.get(&var.end_at) else {
        // TODO Handle this error safely
        return Err(trace(
            "Slot hash unavailable",
            ProgramError::InvalidAccountData,
        ));
    };

    // Record the sampled slot hash.
    var.slot_hash = slot_hash.to_bytes();

    Ok(())
}
