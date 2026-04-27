use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;
use crate::errors::VaultError;
use crate::events::Liquidation;
use crate::math::checked_ltv_bps;
use crate::external_cpi::usd_ai_lend;

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(seeds = [b"vault_config"], bump = vault_config.bump)]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        mut,
        has_one = oracle_feed @ VaultError::WrongOracleFeed,
        constraint = collateral.status == GpuStatus::Borrowing @ VaultError::NotBorrowing,
    )]
    pub collateral: Account<'info, GpuCollateral>,

    #[account(
        constraint = nft_mint.key() == collateral.gpu_nft_mint @ VaultError::WrongMint,
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(
        mut,
        constraint = liquidator_usdai.mint == vault_config.usdai_mint @ VaultError::WrongMint,
        constraint = liquidator_usdai.owner == liquidator.key() @ VaultError::Unauthorized,
    )]
    pub liquidator_usdai: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = vault_config,
    )]
    pub vault_nft_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = nft_mint,
        associated_token::authority = liquidator,
    )]
    pub liquidator_nft_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = vault_config.usdai_mint,
        associated_token::authority = vault_config,
    )]
    pub vault_usdai_account: Account<'info, TokenAccount>,

    /// CHECK: Chainlink oracle price feed
    #[account(constraint = oracle_feed.key() == collateral.oracle_feed)]
    pub oracle_feed: AccountInfo<'info>,

    /// CHECK: Existing USD.AI lend program
    #[account(constraint = usd_ai_lend.key() == vault_config.usd_ai_lend_program)]
    pub usd_ai_lend: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<Liquidate>) -> Result<()> {
    let collateral = &mut ctx.accounts.collateral;
    let config = &ctx.accounts.vault_config;
    require!(!config.paused, VaultError::Paused);

    // Use cached valuation (oracle feed is validated by has_one constraint)
    let current_value = collateral.valuation_usd;
    require!(current_value > 0, VaultError::ZeroValuation);

    let ltv_bps = checked_ltv_bps(collateral.borrowed_usdai, current_value)?;

    require!(
        ltv_bps >= config.liquidation_ltv_bps,
        VaultError::NotLiquidatable
    );

    let debt = collateral.borrowed_usdai;

    // 1. Liquidator covers debt
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.liquidator_usdai.to_account_info(),
                to: ctx.accounts.vault_usdai_account.to_account_info(),
                authority: ctx.accounts.liquidator.to_account_info(),
            },
        ),
        debt,
    )?;

    // 2. Transfer GPU NFT escrow to liquidator
    let vault_bump = config.bump;
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.vault_nft_account.to_account_info(),
                to: ctx.accounts.liquidator_nft_account.to_account_info(),
                authority: ctx.accounts.vault_config.to_account_info(),
            },
            &[&[b"vault_config", &[vault_bump]]],
        ),
        1,
    )?;

    // CPI to external lending program to settle debt
    usd_ai_lend::repay_cpi(
        &ctx.accounts.usd_ai_lend,
        usd_ai_lend::UsdAiLendRepayAccounts {
            borrower: ctx.accounts.liquidator.to_account_info(),
            usdai_source: ctx.accounts.vault_usdai_account.to_account_info(),
            collateral_vault: ctx.accounts.vault_usdai_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
        debt,
        &[&[b"vault_config", &[ctx.accounts.vault_config.bump]]],
    )?;

    collateral.status = GpuStatus::Liquidated;
    collateral.borrowed_usdai = 0;

    emit!(Liquidation {
        collateral: collateral.key(),
        liquidator: ctx.accounts.liquidator.key(),
        debt_repaid: debt,
    });

    Ok(())
}
