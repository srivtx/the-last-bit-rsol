pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

declare_id!("B9ZYPDfh1uUuZTYrbJeigx8oumftp2oiq2JsL2FEdJF8");

#[program]
pub mod zeroxnft_staking {
    use super::*;

    pub fn bootstrap_collection(ctx: Context<BootstrapCollection>) -> Result<()> {
        bootstrap_collection::handler(ctx)
    }

    pub fn initialize(ctx: Context<Initialize>, reward_per_second: u64) -> Result<()> {
        initialize::handler(ctx, reward_per_second)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        stake::handler(ctx)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        claim_rewards::handler(ctx)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        unstake::handler(ctx)
    }
}
