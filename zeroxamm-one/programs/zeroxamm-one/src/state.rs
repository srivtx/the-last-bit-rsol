use anchor_lang::prelude::* ; 


#[account]
pub struct PoolState {
    pub bump: u8,
    pub authority_bump: u8,
    pub pool_id: u16,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub pool_authority: Pubkey,
    pub reserve_a: u64,
    pub reserve_b: u64,
}

impl PoolState {
    pub const LEN: usize = 8 + 1 + 1 + 2 + 32 + 32 + 32 + 32 + 32 + 8 + 8;
}