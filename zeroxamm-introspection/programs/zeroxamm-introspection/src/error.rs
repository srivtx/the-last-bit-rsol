use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Amount out is zero")]
    ZeroAmountOut,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("No prior instruction in transaction")]
    MissingPriorInstruction,
    #[msg("Prior instruction is not from this program")]
    InvalidPriorProgram,
    #[msg("Prior instruction is not burn_for_swap")]
    InvalidPriorDiscriminator,
    #[msg("Prior instruction data does not match")]
    PriorInstructionDataMismatch,
    #[msg("Prior instruction accounts do not match")]
    PriorInstructionAccountsMismatch,
}
