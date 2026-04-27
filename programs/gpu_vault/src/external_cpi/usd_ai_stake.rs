use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};

#[derive(AnchorSerialize, AnchorDeserialize)]
struct StakeData {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct UnstakeData {
    pub amount: u64,
}

pub struct UsdAiStakeAccounts<'info> {
    pub staker: AccountInfo<'info>,
    pub chip_source: AccountInfo<'info>,
    pub s_chip_destination: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

pub fn stake_cpi<'info>(
    usd_ai_stake_program: &AccountInfo<'info>,
    accounts: UsdAiStakeAccounts<'info>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let data = StakeData { amount }.try_to_vec()?;
    let ix = Instruction {
        program_id: usd_ai_stake_program.key(),
        accounts: vec![
            AccountMeta::new_readonly(accounts.staker.key(), true),
            AccountMeta::new(accounts.chip_source.key(), false),
            AccountMeta::new(accounts.s_chip_destination.key(), false),
            AccountMeta::new_readonly(accounts.token_program.key(), false),
        ],
        data,
    };
    invoke_signed(
        &ix,
        &[
            accounts.staker,
            accounts.chip_source,
            accounts.s_chip_destination,
            accounts.token_program,
        ],
        signer_seeds,
    )?;
    Ok(())
}

pub fn unstake_cpi<'info>(
    usd_ai_stake_program: &AccountInfo<'info>,
    accounts: UnstakeAccounts<'info>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let data = UnstakeData { amount }.try_to_vec()?;
    let ix = Instruction {
        program_id: usd_ai_stake_program.key(),
        accounts: vec![
            AccountMeta::new_readonly(accounts.staker.key(), true),
            AccountMeta::new(accounts.s_chip_source.key(), false),
            AccountMeta::new(accounts.chip_destination.key(), false),
            AccountMeta::new_readonly(accounts.token_program.key(), false),
        ],
        data,
    };
    invoke_signed(
        &ix,
        &[
            accounts.staker,
            accounts.s_chip_source,
            accounts.chip_destination,
            accounts.token_program,
        ],
        signer_seeds,
    )?;
    Ok(())
}

pub struct UnstakeAccounts<'info> {
    pub staker: AccountInfo<'info>,
    pub s_chip_source: AccountInfo<'info>,
    pub chip_destination: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}
