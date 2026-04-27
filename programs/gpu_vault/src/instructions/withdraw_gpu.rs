use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;
use crate::errors::VaultError;
use crate::events::GpuWithdrawn;

#[derive(Accounts)]
pub struct WithdrawGpu<'info> {
    #[account(seeds = [b"vault_config"], bump = vault_config.bump)]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        mut,
        has_one = owner @ VaultError::Unauthorized,
        constraint = collateral.status == GpuStatus::Active @ VaultError::NotActive,
        constraint = collateral.borrowed_usdai == 0 @ VaultError::OutstandingDebt,
    )]
    pub collateral: Account<'info, GpuCollateral>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = collateral.gpu_nft_mint,
        associated_token::authority = vault_config,
    )]
    pub vault_nft_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = owner_nft_account.mint == collateral.gpu_nft_mint @ VaultError::WrongMint,
        constraint = owner_nft_account.owner == owner.key() @ VaultError::Unauthorized,
    )]
    pub owner_nft_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WithdrawGpu>) -> Result<()> {
    let collateral = &mut ctx.accounts.collateral;
    let config = &ctx.accounts.vault_config;
    require!(!config.paused, VaultError::Paused);

    // Transfer GPU NFT from vault escrow back to owner
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.vault_nft_account.to_account_info(),
                to: ctx.accounts.owner_nft_account.to_account_info(),
                authority: ctx.accounts.vault_config.to_account_info(),
            },
            &[&[b"vault_config", &[config.bump]]],
        ),
        1,
    )?;

    collateral.status = GpuStatus::Withdrawn;

    emit!(GpuWithdrawn {
        collateral: collateral.key(),
        owner: ctx.accounts.owner.key(),
    });

    msg!("GPU withdrawn: collateral={}, owner={}", collateral.key(), ctx.accounts.owner.key());

    Ok(())
}
