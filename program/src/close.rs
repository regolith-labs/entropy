use steel::*;

/// Closes the var account and returns lamports to the signer.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    let [signer_info, var_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    var_info.has_owner(&entropy_api::ID)?;

    let lamports = var_info.lamports();
    **var_info.try_borrow_mut_lamports()? = 0;
    **signer_info.try_borrow_mut_lamports()? += lamports;
    var_info.assign(&system_program::ID);
    var_info.realloc(0, true)?;

    Ok(())
}
