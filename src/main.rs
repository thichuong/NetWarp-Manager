#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod net_utils;
mod warp;
mod wifi;

use slint::{ComponentHandle, Model};
use std::rc::Rc;
use std::time::Instant;

slint::include_modules!();

/// Converts a size in bytes to a human-readable string (KB, MB, GB, etc.)
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Converts speed in KB/s to human-readable string
fn format_speed(kb_s: f64) -> String {
    if kb_s >= 1024.0 {
        format!("{:.2} MB/s", kb_s / 1024.0)
    } else {
        format!("{:.2} KB/s", kb_s)
    }
}

/// Helper function to append a system log to the UI logs terminal
fn append_log(ui: &AppWindow, message: &str) {
    let logs_rc = ui.get_console_logs();
    let mut logs: Vec<String> = logs_rc.iter().map(|s| s.to_string()).collect();

    // Add timestamp prefix
    let local_time = chrono::Local::now();
    let time_str = local_time.format("[%H:%M:%S]").to_string();
    logs.push(format!("{} {}", time_str, message));

    // Maintain maximum 100 log lines to save memory
    if logs.len() > 100 {
        logs.remove(0);
    }

    let slint_logs: Vec<slint::SharedString> =
        logs.into_iter().map(slint::SharedString::from).collect();
    ui.set_console_logs(Rc::new(slint::VecModel::from(slint_logs)).into());
}

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    // 1. Initialize the main Slint UI Application Window
    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();

    // 2. Setup Vector Models to handle dynamic arrays on Slint UI
    let wifi_list_model = Rc::new(slint::VecModel::<WifiNetwork>::default());
    ui.set_wifi_list(wifi_list_model.clone().into());

    let download_history_model = Rc::new(slint::VecModel::<f32>::from(vec![0.0; 25]));
    ui.set_download_history(download_history_model.clone().into());

    let upload_history_model = Rc::new(slint::VecModel::<f32>::from(vec![0.0; 25]));
    ui.set_upload_history(upload_history_model.clone().into());

    // 3. Register UI callbacks interacting with backend logic modules

    // Close all modal overlays
    let ui_close_weak = ui_weak.clone();
    ui.on_close_modals(move || {
        if let Some(ui) = ui_close_weak.upgrade() {
            ui.set_show_wifi_modal(false);
            ui.set_show_password_modal(false);
        }
    });

    // Trigger Wi-Fi network change list
    let ui_change_weak = ui_weak.clone();
    ui.on_change_network_clicked(move || {
        if let Some(ui) = ui_change_weak.upgrade() {
            ui.set_show_wifi_modal(true);
            ui.set_is_scanning(true);
            append_log(&ui, "[Wi-Fi] Initiating active airwaves scan...");

            let ui_inner_weak = ui_change_weak.clone();

            // Execute Wi-Fi scan in background thread
            tokio::spawn(async move {
                match wifi::get_wifi_list().await {
                    Ok(list) => {
                        let slint_list: Vec<WifiNetwork> = list
                            .into_iter()
                            .map(|net| WifiNetwork {
                                bssid: net.bssid.into(),
                                ssid: net.ssid.into(),
                                channel: net.channel,
                                frequency: net.frequency.into(),
                                band: net.band.into(),
                                signal: net.signal,
                                security: net.security.into(),
                                active: net.active,
                                rate: net.rate.unwrap_or_default().into(),
                                device: net.device.unwrap_or_default().into(),
                                mac: net.mac.unwrap_or_default().into(),
                                ip_address: net.ip_address.unwrap_or_default().into(),
                                gateway: net.gateway.unwrap_or_default().into(),
                                dns_primary: net.dns_primary.unwrap_or_default().into(),
                                dns_secondary: net.dns_secondary.unwrap_or_default().into(),
                            })
                            .collect();

                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            let new_model = Rc::new(slint::VecModel::from(slint_list));
                            ui.set_wifi_list(new_model.into());
                            ui.set_is_scanning(false);
                            append_log(&ui, "[Wi-Fi] Airwaves scan completed successfully.");
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            ui.set_is_scanning(false);
                            append_log(&ui, &format!("[Wi-Fi] Scan failed: {}", e));
                        });
                    }
                }
            });
        }
    });

    // Re-trigger scanner range from modal
    let ui_scan_weak = ui_weak.clone();
    ui.on_scan_range_clicked(move || {
        if let Some(ui) = ui_scan_weak.upgrade() {
            ui.set_is_scanning(true);
            append_log(&ui, "[Wi-Fi] Scanning nearby frequencies...");

            let ui_inner_weak = ui_scan_weak.clone();

            tokio::spawn(async move {
                match wifi::get_wifi_list().await {
                    Ok(list) => {
                        let slint_list: Vec<WifiNetwork> = list
                            .into_iter()
                            .map(|net| WifiNetwork {
                                bssid: net.bssid.into(),
                                ssid: net.ssid.into(),
                                channel: net.channel,
                                frequency: net.frequency.into(),
                                band: net.band.into(),
                                signal: net.signal,
                                security: net.security.into(),
                                active: net.active,
                                rate: net.rate.unwrap_or_default().into(),
                                device: net.device.unwrap_or_default().into(),
                                mac: net.mac.unwrap_or_default().into(),
                                ip_address: net.ip_address.unwrap_or_default().into(),
                                gateway: net.gateway.unwrap_or_default().into(),
                                dns_primary: net.dns_primary.unwrap_or_default().into(),
                                dns_secondary: net.dns_secondary.unwrap_or_default().into(),
                            })
                            .collect();

                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            let new_model = Rc::new(slint::VecModel::from(slint_list));
                            ui.set_wifi_list(new_model.into());
                            ui.set_is_scanning(false);
                            append_log(&ui, "[Wi-Fi] Scan range completed.");
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            ui.set_is_scanning(false);
                            append_log(&ui, &format!("[Wi-Fi] Scan range failed: {}", e));
                        });
                    }
                }
            });
        }
    });

    // Selecting a Wi-Fi network from the list modal
    let ui_select_weak = ui_weak.clone();
    ui.on_wifi_selected(move |ssid, bssid| {
        if let Some(ui) = ui_select_weak.upgrade() {
            ui.set_selected_wifi_ssid(ssid.clone());
            ui.set_selected_wifi_bssid(bssid.clone());
            ui.set_show_password_modal(true);
            ui.set_pwd_input_val("".into()); // Clear old input

            append_log(&ui, &format!("[Wi-Fi] Selected AP: {} ({})", ssid, bssid));

            // Background load saved details if profile already exists
            let ui_inner_weak = ui_select_weak.clone();
            let ssid_str = ssid.to_string();
            tokio::spawn(async move {
                let saved_pwd = wifi::get_wifi_password(ssid_str.clone())
                    .await
                    .unwrap_or_default();
                let locked_bssid = wifi::get_wifi_locked_bssid(ssid_str)
                    .await
                    .unwrap_or_default();
                let has_lock = !locked_bssid.trim().is_empty();

                let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                    if !saved_pwd.is_empty() {
                        ui.set_pwd_input_val(saved_pwd.into());
                        append_log(
                            &ui,
                            "[Wi-Fi] Saved security key loaded from system keyring.",
                        );
                    }
                    ui.set_lock_bssid(has_lock);
                });
            });
        }
    });

    // Submitting password to connect to Wi-Fi
    let ui_conn_weak = ui_weak.clone();
    ui.on_connect_wifi_clicked(move |bssid, ssid, pwd, lock| {
        if let Some(ui) = ui_conn_weak.upgrade() {
            ui.set_show_password_modal(false);
            ui.set_show_wifi_modal(false);
            append_log(&ui, &format!("[Wi-Fi] Associating with SSID: {}...", ssid));

            let bssid_str = bssid.to_string();
            let ssid_str = ssid.to_string();
            let pwd_opt = if pwd.trim().is_empty() {
                None
            } else {
                Some(pwd.to_string())
            };

            let ui_inner_weak = ui_conn_weak.clone();
            tokio::spawn(async move {
                match wifi::connect_wifi(bssid_str, ssid_str, pwd_opt, lock).await {
                    Ok(success_msg) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            append_log(
                                &ui,
                                &format!("[Wi-Fi] Association successful: {}", success_msg),
                            );

                            // Trigger immediate active connection refresh
                            let ui_refresh_weak = ui.as_weak();
                            tokio::spawn(async move {
                                if let Ok(Some(active)) = wifi::get_active_wifi().await {
                                    let slint_active = WifiNetwork {
                                        bssid: active.bssid.into(),
                                        ssid: active.ssid.into(),
                                        channel: active.channel,
                                        frequency: active.frequency.into(),
                                        band: active.band.into(),
                                        signal: active.signal,
                                        security: active.security.into(),
                                        active: active.active,
                                        rate: active.rate.unwrap_or_default().into(),
                                        device: active.device.unwrap_or_default().into(),
                                        mac: active.mac.unwrap_or_default().into(),
                                        ip_address: active.ip_address.unwrap_or_default().into(),
                                        gateway: active.gateway.unwrap_or_default().into(),
                                        dns_primary: active.dns_primary.unwrap_or_default().into(),
                                        dns_secondary: active
                                            .dns_secondary
                                            .unwrap_or_default()
                                            .into(),
                                    };
                                    let _ = ui_refresh_weak.upgrade_in_event_loop(move |ui| {
                                        ui.set_active_wifi(slint_active);
                                    });
                                }
                            });
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            append_log(&ui, &format!("[Wi-Fi] Association failed: {}", e));
                        });
                    }
                }
            });
        }
    });

    // Cloudflare WARP Toggle connection switch
    let ui_warp_weak = ui_weak.clone();
    ui.on_warp_toggle_clicked(move |connect| {
        if let Some(ui) = ui_warp_weak.upgrade() {
            let state_str = if connect {
                "connecting"
            } else {
                "disconnecting"
            };
            append_log(&ui, &format!("[WARP] Triggering client {}...", state_str));
            ui.set_warp_status_text("Connecting...".into());
            ui.set_warp_status_color("#f59e0b".into()); // Orange pulse

            let ui_inner_weak = ui_warp_weak.clone();
            tokio::spawn(async move {
                match warp::warp_toggle(connect).await {
                    Ok(msg) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            append_log(&ui, &format!("[WARP] Operation finished: {}", msg));
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            append_log(&ui, &format!("[WARP] Operation failed: {}", e));
                            ui.set_warp_status_text("Error".into());
                            ui.set_warp_status_color("#f43f5e".into()); // Red error
                        });
                    }
                }
            });
        }
    });

    // Cloudflare WARP tunnel mode switch
    let ui_mode_weak = ui_weak.clone();
    ui.on_warp_mode_clicked(move |mode| {
        if let Some(ui) = ui_mode_weak.upgrade() {
            append_log(
                &ui,
                &format!("[WARP] Configuring operating tunnel mode to: {}...", mode),
            );
            let mode_str = mode.to_string();

            let ui_inner_weak = ui_mode_weak.clone();
            tokio::spawn(async move {
                match warp::set_warp_mode(mode_str).await {
                    Ok(msg) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            append_log(&ui, &format!("[WARP] Operating mode changed: {}", msg));
                        });

                        // Query and update warp mode details immediately
                        let warp_mode = warp::get_warp_mode()
                            .await
                            .unwrap_or_else(|_| "DoH".to_string());

                        let ui_mode_update = ui_inner_weak.clone();
                        let warp_mode_clone = warp_mode.clone();
                        let _ = ui_mode_update.upgrade_in_event_loop(move |ui| {
                            ui.set_warp_mode_badge(format!("Mode: {}", warp_mode_clone).into());
                            ui.set_warp_mode_doh_active(
                                !warp_mode_clone.to_lowercase().contains("warp"),
                            );
                        });

                        // 1. Immediately refresh Public IP & Geolocation
                        let ui_geo = ui_inner_weak.clone();
                        tokio::spawn(async move {
                            if let Ok(raw_json) = net_utils::trace_ip().await {
                                if let Ok(parsed) =
                                    serde_json::from_str::<serde_json::Value>(&raw_json)
                                {
                                    let ip = parsed
                                        .get("query")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("Unknown")
                                        .to_string();
                                    let isp = parsed
                                        .get("isp")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("Unknown")
                                        .to_string();
                                    let city = parsed
                                        .get("city")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let country = parsed
                                        .get("country")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let location = if city.is_empty() {
                                        country
                                    } else {
                                        format!("{}, {}", city, country)
                                    };
                                    let lat =
                                        parsed.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let lon =
                                        parsed.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let coords = format!("{:.4}, {:.4}", lat, lon);

                                    let is_warp = isp.to_lowercase().contains("cloudflare");
                                    let badge = if is_warp { "WARP" } else { "DIRECT" };

                                    let slint_geo = IPGeolocatorInfo {
                                        ip: ip.into(),
                                        isp: isp.clone().into(),
                                        location: location.into(),
                                        coordinates: coords.into(),
                                        warp_badge: badge.into(),
                                    };
                                    let _ = ui_geo.upgrade_in_event_loop(move |ui| {
                                        ui.set_geo_info(slint_geo);
                                        append_log(
                                            &ui,
                                            &format!(
                                                "[GeoIP] Coordinates synced. ISP: {} ({})",
                                                isp, badge
                                            ),
                                        );
                                    });
                                }
                            }
                        });

                        // 2. Immediately refresh Ping latencies
                        let ui_ping = ui_inner_weak.clone();
                        tokio::spawn(async move {
                            let targets = vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()];
                            if let Ok(results) = net_utils::ping_multiple(targets).await {
                                let _ = ui_ping.upgrade_in_event_loop(move |ui| {
                                    for res in results {
                                        let slint_res = PingResult {
                                            target: res.target.clone().into(),
                                            latency: res.latency.unwrap_or(999.0) as f32,
                                            status: res.status.into(),
                                        };

                                        if res.target == "1.1.1.1" {
                                            ui.set_ping1(slint_res);
                                        } else if res.target == "8.8.8.8" {
                                            ui.set_ping2(slint_res);
                                        }
                                    }
                                    append_log(&ui, "[Diagnostics] Latency diagnostics refreshed.");
                                });
                            }
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            append_log(&ui, &format!("[WARP] Mode change failed: {}", e));
                        });
                    }
                }
            });
        }
    });

    // Install Cloudflare WARP Daemon package via Polkit
    let ui_install_weak = ui_weak.clone();
    ui.on_install_rpm_clicked(move || {
        if let Some(ui) = ui_install_weak.upgrade() {
            append_log(
                &ui,
                "[System] Initializing warp-cli Polkit deployment wrapper...",
            );

            let ui_inner_weak = ui_install_weak.clone();
            tokio::spawn(async move {
                match warp::install_warp().await {
                    Ok(msg) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            append_log(
                                &ui,
                                &format!("[System] Polkit deployment success: {}", msg),
                            );
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            append_log(&ui, &format!("[System] Polkit deployment failed: {}", e));
                        });
                    }
                }
            });
        }
    });

    // --- 4. Spawn Background Polling Task Loops via Tokio ---

    // Loop 1: Pulse animations timer (500ms intervals)
    let ui_pulse_weak = ui_weak.clone();
    tokio::spawn(async move {
        let mut pulse = false;
        let mut step = 0;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            pulse = !pulse;
            step = (step + 1) % 4;

            let _ = ui_pulse_weak.upgrade_in_event_loop(move |ui| {
                ui.set_pulse_led(pulse);
                ui.set_radar_step(step);
            });
        }
    });

    // Loop 2: Network Bandwidth IO speed monitoring (1 second interval)
    let ui_speed_weak = ui_weak.clone();
    tokio::spawn(async move {
        let mut last_rx = 0;
        let mut last_tx = 0;
        let mut last_time = Instant::now();
        let mut peak_download = 0.0;
        let mut peak_upload = 0.0;
        let mut total_session_usage = 0;
        let mut first_run = true;

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

            if let Ok(io) = net_utils::get_network_io().await {
                // Read exact bytes parsed directly from /proc/net/dev
                let rx = io.rx_bytes;
                let tx = io.tx_bytes;
                let now = Instant::now();
                let duration_sec = now.duration_since(last_time).as_secs_f64();

                if first_run {
                    last_rx = rx;
                    last_tx = tx;
                    last_time = now;
                    first_run = false;
                    continue;
                }

                // Compute instant download & upload speed
                let mut speed_dl_kb = 0.0;
                let mut speed_ul_kb = 0.0;

                if rx >= last_rx && duration_sec > 0.0 {
                    let diff_rx = rx - last_rx;
                    speed_dl_kb = (diff_rx as f64 / 1024.0) / duration_sec;
                    total_session_usage += diff_rx;
                }

                if tx >= last_tx && duration_sec > 0.0 {
                    let diff_tx = tx - last_tx;
                    speed_ul_kb = (diff_tx as f64 / 1024.0) / duration_sec;
                    total_session_usage += diff_tx;
                }

                last_rx = rx;
                last_tx = tx;
                last_time = now;

                // Adjust peak records
                if speed_dl_kb > peak_download {
                    peak_download = speed_dl_kb;
                }
                if speed_ul_kb > peak_upload {
                    peak_upload = speed_ul_kb;
                }

                let speed_stats = NetworkSpeed {
                    download_speed: format_speed(speed_dl_kb).into(),
                    upload_speed: format_speed(speed_ul_kb).into(),
                    peak_download: format_speed(peak_download).into(),
                    peak_upload: format_speed(peak_upload).into(),
                    total_usage: format_bytes(total_session_usage).into(),
                };

                // Push new history to rolling models of graph visualization
                let _ = ui_speed_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_speed_stats(speed_stats);

                    // Slide download model array
                    let dl_model = ui.get_download_history();
                    let mut dl_vec: Vec<f32> = dl_model.iter().collect();
                    if !dl_vec.is_empty() {
                        dl_vec.remove(0);
                    }
                    dl_vec.push(speed_dl_kb as f32);
                    ui.set_download_history(Rc::new(slint::VecModel::from(dl_vec)).into());

                    // Slide upload model array
                    let ul_model = ui.get_upload_history();
                    let mut ul_vec: Vec<f32> = ul_model.iter().collect();
                    if !ul_vec.is_empty() {
                        ul_vec.remove(0);
                    }
                    ul_vec.push(speed_ul_kb as f32);
                    ui.set_upload_history(Rc::new(slint::VecModel::from(ul_vec)).into());
                });
            }
        }
    });

    // Loop 3: Wi-Fi active interface, Cloudflare WARP Daemon status and Mode (1 second interval)
    let ui_status_weak = ui_weak.clone();
    tokio::spawn(async move {
        let mut last_warp_state = String::new();
        let mut last_warp_mode = String::new();
        let mut last_wifi_ssid = String::new();
        let mut geo_cooldown_counter = 0;

        loop {
            let mut current_wifi_ssid = String::new();

            // Polling active Wi-Fi connection
            if let Ok(active_opt) = wifi::get_active_wifi().await {
                if let Some(ref active) = active_opt {
                    current_wifi_ssid = active.ssid.clone();
                }

                let ui_wifi_weak = ui_status_weak.clone();
                let _ = ui_wifi_weak.upgrade_in_event_loop(move |ui| {
                    if let Some(active) = active_opt {
                        let slint_active = WifiNetwork {
                            bssid: active.bssid.into(),
                            ssid: active.ssid.into(),
                            channel: active.channel,
                            frequency: active.frequency.into(),
                            band: active.band.into(),
                            signal: active.signal,
                            security: active.security.into(),
                            active: active.active,
                            rate: active.rate.unwrap_or_default().into(),
                            device: active.device.unwrap_or_default().into(),
                            mac: active.mac.unwrap_or_default().into(),
                            ip_address: active.ip_address.unwrap_or_default().into(),
                            gateway: active.gateway.unwrap_or_default().into(),
                            dns_primary: active.dns_primary.unwrap_or_default().into(),
                            dns_secondary: active.dns_secondary.unwrap_or_default().into(),
                        };
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
                });
            }

            // Polling Cloudflare WARP Status
            let warp_status = warp::get_warp_status()
                .await
                .unwrap_or_else(|_| "Disconnected".to_string());
            let warp_mode = warp::get_warp_mode()
                .await
                .unwrap_or_else(|_| "DoH".to_string());

            let ui_warp_inner = ui_status_weak.clone();

            // Detect if connection state, warp mode, or wifi SSID changed to trigger immediate Geo IP refresh
            let state_changed = warp_status != last_warp_state
                || warp_mode != last_warp_mode
                || current_wifi_ssid != last_wifi_ssid
                || geo_cooldown_counter >= 30; // Periodic check every 30 seconds

            if state_changed {
                last_warp_state = warp_status.clone();
                last_warp_mode = warp_mode.clone();
                last_wifi_ssid = current_wifi_ssid.clone();
                geo_cooldown_counter = 0;
            } else {
                geo_cooldown_counter += 1;
            }

            let _ = ui_warp_inner.upgrade_in_event_loop(move |ui| {
                ui.set_warp_status_text(warp_status.clone().into());

                if warp_status.to_lowercase().contains("connected") {
                    ui.set_warp_status_color("#10b981".into()); // Green
                    ui.set_warp_network_text(
                        "Your network traffic is encrypted & protected.".into(),
                    );
                    ui.set_warp_toggle_state(true);
                } else if warp_status.to_lowercase().contains("connecting") {
                    ui.set_warp_status_color("#f59e0b".into()); // Orange pulse
                    ui.set_warp_network_text("Establishing secure Cloudflare tunnel...".into());
                    ui.set_warp_toggle_state(true);
                } else {
                    ui.set_warp_status_color("#f43f5e".into()); // Red
                    ui.set_warp_network_text(
                        "Your network traffic is direct & unprotected.".into(),
                    );
                    ui.set_warp_toggle_state(false);
                }

                ui.set_warp_mode_badge(format!("Mode: {}", warp_mode).into());
                ui.set_warp_mode_doh_active(!warp_mode.to_lowercase().contains("warp"));

                // Immediate trigger Geolocation curl fetch if state toggled
                if state_changed {
                    let ui_geo_trigger = ui.as_weak();
                    tokio::spawn(async move {
                        if let Ok(raw_json) = net_utils::trace_ip().await {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw_json)
                            {
                                let ip = parsed
                                    .get("query")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown")
                                    .to_string();
                                let isp = parsed
                                    .get("isp")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown")
                                    .to_string();
                                let city = parsed
                                    .get("city")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let country = parsed
                                    .get("country")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let location = if city.is_empty() {
                                    country
                                } else {
                                    format!("{}, {}", city, country)
                                };
                                let lat = parsed.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let lon = parsed.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let coords = format!("{:.4}, {:.4}", lat, lon);

                                let is_warp = isp.to_lowercase().contains("cloudflare");
                                let badge = if is_warp { "WARP" } else { "DIRECT" };

                                let slint_geo = IPGeolocatorInfo {
                                    ip: ip.into(),
                                    isp: isp.clone().into(),
                                    location: location.into(),
                                    coordinates: coords.into(),
                                    warp_badge: badge.into(),
                                };
                                let _ = ui_geo_trigger.upgrade_in_event_loop(move |ui| {
                                    ui.set_geo_info(slint_geo);
                                    append_log(
                                        &ui,
                                        &format!(
                                            "[GeoIP] Coordinates synced. ISP: {} ({})",
                                            isp, badge
                                        ),
                                    );
                                });
                            }
                        }
                    });
                }
            });

            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });

    // Loop 4: Ping Diagnostics Latencies (1 second interval)
    let ui_ping_weak = ui_weak.clone();
    tokio::spawn(async move {
        loop {
            let targets = vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()];
            if let Ok(results) = net_utils::ping_multiple(targets).await {
                let ui_ping_inner = ui_ping_weak.clone();
                let _ = ui_ping_inner.upgrade_in_event_loop(move |ui| {
                    for res in results {
                        let slint_res = PingResult {
                            target: res.target.clone().into(),
                            latency: res.latency.unwrap_or(999.0) as f32,
                            status: res.status.into(),
                        };

                        if res.target == "1.1.1.1" {
                            ui.set_ping1(slint_res);
                        } else if res.target == "8.8.8.8" {
                            ui.set_ping2(slint_res);
                        }
                    }
                });
            }
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    });

    // Loop 5: Initial Geolocation sync on application launch
    let ui_init_weak = ui_weak.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        if let Ok(raw_json) = net_utils::trace_ip().await {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw_json) {
                let ip = parsed
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let isp = parsed
                    .get("isp")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let city = parsed
                    .get("city")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let country = parsed
                    .get("country")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let location = if city.is_empty() {
                    country
                } else {
                    format!("{}, {}", city, country)
                };
                let lat = parsed.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let lon = parsed.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let coords = format!("{:.4}, {:.4}", lat, lon);

                let is_warp = isp.to_lowercase().contains("cloudflare");
                let badge = if is_warp { "WARP" } else { "DIRECT" };

                let slint_geo = IPGeolocatorInfo {
                    ip: ip.clone().into(),
                    isp: isp.into(),
                    location: location.into(),
                    coordinates: coords.into(),
                    warp_badge: badge.into(),
                };
                let _ = ui_init_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_geo_info(slint_geo);
                    append_log(
                        &ui,
                        &format!(
                            "[GeoIP] Initial launch sync complete. IP: {} ({})",
                            ip, badge
                        ),
                    );
                });
            }
        }
    });

    // 5. Run the Slint Event Loop (This blocks until the window is closed)
    ui.run()?;
    Ok(())
}
