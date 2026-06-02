pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("44pReZEfppDqb1nspgSs7wizn8DkqTdEMKKijj6iCBaV");

#[program]
pub mod zeroxcounter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn increment(ctx : Context<Increment>) -> Result<()> {
        increment::handler(ctx) 
    }
}
