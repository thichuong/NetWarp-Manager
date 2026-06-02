use crate::{AppWindow, WifiNetwork, helpers, warp, wifi};
use slint::ComponentHandle;
use std::rc::Rc;

/// Registers all user interaction callbacks from Slint UI to Rust backend logic.
pub fn register_callbacks(ui: &AppWindow) {
    let ui_weak = ui.as_weak();

    // Callback 1: Close all modal overlays
    let ui_close_weak = ui_weak.clone();
    ui.on_close_modals(move || {
        if let Some(ui) = ui_close_weak.upgrade() {
            ui.set_show_wifi_modal(false);
            ui.set_show_password_modal(false);
        }
    });

    // Callback 2: Toggle fullscreen mode
    let ui_fs_weak = ui_weak.clone();
    ui.on_toggle_fullscreen_clicked(move || {
        if let Some(ui) = ui_fs_weak.upgrade() {
            let is_fullscreen = ui.window().is_fullscreen();
            let next_fullscreen = !is_fullscreen;
            ui.window().set_fullscreen(next_fullscreen);
            ui.set_is_fullscreen(next_fullscreen);
            helpers::append_log(
                &ui,
                &format!(
                    "[System] Fullscreen toggled {}",
                    if next_fullscreen { "ON" } else { "OFF" }
                ),
            );
        }
    });

    // Callback 3: Trigger Wi-Fi network change list
    let ui_change_weak = ui_weak.clone();
    ui.on_change_network_clicked(move || {
        if let Some(ui) = ui_change_weak.upgrade() {
            ui.set_show_wifi_modal(true);
            ui.set_is_scanning(true);
            helpers::append_log(&ui, "[Wi-Fi] Initiating active airwaves scan...");

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
                            helpers::append_log(
                                &ui,
                                "[Wi-Fi] Airwaves scan completed successfully.",
                            );
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            ui.set_is_scanning(false);
                            helpers::append_log(&ui, &format!("[Wi-Fi] Scan failed: {}", e));
                        });
                    }
                }
            });
        }
    });

    // Callback 4: Re-trigger scanner range from modal
    let ui_scan_weak = ui_weak.clone();
    ui.on_scan_range_clicked(move || {
        if let Some(ui) = ui_scan_weak.upgrade() {
            ui.set_is_scanning(true);
            helpers::append_log(&ui, "[Wi-Fi] Scanning nearby frequencies...");

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
                            helpers::append_log(&ui, "[Wi-Fi] Scan range completed.");
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            ui.set_is_scanning(false);
                            helpers::append_log(&ui, &format!("[Wi-Fi] Scan range failed: {}", e));
                        });
                    }
                }
            });
        }
    });

    // Callback 5: Selecting a Wi-Fi network from the list modal
    let ui_select_weak = ui_weak.clone();
    ui.on_wifi_selected(move |ssid, bssid| {
        if let Some(ui) = ui_select_weak.upgrade() {
            ui.set_selected_wifi_ssid(ssid.clone());
            ui.set_selected_wifi_bssid(bssid.clone());
            ui.set_show_password_modal(true);
            ui.set_pwd_input_val("".into()); // Clear old input

            helpers::append_log(&ui, &format!("[Wi-Fi] Selected AP: {} ({})", ssid, bssid));

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
                        helpers::append_log(
                            &ui,
                            "[Wi-Fi] Saved security key loaded from system keyring.",
                        );
                    }
                    ui.set_lock_bssid(has_lock);
                });
            });
        }
    });

    // Callback 6: Submitting password to connect to Wi-Fi
    let ui_conn_weak = ui_weak.clone();
    ui.on_connect_wifi_clicked(move |bssid, ssid, pwd, lock| {
        if let Some(ui) = ui_conn_weak.upgrade() {
            ui.set_show_password_modal(false);
            ui.set_show_wifi_modal(false);
            helpers::append_log(&ui, &format!("[Wi-Fi] Associating with SSID: {}...", ssid));

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
                            helpers::append_log(
                                &ui,
                                &format!("[Wi-Fi] Association successful: {}", success_msg),
                            );

                            // Trigger immediate active connection refresh
                            let ui_refresh_weak = ui.as_weak();
                            tokio::spawn(async move {
                                if let Ok(Some(active)) = wifi::get_active_wifi(true).await {
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
                            helpers::append_log(&ui, &format!("[Wi-Fi] Association failed: {}", e));
                        });
                    }
                }
            });
        }
    });

    // Callback 7: Cloudflare WARP Toggle connection switch
    let ui_warp_weak = ui_weak.clone();
    ui.on_warp_toggle_clicked(move |connect| {
        if let Some(ui) = ui_warp_weak.upgrade() {
            let state_str = if connect {
                "connecting"
            } else {
                "disconnecting"
            };
            helpers::append_log(&ui, &format!("[WARP] Triggering client {}...", state_str));
            ui.set_warp_status_text("Connecting...".into());
            ui.set_warp_status_color("#f59e0b".into()); // Orange pulse

            let ui_inner_weak = ui_warp_weak.clone();
            tokio::spawn(async move {
                match warp::warp_toggle(connect).await {
                    Ok(msg) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            helpers::append_log(
                                &ui,
                                &format!("[WARP] Operation finished: {}", msg),
                            );
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            helpers::append_log(&ui, &format!("[WARP] Operation failed: {}", e));
                            ui.set_warp_status_text("Error".into());
                            ui.set_warp_status_color("#f43f5e".into()); // Red error
                        });
                    }
                }
            });
        }
    });

    // Callback 8: Cloudflare WARP tunnel mode switch
    let ui_mode_weak = ui_weak.clone();
    ui.on_warp_mode_clicked(move |mode| {
        if let Some(ui) = ui_mode_weak.upgrade() {
            helpers::append_log(
                &ui,
                &format!("[WARP] Configuring operating tunnel mode to: {}...", mode),
            );
            let mode_str = mode.to_string();

            let ui_inner_weak = ui_mode_weak.clone();
            tokio::spawn(async move {
                match warp::set_warp_mode(mode_str).await {
                    Ok(msg) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            helpers::append_log(
                                &ui,
                                &format!("[WARP] Operating mode changed: {}", msg),
                            );
                        });

                        // Query and update warp mode details immediately
                        if let Ok(warp_mode) = warp::get_warp_mode().await {
                            let ui_mode_update = ui_inner_weak.clone();
                            let warp_mode_clone = warp_mode.clone();
                            let _ = ui_mode_update.upgrade_in_event_loop(move |ui| {
                                ui.set_warp_mode_badge(format!("Mode: {}", warp_mode_clone).into());
                                ui.set_warp_mode_doh_active(
                                    !warp_mode_clone.to_lowercase().contains("warp"),
                                );
                            });
                        }

                        // 1. Immediately refresh Public IP & Geolocation
                        tokio::spawn(helpers::refresh_geoip(ui_inner_weak.clone()));

                        // 2. Immediately refresh Ping latencies
                        tokio::spawn(helpers::refresh_ping(ui_inner_weak.clone()));
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            helpers::append_log(&ui, &format!("[WARP] Mode change failed: {}", e));
                        });
                    }
                }
            });
        }
    });

    // Callback 9: Install Cloudflare WARP Daemon package via Polkit
    let ui_install_weak = ui_weak.clone();
    ui.on_install_rpm_clicked(move || {
        if let Some(ui) = ui_install_weak.upgrade() {
            helpers::append_log(
                &ui,
                "[System] Initializing warp-cli Polkit deployment wrapper...",
            );

            let ui_inner_weak = ui_install_weak.clone();
            tokio::spawn(async move {
                match warp::install_warp().await {
                    Ok(msg) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            helpers::append_log(
                                &ui,
                                &format!("[System] Polkit deployment success: {}", msg),
                            );
                        });
                    }
                    Err(e) => {
                        let _ = ui_inner_weak.upgrade_in_event_loop(move |ui| {
                            helpers::append_log(
                                &ui,
                                &format!("[System] Polkit deployment failed: {}", e),
                            );
                        });
                    }
                }
            });
        }
    });
}
