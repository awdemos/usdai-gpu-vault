#!/usr/bin/env bash
set -euo pipefail

echo "=== USD.AI GPU Vault — Mainnet Governance Proposal ==="

# Configuration
PROGRAM_ID="${PROGRAM_ID:-<FILL_ME_IN>}"
MULTISIG="${MULTISIG:-<DAO_MULTISIG>}"
UPGRADE_AUTHORITY="${UPGRADE_AUTHORITY:-<UPGRADE_AUTHORITY>}"

echo "Program ID: $PROGRAM_ID"
echo "DAO Multisig: $MULTISIG"
echo "Upgrade Authority: $UPGRADE_AUTHORITY"

# Verify program is built for mainnet
anchor build

# Verify checksum
CHECKSUM=$(sha256sum target/deploy/gpu_vault.so | awk '{print $1}')
echo "Program checksum: $CHECKSUM"

# Create governance proposal metadata
mkdir -p target/governance
cat > target/governance/proposal.md <<EOF
# Proposal: Deploy GPU Vault to Mainnet

## Summary
Deploy the GPU Collateral Vault service to Solana mainnet-beta.

## Program Details
- **Program ID:** \`$PROGRAM_ID\`
- **Upgrade Authority:** \`$UPGRADE_AUTHORITY\`
- **Source:** https://github.com/your-org/usdai-gpu-vault
- **Checksum (SHA-256):** \`$CHECKSUM\`

## Parameters
- Max LTV: 70% (7000 bps)
- Liquidation Threshold: 85% (8500 bps)
- Protocol Fee: 0.1% (10 bps)
- Oracle Staleness: 5 minutes

## Audit
- [OtterSec / Trail of Bits report link]
- Devnet deployment: [explorer link]

## Treasury
- Fee destination: \`$MULTISIG\`

## Voting
- Period: 7 days
- Quorum: 10M \$CHIP
EOF

echo ""
echo "Governance proposal written to target/governance/proposal.md"
echo ""
echo "Steps to execute after proposal passes:"
echo "  1. solana program deploy target/deploy/gpu_vault.so --program-id $PROGRAM_ID"
echo "  2. anchor idl init --filepath target/idl/gpu_vault.json $PROGRAM_ID"
echo "  3. Run initialize_vault with mainnet token/program addresses"
