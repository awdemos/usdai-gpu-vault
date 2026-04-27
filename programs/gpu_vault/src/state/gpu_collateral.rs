use anchor_lang::prelude::*;

#[account]
pub struct GpuCollateral {
    pub owner: Pubkey,
    pub gpu_nft_mint: Pubkey,
    pub valuation_usd: u64,
    pub borrowed_usdai: u64,
    pub model: GpuModel,
    pub status: GpuStatus,
    pub oracle_feed: Pubkey,
    pub last_valuation_ts: i64,
    pub bump: u8,
}

impl GpuCollateral {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 1 + 1 + 32 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GpuModel {
    A100,
    A100Cluster8,
    H100,
    H200,
    Unknown,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GpuStatus {
    Active,
    Borrowing,
    Liquidated,
    Withdrawn,
}
