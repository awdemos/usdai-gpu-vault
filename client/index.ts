import { PublicKey } from "@solana/web3.js";

export const DEFAULT_PROGRAM_ID = new PublicKey(
  "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
);

export const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

/** Derive the vault config PDA. */
export function getVaultConfigPda(programId: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("vault_config")],
    programId
  )[0];
}

/** Derive a GPU collateral PDA for a given NFT mint. */
export function getGpuCollateralPda(
  programId: PublicKey,
  vaultConfig: PublicKey,
  nftMint: PublicKey
) {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("gpu_collateral"),
      vaultConfig.toBuffer(),
      nftMint.toBuffer(),
    ],
    programId
  )[0];
}

/** Derive the Metaplex metadata PDA for a mint. */
export function getMetadataPda(mint: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  )[0];
}

/** Derive the vault's ATA for a given mint. */
export function getVaultAta(mint: PublicKey, vaultConfigPda: PublicKey) {
  // Uses ATA program — compatible with both Token and Token-2022
  const { getAssociatedTokenAddressSync } = require("@solana/spl-token");
  return getAssociatedTokenAddressSync(mint, vaultConfigPda, true);
}

export * from "../target/types/gpu_vault";
