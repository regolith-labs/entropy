use entropy_api::prelude::*;
use steel::*;

pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let clock = Clock::get()?;
    let args = Open::try_from_bytes(data)?;
    let samples = args.samples;

    // Load accounts.
    let [authority_info, entropy_info, var_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    authority_info.is_signer()?;
    entropy_info.is_signer()?.has_address(&ENTROPY_PROVIDER)?;
    var_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[VAR, &authority_info.key.to_bytes()], &entropy_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Create var account.
    create_program_account::<Var>(
        var_info,
        system_program,
        authority_info,
        &entropy_api::ID,
        &[VAR, &authority_info.key.to_bytes()],
    )?;
    let var = var_info.as_account_mut::<Var>(&entropy_api::ID)?;
    var.authority = *authority_info.key;
    var.commit = entropy_info.key.to_bytes();
    var.seed = [0; 32];
    var.slot_hash = [0; 32];
    var.value = [0; 32];
    var.samples = samples;
    var.start_at = clock.slot;
    var.end_at = clock.slot + samples;

    Ok(())
}
