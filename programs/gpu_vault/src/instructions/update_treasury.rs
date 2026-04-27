use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::VaultError;
use crate::events::TreasuryUpdated;

#[derive(Accounts)]
pub struct UpdateTreasury<'info> {
    #[account(
        mut,
        has_one = authority @ VaultError::Unauthorized,
    )]
    pub vault_config: Account<'info, VaultConfig>,

    pub authority: Signer<'info>,

    /// CHECK: New treasury wallet address
    pub new_treasury: AccountInfo<'info>,
}

pub fn handler(ctx: Context<UpdateTreasury>) -> Result<()> {
    let old = ctx.accounts.vault_config.treasury;
    ctx.accounts.vault_config.treasury = ctx.accounts.new_treasury.key();

    emit!(TreasuryUpdated {
        old_treasury: old,
        new_treasury: ctx.accounts.new_treasury.key(),
    });

    Ok(())
}
