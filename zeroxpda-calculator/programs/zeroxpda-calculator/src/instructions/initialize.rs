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
