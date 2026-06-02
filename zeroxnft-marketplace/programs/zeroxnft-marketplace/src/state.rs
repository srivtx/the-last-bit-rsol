use anchor_lang::prelude::*;

#[account]
pub struct MarketplaceConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub fee_bps: u16,
    pub bump: u8,
}

impl MarketplaceConfig {
    pub const SIZE: usize = 32 + 32 + 2 + 1;
}

#[account]
pub struct Listing {
    pub maker: Pubkey,
    pub asset: Pubkey,
    pub price: u64,
    pub payment_mint: Pubkey, // Pubkey::default() = SOL
    pub created_at: i64,
    pub bump: u8,
}

impl Listing {
    pub const SIZE: usize = 32 + 32 + 8 + 32 + 8 + 1;
}

#[account]
pub struct Offer {
    pub buyer: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

impl Offer {
    pub const SIZE: usize = 32 + 32 + 8 + 1;
}

