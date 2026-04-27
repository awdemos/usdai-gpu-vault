#!/usr/bin/env bash
set -euo pipefail

echo "=== USD.AI GPU Vault — Devnet Deploy ==="

# Ensure we have a devnet keypair
if [ ! -f "~/.config/solana/id.json" ]; then
    echo "Generating devnet keypair..."
    solana-keygen new --no-bip39-passphrase -s -o ~/.config/solana/id.json
fi

# Set cluster to devnet
solana config set --url devnet

# Airdrop if low balance
BALANCE=$(solana balance | awk '{print $1}')
if (( $(echo "$BALANCE < 2" | bc -l) )); then
    echo "Airdropping 2 SOL..."
    solana airdrop 2
fi

# Build
anchor build

# Deploy
anchor deploy --provider.cluster devnet --program-name gpu_vault

# Sync program ID in code and Anchor.toml
anchor keys sync

# Run post-deploy init (update with real external addresses)
PROGRAM_ID=$(solana address -k target/deploy/gpu_vault-keypair.json)
echo "Deployed program: $PROGRAM_ID"

echo ""
echo "Next: initialize the vault with real token/program IDs:"
echo "  cargo run -- initialize \\"
echo "    --usdai-mint <USDai_MINT> \\"
echo "    --chip-mint <CHIP_MINT> \\"
echo "    --s-chip-mint <sCHIP_MINT> \\"
echo "    --usd-ai-lend <LEND_PROGRAM> \\"
echo "    --usd-ai-stake <STAKE_PROGRAM> \\"
echo "    --treasury <TREASURY_WALLET>"
