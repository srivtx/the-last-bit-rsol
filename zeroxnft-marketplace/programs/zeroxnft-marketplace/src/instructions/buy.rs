use anchor_lang::prelude::*;

use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::TransferV1CpiBuilder,
    types::UpdateAuthority,
    ID as CORE_PROGRAM_ID,
};

use crate::{
    constants::{LISTING_SEED, MARKETPLACE_SEED},
    error::MarketplaceError,
    state::{Listing, MarketplaceConfig},
    utils::split_fee,
};

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [MARKETPLACE_SEED, marketplace.authority.as_ref()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, MarketplaceConfig>,

    /// Maker receives SOL proceeds.
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    /// Treasury receives SOL fees.
    #[account(mut, constraint = treasury.key() == marketplace.treasury @ MarketplaceError::Unauthorized)]
    pub treasury: SystemAccount<'info>,

    #[account(
        mut,
        close = maker,
        seeds = [LISTING_SEED, asset.key().as_ref()],
        bump = listing.bump,
        constraint = listing.maker == maker.key() @ MarketplaceError::Unauthorized,
    )]
    pub listing: Account<'info, Listing>,

    #[account(
        mut,
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

pub fn handler(ctx: Context<Buy>) -> Result<()> {
    require!(
        ctx.accounts.listing.payment_mint == Pubkey::default(),
        MarketplaceError::InvalidPaymentMint
    );

    let price = ctx.accounts.listing.price;
    let (maker_amount, fee_amount) = split_fee(price, ctx.accounts.marketplace.fee_bps)?;

    require!(
        ctx.accounts.buyer.lamports() >= price,
        MarketplaceError::InsufficientFunds
    );

    // Buyer pays maker.
    anchor_lang::solana_program::program::invoke(
        &anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.maker.key(),
            maker_amount,
        ),
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.maker.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Buyer pays treasury fee.
    if fee_amount > 0 {
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.buyer.key(),
                &ctx.accounts.treasury.key(),
                fee_amount,
            ),
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.treasury.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }

    // Transfer asset to buyer, using listing PDA as delegate authority.
    let asset_key = ctx.accounts.asset.key();
    let listing_seeds: &[&[u8]] = &[LISTING_SEED, asset_key.as_ref(), &[ctx.accounts.listing.bump]];

    // NOTE: mpl-core CPI builder supports signed invocation through the underlying CPI.
    TransferV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.listing.to_account_info()))
        .new_owner(&ctx.accounts.buyer.to_account_info())
        .system_program(Some(&ctx.accounts.system_program.to_account_info()))
        .invoke_signed(&[listing_seeds])?;

    Ok(())
}

