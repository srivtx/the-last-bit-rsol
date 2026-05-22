use anchor_lang::prelude::*;

use crate::state::Counter ; 


#[derive(Accounts)]

pub struct Initialize<'info> {
    #[account( 
        init , 
        payer = user , 
        space = 8 + 8 

    )]
    
    pub counter : Account<'info , Counter> , 

    #[account(mut)]
    pub user : Signer < 'info > , 

    pub system_program : Program<'info, System > , 
}


pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.counter.count = 0 ; 
    Ok(()) 
}


