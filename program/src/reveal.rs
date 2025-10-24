use entropy_api::prelude::*;
use steel::*;

pub fn process_reveal(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let clock = Clock::get()?;
    let args = Reveal::try_from_bytes(data)?;
    let seed = args.seed;

    // Load accounts.
    let [signer_info, commitment_info, var_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let commitment = commitment_info
        .as_account_mut::<Commitment>(&entropy_api::ID)?
        .assert_mut(|c| c.authority == *signer_info.key)?
        .assert_mut(|c| c.var == *var_info.key)?;
    let var = var_info
        .as_account_mut::<Var>(&entropy_api::ID)?
        .assert_mut(|v| clock.slot < v.last_reveal_at)?;
    system_program.is_program(&system_program::ID)?;

    // Validate the seed.
    if !commitment.is_valid(seed) {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Update the digest.
    for (i, byte) in seed.iter().enumerate() {
        var.digest[i] ^= byte;
    }

    // Update the var.
    var.reveal_count += 1;

    Ok(())
}
