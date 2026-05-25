use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Amount out is zero")]
    ZeroAmountOut,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
}
