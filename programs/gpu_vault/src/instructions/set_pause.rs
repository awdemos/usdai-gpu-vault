use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::VaultError;
use crate::events::PauseSet;

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(
        mut,
        has_one = authority @ VaultError::Unauthorized,
    )]
    pub vault_config: Account<'info, VaultConfig>,

    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    ctx.accounts.vault_config.paused = paused;

    emit!(PauseSet {
        paused,
        authority: ctx.accounts.authority.key(),
    });

    Ok(())
}
