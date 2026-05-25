use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::PoolState;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    #[account(
        mut,
        constraint = user_token_a.mint == pool_state.token_mint_a,
        constraint = user_token_a.owner == payer.key(),
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_b.mint == pool_state.token_mint_b,
        constraint = user_token_b.owner == payer.key(),
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_vault_a.key() == pool_state.token_vault_a,
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_vault_b.key() == pool_state.token_vault_b,
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
    require_gt!(amount_a, 0);
    require_gt!(amount_b, 0);

    let cpi_ctx_a = CpiContext::new(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.user_token_a.to_account_info(),
            to: ctx.accounts.token_vault_a.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        },
    );
    token::transfer(cpi_ctx_a, amount_a)?;

    let cpi_ctx_b = CpiContext::new(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.user_token_b.to_account_info(),
            to: ctx.accounts.token_vault_b.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        },
    );
    token::transfer(cpi_ctx_b, amount_b)?;

    let pool = &mut ctx.accounts.pool_state;
    pool.reserve_a = pool.reserve_a.checked_add(amount_a).unwrap();
    pool.reserve_b = pool.reserve_b.checked_add(amount_b).unwrap();

    msg!("Liquidity added: {} A, {} B", amount_a, amount_b);
    Ok(())
}
