use crate::{AppWindow, NetworkSpeed, PingResult, WifiNetwork, helpers, net_utils, warp, wifi};
use slint::ComponentHandle;
use std::rc::Rc;
use std::time::Instant;

// Interval and size constants for background polling engines
const SPEED_POLL_MS: u64 = 1000;
const STATUS_POLL_MS: u64 = 1500;
const PING_POLL_MS: u64 = 1500;
const HISTORY_SIZE: usize = 25;

/// Starts all background polling loop engines for network status, speeds, and pings.
// Developer Warning: Refer to architecture.md Section 6 for full Slint-Rust
// synchronization rules before modifying state polling loops here!
pub fn start_polling_loops(ui: &AppWindow) {
    let ui_weak = ui.as_weak();

    let (tx, rx) = tokio::sync::oneshot::channel::<(
        slint::SharedString,
        slint::SharedString,
        Option<WifiNetwork>,
    )>();

    // Spawn task to fetch initial states concurrently and hydrate UI
    let ui_init = ui_weak.clone();
    tokio::spawn(async move {
        let (mode_res, status_res, wifi_res, _) = tokio::join!(
            warp::get_warp_mode(),
            warp::get_warp_status(),
            wifi::get_active_wifi(true),
            helpers::refresh_geoip(ui_init.clone()),
        );

        let initial_warp_mode = mode_res.unwrap_or_else(|e| {
            eprintln!("[WARN] Failed to get WARP mode: {e}");
            "DoH".to_string()
        });

        let initial_warp_status = status_res.unwrap_or_else(|e| {
            eprintln!("[WARN] Failed to get WARP status: {e}");
            "Disconnected".to_string()
        });
        let init_lower = initial_warp_status.to_lowercase();
        let init_connected = init_lower.contains("connected");
        let init_connecting = init_lower.contains("connecting");
        let warp_state = slint::SharedString::from(&initial_warp_status);

        let mut wifi_ssid = slint::SharedString::new();
        let mut wifi_cache: Option<WifiNetwork> = None;

        if let Ok(Some(active_full)) = wifi_res {
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
            wifi_cache = Some(slint_active);
        }

        let warp_state_ui = warp_state.clone();
        let wifi_cache_ui = wifi_cache.clone();

        let _ = ui_init.upgrade_in_event_loop(move |ui| {
            ui.set_warp_mode_badge(format!("Mode: {}", initial_warp_mode).into());
            ui.set_warp_mode_doh_active(!initial_warp_mode.to_lowercase().contains("warp"));

            ui.set_warp_status_text(warp_state_ui);
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

            if let Some(active) = wifi_cache_ui {
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

        let _ = tx.send((warp_state, wifi_ssid, wifi_cache));
    });

    // Loop 1: Network Bandwidth IO speed monitoring
    let ui_speed_weak = ui_weak.clone();
    tokio::spawn(async move {
        let mut last_rx = 0;
        let mut last_tx = 0;
        let mut last_time = Instant::now();
        let mut peak_download = 0.0;
        let mut peak_upload = 0.0;
        let mut total_session_usage = 0;
        let mut first_run = true;

        // Use pre-allocated Ring Buffers to eliminate massive allocations and O(n) shifts in Slint's Event Loop
        let mut dl_ring = std::collections::VecDeque::from(vec![0.0f32; HISTORY_SIZE]);
        let mut ul_ring = std::collections::VecDeque::from(vec![0.0f32; HISTORY_SIZE]);

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(SPEED_POLL_MS)).await;

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

                // Slide ring buffers with O(1) efficiency
                dl_ring.pop_front();
                dl_ring.push_back(speed_dl_kb as f32);
                ul_ring.pop_front();
                ul_ring.push_back(speed_ul_kb as f32);

                let dl_vec: Vec<f32> = dl_ring.iter().copied().collect();
                let ul_vec: Vec<f32> = ul_ring.iter().copied().collect();

                // Push new history to rolling models of graph visualization
                let _ = ui_speed_weak.upgrade_in_event_loop(move |ui| {
                    ui.set_speed_stats(speed_stats);

                    let max_dl = dl_vec.iter().copied().fold(0.0f32, f32::max);
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
                });
            }
        }
    });

    // Loop 2: Wi-Fi active interface, Cloudflare WARP Daemon status and Mode
    let ui_status_weak = ui_weak.clone();
    tokio::spawn(async move {
        let (mut last_warp_state, mut last_wifi_ssid, mut cached_wifi_details) = match rx.await {
            Ok(states) => states,
            Err(_) => (slint::SharedString::new(), slint::SharedString::new(), None),
        };

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(STATUS_POLL_MS)).await;

            let mut current_wifi_ssid = slint::SharedString::new();
            let mut active_wifi_to_set = None;
            let mut should_update_wifi_ui = false;

            // Polling active Wi-Fi connection (optimized)
            match wifi::get_active_wifi(false).await {
                Ok(Some(active)) => {
                    let has_cache = cached_wifi_details.is_some();
                    let matches_last = active.ssid.as_str() == last_wifi_ssid.as_str();

                    if matches_last && has_cache {
                        // Apply cached static details (MAC, IP, Gateway, DNS) from Slint cache to save CPU process forks
                        if let Some(ref mut cache) = cached_wifi_details {
                            let new_rate = active.rate.unwrap_or_default();
                            let rate_changed = cache.rate.as_str() != new_rate.as_str();
                            let signal_changed = cache.signal != active.signal;

                            if rate_changed || signal_changed {
                                cache.bssid = active.bssid.into();
                                cache.ssid = active.ssid.into();
                                cache.channel = active.channel;
                                cache.frequency = active.frequency.into();
                                cache.band = active.band.into();
                                cache.signal = active.signal;
                                cache.security = active.security.into();
                                cache.active = active.active;
                                cache.rate = new_rate.into();

                                active_wifi_to_set = Some(cache.clone());
                                should_update_wifi_ui = true;
                            }
                            current_wifi_ssid = cache.ssid.clone();
                        }
                    } else {
                        // SSID changed or cache is empty, fetch full details with CLI forks
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
                            cached_wifi_details = Some(slint_active.clone());
                            last_wifi_ssid = slint_active.ssid.clone();
                            current_wifi_ssid = slint_active.ssid.clone();
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
                            current_wifi_ssid = slint_active.ssid.clone();
                            active_wifi_to_set = Some(slint_active);
                            should_update_wifi_ui = true;
                        }
                    }
                }
                Ok(None) => {
                    if cached_wifi_details.is_some() {
                        cached_wifi_details = None;
                        last_wifi_ssid = slint::SharedString::new();
                        should_update_wifi_ui = true; // Set to "Not Connected"
                    }
                }
                Err(_) => {}
            }

            // Polling Cloudflare WARP Status
            let warp_status = slint::SharedString::from(
                warp::get_warp_status()
                    .await
                    .unwrap_or_else(|_| "Disconnected".to_string()),
            );

            // Detect if connection state or wifi SSID changed to trigger immediate Geo IP refresh
            let state_changed = warp_status != last_warp_state
                || current_wifi_ssid != last_wifi_ssid;

            if state_changed {
                last_warp_state = warp_status.clone();
                last_wifi_ssid = current_wifi_ssid.clone();
            }

            // Precompute lowering and status booleans to avoid overhead inside the event loop closure
            let warp_lower = warp_status.to_lowercase();
            let is_connected = warp_lower.contains("connected");
            let is_connecting = warp_lower.contains("connecting");

            let ui_status_weak_inner = ui_status_weak.clone();

            // Consolidate updates into a single event loop call to minimize cross-thread message passing
            let _ = ui_status_weak_inner.upgrade_in_event_loop(move |ui| {
                if should_update_wifi_ui {
                    if let Some(active) = active_wifi_to_set {
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
                }

                ui.set_warp_status_text(warp_status);

                if is_connected {
                    ui.set_warp_network_text(
                        "Your network traffic is encrypted & protected.".into(),
                    );
                    ui.set_warp_toggle_state(true);
                } else if is_connecting {
                    ui.set_warp_network_text("Establishing secure Cloudflare tunnel...".into());
                    ui.set_warp_toggle_state(true);
                } else {
                    ui.set_warp_network_text(
                        "Your network traffic is direct & unprotected.".into(),
                    );
                    ui.set_warp_toggle_state(false);
                }

                // Immediate trigger Geolocation curl fetch and WARP mode refresh if state toggled
                if state_changed {
                    let ui_geo_trigger = ui.as_weak();
                    tokio::spawn(helpers::refresh_geoip(ui_geo_trigger));

                    // Asynchronously fetch current operating mode and update UI badge to stay in sync
                    let ui_mode_trigger = ui.as_weak();
                    tokio::spawn(async move {
                        if let Ok(warp_mode) = warp::get_warp_mode().await {
                            let _ = ui_mode_trigger.upgrade_in_event_loop(move |ui| {
                                ui.set_warp_mode_badge(format!("Mode: {}", warp_mode).into());
                                ui.set_warp_mode_doh_active(
                                    !warp_mode.to_lowercase().contains("warp"),
                                );
                            });
                        }
                    });
                }
            });
        }
    });

    // Loop 3: Ping Diagnostics Latencies
    let ui_ping_weak = ui_weak.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(results) = net_utils::ping_multiple(&["1.1.1.1", "8.8.8.8"]).await {
                let ui_ping_inner = ui_ping_weak.clone();
                let _ = ui_ping_inner.upgrade_in_event_loop(move |ui| {
                    for res in results {
                        let is_ping1 = res.target == "1.1.1.1";
                        let is_ping2 = res.target == "8.8.8.8";
                        let slint_res = PingResult {
                            target: res.target.into(),
                            latency: res.latency.unwrap_or(999.0) as f32,
                            status: res.status.into(),
                        };

                        if is_ping1 {
                            ui.set_ping1(slint_res);
                        } else if is_ping2 {
                            ui.set_ping2(slint_res);
                        }
                    }
                });
            }
            tokio::time::sleep(std::time::Duration::from_millis(PING_POLL_MS)).await;
        }
    });

}
