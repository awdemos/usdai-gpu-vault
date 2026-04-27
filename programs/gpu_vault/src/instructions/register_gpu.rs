use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;
use crate::errors::VaultError;
use crate::events::GpuRegistered;
use mpl_token_metadata::instructions::CreateMetadataAccountV3;
use mpl_token_metadata::types::DataV2;

#[derive(Accounts)]
#[instruction(params: RegisterGpuParams)]
pub struct RegisterGpu<'info> {
    #[account(seeds = [b"vault_config"], bump = vault_config.bump)]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        init,
        payer = owner,
        space = GpuCollateral::LEN,
        seeds = [
            b"gpu_collateral",
            vault_config.key().as_ref(),
            gpu_nft_mint.key().as_ref(),
        ],
        bump
    )]
    pub gpu_collateral: Account<'info, GpuCollateral>,

    #[account(
        init,
        payer = owner,
        mint::decimals = 0,
        mint::authority = vault_config,
        mint::freeze_authority = vault_config,
    )]
    pub gpu_nft_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = owner,
        associated_token::mint = gpu_nft_mint,
        associated_token::authority = vault_config,
    )]
    pub vault_nft_account: Account<'info, TokenAccount>,

    /// CHECK: Metaplex metadata PDA
    #[account(
        mut,
        seeds = [
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            gpu_nft_mint.key().as_ref(),
        ],
        bump,
        seeds::program = mpl_token_metadata::ID,
    )]
    pub metadata_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    /// CHECK: Chainlink price feed for this GPU class
    pub oracle_feed: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RegisterGpuParams {
    pub model: String,
    pub specs: String,
    pub cluster_id: Pubkey,
    pub prometheus_url: String,
}

pub fn handler(
    ctx: Context<RegisterGpu>,
    params: RegisterGpuParams,
) -> Result<()> {
    require!(
        params.model.len() <= 32
            && params.specs.len() <= 64
            && params.prometheus_url.len() <= 128,
        VaultError::MetadataTooLong,
    );

    require!(!ctx.accounts.vault_config.paused, VaultError::Paused);

    let collateral = &mut ctx.accounts.gpu_collateral;
    let clock = Clock::get()?;

    collateral.owner = ctx.accounts.owner.key();
    collateral.gpu_nft_mint = ctx.accounts.gpu_nft_mint.key();
    collateral.valuation_usd = 0;
    collateral.borrowed_usdai = 0;
    collateral.model = parse_model(&params.model)?;
    collateral.status = GpuStatus::Active;
    collateral.oracle_feed = ctx.accounts.oracle_feed.key();
    collateral.last_valuation_ts = 0;
    collateral.bump = ctx.bumps.gpu_collateral;

    // Attach Metaplex metadata to the GPU NFT
    let metadata_ix = CreateMetadataAccountV3 {
        metadata: ctx.accounts.metadata_account.key(),
        mint: ctx.accounts.gpu_nft_mint.key(),
        mint_authority: ctx.accounts.vault_config.key(),
        payer: ctx.accounts.owner.key(),
        update_authority: (ctx.accounts.vault_config.key(), false),
        system_program: ctx.accounts.system_program.key(),
        rent: Some(ctx.accounts.rent.key()),
    }.instruction(
        mpl_token_metadata::instructions::CreateMetadataAccountV3InstructionArgs {
            data: DataV2 {
                name: format!("GPU: {}", params.model),
                symbol: "GPU".to_string(),
                uri: format!("https://metadata.usd.ai/gpu/{}", ctx.accounts.gpu_nft_mint.key()),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            is_mutable: true,
            collection_details: None,
        }
    );

    let vault_config_ai = ctx.accounts.vault_config.to_account_info();
    invoke_signed(
        &metadata_ix,
        &[
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.gpu_nft_mint.to_account_info(),
            vault_config_ai.clone(),              // mint_authority (signer via PDA)
            ctx.accounts.owner.to_account_info(), // payer
            vault_config_ai,                      // update_authority
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        &[&[b"vault_config", &[ctx.accounts.vault_config.bump]]],
    )?;

    emit!(GpuRegistered {
        collateral: collateral.key(),
        nft_mint: collateral.gpu_nft_mint,
        owner: collateral.owner,
        oracle_feed: collateral.oracle_feed,
    });

    msg!("GPU registered: mint={}, owner={}, ts={}",
        collateral.gpu_nft_mint,
        collateral.owner,
        clock.unix_timestamp
    );

    Ok(())
}

fn parse_model(s: &str) -> Result<GpuModel> {
    match s {
        "A100" => Ok(GpuModel::A100),
        "A100x8" | "A100-Cluster-8" => Ok(GpuModel::A100Cluster8),
        "H100" => Ok(GpuModel::H100),
        "H200" => Ok(GpuModel::H200),
        _ => Ok(GpuModel::Unknown),
    }
}
