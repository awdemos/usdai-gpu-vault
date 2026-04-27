use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;
use crate::errors::VaultError;
use crate::events::UsdaiBorrowed;
use crate::external_cpi::usd_ai_lend;
use crate::math::checked_bps_mul;

#[derive(Accounts)]
pub struct BorrowUsdai<'info> {
    #[account(seeds = [b"vault_config"], bump = vault_config.bump)]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        mut,
        has_one = owner @ VaultError::Unauthorized,
        has_one = oracle_feed @ VaultError::WrongOracleFeed,
        constraint = collateral.status == GpuStatus::Active @ VaultError::NotActive,
    )]
    pub collateral: Account<'info, GpuCollateral>,

    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK: Chainlink price feed
    #[account(constraint = oracle_feed.key() == collateral.oracle_feed)]
    pub oracle_feed: AccountInfo<'info>,

    /// CHECK: Existing USD.AI lending program
    #[account(constraint = usd_ai_lend.key() == vault_config.usd_ai_lend_program)]
    pub usd_ai_lend: AccountInfo<'info>,

    #[account(
        mut,
        constraint = owner_usdai_account.mint == vault_config.usdai_mint @ VaultError::WrongMint,
    )]
    pub owner_usdai_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = vault_config.usdai_mint,
        associated_token::authority = vault_config,
    )]
    pub vault_usdai_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = treasury_usdai_account.mint == vault_config.usdai_mint @ VaultError::WrongMint,
        constraint = treasury_usdai_account.owner == vault_config.treasury @ VaultError::InvalidTreasury,
    )]
    pub treasury_usdai_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<BorrowUsdai>, amount: u64) -> Result<()> {
    let collateral = &mut ctx.accounts.collateral;
    let config = &ctx.accounts.vault_config;
    require!(!config.paused, VaultError::Paused);
    let clock = Clock::get()?;

    // Validate oracle freshness
    require!(
        clock.unix_timestamp - collateral.last_valuation_ts <= config.min_oracle_staleness,
        VaultError::StaleOracle,
    );

    // Use cached valuation (staleness checked above)
    let current_value = collateral.valuation_usd;
    require!(current_value > 0, VaultError::ZeroValuation);

    // Compute max borrow based on LTV
    let max_borrow = checked_bps_mul(current_value, config.max_ltv_bps)?;

    let new_borrowed = collateral
        .borrowed_usdai
        .checked_add(amount)
        .ok_or(VaultError::MathOverflow)?;

    require!(new_borrowed <= max_borrow, VaultError::ExceedsLtv);

    // CPI to existing USD.AI lending program — funds land in vault buffer
    usd_ai_lend::borrow_cpi(
        &ctx.accounts.usd_ai_lend,
        usd_ai_lend::UsdAiLendBorrowAccounts {
            borrower: ctx.accounts.owner.to_account_info(),
            usdai_destination: ctx.accounts.vault_usdai_account.to_account_info(),
            collateral_vault: ctx.accounts.vault_usdai_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
        amount,
        collateral.key(),
        &[&[b"vault_config", &[config.bump]]],
    )?;

    // Calculate protocol fee (0.1% default)
    let fee = checked_bps_mul(amount, config.protocol_fee_bps)?;

    let to_user = amount.checked_sub(fee).ok_or(VaultError::MathOverflow)?;

    // Transfer net amount to user
    if to_user > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.vault_usdai_account.to_account_info(),
                    to: ctx.accounts.owner_usdai_account.to_account_info(),
                    authority: ctx.accounts.vault_config.to_account_info(),
                },
                &[&[b"vault_config", &[config.bump]]],
            ),
            to_user,
        )?;
    }

    // Transfer fee to treasury
    if fee > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.vault_usdai_account.to_account_info(),
                    to: ctx.accounts.treasury_usdai_account.to_account_info(),
                    authority: ctx.accounts.vault_config.to_account_info(),
                },
                &[&[b"vault_config", &[config.bump]]],
            ),
            fee,
        )?;
    }

    collateral.borrowed_usdai = new_borrowed;
    collateral.status = GpuStatus::Borrowing;

    emit!(UsdaiBorrowed {
        collateral: collateral.key(),
        amount,
        fee,
        new_total: new_borrowed,
    });

    Ok(())
}
