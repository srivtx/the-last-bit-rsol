use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    #[msg("Fee bps too high")]
    FeeTooHigh,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid payment mint")]
    InvalidPaymentMint,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Math overflow")]
    Overflow,
    #[msg("Math underflow")]
    Underflow,
}

