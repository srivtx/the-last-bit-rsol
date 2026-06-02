use anchor_lang::prelude::*;

#[account]
pub struct StakeConfig {
    pub authority: Pubkey,
    pub collection: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_per_second: u64,
    pub bump: u8,
    pub vault_bump: u8,
}

impl StakeConfig {
    pub const SIZE: usize = 32 + 32 + 32 + 8 + 1 + 1;
}
