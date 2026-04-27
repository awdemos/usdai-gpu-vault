use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::errors::VaultError;
use crate::events::ChipStaked;
use crate::external_cpi::usd_ai_stake;

#[derive(Accounts)]
pub struct StakeChip<'info> {
    #[account(seeds = [b"vault_config"], bump = vault_config.bump)]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = owner_chip_account.mint == vault_config.chip_mint @ VaultError::WrongMint,
    )]
    pub owner_chip_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = owner_schip_account.mint == vault_config.s_chip_mint @ VaultError::WrongMint,
    )]
    pub owner_schip_account: Account<'info, TokenAccount>,

    /// CHECK: Existing USD.AI staking program
    #[account(constraint = usd_ai_stake.key() == vault_config.usd_ai_stake_program)]
    pub usd_ai_stake: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<StakeChip>, amount: u64) -> Result<()> {
    require!(!ctx.accounts.vault_config.paused, VaultError::Paused);
    require!(amount > 0, VaultError::ZeroAmount);

    usd_ai_stake::stake_cpi(
        &ctx.accounts.usd_ai_stake,
        usd_ai_stake::UsdAiStakeAccounts {
            staker: ctx.accounts.owner.to_account_info(),
            chip_source: ctx.accounts.owner_chip_account.to_account_info(),
            s_chip_destination: ctx.accounts.owner_schip_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
        amount,
        &[],
    )?;

    emit!(ChipStaked {
        staker: ctx.accounts.owner.key(),
        amount,
    });

    Ok(())
}
