#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod callbacks;
pub mod error;
mod helpers;
mod net_utils;
mod polling;
mod warp;
mod wifi;

pub use error::AppError;

use slint::ComponentHandle;
use std::rc::Rc;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    // 1. Initialize the main Slint UI Application Window
    let ui = AppWindow::new()?;

    // Set the detected OS Platform name for UI display
    ui.set_os_platform(helpers::detect_os_name().into());

    // 2. Setup Vector Models to handle dynamic arrays on Slint UI
    let wifi_list_model = Rc::new(slint::VecModel::<WifiNetwork>::default());
    ui.set_wifi_list(wifi_list_model.into());

    let download_history_model = Rc::new(slint::VecModel::<f32>::from(vec![0.0; 25]));
    ui.set_download_history(download_history_model.into());

    let upload_history_model = Rc::new(slint::VecModel::<f32>::from(vec![0.0; 25]));
    ui.set_upload_history(upload_history_model.into());

    // 3. Register UI callbacks
    callbacks::register_callbacks(&ui);

    // 4. Fetch initial states before starting window and loops
    let initial_warp_mode = warp::get_warp_mode()
        .await
        .unwrap_or_else(|_| "DoH".to_string());
    ui.set_warp_mode_badge(format!("Mode: {}", initial_warp_mode).into());
    ui.set_warp_mode_doh_active(!initial_warp_mode.to_lowercase().contains("warp"));

    let initial_warp_status = warp::get_warp_status()
        .await
        .unwrap_or_else(|_| "Disconnected".to_string());
    let init_lower = initial_warp_status.to_lowercase();
    let init_connected = init_lower.contains("connected");
    let init_connecting = init_lower.contains("connecting");
    let warp_state = slint::SharedString::from(&initial_warp_status);

    ui.set_warp_status_text(slint::SharedString::from(&initial_warp_status));
    if init_connected {
        ui.set_warp_network_text("Your network traffic is encrypted & protected.".into());
        ui.set_warp_toggle_state(true);
    } else if init_connecting {
        ui.set_warp_network_text("Establishing secure Cloudflare tunnel...".into());
        ui.set_warp_toggle_state(true);
    } else {
        ui.set_warp_network_text("Your network traffic is direct & unprotected.".into());
        ui.set_warp_toggle_state(false);
    }

    let mut wifi_ssid = slint::SharedString::new();
    let mut wifi_cache: Option<WifiNetwork> = None;

    if let Ok(Some(active_full)) = wifi::get_active_wifi(true).await {
        let slint_active = WifiNetwork {
            bssid: active_full.bssid.into(),
            ssid: active_full.ssid.into(),
            channel: active_full.channel,
            frequency: active_full.frequency.into(),
            band: active_full.band.into(),
            signal: active_full.signal,
            security: active_full.security.into(),
            active: active_full.active,
            rate: active_full.rate.unwrap_or_default().into(),
            device: active_full.device.unwrap_or_default().into(),
            mac: active_full.mac.unwrap_or_default().into(),
            ip_address: active_full.ip_address.unwrap_or_default().into(),
            gateway: active_full.gateway.unwrap_or_default().into(),
            dns_primary: active_full.dns_primary.unwrap_or_default().into(),
            dns_secondary: active_full.dns_secondary.unwrap_or_default().into(),
        };
        wifi_ssid = slint_active.ssid.clone();
        wifi_cache = Some(slint_active.clone());
        ui.set_active_wifi(slint_active);
    } else {
        ui.set_active_wifi(WifiNetwork {
            ssid: "Not Connected".into(),
            active: false,
            signal: 0,
            bssid: "--".into(),
            security: "--".into(),
            mac: "--".into(),
            device: "--".into(),
            ip_address: "--".into(),
            gateway: "--".into(),
            dns_primary: "--".into(),
            dns_secondary: "--".into(),
            rate: "--".into(),
            band: "--".into(),
            channel: 0,
            frequency: "--".into(),
        });
    }

    // 5. Start background polling loops passing initial states
    polling::start_polling_loops(&ui, warp_state, wifi_ssid, wifi_cache);

    // 6. Run the Slint Event Loop (This blocks until the window is closed)
    ui.run()?;
    Ok(())
}
