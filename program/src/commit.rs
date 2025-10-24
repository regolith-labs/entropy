use entropy_api::prelude::*;
use steel::*;

pub fn process_commit(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Commit::try_from_bytes(data)?;
    let hash = args.hash;

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, commitment_info, var_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let var = var_info
        .as_account_mut::<Var>(&entropy_api::ID)?
        .assert_mut(|v| clock.slot < v.last_commit_at)?;
    system_program.is_program(&system_program::ID)?;

    // Commit to the var.
    if commitment_info.data_is_empty() {
        // Create the commitment account.
        create_program_account::<Commitment>(
            commitment_info,
            system_program,
            &signer_info,
            &entropy_api::ID,
            &[
                COMMITMENT,
                &signer_info.key.to_bytes(),
                &var_info.key.to_bytes(),
            ],
        )?;
        let commitment = commitment_info.as_account_mut::<Commitment>(&entropy_api::ID)?;
        commitment.authority = *signer_info.key;
        commitment.var = *var_info.key;
        commitment.hash = hash;

        // Make deposit.
        var_info.collect(var.deposit, &signer_info)?;

        // Update var.
        var.commit_count += 1;
    } else {
        // Update the commitment account.
        let commitment = commitment_info
            .as_account_mut::<Commitment>(&entropy_api::ID)?
            .assert_mut(|c| c.authority == *signer_info.key)?
            .assert_mut(|c| c.var == *var_info.key)?;
        commitment.hash = hash;
    }

    Ok(())
}
