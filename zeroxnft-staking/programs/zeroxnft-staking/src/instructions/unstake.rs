use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{
        RemovePluginV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder,
    },
    types::{
        Attributes, FreezeDelegate, Plugin, PluginType, UpdateAuthority,
    },
    ID as CORE_PROGRAM_ID,
};

use crate::state::StakeConfig;
use crate::utils::build_unstake_attributes;

#[derive(Accounts)]
pub struct Unstake<'info> {
    pub owner: Signer<'info>,
    pub update_authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(has_one = collection)]
    pub stake_config: Account<'info, StakeConfig>,

    #[account(
        mut,
        has_one = owner,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()),
    )]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(mut, has_one = update_authority)]
    pub collection: Account<'info, BaseCollectionV1>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: validated by address constraint
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Unstake>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    let fetched = fetch_plugin::<BaseAssetV1, Attributes>(
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    )
    .map_err(|_| crate::error::StakingError::AttributesNotInitialized)?;
    let (_, asset_attrs, _) = fetched;
    let list = build_unstake_attributes(&asset_attrs.attribute_list, now)?;

    UpdatePluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes { attribute_list: list }))
        .invoke()?;

    UpdatePluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
        .invoke()?;

    RemovePluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.owner.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin_type(PluginType::FreezeDelegate)
        .invoke()?;

    let collection_attrs = fetch_plugin::<BaseCollectionV1, Attributes>(
        &ctx.accounts.collection.to_account_info(),
        PluginType::Attributes,
    )
    .map_err(|_| crate::error::StakingError::CollectionAttributesMissing)?;
    let (_, collection_list, _) = collection_attrs;
    let list = crate::utils::build_collection_count_attributes(&collection_list.attribute_list, -1)?;

    UpdateCollectionPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes { attribute_list: list }))
        .invoke()?;

    Ok(())
}
