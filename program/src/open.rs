use entropy_api::prelude::*;
use steel::*;

pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let clock = Clock::get()?;
    let args = Open::try_from_bytes(data)?;
    let deposit = args.deposit;
    let id = u64::from_le_bytes(args.id);
    let last_commit_at = args.last_commit_at;
    let last_reveal_at = args.last_reveal_at;
    let close_at = args.close_at;

    // Validate args.
    assert!(clock.slot <= last_commit_at);
    assert!(last_commit_at <= last_reveal_at);
    assert!(last_reveal_at <= close_at);

    // Load accounts.
    let [signer_info, fee_collector_info, var_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    var_info.is_empty()?.is_writable()?.has_seeds(
        &[VAR, &signer_info.key.to_bytes(), &id.to_le_bytes()],
        &entropy_api::ID,
    )?;
    system_program.is_program(&system_program::ID)?;

    // Create var account.
    create_program_account::<Var>(
        var_info,
        system_program,
        signer_info,
        &entropy_api::ID,
        &[VAR, &signer_info.key.to_bytes(), &id.to_le_bytes()],
    )?;
    let var = var_info.as_account_mut::<Var>(&entropy_api::ID)?;
    var.authority = *signer_info.key;
    var.deposit = deposit;
    var.digest = solana_program::keccak::hashv(&[
        &signer_info.key.to_bytes(),
        &id.to_le_bytes(),
        &clock.slot.to_le_bytes(),
    ])
    .to_bytes();
    var.fee_collector = *fee_collector_info.key;
    var.id = id;
    var.last_commit_at = last_commit_at;
    var.last_reveal_at = last_reveal_at;
    var.close_at = close_at;

    Ok(())
}
