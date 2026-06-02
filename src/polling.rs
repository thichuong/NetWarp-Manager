use crate::{AppWindow, NetworkSpeed, PingResult, WifiNetwork, helpers, net_utils, warp, wifi};
use slint::{ComponentHandle, Model};
use std::rc::Rc;
use std::time::Instant;

/// Starts all background polling loop engines for network status, speeds, pings, and animations.
// Developer Warning: Refer to architecture.md Section 6 for full Slint-Rust 
// synchronization rules before modifying state polling loops here!
pub fn start_polling_loops(ui: &AppWindow) {
    let ui_weak = ui.as_weak();

    // Loop 1: Pulse animations timer (500ms intervals when active, 2s when idle)
    let ui_pulse_weak = ui_weak.clone();
    tokio::spawn(async move {
        let mut pulse = false;
        let mut step = 0;
        loop {
            let mut sleep_ms = 500;
            let mut should_pulse = false;

            if let Some(ui) = ui_pulse_weak.upgrade() {
                let status = ui.get_warp_status_text().to_string();
                if status.to_lowercase().contains("connecting") {
                    should_pulse = true;
                }
            }

            if should_pulse {
                pulse = !pulse;
                step = (step + 1) % 4;

                let _ = ui_pulse_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_pulse_led(pulse);
                    ui.set_radar_step(step);
                });
            } else {
                let _ = ui_pulse_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_pulse_led(false);
                });
                sleep_ms = 2000; // Sleep longer when idle to prevent GPU/CPU redrawing loops
            }

            tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
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
                    download_speed: helpers::format_speed(speed_dl_kb).into(),
                    upload_speed: helpers::format_speed(speed_ul_kb).into(),
                    peak_download: helpers::format_speed(peak_download).into(),
                    peak_upload: helpers::format_speed(peak_upload).into(),
                    total_usage: helpers::format_bytes(total_session_usage).into(),
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
                    let max_dl = dl_vec.iter().copied().fold(0.0f32, f32::max);

                    // Slide upload model array
                    let ul_model = ui.get_upload_history();
                    let mut ul_vec: Vec<f32> = ul_model.iter().collect();
                    if !ul_vec.is_empty() {
                        ul_vec.remove(0);
                    }
                    ul_vec.push(speed_ul_kb as f32);
                    let max_ul = ul_vec.iter().copied().fold(0.0f32, f32::max);

                    // Calculate the peak value across both historical channels for auto-scaling
                    let max_val = max_dl.max(max_ul);

                    // Clamp to a safe minimum baseline (100.0 KB/s) to prevent division by zero or overly micro-scaled waves when idle
                    let max_val_safe = if max_val < 100.0 { 100.0 } else { max_val };

                    let chart_w = ui.get_chart_width();
                    let chart_h = ui.get_chart_height();

                    // Generate dynamic line and area SVG paths for direct rendering on unified axis
                    let dl_line =
                        helpers::generate_svg_path(&dl_vec, max_val_safe, chart_w, chart_h, false);
                    let dl_area =
                        helpers::generate_svg_path(&dl_vec, max_val_safe, chart_w, chart_h, true);
                    let ul_line =
                        helpers::generate_svg_path(&ul_vec, max_val_safe, chart_w, chart_h, false);
                    let ul_area =
                        helpers::generate_svg_path(&ul_vec, max_val_safe, chart_w, chart_h, true);

                    ui.set_download_line_path(dl_line.into());
                    ui.set_download_area_path(dl_area.into());
                    ui.set_upload_line_path(ul_line.into());
                    ui.set_upload_area_path(ul_area.into());

                    // Move vectors to Slint VecModels
                    ui.set_download_history(Rc::new(slint::VecModel::from(dl_vec)).into());
                    ui.set_upload_history(Rc::new(slint::VecModel::from(ul_vec)).into());

                    // Format the peak speed label dynamically (converting to MB/s if rate is >= 1024 KB/s)
                    let max_label = if max_val_safe >= 1024.0 {
                        format!("{:.2} MB/s", max_val_safe / 1024.0)
                    } else {
                        format!("{:.0} KB/s", max_val_safe)
                    };

                    ui.set_max_history_value(max_val_safe);
                    ui.set_max_history_label(max_label.into());
                });
            }
        }
    });

    // Loop 3: Wi-Fi active interface, Cloudflare WARP Daemon status and Mode (3 second interval)
    let ui_status_weak = ui_weak.clone();
    tokio::spawn(async move {
        let mut last_warp_state = String::new();
        let mut last_wifi_ssid = String::new();
        let mut cached_wifi_details: Option<WifiNetwork> = None;
        let mut geo_cooldown_counter = 0;

        // Fetch initial WARP Mode on application launch
        let initial_warp_mode = warp::get_warp_mode()
            .await
            .unwrap_or_else(|_| "DoH".to_string());
        let ui_init_mode = ui_status_weak.clone();
        let warp_mode_init_clone = initial_warp_mode.clone();
        let _ = ui_init_mode.upgrade_in_event_loop(move |ui| {
            ui.set_warp_mode_badge(format!("Mode: {}", warp_mode_init_clone).into());
            ui.set_warp_mode_doh_active(!warp_mode_init_clone.to_lowercase().contains("warp"));
        });

        loop {
            let mut current_wifi_ssid = String::new();
            let mut active_wifi_to_set = None;
            let mut should_update_wifi_ui = false;

            // Polling active Wi-Fi connection (optimized)
            match wifi::get_active_wifi(false).await {
                Ok(Some(active)) => {
                    current_wifi_ssid = active.ssid.clone();

                    if active.ssid == last_wifi_ssid && cached_wifi_details.is_some() {
                        // Apply cached static details (MAC, IP, Gateway, DNS) from Slint cache to save CPU process forks
                        if let Some(ref cache) = cached_wifi_details {
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
                                device: cache.device.clone(),
                                mac: cache.mac.clone(),
                                ip_address: cache.ip_address.clone(),
                                gateway: cache.gateway.clone(),
                                dns_primary: cache.dns_primary.clone(),
                                dns_secondary: cache.dns_secondary.clone(),
                            };
                            active_wifi_to_set = Some(slint_active);
                            should_update_wifi_ui = true;
                        }
                    } else {
                        // SSID changed or cache is empty, fetch full details with CLI forks
                        if let Ok(Some(active_full)) = wifi::get_active_wifi(true).await {
                            let slint_active = WifiNetwork {
                                bssid: active_full.bssid.into(),
                                ssid: active_full.ssid.clone().into(),
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
                            cached_wifi_details = Some(slint_active.clone());
                            last_wifi_ssid = active_full.ssid.clone();
                            active_wifi_to_set = Some(slint_active);
                            should_update_wifi_ui = true;
                        } else {
                            // Fallback if full info query failed
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
                                device: "--".into(),
                                mac: "--".into(),
                                ip_address: "--".into(),
                                gateway: "--".into(),
                                dns_primary: "--".into(),
                                dns_secondary: "--".into(),
                            };
                            active_wifi_to_set = Some(slint_active);
                            should_update_wifi_ui = true;
                        }
                    }
                }
                Ok(None) => {
                    cached_wifi_details = None;
                    last_wifi_ssid = String::new();
                    should_update_wifi_ui = true; // Set to "Not Connected"
                }
                Err(_) => {}
            }

            if should_update_wifi_ui {
                let ui_wifi_weak = ui_status_weak.clone();
                let active_opt = active_wifi_to_set.clone();
                let _ = ui_wifi_weak.upgrade_in_event_loop(move |ui| {
                    if let Some(active) = active_opt {
                        ui.set_active_wifi(active);
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

            // Detect if connection state or wifi SSID changed to trigger immediate Geo IP refresh
            // 3-second cycle: geo_cooldown_counter >= 10 matches 30 seconds interval
            let state_changed = warp_status != last_warp_state
                || current_wifi_ssid != last_wifi_ssid
                || geo_cooldown_counter >= 10;

            if state_changed {
                last_warp_state = warp_status.clone();
                last_wifi_ssid = current_wifi_ssid.clone();
                geo_cooldown_counter = 0;
            } else {
                geo_cooldown_counter += 1;
            }

            let ui_warp_inner = ui_status_weak.clone();
            let warp_status_clone = warp_status.clone();
            let ui_geo_trigger = ui_warp_inner.clone();

            let _ = ui_warp_inner.upgrade_in_event_loop(move |ui| {
                ui.set_warp_status_text(warp_status_clone.clone().into());

                if warp_status_clone.to_lowercase().contains("connected") {
                    ui.set_warp_status_color("#10b981".into()); // Green
                    ui.set_warp_network_text(
                        "Your network traffic is encrypted & protected.".into(),
                    );
                    ui.set_warp_toggle_state(true);
                } else if warp_status_clone.to_lowercase().contains("connecting") {
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

                // Immediate trigger Geolocation curl fetch if state toggled
                if state_changed {
                    tokio::spawn(helpers::refresh_geoip(ui_geo_trigger));
                }
            });

            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        }
    });

    // Loop 4: Ping Diagnostics Latencies (5 second interval)
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
            tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
        }
    });

    // Loop 5: Initial Geolocation sync on application launch
    let ui_init_weak = ui_weak.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        helpers::refresh_geoip(ui_init_weak).await;
    });
}
