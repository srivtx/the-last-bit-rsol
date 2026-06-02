use anchor_lang::prelude::*;

use crate::{
    constants::OFFER_SEED,
    error::MarketplaceError,
    state::Offer,
};

#[derive(Accounts)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: recorded in Offer PDA
    pub asset: UncheckedAccount<'info>,

    #[account(
        init,
        payer = buyer,
        space = 8 + Offer::SIZE,
        seeds = [OFFER_SEED, asset.key().as_ref(), buyer.key().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<MakeOffer>, amount: u64) -> Result<()> {
    require!(amount > 0, MarketplaceError::InvalidAmount);

    let offer = &mut ctx.accounts.offer;
    offer.buyer = ctx.accounts.buyer.key();
    offer.asset = ctx.accounts.asset.key();
    offer.amount = amount;
    offer.bump = ctx.bumps.offer;

    // Escrow SOL into the Offer PDA.
    require!(
        ctx.accounts.buyer.lamports() >= amount,
        MarketplaceError::InsufficientFunds
    );
    anchor_lang::solana_program::program::invoke(
        &anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.offer.key(),
            amount,
        ),
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.offer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    Ok(())
}

