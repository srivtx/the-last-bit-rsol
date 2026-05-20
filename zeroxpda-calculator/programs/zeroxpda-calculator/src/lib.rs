pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Ek9j58LBsfzzeQ79uTnp2PJ2iobxtzoB89VybvgfJXPT");

#[program]
pub mod zeroxpda_calculator {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed : u64 ) -> Result<()> {
        initialize::handler(ctx , seed )
    }

    pub fn derive_with_maker( ctx: Context<Initialize> , seed : u64 , maker: Pubkey ) -> Result<()> { 
        initialize::derive_with_maker(ctx, seed, maker) 
    }
}
