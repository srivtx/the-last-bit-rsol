use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::PoolState;

#[derive(Accounts)]
pub struct BurnForSwap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<BurnForSwap>, amount_in: u64, is_a_to_b: bool) -> Result<()> {
    require_gt!(amount_in, 0);

    let pool = &ctx.accounts.pool_state;

    require_keys_eq!(ctx.accounts.vault_a.key(), pool.token_vault_a);
    require_keys_eq!(ctx.accounts.vault_b.key(), pool.token_vault_b);

    if is_a_to_b {
        require_keys_eq!(ctx.accounts.user_token_a.mint, pool.token_mint_a);
        require_keys_eq!(ctx.accounts.user_token_b.mint, pool.token_mint_b);
    } else {
        require_keys_eq!(ctx.accounts.user_token_a.mint, pool.token_mint_b);
        require_keys_eq!(ctx.accounts.user_token_b.mint, pool.token_mint_a);
    }

    let (user_token_in, vault_in) = if is_a_to_b {
        (&ctx.accounts.user_token_a, &ctx.accounts.vault_a)
    } else {
        (&ctx.accounts.user_token_b, &ctx.accounts.vault_b)
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: user_token_in.to_account_info(),
            to: vault_in.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, amount_in)?;

    msg!("Burn for swap: {} in (a_to_b={})", amount_in, is_a_to_b);
    Ok(())
}
