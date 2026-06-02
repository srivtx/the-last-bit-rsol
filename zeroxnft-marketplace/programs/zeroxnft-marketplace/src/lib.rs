use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

declare_id!("HUvNAtQRRPiSrXriX1avomCXSPw7B6BzkmBpbSPGGvJj");

#[program]
pub mod zeroxnft_marketplace {
    use super::*;

    pub fn initialize_marketplace(ctx: Context<InitializeMarketplace>, fee_bps: u16) -> Result<()> {
        initialize_marketplace::handler(ctx, fee_bps)
    }

    pub fn list(ctx: Context<List>, price: u64, payment_mint: Pubkey) -> Result<()> {
        list::handler(ctx, price, payment_mint)
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        delist::handler(ctx)
    }

    pub fn buy(ctx: Context<Buy>) -> Result<()> {
        buy::handler(ctx)
    }

    pub fn buy_with_token(ctx: Context<BuyWithToken>) -> Result<()> {
        buy_with_token::handler(ctx)
    }

    pub fn make_offer(ctx: Context<MakeOffer>, amount: u64) -> Result<()> {
        make_offer::handler(ctx, amount)
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        accept_offer::handler(ctx)
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        cancel_offer::handler(ctx)
    }
}
