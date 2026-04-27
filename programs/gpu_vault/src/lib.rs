pub mod instructions;
pub mod state;
pub mod errors;
pub mod events;
pub mod external_cpi;
pub mod math;

use anchor_lang::prelude::*;
use instructions::*;

// TODO: Replace with actual deployed program ID after `anchor keys sync`
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod gpu_vault {
    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        max_ltv_bps: u16,
        liquidation_ltv_bps: u16,
    ) -> Result<()> {
        instructions::initialize_vault::handler(ctx, max_ltv_bps, liquidation_ltv_bps)
    }

    pub fn register_gpu(
        ctx: Context<RegisterGpu>,
        params: RegisterGpuParams,
    ) -> Result<()> {
        instructions::register_gpu::handler(ctx, params)
    }

    pub fn borrow_usdai(
        ctx: Context<BorrowUsdai>,
        amount: u64,
    ) -> Result<()> {
        instructions::borrow_usdai::handler(ctx, amount)
    }

    pub fn repay_usdai(
        ctx: Context<RepayUsdai>,
        amount: u64,
    ) -> Result<()> {
        instructions::repay_usdai::handler(ctx, amount)
    }

    pub fn stake_chip(
        ctx: Context<StakeChip>,
        amount: u64,
    ) -> Result<()> {
        instructions::stake_chip::handler(ctx, amount)
    }

    pub fn unstake_chip(
        ctx: Context<UnstakeChip>,
        amount: u64,
    ) -> Result<()> {
        instructions::unstake_chip::handler(ctx, amount)
    }

    pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
        instructions::liquidate::handler(ctx)
    }

    pub fn update_oracle(ctx: Context<UpdateOracle>) -> Result<()> {
        instructions::update_oracle::handler(ctx)
    }

    pub fn withdraw_gpu(ctx: Context<WithdrawGpu>) -> Result<()> {
        instructions::withdraw_gpu::handler(ctx)
    }

    pub fn set_authority(ctx: Context<SetAuthority>) -> Result<()> {
        instructions::set_authority::handler(ctx)
    }

    pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
        instructions::set_pause::handler(ctx, paused)
    }

    pub fn update_treasury(ctx: Context<UpdateTreasury>) -> Result<()> {
        instructions::update_treasury::handler(ctx)
    }
}
