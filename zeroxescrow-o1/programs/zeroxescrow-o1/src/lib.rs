pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("CLTjXNTG8Ph9m3kiAD948sjXNhGoRMuRCA9DtJ28tBBU");

#[program]
pub mod zeroxescrow_o1 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn increment(ctx : Context<Increment>) -> Result<()> {
        increment::handler(ctx) ; 
        Ok(())
    }
}
