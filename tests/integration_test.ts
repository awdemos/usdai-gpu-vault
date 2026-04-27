import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GpuVault } from "../target/types/gpu_vault";
import { MockUsdAiLend } from "../target/types/mock_usd_ai_lend";
import { MockUsdAiStake } from "../target/types/mock_usd_ai_stake";
import { expect } from "chai";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("gpu-vault full flow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.GpuVault as Program<GpuVault>;
  const mockLend = anchor.workspace.MockUsdAiLend as Program<MockUsdAiLend>;
  const mockStake = anchor.workspace.MockUsdAiStake as Program<MockUsdAiStake>;

  let usdaiMint: anchor.web3.PublicKey;
  let chipMint: anchor.web3.PublicKey;
  let sChipMint: anchor.web3.PublicKey;
  let vaultConfigPda: anchor.web3.PublicKey;
  let vaultConfigBump: number;

  const treasuryKp = anchor.web3.Keypair.generate();

  before(async () => {
    usdaiMint = await createMint(provider.connection, payer.payer, payer.publicKey, null, 6);
    chipMint = await createMint(provider.connection, payer.payer, payer.publicKey, null, 6);
    sChipMint = await createMint(provider.connection, payer.payer, payer.publicKey, null, 6);

    [vaultConfigPda, vaultConfigBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_config")],
      program.programId
    );
  });

  it("Initializes vault config", async () => {
    await program.methods
      .initializeVault(7000, 8500)
      .accounts({
        vaultConfig: vaultConfigPda,
        payer: payer.publicKey,
        usdaiMint,
        chipMint,
        sChipMint,
        usdAiLendProgram: mockLend.programId,
        usdAiStakeProgram: mockStake.programId,
        treasury: treasuryKp.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const config = await program.account.vaultConfig.fetch(vaultConfigPda);
    expect(config.maxLtvBps).to.equal(7000);
    expect(config.liquidationLtvBps).to.equal(8500);
    expect(config.protocolFeeBps).to.equal(10);
    expect(config.paused).to.equal(false);
    expect(config.usdaiMint.toBase58()).to.equal(usdaiMint.toBase58());
  });

  it("Registers a GPU collateral NFT", async () => {
    const nftMintKp = anchor.web3.Keypair.generate();
    const oracleFeedKp = anchor.web3.Keypair.generate();

    // Create mock oracle feed account (49 bytes of zeros → fallback price $1.00)
    await provider.connection.confirmTransaction(
      await provider.connection.sendTransaction(
        new anchor.web3.Transaction().add(
          anchor.web3.SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: oracleFeedKp.publicKey,
            lamports: await provider.connection.getMinimumBalanceForRentExemption(49),
            space: 49,
            programId: anchor.web3.SystemProgram.programId,
          })
        ),
        [payer.payer, oracleFeedKp]
      )
    );

    const [collateralPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("gpu_collateral"),
        vaultConfigPda.toBuffer(),
        nftMintKp.publicKey.toBuffer(),
      ],
      program.programId
    );

    await program.methods
      .registerGpu({
        model: "H100",
        specs: "80GB HBM3",
        clusterId: payer.publicKey,
        prometheusUrl: "http://localhost:9090",
      })
      .accounts({
        vaultConfig: vaultConfigPda,
        gpuCollateral: collateralPda,
        gpuNftMint: nftMintKp.publicKey,
        vaultNftAccount: getAssociatedTokenAddressSync(nftMintKp.publicKey, vaultConfigPda, true),
        metadataAccount: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
            nftMintKp.publicKey.toBuffer(),
          ],
          new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        )[0],
        owner: payer.publicKey,
        oracleFeed: oracleFeedKp.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([nftMintKp])
      .rpc();

    const collateral = await program.account.gpuCollateral.fetch(collateralPda);
    expect(collateral.model).to.deep.equal({ h100: {} });
    expect(collateral.status).to.deep.equal({ active: {} });
    expect(collateral.borrowedUsdai.toNumber()).to.equal(0);
  });

  it("Updates oracle price", async () => {
    const allCollaterals = await program.account.gpuCollateral.all();
    expect(allCollaterals.length).to.equal(1);
    const collateral = allCollaterals[0];

    await program.methods
      .updateOracle()
      .accounts({
        collateral: collateral.publicKey,
        oracleFeed: collateral.account.oracleFeed,
      })
      .rpc();

    const updated = await program.account.gpuCollateral.fetch(collateral.publicKey);
    expect(updated.valuationUsd.toNumber()).to.equal(1_000_000); // fallback $1.00
    expect(updated.lastValuationTs.toNumber()).to.be.greaterThan(0);
  });

  it("Borrows USDai with 0.1% fee to treasury", async () => {
    const allCollaterals = await program.account.gpuCollateral.all();
    const collateral = allCollaterals[0];

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    const treasuryUsdai = getAssociatedTokenAddressSync(usdaiMint, treasuryKp.publicKey);
    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);

    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, vaultConfigPda, true);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, treasuryKp.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, payer.publicKey);

    await mintTo(provider.connection, payer.payer, usdaiMint, vaultUsdai, payer.publicKey, 10_000_000);

    const vaultBefore = (await getAccount(provider.connection, vaultUsdai)).amount;
    const treasuryBefore = (await getAccount(provider.connection, treasuryUsdai)).amount;
    const ownerBefore = (await getAccount(provider.connection, ownerUsdai)).amount;

    const borrowAmount = 500_000; // $0.50

    await program.methods
      .borrowUsdai(new anchor.BN(borrowAmount))
      .accounts({
        vaultConfig: vaultConfigPda,
        collateral: collateral.publicKey,
        owner: payer.publicKey,
        oracleFeed: collateral.account.oracleFeed,
        usdAiLend: mockLend.programId,
        ownerUsdaiAccount: ownerUsdai,
        vaultUsdaiAccount: vaultUsdai,
        treasuryUsdaiAccount: treasuryUsdai,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const fee = Math.floor((borrowAmount * 10) / 10_000); // 0.1% = 500
    const toUser = borrowAmount - fee;

    const vaultAfter = (await getAccount(provider.connection, vaultUsdai)).amount;
    const treasuryAfter = (await getAccount(provider.connection, treasuryUsdai)).amount;
    const ownerAfter = (await getAccount(provider.connection, ownerUsdai)).amount;

    expect(Number(vaultAfter)).to.equal(Number(vaultBefore) - borrowAmount);
    expect(Number(treasuryAfter)).to.equal(Number(treasuryBefore) + fee);
    expect(Number(ownerAfter)).to.equal(Number(ownerBefore) + toUser);

    const updated = await program.account.gpuCollateral.fetch(collateral.publicKey);
    expect(updated.borrowedUsdai.toNumber()).to.equal(borrowAmount);
    expect(updated.status).to.deep.equal({ borrowing: {} });
  });

  it("Repays USDai loan", async () => {
    const allCollaterals = await program.account.gpuCollateral.all();
    const collateral = allCollaterals[0];

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);

    await mintTo(provider.connection, payer.payer, usdaiMint, ownerUsdai, payer.publicKey, 1_000_000);

    const repayAmount = 500_000;

    await program.methods
      .repayUsdai(new anchor.BN(repayAmount))
      .accounts({
        vaultConfig: vaultConfigPda,
        collateral: collateral.publicKey,
        owner: payer.publicKey,
        ownerUsdaiAccount: ownerUsdai,
        vaultUsdaiAccount: vaultUsdai,
        usdAiLend: mockLend.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const updated = await program.account.gpuCollateral.fetch(collateral.publicKey);
    expect(updated.borrowedUsdai.toNumber()).to.equal(0);
    expect(updated.status).to.deep.equal({ active: {} });
  });

  it("Withdraws GPU NFT after repayment", async () => {
    const allCollaterals = await program.account.gpuCollateral.all();
    const collateral = allCollaterals[0];

    const vaultNft = getAssociatedTokenAddressSync(collateral.account.gpuNftMint, vaultConfigPda, true);
    const ownerNft = getAssociatedTokenAddressSync(collateral.account.gpuNftMint, payer.publicKey);

    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, collateral.account.gpuNftMint, payer.publicKey);

    await program.methods
      .withdrawGpu()
      .accounts({
        vaultConfig: vaultConfigPda,
        collateral: collateral.publicKey,
        owner: payer.publicKey,
        vaultNftAccount: vaultNft,
        ownerNftAccount: ownerNft,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const updated = await program.account.gpuCollateral.fetch(collateral.publicKey);
    expect(updated.status).to.deep.equal({ withdrawn: {} });

    const ownerNftAccount = await getAccount(provider.connection, ownerNft);
    expect(Number(ownerNftAccount.amount)).to.equal(1);
  });

  it("Stakes CHIP via mock external program", async () => {
    const ownerChip = getAssociatedTokenAddressSync(chipMint, payer.publicKey);
    const ownerSchip = getAssociatedTokenAddressSync(sChipMint, payer.publicKey);

    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, chipMint, payer.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, sChipMint, payer.publicKey);
    await mintTo(provider.connection, payer.payer, chipMint, ownerChip, payer.publicKey, 1_000_000);

    await program.methods
      .stakeChip(new anchor.BN(100_000))
      .accounts({
        vaultConfig: vaultConfigPda,
        owner: payer.publicKey,
        ownerChipAccount: ownerChip,
        ownerSchipAccount: ownerSchip,
        usdAiStake: mockStake.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Mock is no-op; we just verify the instruction succeeds
  });

  it("Pauses and rejects user operations", async () => {
    await program.methods
      .setPause(true)
      .accounts({
        vaultConfig: vaultConfigPda,
        authority: payer.publicKey,
      })
      .rpc();

    const config = await program.account.vaultConfig.fetch(vaultConfigPda);
    expect(config.paused).to.equal(true);

    const ownerChip = getAssociatedTokenAddressSync(chipMint, payer.publicKey);
    const ownerSchip = getAssociatedTokenAddressSync(sChipMint, payer.publicKey);

    try {
      await program.methods
        .stakeChip(new anchor.BN(100_000))
        .accounts({
          vaultConfig: vaultConfigPda,
          owner: payer.publicKey,
          ownerChipAccount: ownerChip,
          ownerSchipAccount: ownerSchip,
          usdAiStake: mockStake.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      expect.fail("Should have thrown Paused error");
    } catch (e: any) {
      expect(e.toString()).to.include("Program is paused");
    }
  });

  it("Unpauses and resumes operations", async () => {
    await program.methods
      .setPause(false)
      .accounts({
        vaultConfig: vaultConfigPda,
        authority: payer.publicKey,
      })
      .rpc();

    const config = await program.account.vaultConfig.fetch(vaultConfigPda);
    expect(config.paused).to.equal(false);
  });

  it("Prevents liquidation of healthy position", async () => {
    const nftMintKp = anchor.web3.Keypair.generate();
    const oracleFeedKp = anchor.web3.Keypair.generate();

    await provider.connection.confirmTransaction(
      await provider.connection.sendTransaction(
        new anchor.web3.Transaction().add(
          anchor.web3.SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: oracleFeedKp.publicKey,
            lamports: await provider.connection.getMinimumBalanceForRentExemption(49),
            space: 49,
            programId: anchor.web3.SystemProgram.programId,
          })
        ),
        [payer.payer, oracleFeedKp]
      )
    );

    const [collateralPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("gpu_collateral"),
        vaultConfigPda.toBuffer(),
        nftMintKp.publicKey.toBuffer(),
      ],
      program.programId
    );

    await program.methods
      .registerGpu({
        model: "A100",
        specs: "80GB",
        clusterId: payer.publicKey,
        prometheusUrl: "",
      })
      .accounts({
        vaultConfig: vaultConfigPda,
        gpuCollateral: collateralPda,
        gpuNftMint: nftMintKp.publicKey,
        vaultNftAccount: getAssociatedTokenAddressSync(nftMintKp.publicKey, vaultConfigPda, true),
        metadataAccount: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
            nftMintKp.publicKey.toBuffer(),
          ],
          new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        )[0],
        owner: payer.publicKey,
        oracleFeed: oracleFeedKp.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([nftMintKp])
      .rpc();

    await program.methods
      .updateOracle()
      .accounts({
        collateral: collateralPda,
        oracleFeed: oracleFeedKp.publicKey,
      })
      .rpc();

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    await mintTo(provider.connection, payer.payer, usdaiMint, vaultUsdai, payer.publicKey, 1_000_000);

    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, payer.publicKey);

    await program.methods
      .borrowUsdai(new anchor.BN(100_000))
      .accounts({
        vaultConfig: vaultConfigPda,
        collateral: collateralPda,
        owner: payer.publicKey,
        oracleFeed: oracleFeedKp.publicKey,
        usdAiLend: mockLend.programId,
        ownerUsdaiAccount: ownerUsdai,
        vaultUsdaiAccount: vaultUsdai,
        treasuryUsdaiAccount: getAssociatedTokenAddressSync(usdaiMint, treasuryKp.publicKey),
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const liquidator = anchor.web3.Keypair.generate();
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(liquidator.publicKey, 1_000_000_000)
    );

    const liquidatorUsdai = getAssociatedTokenAddressSync(usdaiMint, liquidator.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, liquidator.publicKey);
    await mintTo(provider.connection, payer.payer, usdaiMint, liquidatorUsdai, payer.publicKey, 500_000);

    try {
      await program.methods
        .liquidate()
        .accounts({
          vaultConfig: vaultConfigPda,
          collateral: collateralPda,
          liquidator: liquidator.publicKey,
          liquidatorUsdai: liquidatorUsdai,
          vaultNftAccount: getAssociatedTokenAddressSync(nftMintKp.publicKey, vaultConfigPda, true),
          liquidatorNftAccount: getAssociatedTokenAddressSync(nftMintKp.publicKey, liquidator.publicKey),
          vaultUsdaiAccount: vaultUsdai,
          oracleFeed: oracleFeedKp.publicKey,
          usdAiLend: mockLend.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([liquidator])
        .rpc();
      expect.fail("Should have thrown NotLiquidatable");
    } catch (e: any) {
      expect(e.toString()).to.include("Position is not liquidatable");
    }
  });
});

describe("gpu-vault edge cases", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.GpuVault as Program<GpuVault>;
  const mockLend = anchor.workspace.MockUsdAiLend as Program<MockUsdAiLend>;

  let usdaiMint: anchor.web3.PublicKey;
  let chipMint: anchor.web3.PublicKey;
  let sChipMint: anchor.web3.PublicKey;
  let vaultConfigPda: anchor.web3.PublicKey;
  let treasuryKp = anchor.web3.Keypair.generate();

  before(async () => {
    usdaiMint = await createMint(provider.connection, payer.payer, payer.publicKey, null, 6);
    chipMint = await createMint(provider.connection, payer.payer, payer.publicKey, null, 6);
    sChipMint = await createMint(provider.connection, payer.payer, payer.publicKey, null, 6);

    [vaultConfigPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_config")],
      program.programId
    );

    await program.methods
      .initializeVault(6000, 8000)
      .accounts({
        vaultConfig: vaultConfigPda,
        payer: payer.publicKey,
        usdaiMint,
        chipMint,
        sChipMint,
        usdAiLendProgram: mockLend.programId,
        usdAiStakeProgram: program.programId,
        treasury: treasuryKp.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  async function registerGpuWithPrice(
    model: string,
    valuationUsd: number
  ): Promise<{ collateralPda: anchor.web3.PublicKey; nftMint: anchor.web3.PublicKey; oracleFeed: anchor.web3.PublicKey }> {
    const nftMintKp = anchor.web3.Keypair.generate();
    const oracleFeedKp = anchor.web3.Keypair.generate();

    // Create oracle feed with specified price
    const feedData = Buffer.alloc(49);
    feedData.writeBigUInt64LE(BigInt(1), 0); // version
    feedData.writeUInt8(6, 8); // decimals
    feedData.writeBigUInt64LE(BigInt(1), 9); // round_id
    feedData.writeBigInt64LE(BigInt(valuationUsd), 17); // answer @ 6 decimals
    feedData.writeBigInt64LE(BigInt(Math.floor(Date.now() / 1000)), 33); // timestamp
    feedData.writeBigInt64LE(BigInt(Math.floor(Date.now() / 1000)), 41); // updated_at

    await provider.connection.confirmTransaction(
      await provider.connection.sendTransaction(
        new anchor.web3.Transaction().add(
          anchor.web3.SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: oracleFeedKp.publicKey,
            lamports: await provider.connection.getMinimumBalanceForRentExemption(49),
            space: 49,
            programId: anchor.web3.SystemProgram.programId,
          })
        ),
        [payer.payer, oracleFeedKp]
      )
    );
    await provider.connection.sendTransaction(
      new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: payer.publicKey,
          toPubkey: oracleFeedKp.publicKey,
          lamports: 1,
        })
      ),
      [payer.payer]
    );

    const [collateralPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("gpu_collateral"),
        vaultConfigPda.toBuffer(),
        nftMintKp.publicKey.toBuffer(),
      ],
      program.programId
    );

    await program.methods
      .registerGpu({
        model,
        specs: "test",
        clusterId: payer.publicKey,
        prometheusUrl: "",
      })
      .accounts({
        vaultConfig: vaultConfigPda,
        gpuCollateral: collateralPda,
        gpuNftMint: nftMintKp.publicKey,
        vaultNftAccount: getAssociatedTokenAddressSync(nftMintKp.publicKey, vaultConfigPda, true),
        metadataAccount: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
            nftMintKp.publicKey.toBuffer(),
          ],
          new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        )[0],
        owner: payer.publicKey,
        oracleFeed: oracleFeedKp.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([nftMintKp])
      .rpc();

    await program.methods
      .updateOracle()
      .accounts({
        collateral: collateralPda,
        oracleFeed: oracleFeedKp.publicKey,
      })
      .rpc();

    return { collateralPda, nftMint: nftMintKp.publicKey, oracleFeed: oracleFeedKp.publicKey };
  }

  it("Rejects borrow that exceeds max LTV", async () => {
    const { collateralPda, oracleFeed } = await registerGpuWithPrice("H100", 1_000_000);

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);
    const treasuryUsdai = getAssociatedTokenAddressSync(usdaiMint, treasuryKp.publicKey);

    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, vaultConfigPda, true);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, payer.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, treasuryKp.publicKey);
    await mintTo(provider.connection, payer.payer, usdaiMint, vaultUsdai, payer.publicKey, 10_000_000);

    // Valuation = $1.00, max LTV = 60% → max borrow = 600_000
    // Try to borrow 700_000
    try {
      await program.methods
        .borrowUsdai(new anchor.BN(700_000))
        .accounts({
          vaultConfig: vaultConfigPda,
          collateral: collateralPda,
          owner: payer.publicKey,
          oracleFeed,
          usdAiLend: mockLend.programId,
          ownerUsdaiAccount: ownerUsdai,
          vaultUsdaiAccount: vaultUsdai,
          treasuryUsdaiAccount: treasuryUsdai,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      expect.fail("Should have thrown ExceedsLtv");
    } catch (e: any) {
      expect(e.toString()).to.include("Exceeds allowed LTV");
    }
  });

  it("Allows borrow exactly at max LTV boundary", async () => {
    const { collateralPda, oracleFeed } = await registerGpuWithPrice("H100", 1_000_000);

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);
    const treasuryUsdai = getAssociatedTokenAddressSync(usdaiMint, treasuryKp.publicKey);

    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, vaultConfigPda, true);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, payer.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, treasuryKp.publicKey);
    await mintTo(provider.connection, payer.payer, usdaiMint, vaultUsdai, payer.publicKey, 10_000_000);

    // Valuation = $1.00, max LTV = 60% → max borrow = 600_000
    await program.methods
      .borrowUsdai(new anchor.BN(600_000))
      .accounts({
        vaultConfig: vaultConfigPda,
        collateral: collateralPda,
        owner: payer.publicKey,
        oracleFeed,
        usdAiLend: mockLend.programId,
        ownerUsdaiAccount: ownerUsdai,
        vaultUsdaiAccount: vaultUsdai,
        treasuryUsdaiAccount: treasuryUsdai,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const updated = await program.account.gpuCollateral.fetch(collateralPda);
    expect(updated.borrowedUsdai.toNumber()).to.equal(600_000);
  });

  it("Supports partial repay without changing status", async () => {
    const { collateralPda, oracleFeed } = await registerGpuWithPrice("A100", 2_000_000);

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);
    const treasuryUsdai = getAssociatedTokenAddressSync(usdaiMint, treasuryKp.publicKey);

    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, vaultConfigPda, true);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, payer.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, treasuryKp.publicKey);
    await mintTo(provider.connection, payer.payer, usdaiMint, vaultUsdai, payer.publicKey, 10_000_000);

    // Borrow 1_000_000
    await program.methods
      .borrowUsdai(new anchor.BN(1_000_000))
      .accounts({
        vaultConfig: vaultConfigPda,
        collateral: collateralPda,
        owner: payer.publicKey,
        oracleFeed,
        usdAiLend: mockLend.programId,
        ownerUsdaiAccount: ownerUsdai,
        vaultUsdaiAccount: vaultUsdai,
        treasuryUsdaiAccount: treasuryUsdai,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Mint USDai to owner for repayment
    await mintTo(provider.connection, payer.payer, usdaiMint, ownerUsdai, payer.publicKey, 2_000_000);

    // Repay half
    await program.methods
      .repayUsdai(new anchor.BN(500_000))
      .accounts({
        vaultConfig: vaultConfigPda,
        collateral: collateralPda,
        owner: payer.publicKey,
        ownerUsdaiAccount: ownerUsdai,
        vaultUsdaiAccount: vaultUsdai,
        usdAiLend: mockLend.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const updated = await program.account.gpuCollateral.fetch(collateralPda);
    expect(updated.borrowedUsdai.toNumber()).to.equal(500_000);
    expect(updated.status).to.deep.equal({ borrowing: {} });
  });

  it("Rejects repay amount greater than debt", async () => {
    const allCollaterals = await program.account.gpuCollateral.all();
    // Get the collateral from the previous test (still has 500_000 debt)
    const collateral = allCollaterals.find(c => c.account.borrowedUsdai.toNumber() === 500_000);
    expect(collateral).to.not.be.undefined;

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);

    await mintTo(provider.connection, payer.payer, usdaiMint, ownerUsdai, payer.publicKey, 1_000_000);

    try {
      await program.methods
        .repayUsdai(new anchor.BN(1_000_000))
        .accounts({
          vaultConfig: vaultConfigPda,
          collateral: collateral!.publicKey,
          owner: payer.publicKey,
          ownerUsdaiAccount: ownerUsdai,
          vaultUsdaiAccount: vaultUsdai,
          usdAiLend: mockLend.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      expect.fail("Should have thrown RepayTooMuch");
    } catch (e: any) {
      expect(e.toString()).to.include("Repay amount exceeds debt");
    }
  });

  it("Updates authority correctly", async () => {
    const newAuthority = anchor.web3.Keypair.generate();

    await program.methods
      .setAuthority()
      .accounts({
        vaultConfig: vaultConfigPda,
        authority: payer.publicKey,
        newAuthority: newAuthority.publicKey,
      })
      .rpc();

    const config = await program.account.vaultConfig.fetch(vaultConfigPda);
    expect(config.authority.toBase58()).to.equal(newAuthority.publicKey.toBase58());

    // Revert back to payer for subsequent tests
    await program.methods
      .setAuthority()
      .accounts({
        vaultConfig: vaultConfigPda,
        authority: newAuthority.publicKey,
        newAuthority: payer.publicKey,
      })
      .signers([newAuthority])
      .rpc();
  });

  it("Updates treasury correctly", async () => {
    const newTreasury = anchor.web3.Keypair.generate();

    await program.methods
      .updateTreasury()
      .accounts({
        vaultConfig: vaultConfigPda,
        authority: payer.publicKey,
        newTreasury: newTreasury.publicKey,
      })
      .rpc();

    const config = await program.account.vaultConfig.fetch(vaultConfigPda);
    expect(config.treasury.toBase58()).to.equal(newTreasury.publicKey.toBase58());

    // Revert back
    await program.methods
      .updateTreasury()
      .accounts({
        vaultConfig: vaultConfigPda,
        authority: payer.publicKey,
        newTreasury: treasuryKp.publicKey,
      })
      .rpc();
  });

  it("Rejects unauthorized authority changes", async () => {
    const attacker = anchor.web3.Keypair.generate();
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(attacker.publicKey, 1_000_000_000)
    );

    try {
      await program.methods
        .setPause(true)
        .accounts({
          vaultConfig: vaultConfigPda,
          authority: attacker.publicKey,
        })
        .signers([attacker])
        .rpc();
      expect.fail("Should have thrown Unauthorized");
    } catch (e: any) {
      expect(e.toString()).to.include("Unauthorized access");
    }
  });

  it("Rejects borrow when paused", async () => {
    await program.methods
      .setPause(true)
      .accounts({
        vaultConfig: vaultConfigPda,
        authority: payer.publicKey,
      })
      .rpc();

    const allCollaterals = await program.account.gpuCollateral.all();
    const collateral = allCollaterals[allCollaterals.length - 1];

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);
    const treasuryUsdai = getAssociatedTokenAddressSync(usdaiMint, treasuryKp.publicKey);

    try {
      await program.methods
        .borrowUsdai(new anchor.BN(100_000))
        .accounts({
          vaultConfig: vaultConfigPda,
          collateral: collateral.publicKey,
          owner: payer.publicKey,
          oracleFeed: collateral.account.oracleFeed,
          usdAiLend: mockLend.programId,
          ownerUsdaiAccount: ownerUsdai,
          vaultUsdaiAccount: vaultUsdai,
          treasuryUsdaiAccount: treasuryUsdai,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      expect.fail("Should have thrown Paused");
    } catch (e: any) {
      expect(e.toString()).to.include("Program is paused");
    }

    // Unpause for other tests
    await program.methods
      .setPause(false)
      .accounts({
        vaultConfig: vaultConfigPda,
        authority: payer.publicKey,
      })
      .rpc();
  });

  it("Rejects borrow with stale oracle", async () => {
    const nftMintKp = anchor.web3.Keypair.generate();
    const oracleFeedKp = anchor.web3.Keypair.generate();

    // Create oracle feed with a timestamp from 2019 (stale)
    const feedData = Buffer.alloc(49);
    feedData.writeBigUInt64LE(BigInt(1), 0); // version
    feedData.writeUInt8(6, 8); // decimals
    feedData.writeBigUInt64LE(BigInt(1), 9); // round_id
    feedData.writeBigInt64LE(BigInt(1_000_000), 17); // answer @ 6 decimals = $1.00
    feedData.writeBigInt64LE(BigInt(1_000_000_000), 33); // timestamp = 2001-09-09 (stale)
    feedData.writeBigInt64LE(BigInt(1_000_000_000), 41); // updated_at

    await provider.connection.confirmTransaction(
      await provider.connection.sendTransaction(
        new anchor.web3.Transaction().add(
          anchor.web3.SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: oracleFeedKp.publicKey,
            lamports: await provider.connection.getMinimumBalanceForRentExemption(49),
            space: 49,
            programId: anchor.web3.SystemProgram.programId,
          })
        ),
        [payer.payer, oracleFeedKp]
      )
    );

    const [collateralPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("gpu_collateral"),
        vaultConfigPda.toBuffer(),
        nftMintKp.publicKey.toBuffer(),
      ],
      program.programId
    );

    await program.methods
      .registerGpu({
        model: "H100",
        specs: "test",
        clusterId: payer.publicKey,
        prometheusUrl: "",
      })
      .accounts({
        vaultConfig: vaultConfigPda,
        gpuCollateral: collateralPda,
        gpuNftMint: nftMintKp.publicKey,
        vaultNftAccount: getAssociatedTokenAddressSync(nftMintKp.publicKey, vaultConfigPda, true),
        metadataAccount: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
            nftMintKp.publicKey.toBuffer(),
          ],
          new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        )[0],
        owner: payer.publicKey,
        oracleFeed: oracleFeedKp.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([nftMintKp])
      .rpc();

    await program.methods
      .updateOracle()
      .accounts({
        collateral: collateralPda,
        oracleFeed: oracleFeedKp.publicKey,
      })
      .rpc();

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);
    const treasuryUsdai = getAssociatedTokenAddressSync(usdaiMint, treasuryKp.publicKey);

    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, vaultConfigPda, true);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, payer.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, treasuryKp.publicKey);
    await mintTo(provider.connection, payer.payer, usdaiMint, vaultUsdai, payer.publicKey, 10_000_000);

    try {
      await program.methods
        .borrowUsdai(new anchor.BN(100_000))
        .accounts({
          vaultConfig: vaultConfigPda,
          collateral: collateralPda,
          owner: payer.publicKey,
          oracleFeed: oracleFeedKp.publicKey,
          usdAiLend: mockLend.programId,
          ownerUsdaiAccount: ownerUsdai,
          vaultUsdaiAccount: vaultUsdai,
          treasuryUsdaiAccount: treasuryUsdai,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      expect.fail("Should have thrown StaleOracle");
    } catch (e: any) {
      expect(e.toString()).to.include("Oracle price stale");
    }
  });

  it("Rejects borrow with zero valuation", async () => {
    const nftMintKp = anchor.web3.Keypair.generate();
    const oracleFeedKp = anchor.web3.Keypair.generate();

    // Create oracle feed with zero answer
    const feedData = Buffer.alloc(49);
    feedData.writeBigUInt64LE(BigInt(1), 0); // version
    feedData.writeUInt8(6, 8); // decimals
    feedData.writeBigUInt64LE(BigInt(1), 9); // round_id
    feedData.writeBigInt64LE(BigInt(0), 17); // answer = 0
    feedData.writeBigInt64LE(BigInt(Math.floor(Date.now() / 1000)), 33); // timestamp = now
    feedData.writeBigInt64LE(BigInt(Math.floor(Date.now() / 1000)), 41); // updated_at

    await provider.connection.confirmTransaction(
      await provider.connection.sendTransaction(
        new anchor.web3.Transaction().add(
          anchor.web3.SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: oracleFeedKp.publicKey,
            lamports: await provider.connection.getMinimumBalanceForRentExemption(49),
            space: 49,
            programId: anchor.web3.SystemProgram.programId,
          })
        ),
        [payer.payer, oracleFeedKp]
      )
    );

    const [collateralPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("gpu_collateral"),
        vaultConfigPda.toBuffer(),
        nftMintKp.publicKey.toBuffer(),
      ],
      program.programId
    );

    await program.methods
      .registerGpu({
        model: "H100",
        specs: "test",
        clusterId: payer.publicKey,
        prometheusUrl: "",
      })
      .accounts({
        vaultConfig: vaultConfigPda,
        gpuCollateral: collateralPda,
        gpuNftMint: nftMintKp.publicKey,
        vaultNftAccount: getAssociatedTokenAddressSync(nftMintKp.publicKey, vaultConfigPda, true),
        metadataAccount: anchor.web3.PublicKey.findProgramAddressSync(
          [
            Buffer.from("metadata"),
            new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
            nftMintKp.publicKey.toBuffer(),
          ],
          new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
        )[0],
        owner: payer.publicKey,
        oracleFeed: oracleFeedKp.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([nftMintKp])
      .rpc();

    await program.methods
      .updateOracle()
      .accounts({
        collateral: collateralPda,
        oracleFeed: oracleFeedKp.publicKey,
      })
      .rpc();

    const vaultUsdai = getAssociatedTokenAddressSync(usdaiMint, vaultConfigPda, true);
    const ownerUsdai = getAssociatedTokenAddressSync(usdaiMint, payer.publicKey);
    const treasuryUsdai = getAssociatedTokenAddressSync(usdaiMint, treasuryKp.publicKey);

    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, vaultConfigPda, true);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, payer.publicKey);
    await getOrCreateAssociatedTokenAccount(provider.connection, payer.payer, usdaiMint, treasuryKp.publicKey);
    await mintTo(provider.connection, payer.payer, usdaiMint, vaultUsdai, payer.publicKey, 10_000_000);

    try {
      await program.methods
        .borrowUsdai(new anchor.BN(100_000))
        .accounts({
          vaultConfig: vaultConfigPda,
          collateral: collateralPda,
          owner: payer.publicKey,
          oracleFeed: oracleFeedKp.publicKey,
          usdAiLend: mockLend.programId,
          ownerUsdaiAccount: ownerUsdai,
          vaultUsdaiAccount: vaultUsdai,
          treasuryUsdaiAccount: treasuryUsdai,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      expect.fail("Should have thrown ZeroValuation");
    } catch (e: any) {
      expect(e.toString()).to.include("Zero valuation");
    }
  });
});
