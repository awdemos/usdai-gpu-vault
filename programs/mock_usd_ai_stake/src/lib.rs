use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

declare_id!("Stake11111111111111111111111111111111111111");

#[program]
pub mod mock_usd_ai_stake {
    use super::*;

    pub fn stake(
        _ctx: Context<Stake>,
        amount: u64,
    ) -> Result<()> {
        msg!("Mock stake: amount={}", amount);
        Ok(())
    }

    pub fn unstake(
        _ctx: Context<Unstake>,
        amount: u64,
    ) -> Result<()> {
        msg!("Mock unstake: amount={}", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Stake<'info> {
    pub staker: Signer<'info>,
    #[account(mut)]
    pub chip_source: Account<'info, TokenAccount>,
    #[account(mut)]
    pub s_chip_destination: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    pub staker: Signer<'info>,
    #[account(mut)]
    pub s_chip_source: Account<'info, TokenAccount>,
    #[account(mut)]
    pub chip_destination: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
