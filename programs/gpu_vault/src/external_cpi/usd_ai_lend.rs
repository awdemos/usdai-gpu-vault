use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};

#[derive(AnchorSerialize, AnchorDeserialize)]
struct BorrowData {
    pub amount: u64,
    pub collateral: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct RepayData {
    pub amount: u64,
}

pub struct UsdAiLendBorrowAccounts<'info> {
    pub borrower: AccountInfo<'info>,
    pub usdai_destination: AccountInfo<'info>,
    pub collateral_vault: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

pub struct UsdAiLendRepayAccounts<'info> {
    pub borrower: AccountInfo<'info>,
    pub usdai_source: AccountInfo<'info>,
    pub collateral_vault: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

pub fn borrow_cpi<'info>(
    usd_ai_lend_program: &AccountInfo<'info>,
    accounts: UsdAiLendBorrowAccounts<'info>,
    amount: u64,
    collateral: Pubkey,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let data = BorrowData { amount, collateral }.try_to_vec()?;
    let ix = Instruction {
        program_id: usd_ai_lend_program.key(),
        accounts: vec![
            AccountMeta::new_readonly(accounts.borrower.key(), true),
            AccountMeta::new(accounts.usdai_destination.key(), false),
            AccountMeta::new(accounts.collateral_vault.key(), false),
            AccountMeta::new_readonly(accounts.token_program.key(), false),
        ],
        data,
    };
    invoke_signed(
        &ix,
        &[
            accounts.borrower,
            accounts.usdai_destination,
            accounts.collateral_vault,
            accounts.token_program,
        ],
        signer_seeds,
    )?;
    Ok(())
}

pub fn repay_cpi<'info>(
    usd_ai_lend_program: &AccountInfo<'info>,
    accounts: UsdAiLendRepayAccounts<'info>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let data = RepayData { amount }.try_to_vec()?;
    let ix = Instruction {
        program_id: usd_ai_lend_program.key(),
        accounts: vec![
            AccountMeta::new_readonly(accounts.borrower.key(), true),
            AccountMeta::new(accounts.usdai_source.key(), false),
            AccountMeta::new(accounts.collateral_vault.key(), false),
            AccountMeta::new_readonly(accounts.token_program.key(), false),
        ],
        data,
    };
    invoke_signed(
        &ix,
        &[
            accounts.borrower,
            accounts.usdai_source,
            accounts.collateral_vault,
            accounts.token_program,
        ],
        signer_seeds,
    )?;
    Ok(())
}
