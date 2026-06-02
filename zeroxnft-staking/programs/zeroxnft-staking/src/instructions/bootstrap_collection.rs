use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseCollectionV1,
    instructions::AddCollectionPluginV1CpiBuilder,
    types::{Attribute, Attributes, Plugin, PluginAuthority},
    ID as CORE_PROGRAM_ID,
};

use crate::constants::STAKED_COUNT_KEY;

#[derive(Accounts)]
pub struct BootstrapCollection<'info> {
    pub update_authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, has_one = update_authority)]
    pub collection: Account<'info, BaseCollectionV1>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: validated by address constraint
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BootstrapCollection>) -> Result<()> {
    AddCollectionPluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes {
            attribute_list: vec![Attribute {
                key: STAKED_COUNT_KEY.to_string(),
                value: "0".to_string(),
            }],
        }))
        .init_authority(PluginAuthority::UpdateAuthority)
        .invoke()?;
    Ok(())
}
