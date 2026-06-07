use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

pub use instructions::*;
pub use state::*;

declare_id!("7xKp2mNqR8vYw3tZfHjL9sDc4eUb6aFg1nXi5oPr7QwE");

#[program]
pub mod zeroxamm_introspection {
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

    pub fn burn_for_swap(
        ctx: Context<BurnForSwap>,
        amount_in: u64,
        is_a_to_b: bool,
    ) -> Result<()> {
        burn_for_swap::handler(ctx, amount_in, is_a_to_b)
    }

    pub fn swap_payout(
        ctx: Context<SwapPayout>,
        amount_in: u64,
        min_amount_out: u64,
        is_a_to_b: bool,
    ) -> Result<()> {
        swap_payout::handler(ctx, amount_in, min_amount_out, is_a_to_b)
    }
}
