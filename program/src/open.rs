use entropy_api::prelude::*;
use steel::*;

pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let clock = Clock::get()?;
    let args = Open::try_from_bytes(data)?;
    let id = u64::from_le_bytes(args.id);
    let samples = u64::from_le_bytes(args.samples);
    let end_at = u64::from_le_bytes(args.end_at);
    let is_auto = u64::from_le_bytes(args.is_auto);
    let commit = args.commit;

    // Load accounts.
    let [authority_info, payer_info, provider_info, var_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    authority_info.is_signer()?;
    payer_info.is_signer()?;
    // provider_info.is_signer()?;
    var_info.is_empty()?.is_writable()?;
    system_program.is_program(&system_program::ID)?;

    // Validate the end at slot.
    assert!(
        end_at > clock.slot,
        "End at must be greater than current slot"
    );

    // Create var account.
    create_program_account::<Var>(
        var_info,
        system_program,
        payer_info,
        &entropy_api::ID,
        &[VAR, &authority_info.key.to_bytes(), &id.to_le_bytes()],
    )?;
    let var = var_info.as_account_mut::<Var>(&entropy_api::ID)?;
    var.authority = *authority_info.key;
    var.id = id;
    var.provider = *provider_info.key;
    var.commit = commit;
    var.seed = [0; 32];
    var.slot_hash = [0; 32];
    var.value = [0; 32];
    var.is_auto = is_auto;
    var.samples = samples;
    var.start_at = clock.slot;
    var.end_at = end_at;

    Ok(())
}
