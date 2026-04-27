import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

interface VaultConfig {
  authority: string;
  usdai_mint: string;
  chip_mint: string;
  s_chip_mint: string;
  max_ltv_bps: number;
  liquidation_ltv_bps: number;
  protocol_fee_bps: number;
  paused: boolean;
}

interface Collateral {
  mint: string;
  valuation_usd: number;
  borrowed_usdai: number;
  status: string;
}

async function loadVaultConfig() {
  try {
    const config = await invoke<VaultConfig>("get_vault_config", {});
    const el = document.getElementById("config-display")!;
    el.innerHTML = `
      <div><strong>Authority:</strong> ${config.authority}</div>
      <div><strong>USDai Mint:</strong> ${config.usdai_mint}</div>
      <div><strong>CHIP Mint:</strong> ${config.chip_mint}</div>
      <div><strong>sCHIP Mint:</strong> ${config.s_chip_mint}</div>
      <div><strong>Max LTV:</strong> ${(config.max_ltv_bps / 100).toFixed(0)}%</div>
      <div><strong>Liquidation LTV:</strong> ${(config.liquidation_ltv_bps / 100).toFixed(0)}%</div>
      <div><strong>Protocol Fee:</strong> ${(config.protocol_fee_bps / 100).toFixed(2)}%</div>
      <div><strong>Status:</strong> ${config.paused ? "🛑 PAUSED" : "✅ Active"}</div>
    `;
  } catch (e) {
    document.getElementById("config-display")!.textContent = "Error: " + String(e);
  }
}

async function loadCollaterals() {
  try {
    const collaterals = await invoke<Collateral[]>("get_gpu_collaterals", {});
    const list = document.getElementById("collateral-list")!;
    if (collaterals.length === 0) {
      list.innerHTML = "<p>No GPU collaterals found</p>";
      return;
    }
    list.innerHTML = collaterals.map((c) => `
      <div class="card">
        <div><strong>NFT Mint:</strong> ${c.mint}</div>
        <div><strong>Valuation:</strong> $${(c.valuation_usd / 1_000_000).toFixed(2)}</div>
        <div><strong>Borrowed:</strong> $${(c.borrowed_usdai / 1_000_000).toFixed(2)}</div>
        <div><strong>Status:</strong> ${c.status}</div>
      </div>
    `).join("");
  } catch (e) {
    document.getElementById("collateral-list")!.textContent = "Error: " + String(e);
  }
}

async function init() {
  const app = document.getElementById("app")!;
  app.innerHTML = `
    <header>
      <h1>USDai GPU Vault</h1>
      <div id="connection-status">Ready</div>
    </header>
    <main>
      <section id="vault-config">
        <h2>Vault Config</h2>
        <div id="config-display">Loading...</div>
      </section>
      <section id="collaterals">
        <h2>GPU Collaterals</h2>
        <div id="collateral-list">Loading...</div>
      </section>
      <section id="actions">
        <h2>Actions</h2>
        <button id="refresh">Refresh Data</button>
        <div class="hint">Connect wallet via CLI or backend to perform write operations.</div>
      </section>
    </main>
  `;

  document.getElementById("refresh")!.addEventListener("click", () => {
    loadVaultConfig();
    loadCollaterals();
  });

  await loadVaultConfig();
  await loadCollaterals();
}

init();
