use anchor_lang::prelude::*;

use anchor_spl::token_interface::{
    self, Mint, TokenAccount, TokenInterface, TransferChecked,
};
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
pub struct BuyWithToken<'info> {
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [MARKETPLACE_SEED, marketplace.authority.as_ref()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, MarketplaceConfig>,

    /// CHECK: for SPL payments this is the ATA owner; verified by `treasury_ata.owner`.
    pub treasury: UncheckedAccount<'info>,

    #[account(
        mut,
        close = maker,
        seeds = [LISTING_SEED, asset.key().as_ref()],
        bump = listing.bump,
        constraint = listing.maker == maker.key() @ MarketplaceError::Unauthorized,
    )]
    pub listing: Account<'info, Listing>,

    /// Maker receives SPL proceeds (and receives rent when listing closes).
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(mut)]
    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub maker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()),
    )]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(mut)]
    pub collection: Account<'info, BaseCollectionV1>,

    pub token_program: Interface<'info, TokenInterface>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: validated by address constraint
    pub core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyWithToken>) -> Result<()> {
    require!(
        ctx.accounts.listing.payment_mint == ctx.accounts.payment_mint.key(),
        MarketplaceError::InvalidPaymentMint
    );

    require!(
        ctx.accounts.buyer_ata.owner == ctx.accounts.buyer.key(),
        MarketplaceError::Unauthorized
    );
    require!(
        ctx.accounts.maker_ata.owner == ctx.accounts.maker.key(),
        MarketplaceError::Unauthorized
    );
    require!(
        ctx.accounts.treasury_ata.owner == ctx.accounts.treasury.key(),
        MarketplaceError::Unauthorized
    );
    require!(
        ctx.accounts.buyer_ata.mint == ctx.accounts.payment_mint.key(),
        MarketplaceError::InvalidPaymentMint
    );
    require!(
        ctx.accounts.maker_ata.mint == ctx.accounts.payment_mint.key(),
        MarketplaceError::InvalidPaymentMint
    );
    require!(
        ctx.accounts.treasury_ata.mint == ctx.accounts.payment_mint.key(),
        MarketplaceError::InvalidPaymentMint
    );

    let price = ctx.accounts.listing.price;
    let (maker_amount, fee_amount) = split_fee(price, ctx.accounts.marketplace.fee_bps)?;
    let decimals = ctx.accounts.payment_mint.decimals;

    // Buyer pays maker.
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.payment_mint.to_account_info(),
                from: ctx.accounts.buyer_ata.to_account_info(),
                to: ctx.accounts.maker_ata.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        maker_amount,
        decimals,
    )?;

    // Buyer pays treasury fee.
    if fee_amount > 0 {
        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    mint: ctx.accounts.payment_mint.to_account_info(),
                    from: ctx.accounts.buyer_ata.to_account_info(),
                    to: ctx.accounts.treasury_ata.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            fee_amount,
            decimals,
        )?;
    }

    let asset_key = ctx.accounts.asset.key();
    let listing_seeds: &[&[u8]] = &[LISTING_SEED, asset_key.as_ref(), &[ctx.accounts.listing.bump]];

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

