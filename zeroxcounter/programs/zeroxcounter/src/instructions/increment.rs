use anchor_lang::prelude::* ; 

use crate::state::Counter ; 

#[derive(Accounts)]
pub struct Increment<'info>{
    pub authority : Signer<'info> , 
    #[account(
        mut , 
        has_one  = authority 
    )] 

    pub counter : Account<'info , Counter> , 
}


pub fn handler(ctx : Context<Increment>) -> Result<()> { 
    let counter = &mut ctx.accounts.counter ; 
    counter.count += 1 ; 
    Ok(())
}


