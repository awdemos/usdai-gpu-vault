use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::VaultError;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = payer,
        space = VaultConfig::LEN,
        seeds = [b"vault_config"],
        bump
    )]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Existing USDai mint (verified off-chain or via token registry)
    pub usdai_mint: AccountInfo<'info>,

    /// CHECK: Existing CHIP mint
    pub chip_mint: AccountInfo<'info>,

    /// CHECK: Existing sCHIP mint
    pub s_chip_mint: AccountInfo<'info>,

    /// CHECK: Existing USD.AI lending program ID
    pub usd_ai_lend_program: AccountInfo<'info>,

    /// CHECK: Existing USD.AI staking program ID
    pub usd_ai_stake_program: AccountInfo<'info>,

    /// CHECK: Treasury wallet for fee collection (existing wallet, not ATA)
    pub treasury: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeVault>,
    max_ltv_bps: u16,
    liquidation_ltv_bps: u16,
) -> Result<()> {
    require!(max_ltv_bps <= 8_000, VaultError::LtvTooHigh);
    require!(
        liquidation_ltv_bps > max_ltv_bps,
        VaultError::InvalidLiquidationThreshold
    );

    let config = &mut ctx.accounts.vault_config;
    config.authority = ctx.accounts.payer.key();
    config.usdai_mint = ctx.accounts.usdai_mint.key();
    config.chip_mint = ctx.accounts.chip_mint.key();
    config.s_chip_mint = ctx.accounts.s_chip_mint.key();
    config.usd_ai_lend_program = ctx.accounts.usd_ai_lend_program.key();
    config.usd_ai_stake_program = ctx.accounts.usd_ai_stake_program.key();
    config.treasury = ctx.accounts.treasury.key();
    config.max_ltv_bps = max_ltv_bps;
    config.liquidation_ltv_bps = liquidation_ltv_bps;
    config.protocol_fee_bps = 10; // 0.1%
    config.min_oracle_staleness = 300; // 5 minutes
    config.paused = false;
    config.bump = ctx.bumps.vault_config;

    Ok(())
}
