# USD.AI GPU Collateral Vault

> ⚠️ **Experimental / Unaudited**
> This code is a reference scaffold and has not been audited. Do not use with real funds in production without a professional security review.

GPU collateralization and lending service built on Solana. Uses existing USDai, CHIP, and sCHIP tokens from the USD.AI ecosystem.

## Architecture

```
usdai-gpu-vault/
├── programs/
│   ├── gpu_vault/              # Core vault program
│   ├── mock_usd_ai_lend/       # Mock lending program (tests)
│   └── mock_usd_ai_stake/      # Mock staking program (tests)
├── cli/                        # Rust CLI for operators
├── client/                     # TypeScript SDK (PDAs + helpers)
├── tauri-app/                  # Desktop UI (Tauri v2)
├── tests/
│   └── integration_test.ts     # Full Anchor TS test suite
└── scripts/
    ├── deploy-devnet.sh
    └── propose-mainnet.sh
```

## Prerequisites

- Rust 1.75+
- Anchor 0.30.1
- Solana CLI 1.18+
- Node.js 18+ (for tests & frontend)
- Tauri CLI 2.0+ (optional, for desktop app)

## Quick Start

```bash
# Install JS dependencies
yarn install

# Build all programs
anchor build

# Run tests
anchor test

# Sync program ID
anchor keys sync

# Deploy to devnet
./scripts/deploy-devnet.sh
```

## Program Features

| Feature | Description |
|---------|-------------|
| **Token-2022** | `anchor-spl` built with `token_2022` feature. Pass `spl_token_2022::ID` as `token_program` for Token-2022 mints. |
| **Metaplex NFT** | GPU collateral NFTs get `CreateMetadataAccountV3` metadata on registration. |
| **Treasury Fee** | 0.1% protocol fee on borrows split to treasury ATA automatically. |
| **Pause** | Emergency freeze via `set_pause`. All user operations reject when paused. |
| **Liquidation** | Permissionless. Anyone can trigger if LTV > liquidation threshold. |
| **Oracle** | Chainlink-compatible feed reader with configurable staleness (default 5 min). |

## CLI Usage

```bash
cd cli
cargo run -- initialize \
  --usdai-mint <MINT> \
  --chip-mint <MINT> \
  --s-chip-mint <MINT> \
  --usd-ai-lend <PROGRAM_ID> \
  --usd-ai-stake <PROGRAM_ID> \
  --treasury <WALLET>

cargo run -- register-gpu --model H100 --oracle-feed <CHAINLINK_FEED>
cargo run -- borrow --collateral <PDA> --amount 500000
cargo run -- set-pause --paused true
```

## Client SDK

```bash
cd client
npm install
npm run build
```

```typescript
import { getVaultConfigPda, getGpuCollateralPda, DEFAULT_PROGRAM_ID } from "@usdai/gpu-vault-client";

const programId = DEFAULT_PROGRAM_ID;
const vaultConfig = getVaultConfigPda(programId);
const collateral = getGpuCollateralPda(programId, vaultConfig, nftMint);
```

## Desktop App (Tauri v2)

```bash
cd tauri-app
npm install
npm run tauri dev
```

The Tauri app provides a read-only dashboard for vault config and GPU collaterals. Write operations are delegated to the CLI or backend.

## Key Flows

1. **Register GPU** → Mint 0-decimal NFT, lock in vault escrow, attach Metaplex metadata
2. **Update Oracle** → Refresh USD valuation from Chainlink feed
3. **Borrow** → Borrow USDai against GPU (max 70% LTV). Fee auto-routed to treasury.
4. **Repay** → Return USDai, release debt
5. **Withdraw** → Reclaim GPU NFT after full repayment
6. **Liquidate** → Seize underwater collateral (LTV > 85%)
7. **Stake CHIP** → Route to external USD.AI staking program for sCHIP yields

## Security

- All arithmetic uses checked math (`checked_mul`, `checked_div`).
- Collateral NFTs escrowed by vault PDA while borrowed.
- Oracle staleness checks prevent outdated valuations.
- Emergency pause halts all user operations.
- Liquidation is permissionless for market enforcement.

## License

MIT
