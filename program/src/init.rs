use entropy_api::prelude::*;
use steel::*;

/// Initializes the var account with a zero value.
pub fn process_init(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [payer_info, var_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    payer_info.is_signer()?;
    var_info.is_empty()?.is_writable()?;
    system_program.is_program(&system_program::ID)?;

    // Create var account.
    create_program_account::<Var>(
        var_info,
        system_program,
        payer_info,
        &entropy_api::ID,
        &[VAR],
    )?;
    let var = var_info.as_account_mut::<Var>(&entropy_api::ID)?;
    var.value = [0; 32];
    var.sample_at = 0;

    Ok(())
}
