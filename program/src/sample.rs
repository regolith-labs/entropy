use entropy_api::prelude::*;
use solana_program::{log::sol_log, slot_hashes::SlotHashes};
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

    // Deserialize the slot hashes.
    let slot_hashes =
        bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();

    // Record the sampled slot hash.
    if let Some(slot_hash) = slot_hashes.get(&var.end_at) {
        var.slot_hash = slot_hash.to_bytes();
        sol_log(&format!(
            "Sampled hash at slot {:?}: {:?}",
            var.end_at,
            slot_hash.to_string()
        ));
    } else {
        let hash = solana_program::keccak::hashv(&[&var.end_at.to_le_bytes()]);
        var.slot_hash = hash.to_bytes();
        sol_log(&format!(
            "No hash for slot {:?}. Generated: {:?}",
            var.end_at,
            hash.to_string()
        ));
    }

    Ok(())
}
