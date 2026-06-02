use anchor_lang::prelude::*;

use crate::error::MarketplaceError;

pub fn split_fee(amount: u64, fee_bps: u16) -> Result<(u64, u64)> {
    // fee_bps is in basis points (1/100 of a percent)
    let fee = (amount as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(MarketplaceError::Overflow)?
        .checked_div(10_000)
        .ok_or(MarketplaceError::Underflow)? as u64;
    let maker_amount = amount.checked_sub(fee).ok_or(MarketplaceError::Underflow)?;
    Ok((maker_amount, fee))
}

