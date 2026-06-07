use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::{
    load_current_index_checked, load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
};
use anchor_lang::Discriminator;

use crate::error::ErrorCode;
use crate::instruction;

/// Account indices in `burn_for_swap` (must match `BurnForSwap` account order).
const BURN_IX_USER: usize = 0;
const BURN_IX_POOL: usize = 1;
const BURN_IX_USER_A: usize = 2;
const BURN_IX_USER_B: usize = 3;
const BURN_IX_VAULT_A: usize = 4;
const BURN_IX_VAULT_B: usize = 5;
const BURN_IX_TOKEN_PROGRAM: usize = 6;

pub fn verify_prior_burn_for_swap(
    instruction_sysvar: &AccountInfo<'_>,
    program_id: &Pubkey,
    expected_user: &Pubkey,
    expected_pool: &Pubkey,
    expected_user_a: &Pubkey,
    expected_user_b: &Pubkey,
    expected_vault_a: &Pubkey,
    expected_vault_b: &Pubkey,
    expected_token_program: &Pubkey,
    amount_in: u64,
    is_a_to_b: bool,
) -> Result<()> {
    require_keys_eq!(
        *instruction_sysvar.key,
        INSTRUCTIONS_SYSVAR_ID,
        ErrorCode::MissingPriorInstruction
    );

    let current_index = load_current_index_checked(instruction_sysvar)
        .map_err(|_| error!(ErrorCode::MissingPriorInstruction))?;

    require_gt!(current_index, 0, ErrorCode::MissingPriorInstruction);

    let prior_ix = load_instruction_at_checked(
        (current_index - 1) as usize,
        instruction_sysvar,
    )
    .map_err(|_| error!(ErrorCode::MissingPriorInstruction))?;

    require_keys_eq!(prior_ix.program_id, *program_id, ErrorCode::InvalidPriorProgram);

    let data = prior_ix.data.as_slice();
    require!(
        data.len() >= 8 + 8 + 1,
        ErrorCode::InvalidPriorDiscriminator
    );

    let disc = &data[..8];
    require!(
        disc == instruction::BurnForSwap::DISCRIMINATOR,
        ErrorCode::InvalidPriorDiscriminator
    );

    let parsed_amount = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let parsed_is_a_to_b = data[16] != 0;

    require!(
        parsed_amount == amount_in && parsed_is_a_to_b == is_a_to_b,
        ErrorCode::PriorInstructionDataMismatch
    );

    let accounts = &prior_ix.accounts;
    require!(
        accounts.len() > BURN_IX_TOKEN_PROGRAM,
        ErrorCode::PriorInstructionAccountsMismatch
    );

    require_keys_eq!(
        accounts[BURN_IX_USER].pubkey,
        *expected_user,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_POOL].pubkey,
        *expected_pool,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_USER_A].pubkey,
        *expected_user_a,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_USER_B].pubkey,
        *expected_user_b,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_VAULT_A].pubkey,
        *expected_vault_a,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_VAULT_B].pubkey,
        *expected_vault_b,
        ErrorCode::PriorInstructionAccountsMismatch
    );
    require_keys_eq!(
        accounts[BURN_IX_TOKEN_PROGRAM].pubkey,
        *expected_token_program,
        ErrorCode::PriorInstructionAccountsMismatch
    );

    Ok(())
}
