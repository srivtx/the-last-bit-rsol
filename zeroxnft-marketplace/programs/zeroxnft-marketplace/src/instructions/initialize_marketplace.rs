use anchor_lang::prelude::*;

use crate::constants::MARKETPLACE_SEED;
use crate::error::MarketplaceError;
use crate::state::MarketplaceConfig;

#[derive(Accounts)]
pub struct InitializeMarketplace<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: this can be any fee-collection wallet; in SPL mode it will be an ATA owner.
    pub treasury: UncheckedAccount<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + MarketplaceConfig::SIZE,
        seeds = [MARKETPLACE_SEED, authority.key().as_ref()],
        bump,
    )]
    pub marketplace: Account<'info, MarketplaceConfig>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeMarketplace>, fee_bps: u16) -> Result<()> {
    require!(fee_bps <= 10_000, MarketplaceError::FeeTooHigh);

    let m = &mut ctx.accounts.marketplace;
    m.authority = ctx.accounts.authority.key();
    m.treasury = ctx.accounts.treasury.key();
    m.fee_bps = fee_bps;
    m.bump = ctx.bumps.marketplace;
    Ok(())
}

