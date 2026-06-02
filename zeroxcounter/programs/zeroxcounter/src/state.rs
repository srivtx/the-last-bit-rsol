use anchor_lang::prelude::*  ; 



#[account]
#[derive(InitSpace)]

pub struct Counter { 
    pub authority : Pubkey , 
    pub count : u64 , 
}


