use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::PoolState;

#[derive(Accounts)]
#[instruction(pool_id: u16)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        space = PoolState::LEN,
        seeds = [
            b"pool",
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
            &pool_id.to_le_bytes(),
        ],
        bump
    )]
    pub pool_state: Account<'info, PoolState>,

    #[account(
        seeds = [b"authority", pool_state.key().as_ref()],
        bump
    )]
    /// CHECK: PDA pool authority; no data, used only as signer for vault CPIs.
    pub pool_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        token::mint = token_mint_a,
        token::authority = pool_authority,
        seeds = [b"vault_a", pool_state.key().as_ref()],
        bump
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        token::mint = token_mint_b,
        token::authority = pool_authority,
        seeds = [b"vault_b", pool_state.key().as_ref()],
        bump
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializePool>, pool_id: u16) -> Result<()> {
    let pool = &mut ctx.accounts.pool_state;

    pool.bump = ctx.bumps.pool_state;
    pool.authority_bump = ctx.bumps.pool_authority;
    pool.pool_id = pool_id;
    pool.token_mint_a = ctx.accounts.token_mint_a.key();
    pool.token_mint_b = ctx.accounts.token_mint_b.key();
    pool.token_vault_a = ctx.accounts.token_vault_a.key();
    pool.token_vault_b = ctx.accounts.token_vault_b.key();
    pool.pool_authority = ctx.accounts.pool_authority.key();
    pool.reserve_a = 0;
    pool.reserve_b = 0;

    msg!("Pool initialized: id={}, reserves=0/0", pool_id);
    Ok(())
}
