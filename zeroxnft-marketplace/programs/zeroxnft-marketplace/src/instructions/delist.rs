use anchor_lang::prelude::*;

use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::RemovePluginV1CpiBuilder,
    types::{PluginType, UpdateAuthority},
    ID as CORE_PROGRAM_ID,
};

use crate::{
    constants::LISTING_SEED,
    error::MarketplaceError,
    state::Listing,
};

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

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

pub fn handler(ctx: Context<Delist>) -> Result<()> {
    RemovePluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.maker.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin_type(PluginType::TransferDelegate)
        .invoke()?;
    Ok(())
}

