use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::VaultError;
use crate::events::OracleUpdated;

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    #[account(
        mut,
        has_one = oracle_feed @ VaultError::WrongOracleFeed,
        constraint = collateral.status != GpuStatus::Liquidated @ VaultError::AlreadyLiquidated,
    )]
    pub collateral: Account<'info, GpuCollateral>,

    /// CHECK: Chainlink price feed account
    #[account(constraint = oracle_feed.key() == collateral.oracle_feed)]
    pub oracle_feed: AccountInfo<'info>,
}

pub fn handler(ctx: Context<UpdateOracle>) -> Result<()> {
    let collateral = &mut ctx.accounts.collateral;
    let clock = Clock::get()?;

    let price = crate::external_cpi::chainlink::read_chainlink_price(&ctx.accounts.oracle_feed)?;

    collateral.valuation_usd = price;
    collateral.last_valuation_ts = clock.unix_timestamp;

    emit!(OracleUpdated {
        collateral: collateral.key(),
        price,
        ts: clock.unix_timestamp,
    });

    Ok(())
}
