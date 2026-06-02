use anchor_lang::prelude::* ; 
use anchor_spl::token::{Mint , Token , TokenAccount} ; 


use crate::constants::{CONFIG_SEED , VAULT_SEED} ; 
use crate::state::StakeConfig ; 



#[derive(Accounts)]

pub struct  Initialize <'info> { 
    #[account(mut)]
    pub authority : Signer<'info> , 
    /// CHECK: This is a Metaplex Core collection account; we only record its pubkey in `stake_config`.
    /// Validation that an NFT belongs to this collection happens in later instructions (e.g. stake/unstake).
    pub collection : UncheckedAccount<'info> , 
    pub  reward_mint : Account<'info , Mint> , 

    #[account(
        init , 
        payer = authority , 
        space = 8 + StakeConfig::SIZE , 
        seeds = [CONFIG_SEED , collection.key().as_ref()], 
        bump ,
    )]
    pub stake_config : Account<'info , StakeConfig> , 

    #[account (
        init , 
        payer = authority , 
        token::mint = reward_mint , 
        token::authority = authority , 
        seeds=[VAULT_SEED , stake_config.key().as_ref()], 
        bump, 
    )]

    pub reward_vault : Account<'info , TokenAccount >, 

    pub token_program : Program<'info , Token> , 

    pub system_program : Program<'info , System > , 

}



pub fn handler ( ctx :Context<Initialize > ,reward_per_second : u64 ) -> Result<()> {
    let cfg = &mut ctx.accounts.stake_config ; 
    cfg.authority = ctx.accounts.authority.key() ; 
    cfg.collection = ctx.accounts.collection.key(); 
    cfg.reward_mint = ctx.accounts.reward_mint.key(); 
    cfg.reward_per_second = reward_per_second ; 
    cfg.bump = ctx.bumps.stake_config  ;
    cfg.vault_bump = ctx.bumps.reward_vault ;
    Ok(())
}