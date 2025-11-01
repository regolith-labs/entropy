use entropy_api::prelude::*;
use steel::*;

pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, var_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    var_info
        .as_account_mut::<Var>(&entropy_api::ID)?
        .assert_mut_msg(|v| v.authority == *signer_info.key, "Invalid var authority")?;
    system_program.is_program(&system_program::ID)?;

    // Close var account.
    var_info.close(signer_info)?;

    Ok(())
}
