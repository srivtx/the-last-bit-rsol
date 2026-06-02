use anchor_lang::prelude::*;

use crate::{
    constants::OFFER_SEED,
    error::MarketplaceError,
    state::Offer,
};

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: used for PDA seed derivation; validated by address constraint below
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        close = buyer,
        seeds = [OFFER_SEED, asset.key().as_ref(), buyer.key().as_ref()],
        bump = offer.bump,
        constraint = offer.buyer == buyer.key() @ MarketplaceError::Unauthorized,
        constraint = offer.asset == asset.key() @ MarketplaceError::Unauthorized,
    )]
    pub offer: Account<'info, Offer>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CancelOffer>) -> Result<()> {
    let amount = ctx.accounts.offer.amount;

    // Offer PDA is program-owned (has data), so we cannot use system_program::transfer.
    // Move lamports directly instead.
    let offer_info = ctx.accounts.offer.to_account_info();
    let buyer_info = ctx.accounts.buyer.to_account_info();

    let mut offer_lamports = offer_info.try_borrow_mut_lamports()?;
    require!(**offer_lamports >= amount, MarketplaceError::InsufficientFunds);
    **offer_lamports = (**offer_lamports)
        .checked_sub(amount)
        .ok_or(MarketplaceError::Underflow)?;
    drop(offer_lamports);

    let mut buyer_lamports = buyer_info.try_borrow_mut_lamports()?;
    **buyer_lamports = (**buyer_lamports)
        .checked_add(amount)
        .ok_or(MarketplaceError::Overflow)?;

    Ok(())
}

