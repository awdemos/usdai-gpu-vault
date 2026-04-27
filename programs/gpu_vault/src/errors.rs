use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("LTV exceeds maximum 8000 bps (80%)")]
    LtvTooHigh,
    #[msg("Liquidation threshold must exceed max LTV")]
    InvalidLiquidationThreshold,
    #[msg("Utilization must be 0-10000 basis points")]
    InvalidUtilization,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Exceeds allowed LTV")]
    ExceedsLtv,
    #[msg("Collateral already liquidated")]
    AlreadyLiquidated,
    #[msg("Collateral is not active")]
    NotActive,
    #[msg("Collateral is not borrowing")]
    NotBorrowing,
    #[msg("Insufficient sCHIP balance")]
    InsufficientStake,
    #[msg("Oracle price stale")]
    StaleOracle,
    #[msg("Zero valuation")]
    ZeroValuation,
    #[msg("Position is not liquidatable")]
    NotLiquidatable,
    #[msg("Wrong oracle feed for collateral")]
    WrongOracleFeed,
    #[msg("Wrong token mint")]
    WrongMint,
    #[msg("Invalid treasury account")]
    InvalidTreasury,
    #[msg("Metadata strings too long")]
    MetadataTooLong,
    #[msg("Repay amount exceeds debt")]
    RepayTooMuch,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Collateral has outstanding debt")]
    OutstandingDebt,
    #[msg("Nothing to withdraw")]
    NothingToWithdraw,
    #[msg("Program is paused")]
    Paused,
}
