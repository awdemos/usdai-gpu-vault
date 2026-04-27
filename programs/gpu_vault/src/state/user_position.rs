use anchor_lang::prelude::*;

/// Aggregate position tracking per user (optional — for indexing / UI convenience).
#[account]
pub struct UserPosition {
    pub owner: Pubkey,
    pub total_collateral_value: u64,
    pub total_borrowed: u64,
    pub total_staked_chip: u64,
    pub bump: u8,
}

impl UserPosition {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 1;
}
