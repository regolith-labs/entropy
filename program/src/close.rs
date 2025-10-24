use entropy_api::prelude::*;
use steel::*;

pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, commitment_info, commitment_authority_info, fee_collector_info, var_info, var_authority_info, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    system_program.is_program(&system_program::ID)?;

    // Close var account.
    if !var_info.data_is_empty() {
        // Validate var.
        let var = var_info
            .as_account_mut::<Var>(&entropy_api::ID)?
            .assert_mut(|v| v.authority == *var_authority_info.key)?
            .assert_mut(|v| clock.slot < v.close_at)?;

        // Payout fee collector
        if var.reveal_count < var.commit_count {
            fee_collector_info
                .is_writable()?
                .has_address(&var.fee_collector)?;
            let fees = (var.commit_count - var.reveal_count) * var.deposit;
            var_info.send(fees, &fee_collector_info);
        }

        // Close var account.
        var_info.close(var_authority_info)?;
    }

    // Close commitment account.
    if !commitment_info.data_is_empty() {
        commitment_info
            .as_account_mut::<Commitment>(&entropy_api::ID)?
            .assert_mut(|c| c.authority == *commitment_authority_info.key)?
            .assert_mut(|c| c.var == *var_info.key)?;
        commitment_authority_info.is_writable()?;
        commitment_info.close(commitment_authority_info)?;
    }

    Ok(())
}
