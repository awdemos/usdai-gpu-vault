use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::VaultError;
use crate::events::AuthorityChanged;

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    #[account(
        mut,
        has_one = authority @ VaultError::Unauthorized,
    )]
    pub vault_config: Account<'info, VaultConfig>,

    pub authority: Signer<'info>,

    /// CHECK: New authority address
    pub new_authority: AccountInfo<'info>,
}

pub fn handler(ctx: Context<SetAuthority>) -> Result<()> {
    let old = ctx.accounts.vault_config.authority;
    ctx.accounts.vault_config.authority = ctx.accounts.new_authority.key();

    emit!(AuthorityChanged {
        old_authority: old,
        new_authority: ctx.accounts.new_authority.key(),
    });

    Ok(())
}
