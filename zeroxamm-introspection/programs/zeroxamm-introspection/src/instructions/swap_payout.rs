use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::error::ErrorCode;
use crate::state::PoolState;
use crate::utils::introspection::verify_prior_burn_for_swap;

pub fn get_amount_out(amount_in: u64, reserve_in: u64, reserve_out: u64) -> Option<u64> {
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return None;
    }
    let amount_in = amount_in as u128;
    let reserve_in = reserve_in as u128;
    let reserve_out = reserve_out as u128;
    let numerator = amount_in.checked_mul(reserve_out)?;
    let denominator = reserve_in.checked_add(amount_in)?;
    let out = numerator.checked_div(denominator)?;
    Some(out as u64)
}

#[derive(Accounts)]
pub struct SwapPayout<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    /// CHECK: PDA pool authority; no data, used only as signer for vault CPIs.
    #[account(
        seeds = [b"authority", pool_state.key().as_ref()],
        bump = pool_state.authority_bump,
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    /// CHECK: Instructions sysvar for introspection of the prior burn_for_swap ix.
    /// CHECK: Instructions sysvar; validated in introspection helper.
    #[account(address = sysvar::instructions::ID @ ErrorCode::MissingPriorInstruction)]
    pub instruction_sysvar: UncheckedAccount<'info>,
}

pub fn handler(
    ctx: Context<SwapPayout>,
    amount_in: u64,
    min_amount_out: u64,
    is_a_to_b: bool,
) -> Result<()> {
    require_gt!(amount_in, 0);

    verify_prior_burn_for_swap(
        &ctx.accounts.instruction_sysvar.to_account_info(),
        ctx.program_id,
        &ctx.accounts.user.key(),
        &ctx.accounts.pool_state.key(),
        &ctx.accounts.user_token_a.key(),
        &ctx.accounts.user_token_b.key(),
        &ctx.accounts.vault_a.key(),
        &ctx.accounts.vault_b.key(),
        &ctx.accounts.token_program.key(),
        amount_in,
        is_a_to_b,
    )?;

    let pool = &ctx.accounts.pool_state;

    require_keys_eq!(ctx.accounts.vault_a.key(), pool.token_vault_a);
    require_keys_eq!(ctx.accounts.vault_b.key(), pool.token_vault_b);
    require_keys_eq!(
        ctx.accounts.pool_authority.key(),
        pool.pool_authority
    );

    if is_a_to_b {
        require_keys_eq!(ctx.accounts.user_token_a.mint, pool.token_mint_a);
        require_keys_eq!(ctx.accounts.user_token_b.mint, pool.token_mint_b);
    } else {
        require_keys_eq!(ctx.accounts.user_token_a.mint, pool.token_mint_b);
        require_keys_eq!(ctx.accounts.user_token_b.mint, pool.token_mint_a);
    }

    let reserve_in = if is_a_to_b {
        pool.reserve_a
    } else {
        pool.reserve_b
    };
    let reserve_out = if is_a_to_b {
        pool.reserve_b
    } else {
        pool.reserve_a
    };

    let amount_out =
        get_amount_out(amount_in, reserve_in, reserve_out).ok_or(ErrorCode::ZeroAmountOut)?;

    require_gte!(amount_out, min_amount_out, ErrorCode::SlippageExceeded);

    let (user_token_out, vault_out) = if is_a_to_b {
        (&ctx.accounts.user_token_b, &ctx.accounts.vault_b)
    } else {
        (&ctx.accounts.user_token_a, &ctx.accounts.vault_a)
    };

    let seeds = &[
        b"authority",
        ctx.accounts.pool_state.to_account_info().key.as_ref(),
        &[pool.authority_bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_ctx_out = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: vault_out.to_account_info(),
            to: user_token_out.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        },
        signer,
    );
    token::transfer(cpi_ctx_out, amount_out)?;

    let pool = &mut ctx.accounts.pool_state;
    if is_a_to_b {
        pool.reserve_a = pool.reserve_a.checked_add(amount_in).unwrap();
        pool.reserve_b = pool.reserve_b.checked_sub(amount_out).unwrap();
    } else {
        pool.reserve_b = pool.reserve_b.checked_add(amount_in).unwrap();
        pool.reserve_a = pool.reserve_a.checked_sub(amount_out).unwrap();
    }

    msg!("Swap payout: {} in -> {} out", amount_in, amount_out);
    Ok(())
}
