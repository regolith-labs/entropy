use entropy_api::prelude::*;
use steel::*;

/// Initializes the var account with zero values.
pub fn process_init(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    let [payer_info, var_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    payer_info.is_signer()?;
    var_info.is_empty()?.is_writable()?;
    system_program.is_program(&system_program::ID)?;

    create_program_account::<Var>(
        var_info,
        system_program,
        payer_info,
        &entropy_api::ID,
        &[VAR],
    )?;

    // All fields are zero-initialized by create_program_account (Zeroable).
    // sample_at = 0 signals first-sample mode to process_sample.

    Ok(())
}
