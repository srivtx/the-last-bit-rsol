use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize {}

pub fn handler(ctx: Context<Initialize>, seed: u64) -> Result<()> {
    let (pda, bump) =
        Pubkey::find_program_address(&[b"escrow", &seed.to_le_bytes()], ctx.program_id);

        msg!("pda : {}" , pda) ; 
        msg!( " bump : {}" , bump ) ; 
    
    Ok(())
}


pub fn derive_with_maker(ctx  : Context<Initialize> , seed : u64 , maker: Pubkey)  -> Result<()>{
    let ( pda , bump) = Pubkey::find_program_address(&[b"escrow" , &seed.to_le_bytes() , maker.as_ref()], ctx.program_id) ; 
    msg!("PDA with maker: {}", pda);
    msg!("Bump: {}", bump);

    Ok(())
}