use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;
use crate::errors::VaultError;
use crate::events::UsdaiRepaid;
use crate::external_cpi::usd_ai_lend;

#[derive(Accounts)]
pub struct RepayUsdai<'info> {
    #[account(seeds = [b"vault_config"], bump = vault_config.bump)]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        mut,
        has_one = owner @ VaultError::Unauthorized,
        constraint = collateral.status == GpuStatus::Borrowing @ VaultError::NotBorrowing,
    )]
    pub collateral: Account<'info, GpuCollateral>,

    #[account(mut)]
    pub owner: Signer<'info>,

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

    /// CHECK: Existing USD.AI lending program
    #[account(constraint = usd_ai_lend.key() == vault_config.usd_ai_lend_program)]
    pub usd_ai_lend: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<RepayUsdai>, amount: u64) -> Result<()> {
    require!(!ctx.accounts.vault_config.paused, VaultError::Paused);

    let collateral = &mut ctx.accounts.collateral;

    require!(
        amount <= collateral.borrowed_usdai,
        VaultError::RepayTooMuch
    );

    // Transfer USDai from owner back to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.owner_usdai_account.to_account_info(),
                to: ctx.accounts.vault_usdai_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        amount,
    )?;

    // CPI to external lending program to settle debt
    usd_ai_lend::repay_cpi(
        &ctx.accounts.usd_ai_lend,
        usd_ai_lend::UsdAiLendRepayAccounts {
            borrower: ctx.accounts.owner.to_account_info(),
            usdai_source: ctx.accounts.vault_usdai_account.to_account_info(),
            collateral_vault: ctx.accounts.vault_usdai_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
        amount,
        &[&[b"vault_config", &[ctx.accounts.vault_config.bump]]],
    )?;

    collateral.borrowed_usdai = collateral
        .borrowed_usdai
        .checked_sub(amount)
        .ok_or(VaultError::MathOverflow)?;

    if collateral.borrowed_usdai == 0 {
        collateral.status = GpuStatus::Active;
    }

    emit!(UsdaiRepaid {
        collateral: collateral.key(),
        amount,
        remaining: collateral.borrowed_usdai,
    });

    Ok(())
}
