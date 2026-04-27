# Deployment Checklist

## External Dependencies (Required Before Deploy)

These are assumed to exist on-chain. The vault program stores their addresses in `VaultConfig` at initialization.

- [ ] **USDai mint address** — SPL Token or Token-2022 mint for the stablecoin borrowed against GPUs
- [ ] **CHIP mint address** — SPL Token or Token-2022 mint for the governance/utility token
- [ ] **sCHIP mint address** — SPL Token or Token-2022 mint for staked CHIP receipts
- [ ] **USD.AI lending program ID** — On-chain program that handles USDai borrow/repay logic
- [ ] **USD.AI staking program ID** — On-chain program that handles CHIP → sCHIP staking
- [ ] **Treasury wallet** — Destination address for 0.1% protocol fees (multisig recommended)

## Oracle Setup

- [ ] **Chainlink / Switchboard / Pyth feed: A100** — Price feed account for NVIDIA A100 GPU valuation
- [ ] **Chainlink / Switchboard / Pyth feed: H100** — Price feed account for NVIDIA H100 GPU valuation
- [ ] **Chainlink / Switchboard / Pyth feed: H200** — Price feed account for NVIDIA H200 GPU valuation
- [ ] **Chainlink / Switchboard / Pyth feed: A100-Cluster-8** — Price feed for 8x A100 cluster valuation
- [ ] **Feed staleness SLA** — Confirm oracle provider guarantees updates within 5 minutes (300s)

## Program Deployment

- [ ] Run `anchor keys sync` to generate real program ID
- [ ] Replace dummy program ID in:
  - `Anchor.toml`
  - `programs/gpu_vault/src/lib.rs`
  - `cli/src/main.rs`
  - `client/index.ts`
  - `tauri-app/src-tauri/src/lib.rs`
- [ ] Deploy to **devnet** using `./scripts/deploy-devnet.sh`
- [ ] Initialize vault with real token/program addresses via CLI
- [ ] Register 1–3 test GPUs on devnet
- [ ] Execute borrow → repay → withdraw cycle on devnet
- [ ] Confirm treasury receives 0.1% fee

## Testing & Quality

- [ ] Run `cargo clippy --workspace` and fix all warnings
- [ ] Run `anchor test` — all 10 integration tests must pass
- [ ] Run `cargo audit` to check for vulnerable dependencies
- [ ] Test emergency pause/unpause flow on devnet
- [ ] Test permissionless liquidation with an underwater position
- [ ] Test batch register CLI with a JSON manifest

## Security

- [ ] **Security audit** by OtterSec, Trail of Bits, or Neodyme
- [ ] Replace temporary authority with DAO multisig (e.g., Squads)
- [ ] Set up timelock for admin operations (optional but recommended)
- [ ] Verify Metaplex metadata URI is served over HTTPS with proper CORS
- [ ] Review all `UncheckedAccount` usages for safety
- [ ] Confirm `liquidation_ltv_bps > max_ltv_bps` invariant holds in all edge cases

## Mainnet Governance

- [ ] Run `./scripts/propose-mainnet.sh` to generate proposal markdown
- [ ] Publish audit report in proposal
- [ ] Link devnet deployment + transaction history
- [ ] Define `$CHIP` quorum and voting period (suggested: 10M CHIP, 7 days)
- [ ] Execute proposal and deploy to mainnet-beta
- [ ] Verify program ID matches proposal
- [ ] Initialize mainnet vault with mainnet token addresses

## Frontend & UX

- [ ] Connect Tauri app to real RPC endpoint (Helius / QuickNode)
- [ ] Add wallet adapter to Tauri app (custom adapter for WebView context)
- [ ] Implement write operations in Tauri backend (register, borrow, repay, stake)
- [ ] Add real-time collateral value charts
- [ ] Add liquidation risk indicator (green/yellow/red based on LTV)
- [ ] Mobile-responsive layout for Tauri window

## Documentation

- [ ] Update `README.md` with real program ID and token addresses
- [ ] Add architecture diagram (Mermaid or PNG)
- [ ] Document CPI interface expected from USD.AI programs
- [ ] Add troubleshooting guide for common Anchor/Solana errors
- [ ] Document how to add new GPU models to oracle mapping

## Monetization & Incentives

- [ ] Confirm 0.1% fee split (treasury vs. stakers vs. LPs)
- [ ] Set up fee distribution contract or manual multisig distribution
- [ ] Define `$CHIP` grant criteria for GPU providers
- [ ] Create governance proposal for fee parameter changes

## Post-Launch Monitoring

- [ ] Set up Prometheus/Grafana for transaction metrics
- [ ] Monitor oracle staleness alerts
- [ ] Monitor liquidation events
- [ ] Set up Telegram/Discord bot for large borrows/liquidations
- [ ] Create incident response runbook
