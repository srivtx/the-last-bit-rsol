use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Invalid timestamp in attribute")]
    InvalidTimestamp,
    #[msg("NFT is already staked")]
    AlreadyStaked,
    #[msg("NFT is not staked")]
    NotStaked,
    #[msg("Staking attributes are not initialized on this asset")]
    StakingNotInitialized,
    #[msg("Attributes plugin is missing on this asset")]
    AttributesNotInitialized,
    #[msg("Collection attributes plugin is missing")]
    CollectionAttributesMissing,
    #[msg("staked_count key missing on collection")]
    StakedCountMissing,
    #[msg("Integer overflow")]
    Overflow,
    #[msg("Integer underflow")]
    Underflow,
    #[msg("No rewards accrued yet")]
    NothingToClaim,
    #[msg("Reward vault has insufficient balance")]
    InsufficientVaultBalance,
    #[msg("Invalid reward mint")]
    InvalidRewardMint,
}
