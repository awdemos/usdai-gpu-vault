use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

declare_id!("Lend111111111111111111111111111111111111111");

#[program]
pub mod mock_usd_ai_lend {
    use super::*;

    pub fn borrow(
        _ctx: Context<Borrow>,
        amount: u64,
        _collateral: Pubkey,
    ) -> Result<()> {
        msg!("Mock borrow: amount={}", amount);
        Ok(())
    }

    pub fn repay(
        _ctx: Context<Repay>,
        amount: u64,
    ) -> Result<()> {
        msg!("Mock repay: amount={}", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    pub borrower: Signer<'info>,
    #[account(mut)]
    pub usdai_destination: Account<'info, TokenAccount>,
    #[account(mut)]
    pub collateral_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Repay<'info> {
    pub borrower: Signer<'info>,
    #[account(mut)]
    pub usdai_source: Account<'info, TokenAccount>,
    #[account(mut)]
    pub collateral_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
