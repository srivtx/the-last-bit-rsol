use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{
        AddPluginV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder,
    },
    types::{
        Attribute, Attributes, FreezeDelegate, Plugin, PluginAuthority, PluginType,
        UpdateAuthority,
    },
    ID as CORE_PROGRAM_ID,
};

use crate::state::StakeConfig;
use crate::utils::build_stake_start_attributes;

#[derive(Accounts)]
pub struct Stake<'info> {
    pub owner: Signer<'info>,
    pub update_authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(constraint = stake_config.collection == collection.key())]
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

pub fn handler(ctx: Context<Stake>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    match fetch_plugin::<BaseAssetV1, Attributes>(
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    ) {
        Ok((_, fetched, _)) => {
            let list = build_stake_start_attributes(&fetched.attribute_list, now)?;
            UpdatePluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
                .asset(&ctx.accounts.asset.to_account_info())
                .collection(Some(&ctx.accounts.collection.to_account_info()))
                .payer(&ctx.accounts.payer.to_account_info())
                .authority(Some(&ctx.accounts.update_authority.to_account_info()))
                .system_program(&ctx.accounts.system_program.to_account_info())
                .plugin(Plugin::Attributes(Attributes { attribute_list: list }))
                .invoke()?;
        }
        Err(_) => {
            AddPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
                .asset(&ctx.accounts.asset.to_account_info())
                .collection(Some(&ctx.accounts.collection.to_account_info()))
                .payer(&ctx.accounts.payer.to_account_info())
                .authority(Some(&ctx.accounts.update_authority.to_account_info()))
                .system_program(&ctx.accounts.system_program.to_account_info())
                .plugin(Plugin::Attributes(Attributes {
                    attribute_list: vec![
                        Attribute {
                            key: "staked".to_string(),
                            value: now.to_string(),
                        },
                        Attribute {
                            key: "staked_time".to_string(),
                            value: "0".to_string(),
                        },
                    ],
                }))
                .init_authority(PluginAuthority::UpdateAuthority)
                .invoke()?;
        }
    }

    AddPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.owner.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
        .init_authority(PluginAuthority::UpdateAuthority)
        .invoke()?;

    let collection_attrs = fetch_plugin::<BaseCollectionV1, Attributes>(
        &ctx.accounts.collection.to_account_info(),
        PluginType::Attributes,
    )
    .map_err(|_| crate::error::StakingError::CollectionAttributesMissing)?;
    let (_, collection_list, _) = collection_attrs;
    let list = crate::utils::build_collection_count_attributes(&collection_list.attribute_list, 1)?;

    UpdateCollectionPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes { attribute_list: list }))
        .invoke()?;

    Ok(())
}
