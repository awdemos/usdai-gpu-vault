use anchor_lang::prelude::*;

#[event]
pub struct GpuRegistered {
    pub collateral: Pubkey,
    pub nft_mint: Pubkey,
    pub owner: Pubkey,
    pub oracle_feed: Pubkey,
}

#[event]
pub struct UsdaiBorrowed {
    pub collateral: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub new_total: u64,
}

#[event]
pub struct UsdaiRepaid {
    pub collateral: Pubkey,
    pub amount: u64,
    pub remaining: u64,
}

#[event]
pub struct ChipStaked {
    pub staker: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ChipUnstaked {
    pub staker: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Liquidation {
    pub collateral: Pubkey,
    pub liquidator: Pubkey,
    pub debt_repaid: u64,
}

#[event]
pub struct OracleUpdated {
    pub collateral: Pubkey,
    pub price: u64,
    pub ts: i64,
}

#[event]
pub struct GpuWithdrawn {
    pub collateral: Pubkey,
    pub owner: Pubkey,
}

#[event]
pub struct AuthorityChanged {
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
}

#[event]
pub struct PauseSet {
    pub paused: bool,
    pub authority: Pubkey,
}

#[event]
pub struct TreasuryUpdated {
    pub old_treasury: Pubkey,
    pub new_treasury: Pubkey,
}
