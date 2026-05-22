use anchor_lang::{ prelude::*} ; 
use crate::state::Counter ; 


#[derive(Accounts)] 

pub struct Increment<'info > {
    #[account(mut)] 
    pub counter : Account <'info , Counter > , 
}


pub fn handler(ctx:Context<Increment> ) -> Result<()>  { 
    ctx.accounts.counter.count += 1 ; 
    Ok(())
}

