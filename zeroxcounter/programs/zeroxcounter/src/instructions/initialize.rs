use anchor_lang::prelude::*  ; 

use crate::state::Counter;


#[derive(Accounts)]

pub struct Initialize<'info> { 

    #[account(mut)]
    pub authority : Signer<'info> , 
    

    #[account( 
        init , 
        payer = authority , 
        space = 8 + Counter::INIT_SPACE
    )]
    pub counter : Account<'info , Counter > , 
    
    pub system_program : Program<'info , System > 
    
}

pub fn handler (ctx : Context<Initialize>) -> Result<()> {
    let counter = &mut ctx.accounts.counter ; 
    counter.authority = ctx.accounts.authority.key() ; 
    counter.count = 0  ; 
    Ok(())  
}
