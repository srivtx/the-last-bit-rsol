use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::UpdatePluginV1CpiBuilder,
    types::{Attributes, Plugin, PluginType, UpdateAuthority},
    ID as CORE_PROGRAM_ID,
};

use crate::constants::VAULT_SEED;
use crate::state::StakeConfig;
use crate::utils::build_claim_attributes;

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(constraint = stake_config.authority == authority.key())]
    pub authority: Signer<'info>,
    pub owner: Signer<'info>,
    pub update_authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        has_one = collection,
        constraint = stake_config.reward_mint == reward_mint.key() @ crate::error::StakingError::InvalidRewardMint,
    )]
    pub stake_config: Account<'info, StakeConfig>,

    pub reward_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        mut,
        seeds = [VAULT_SEED, stake_config.key().as_ref()],
        bump = stake_config.vault_bump,
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_reward_ata.mint == reward_mint.key(),
        constraint = user_reward_ata.owner == owner.key(),
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        has_one = owner,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()),
    )]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(mut)]
    pub collection: Account<'info, BaseCollectionV1>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: validated by address constraint
    pub core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    let fetched = fetch_plugin::<BaseAssetV1, Attributes>(
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    )
    .map_err(|_| crate::error::StakingError::AttributesNotInitialized)?;
    let (_, asset_attrs, _) = fetched;
    let (list, elapsed) = build_claim_attributes(&asset_attrs.attribute_list, now)?;

    UpdatePluginV1CpiBuilder::new(&ctx.accounts.core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes { attribute_list: list }))
        .invoke()?;

    let amount = (elapsed as u64)
        .checked_mul(ctx.accounts.stake_config.reward_per_second)
        .ok_or(crate::error::StakingError::Overflow)?;

    require!(amount > 0, crate::error::StakingError::NothingToClaim);
    require!(
        ctx.accounts.reward_vault.amount >= amount,
        crate::error::StakingError::InsufficientVaultBalance
    );

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.reward_vault.to_account_info(),
                to: ctx.accounts.user_reward_ata.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}
