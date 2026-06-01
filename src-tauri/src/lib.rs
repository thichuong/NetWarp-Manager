pub mod net_utils;
pub mod warp;
pub mod wifi;

use net_utils::{get_network_io, ping_multiple, ping_target, trace_ip};
use warp::{get_warp_mode, get_warp_status, install_warp, set_warp_mode, warp_toggle};
use wifi::{connect_wifi, get_wifi_list};

/// Main entry point for the Tauri application backend.
/// Configures custom plugins, initializes native event handlers, and exposes commands to the frontend.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_wifi_list,
            connect_wifi,
            install_warp,
            warp_toggle,
            get_warp_status,
            get_warp_mode,
            set_warp_mode,
            ping_target,
            trace_ip,
            get_network_io,
            ping_multiple
        ])
        .run(tauri::generate_context!())
    {
        eprintln!("error while running tauri application: {}", e);
    }
}
