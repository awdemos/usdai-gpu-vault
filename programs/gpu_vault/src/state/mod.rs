pub mod vault_config;
pub mod gpu_collateral;
pub mod user_position;

pub use vault_config::VaultConfig;
pub use gpu_collateral::{GpuCollateral, GpuModel, GpuStatus};
pub use user_position::UserPosition;

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;

    /// Account discriminator size added by Anchor's `#[account]` macro.
    const DISCRIMINATOR: usize = 8;

    #[test]
    fn vault_config_len_matches_layout() {
        // authority: Pubkey (32)
        // usdai_mint: Pubkey (32)
        // chip_mint: Pubkey (32)
        // s_chip_mint: Pubkey (32)
        // usd_ai_lend_program: Pubkey (32)
        // usd_ai_stake_program: Pubkey (32)
        // treasury: Pubkey (32)
        // max_ltv_bps: u16 (2)
        // liquidation_ltv_bps: u16 (2)
        // protocol_fee_bps: u16 (2)
        // min_oracle_staleness: i64 (8)
        // paused: bool (1)
        // bump: u8 (1)
        let expected = DISCRIMINATOR
            + (32 * 7)
            + 2
            + 2
            + 2
            + 8
            + 1
            + 1;
        assert_eq!(VaultConfig::LEN, expected);
    }

    #[test]
    fn gpu_collateral_len_matches_layout() {
        // owner: Pubkey (32)
        // gpu_nft_mint: Pubkey (32)
        // valuation_usd: u64 (8)
        // borrowed_usdai: u64 (8)
        // model: GpuModel enum (1)
        // status: GpuStatus enum (1)
        // oracle_feed: Pubkey (32)
        // last_valuation_ts: i64 (8)
        // bump: u8 (1)
        let expected = DISCRIMINATOR + 32 + 32 + 8 + 8 + 1 + 1 + 32 + 8 + 1;
        assert_eq!(GpuCollateral::LEN, expected);
    }

    #[test]
    fn user_position_len_matches_layout() {
        // owner: Pubkey (32)
        // total_collateral_value: u64 (8)
        // total_borrowed: u64 (8)
        // total_staked_chip: u64 (8)
        // bump: u8 (1)
        let expected = DISCRIMINATOR + 32 + 8 + 8 + 8 + 1;
        assert_eq!(UserPosition::LEN, expected);
    }

    #[test]
    fn gpu_model_variants_are_unique() {
        use GpuModel::*;
        let variants = vec![A100, A100Cluster8, H100, H200, Unknown];
        // Anchor enums serialize as u8 indices starting at 0
        for (i, variant) in variants.iter().enumerate() {
            let mut buf = Vec::new();
            variant.serialize(&mut buf).unwrap();
            assert_eq!(buf[0], i as u8, "variant at index {} serialized incorrectly", i);
        }
    }

    #[test]
    fn gpu_status_variants_are_unique() {
        use GpuStatus::*;
        let variants = vec![Active, Borrowing, Liquidated, Withdrawn];
        for (i, variant) in variants.iter().enumerate() {
            let mut buf = Vec::new();
            variant.serialize(&mut buf).unwrap();
            assert_eq!(buf[0], i as u8, "variant at index {} serialized incorrectly", i);
        }
    }
}
