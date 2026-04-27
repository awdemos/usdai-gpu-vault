use anchor_lang::prelude::*;

#[account]
pub struct VaultConfig {
    pub authority: Pubkey,
    pub usdai_mint: Pubkey,
    pub chip_mint: Pubkey,
    pub s_chip_mint: Pubkey,
    pub usd_ai_lend_program: Pubkey,
    pub usd_ai_stake_program: Pubkey,
    pub treasury: Pubkey,
    pub max_ltv_bps: u16,
    pub liquidation_ltv_bps: u16,
    pub protocol_fee_bps: u16,
    pub min_oracle_staleness: i64,
    pub paused: bool,
    pub bump: u8,
}

impl VaultConfig {
    pub const LEN: usize = 8 + (32 * 7) + 2 + 2 + 2 + 8 + 1 + 1;
}
