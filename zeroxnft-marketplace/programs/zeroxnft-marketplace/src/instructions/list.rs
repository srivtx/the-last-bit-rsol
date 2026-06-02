use anchor_lang::prelude::*;

use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::AddPluginV1CpiBuilder,
    types::{Plugin, PluginAuthority, TransferDelegate, UpdateAuthority},
    ID as CORE_PROGRAM_ID,
};

use crate::{
    constants::{LISTING_SEED, MARKETPLACE_SEED},
    error::MarketplaceError,
    state::{Listing, MarketplaceConfig},
};

#[derive(Accounts)]
pub struct List<'info> {
    pub maker: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [MARKETPLACE_SEED, marketplace.authority.as_ref()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, MarketplaceConfig>,

    #[account(
        init,
        payer = payer,
        space = 8 + Listing::SIZE,
        seeds = [LISTING_SEED, asset.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>,

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

pub fn handler(ctx: Context<List>, price: u64, payment_mint: Pubkey) -> Result<()> {
    require!(price > 0, MarketplaceError::InvalidAmount);

    let now = Clock::get()?.unix_timestamp;

    let listing = &mut ctx.accounts.listing;
    listing.maker = ctx.accounts.maker.key();
    listing.asset = ctx.accounts.asset.key();
    listing.price = price;
    listing.payment_mint = payment_mint;
    listing.created_at = now;
    listing.bump = ctx.bumps.listing;

    // Owner-managed plugin: maker signs to add it. Delegate authority is the listing PDA.
    AddPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.maker.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::TransferDelegate(TransferDelegate {}))
        .init_authority(PluginAuthority::Address {
            address: ctx.accounts.listing.key(),
        })
        .invoke()?;

    Ok(())
}

