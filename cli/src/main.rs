use clap::{Parser, Subcommand};
use solana_sdk::signer::keypair::read_keypair_file;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use anchor_client::{Client, Cluster};
use std::rc::Rc;
use std::str::FromStr;
use std::fs;
use anyhow::Result;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "usdai-gpu")]
#[command(version = "0.1.0")]
#[command(about = "USD.AI GPU Collateral Vault CLI")]
struct Cli {
    #[arg(short, long, default_value = "~/.config/solana/id.json")]
    keypair: String,

    #[arg(short, long, default_value = "devnet")]
    cluster: String,

    #[arg(long, help = "Use cached/oracle-offline mode")]
    offline_oracle: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize vault config with existing token addresses
    Initialize {
        #[arg(default_value = "7000")]
        max_ltv_bps: u16,
        #[arg(default_value = "8500")]
        liquidation_ltv_bps: u16,
        #[arg(long)]
        usdai_mint: String,
        #[arg(long)]
        chip_mint: String,
        #[arg(long)]
        s_chip_mint: String,
        #[arg(long)]
        usd_ai_lend: String,
        #[arg(long)]
        usd_ai_stake: String,
        #[arg(long)]
        treasury: String,
    },
    /// Register a GPU and mint collateral NFT
    RegisterGpu {
        #[arg(short, long)]
        model: String,
        #[arg(short, long)]
        oracle_feed: String,
        #[arg(long, default_value = "")]
        specs: String,
        #[arg(long, default_value = "")]
        prometheus_url: String,
    },
    /// Batch register GPUs from a JSON manifest
    BatchRegister {
        #[arg(short, long)]
        manifest: String,
    },
    /// Update oracle price for a GPU collateral
    UpdateOracle {
        #[arg(short, long)]
        collateral: String,
    },
    /// Borrow USDai against GPU collateral
    Borrow {
        #[arg(short, long)]
        collateral: String,
        #[arg(short, long)]
        amount: u64,
    },
    /// Repay USDai loan
    Repay {
        #[arg(short, long)]
        collateral: String,
        #[arg(short, long)]
        amount: u64,
    },
    /// Withdraw GPU NFT after full repayment
    WithdrawGpu {
        #[arg(short, long)]
        collateral: String,
    },
    /// Stake existing CHIP tokens via USD.AI staking program
    StakeChip {
        #[arg(short, long)]
        amount: u64,
    },
    /// Unstake sCHIP
    UnstakeChip {
        #[arg(short, long)]
        amount: u64,
    },
    /// Trigger liquidation on an underwater position
    Liquidate {
        #[arg(short, long)]
        collateral: String,
    },
    /// Set program authority (admin only)
    SetAuthority {
        #[arg(short, long)]
        new_authority: String,
    },
    /// Pause or unpause the program (admin only)
    SetPause {
        #[arg(long)]
        paused: bool,
    },
    /// Update treasury address (admin only)
    UpdateTreasury {
        #[arg(short, long)]
        treasury: String,
    },
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GpuManifestEntry {
    model: String,
    oracle_feed: String,
    #[serde(default)]
    specs: String,
    #[serde(default)]
    prometheus_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let payer = read_keypair_file(
        shellexpand::tilde(&cli.keypair).into_owned()
    ).map_err(|e| anyhow::anyhow!("Failed to read keypair: {}", e))?;

    let cluster = match cli.cluster.as_str() {
        "devnet" => Cluster::Devnet,
        "mainnet" => Cluster::Mainnet,
        "localnet" | "localhost" => Cluster::Localnet,
        url => Cluster::Custom(url.to_string(), url.to_string()),
    };

    let client = Client::new_with_options(
        cluster,
        Rc::new(payer),
        CommitmentConfig::confirmed(),
    );

    let program_id = gpu_vault::ID;
    let program = client.program(program_id)?;

    match cli.command {
        Commands::Initialize { max_ltv_bps, liquidation_ltv_bps, usdai_mint, chip_mint, s_chip_mint, usd_ai_lend, usd_ai_stake, treasury } => {
            let (vault_config_pda, _bump) = Pubkey::find_program_address(
                &[b"vault_config"],
                &program_id,
            );

            let sig = program
                .request()
                .accounts(gpu_vault::accounts::InitializeVault {
                    vault_config: vault_config_pda,
                    payer: program.payer(),
                    usdai_mint: Pubkey::from_str(&usdai_mint)?,
                    chip_mint: Pubkey::from_str(&chip_mint)?,
                    s_chip_mint: Pubkey::from_str(&s_chip_mint)?,
                    usd_ai_lend_program: Pubkey::from_str(&usd_ai_lend)?,
                    usd_ai_stake_program: Pubkey::from_str(&usd_ai_stake)?,
                    treasury: Pubkey::from_str(&treasury)?,
                    system_program: solana_sdk::system_program::ID,
                })
                .args(gpu_vault::instruction::InitializeVault { max_ltv_bps, liquidation_ltv_bps })
                .send()?;
            println!("Vault initialized: {}", sig);
        }

        Commands::RegisterGpu { model, oracle_feed, specs: _, prometheus_url: _ } => {
            println!("Register GPU: model={}, oracle={}", model, oracle_feed);
            // TODO: Derive PDAs and build full transaction
        }

        Commands::BatchRegister { manifest } => {
            let entries: Vec<GpuManifestEntry> = serde_json::from_str(
                &fs::read_to_string(&manifest)?
            )?;
            println!("Batch registering {} GPUs...", entries.len());
            for (i, entry) in entries.iter().enumerate() {
                println!("  [{}/{}] Registering {} (oracle: {})", i + 1, entries.len(), entry.model, entry.oracle_feed);
                // TODO: Derive PDAs, build and send tx per GPU
            }
        }

        Commands::UpdateOracle { collateral } => {
            println!("Update oracle for collateral: {}", collateral);
        }

        Commands::Borrow { collateral, amount } => {
            println!("Borrow {} USDai against {}", amount, collateral);
        }

        Commands::Repay { collateral, amount } => {
            println!("Repay {} USDai for {}", amount, collateral);
        }

        Commands::WithdrawGpu { collateral } => {
            println!("Withdraw GPU NFT from collateral: {}", collateral);
        }

        Commands::StakeChip { amount } => {
            println!("Stake {} CHIP", amount);
        }

        Commands::UnstakeChip { amount } => {
            println!("Unstake {} sCHIP", amount);
        }

        Commands::Liquidate { collateral } => {
            println!("Liquidate collateral: {}", collateral);
        }

        Commands::SetAuthority { new_authority } => {
            let (vault_config_pda, _bump) = Pubkey::find_program_address(&[b"vault_config"], &program_id);
            let sig = program
                .request()
                .accounts(gpu_vault::accounts::SetAuthority {
                    vault_config: vault_config_pda,
                    authority: program.payer(),
                    new_authority: Pubkey::from_str(&new_authority)?,
                })
                .args(gpu_vault::instruction::SetAuthority {})
                .send()?;
            println!("Authority updated: {}", sig);
        }

        Commands::SetPause { paused } => {
            let (vault_config_pda, _bump) = Pubkey::find_program_address(&[b"vault_config"], &program_id);
            let sig = program
                .request()
                .accounts(gpu_vault::accounts::SetPause {
                    vault_config: vault_config_pda,
                    authority: program.payer(),
                })
                .args(gpu_vault::instruction::SetPause { paused })
                .send()?;
            println!("Pause set to {}: {}", paused, sig);
        }

        Commands::UpdateTreasury { treasury } => {
            let (vault_config_pda, _bump) = Pubkey::find_program_address(&[b"vault_config"], &program_id);
            let sig = program
                .request()
                .accounts(gpu_vault::accounts::UpdateTreasury {
                    vault_config: vault_config_pda,
                    authority: program.payer(),
                    new_treasury: Pubkey::from_str(&treasury)?,
                })
                .args(gpu_vault::instruction::UpdateTreasury {})
                .send()?;
            println!("Treasury updated: {}", sig);
        }
    }

    Ok(())
}
