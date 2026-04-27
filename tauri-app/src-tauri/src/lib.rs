use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    rpc_url: Mutex<String>,
    program_id: Mutex<String>,
}

#[derive(Serialize)]
struct VaultConfigView {
    authority: String,
    usdai_mint: String,
    chip_mint: String,
    s_chip_mint: String,
    max_ltv_bps: u16,
    liquidation_ltv_bps: u16,
    protocol_fee_bps: u16,
    paused: bool,
}

#[derive(Serialize)]
struct CollateralView {
    mint: String,
    valuation_usd: u64,
    borrowed_usdai: u64,
    status: String,
}

#[tauri::command]
fn get_vault_config(state: State<AppState>) -> Result<VaultConfigView, String> {
    let program_id = state.program_id.lock().map_err(|e| e.to_string())?;
    let rpc = state.rpc_url.lock().map_err(|e| e.to_string())?;

    // TODO: Connect via solana-client, fetch account, deserialize with Borsh
    // For the scaffold we return a placeholder
    let _ = (&*program_id, &*rpc);
    Ok(VaultConfigView {
        authority: "11111111111111111111111111111111".to_string(),
        usdai_mint: "USDai1111111111111111111111111111111111111".to_string(),
        chip_mint: "CHiP11111111111111111111111111111111111111".to_string(),
        s_chip_mint: "sCHiP1111111111111111111111111111111111111".to_string(),
        max_ltv_bps: 7000,
        liquidation_ltv_bps: 8500,
        protocol_fee_bps: 10,
        paused: false,
    })
}

#[tauri::command]
fn get_gpu_collaterals(state: State<AppState>) -> Result<Vec<CollateralView>, String> {
    let _ = state.program_id.lock().map_err(|e| e.to_string())?;
    // TODO: Fetch program accounts via getProgramAccounts and filter by discriminator
    Ok(vec![])
}

#[tauri::command]
fn set_rpc_config(
    state: State<AppState>,
    rpc_url: String,
    program_id: String,
) -> Result<(), String> {
    *state.rpc_url.lock().map_err(|e| e.to_string())? = rpc_url;
    *state.program_id.lock().map_err(|e| e.to_string())? = program_id;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            rpc_url: Mutex::new("https://api.devnet.solana.com".to_string()),
            program_id: Mutex::new("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS".to_string()),
        })
        .invoke_handler(tauri::generate_handler![
            get_vault_config,
            get_gpu_collaterals,
            set_rpc_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
