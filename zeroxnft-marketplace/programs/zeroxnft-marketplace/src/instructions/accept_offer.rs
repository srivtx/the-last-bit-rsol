use anchor_lang::prelude::*;

use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::TransferV1CpiBuilder,
    types::UpdateAuthority,
    ID as CORE_PROGRAM_ID,
};

use crate::{
    constants::{MARKETPLACE_SEED, OFFER_SEED},
    error::MarketplaceError,
    state::{MarketplaceConfig, Offer},
    utils::split_fee,
};

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: offer PDA stores the buyer; used as lamport recipient and new asset owner.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,

    #[account(
        seeds = [MARKETPLACE_SEED, marketplace.authority.as_ref()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, MarketplaceConfig>,

    #[account(mut, constraint = treasury.key() == marketplace.treasury @ MarketplaceError::Unauthorized)]
    pub treasury: SystemAccount<'info>,

    /// CHECK: used for PDA seed derivation; validated by `offer.asset`
    pub asset_key: UncheckedAccount<'info>,

    #[account(
        mut,
        close = buyer,
        seeds = [OFFER_SEED, asset_key.key().as_ref(), buyer.key().as_ref()],
        bump = offer.bump,
        constraint = offer.buyer == buyer.key() @ MarketplaceError::Unauthorized,
        constraint = offer.asset == asset_key.key() @ MarketplaceError::Unauthorized,
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        constraint = asset.owner == maker.key() @ MarketplaceError::Unauthorized,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()),
    )]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(mut)]
    pub collection: Account<'info, BaseCollectionV1>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: validated by address constraint
    pub core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AcceptOffer>) -> Result<()> {
    let amount = ctx.accounts.offer.amount;
    require!(amount > 0, MarketplaceError::InvalidAmount);

    let (maker_amount, fee_amount) = split_fee(amount, ctx.accounts.marketplace.fee_bps)?;

    // Transfer asset to buyer using maker (current owner) authority.
    // Do this before moving lamports (which may be restricted across CPIs by the runtime).
    TransferV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.maker.to_account_info()))
        .new_owner(&ctx.accounts.buyer.to_account_info())
        .system_program(Some(&ctx.accounts.system_program.to_account_info()))
        .invoke()?;

    // Offer PDA is program-owned (has data), so pay out by moving lamports directly.
    // Do it in one balanced mutation to avoid runtime balance mismatches.
    let offer_info = ctx.accounts.offer.to_account_info();
    let maker_info = ctx.accounts.maker.to_account_info();
    let treasury_info = ctx.accounts.treasury.to_account_info();

    let total_out = maker_amount
        .checked_add(fee_amount)
        .ok_or(MarketplaceError::Overflow)?;

    let mut offer_lamports = offer_info.try_borrow_mut_lamports()?;
    require!(**offer_lamports >= total_out, MarketplaceError::InsufficientFunds);
    **offer_lamports = (**offer_lamports)
        .checked_sub(total_out)
        .ok_or(MarketplaceError::Underflow)?;
    drop(offer_lamports);

    let mut maker_lamports = maker_info.try_borrow_mut_lamports()?;
    **maker_lamports = (**maker_lamports)
        .checked_add(maker_amount)
        .ok_or(MarketplaceError::Overflow)?;
    drop(maker_lamports);

    if fee_amount > 0 {
        let mut treasury_lamports = treasury_info.try_borrow_mut_lamports()?;
        **treasury_lamports = (**treasury_lamports)
            .checked_add(fee_amount)
            .ok_or(MarketplaceError::Overflow)?;
    }

    Ok(())
}

