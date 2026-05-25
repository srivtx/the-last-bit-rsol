use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BwYSdX5KxrcJzxcBhJ3zSveeJ1Cae9AgN8BDLHvY6E3v");

#[program]
pub mod zeroxamm_one {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn initialize_pool(ctx: Context<InitializePool>, pool_id: u16) -> Result<()> {
        initialize_pool::handler(ctx, pool_id)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        add_liquidity::handler(ctx, amount_a, amount_b)
    }

    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
        is_a_to_b: bool,
    ) -> Result<()> {
        swap::handler(ctx, amount_in, min_amount_out, is_a_to_b)
    }
}
