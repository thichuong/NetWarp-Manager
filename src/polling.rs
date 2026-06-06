use crate::{
    AppWindow, IPGeolocatorInfo, NetworkSpeed, PingResult, WifiNetwork, helpers, net_utils, warp,
    wifi,
};
use slint::{ComponentHandle, Model};
use std::time::Instant;

async fn fetch_and_hydrate_state(
    ui_weak: &slint::Weak<AppWindow>,
) -> (
    slint::SharedString,
    slint::SharedString,
    Option<WifiNetwork>,
    slint::SharedString,
    Option<wifi::WifiNetwork>,
    Option<helpers::CachedGeoInfo>,
) {
    let (mode_res, status_res, wifi_res, geo_res) = tokio::join!(
        warp::get_warp_mode(),
        warp::get_warp_status(),
        wifi::get_active_wifi(true),
        helpers::query_geoip(),
    );

    let initial_warp_mode = slint::SharedString::from(mode_res.unwrap_or_else(|e| {
        eprintln!("[WARN] Failed to get WARP mode: {e}");
        "DoH".to_string()
    }));

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
    let mut active_wifi_raw: Option<wifi::WifiNetwork> = None;

    if let Ok(Some(active_full)) = wifi_res {
        active_wifi_raw = Some(active_full.clone());
        let slint_active = helpers::to_slint_wifi(active_full);
        wifi_ssid = slint_active.ssid.clone();
        wifi_cache = Some(slint_active);
    }

    let mut geo_cache = None;
    if let Ok(geo_info) = geo_res {
        geo_cache = Some(geo_info);
    }

    let warp_state_ui = warp_state.clone();
    let wifi_cache_ui = wifi_cache.clone();
    let geo_cache_ui = geo_cache.clone();
    let initial_warp_mode_clone = initial_warp_mode.clone();

    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
        ui.set_warp_mode_badge(format!("Mode: {}", initial_warp_mode_clone).into());
        ui.set_warp_mode_doh_active(!initial_warp_mode_clone.to_lowercase().contains("warp"));

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
            ui.set_active_wifi(helpers::disconnected_wifi());
        }

        if let Some(ref geo) = geo_cache_ui {
            ui.set_geo_info(IPGeolocatorInfo {
                ip: geo.ip.clone().into(),
                isp: geo.isp.clone().into(),
                location: geo.location.clone().into(),
                coordinates: geo.coordinates.clone().into(),
                warp_badge: geo.warp_badge.clone().into(),
            });
            let log_message = format!(
                "[GeoIP] Coordinates synced. IP: {} | ISP: {} ({})",
                geo.ip, geo.isp, geo.warp_badge
            );
            helpers::append_log(&ui, &log_message);
        }
    });

    (
        warp_state,
        wifi_ssid,
        wifi_cache,
        initial_warp_mode,
        active_wifi_raw,
        geo_cache,
    )
}

/// Message structure representing dynamic updates dispatched asynchronously to the UI event loop.
enum UiUpdateMsg {
    Speed {
        stats: NetworkSpeed,
        dl: Vec<f32>,
        ul: Vec<f32>,
    },
    Wifi(Option<WifiNetwork>),
    WarpStatus(slint::SharedString),
    WarpMode(slint::SharedString),
    GeoIp(Option<helpers::CachedGeoInfo>),
    Pings {
        p1: Option<PingResult>,
        p2: Option<PingResult>,
    },
}

// Interval and size constants for background polling engines
const SPEED_POLL_MS: u64 = 1000;
const STATUS_POLL_MS: u64 = 1500;
const PING_POLL_MS: u64 = 1500;
const HISTORY_SIZE: usize = 25;

/// Starts all background polling loop engines for network status, speeds, and pings.
// Developer Warning: Refer to architecture.md Section 6 for full Slint-Rust
// synchronization rules before modifying state polling loops here!
pub fn start_polling_loops(ui: &AppWindow, shutdown_rx: tokio::sync::watch::Receiver<bool>) {
    let ui_weak = ui.as_weak();

    // Synchronously load cached state (if any) and hydrate UI immediately on startup
    if let Some(cache) = crate::cache::load_state_cache() {
        let init_lower = cache.warp_status.to_lowercase();
        let init_connected = init_lower.contains("connected");
        let init_connecting = init_lower.contains("connecting");

        ui.set_warp_mode_badge(format!("Mode: {}", cache.warp_mode).into());
        ui.set_warp_mode_doh_active(!cache.warp_mode.to_lowercase().contains("warp"));

        ui.set_warp_status_text(cache.warp_status.into());
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

        if let Some(active) = cache.wifi_network {
            ui.set_active_wifi(helpers::to_slint_wifi(active));
        } else {
            ui.set_active_wifi(helpers::disconnected_wifi());
        }

        if let Some(geo) = cache.geo_info {
            ui.set_geo_info(IPGeolocatorInfo {
                ip: geo.ip.into(),
                isp: geo.isp.into(),
                location: geo.location.into(),
                coordinates: geo.coordinates.into(),
                warp_badge: geo.warp_badge.into(),
            });
        }
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<(
        slint::SharedString,
        slint::SharedString,
        Option<WifiNetwork>,
        slint::SharedString,
        Option<helpers::CachedGeoInfo>,
    )>();

    // Spawn task to fetch initial states concurrently and hydrate UI
    let ui_init = ui_weak.clone();
    tokio::spawn(async move {
        let (warp_state, wifi_ssid, wifi_cache, initial_warp_mode, _, geo_cache) =
            fetch_and_hydrate_state(&ui_init).await;

        let _ = tx.send((
            warp_state,
            wifi_ssid,
            wifi_cache,
            initial_warp_mode,
            geo_cache,
        ));
    });

    let (tx_msg, mut rx_msg) = tokio::sync::mpsc::channel::<UiUpdateMsg>(64);
    let ui_dispatcher_weak = ui_weak.clone();

    tokio::spawn(async move {
        // UI Dispatcher Event Loop: Process update messages asynchronously.
        // Once all worker threads shut down and drop their senders, rx_msg.recv() returns None,
        // letting this dispatcher loop exit cleanly, dropping all allocated resources.
        while let Some(first_msg) = rx_msg.recv().await {
            // Local cache to aggregate updates before committing to Slint UI
            let mut speed_stats_to_set = None;
            let mut dl_vec_to_set = None;
            let mut ul_vec_to_set = None;
            let mut wifi_to_set = None;
            let mut wifi_set_disconnected = false;
            let mut warp_status_to_set = None;
            let mut warp_mode_to_set = None;
            let mut geo_info_to_set = None;
            let mut ping1_to_set = None;
            let mut ping2_to_set = None;

            // Process a single message inline to avoid vector allocations and multi-pass loops
            let mut process_msg = |msg: UiUpdateMsg| match msg {
                UiUpdateMsg::Speed { stats, dl, ul } => {
                    speed_stats_to_set = Some(stats);
                    dl_vec_to_set = Some(dl);
                    ul_vec_to_set = Some(ul);
                }
                UiUpdateMsg::Wifi(Some(wifi)) => {
                    wifi_to_set = Some(wifi);
                    wifi_set_disconnected = false;
                }
                UiUpdateMsg::Wifi(None) => {
                    wifi_to_set = None;
                    wifi_set_disconnected = true;
                }
                UiUpdateMsg::WarpStatus(status) => {
                    warp_status_to_set = Some(status);
                }
                UiUpdateMsg::WarpMode(mode) => {
                    warp_mode_to_set = Some(mode);
                }
                UiUpdateMsg::GeoIp(geo) => {
                    geo_info_to_set = Some(geo);
                }
                UiUpdateMsg::Pings { p1, p2 } => {
                    if let Some(p) = p1 {
                        ping1_to_set = Some(p);
                    }
                    if let Some(p) = p2 {
                        ping2_to_set = Some(p);
                    }
                }
            };

            // Process the first message immediately
            process_msg(first_msg);

            // Drain all currently queued pending updates to batch them in a single transaction
            while let Ok(msg) = rx_msg.try_recv() {
                process_msg(msg);
            }

            // Dispatch all consolidated changes to the main Slint UI event loop in one single IPC call
            let _ = ui_dispatcher_weak.upgrade_in_event_loop(move |ui| {
                if let Some(stats) = speed_stats_to_set {
                    ui.set_speed_stats(stats);
                }
                if let (Some(dl), Some(ul)) = (dl_vec_to_set, ul_vec_to_set) {
                    let chart_w = ui.get_chart_width();
                    let chart_h = ui.get_chart_height();

                    let max_dl = dl.iter().copied().fold(0.0f32, f32::max);
                    let max_ul = ul.iter().copied().fold(0.0f32, f32::max);
                    let max_val_safe = max_dl.max(max_ul).max(100.0);

                    let (dl_line, dl_area) =
                        helpers::generate_svg_paths(&dl, max_val_safe, chart_w, chart_h);
                    let (ul_line, ul_area) =
                        helpers::generate_svg_paths(&ul, max_val_safe, chart_w, chart_h);

                    ui.set_download_line_path(dl_line.into());
                    ui.set_download_area_path(dl_area.into());
                    ui.set_upload_line_path(ul_line.into());
                    ui.set_upload_area_path(ul_area.into());

                    helpers::DOWNLOAD_HISTORY_MODEL.with(|m| {
                        if let Some(model) = m.borrow().as_ref() {
                            for (idx, &val) in dl.iter().enumerate() {
                                model.set_row_data(idx, val);
                            }
                        }
                    });
                    helpers::UPLOAD_HISTORY_MODEL.with(|m| {
                        if let Some(model) = m.borrow().as_ref() {
                            for (idx, &val) in ul.iter().enumerate() {
                                model.set_row_data(idx, val);
                            }
                        }
                    });
                }
                if let Some(wifi) = wifi_to_set {
                    ui.set_active_wifi(wifi);
                } else if wifi_set_disconnected {
                    ui.set_active_wifi(helpers::disconnected_wifi());
                }
                if let Some(warp_status) = warp_status_to_set {
                    let warp_lower = warp_status.to_lowercase();
                    let is_connected = warp_lower.contains("connected");
                    let is_connecting = warp_lower.contains("connecting");

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
                }
                if let Some(warp_mode) = warp_mode_to_set {
                    ui.set_warp_mode_badge(format!("Mode: {}", warp_mode).into());
                    ui.set_warp_mode_doh_active(!warp_mode.to_lowercase().contains("warp"));
                }
                if let Some(Some(geo)) = geo_info_to_set {
                    ui.set_geo_info(IPGeolocatorInfo {
                        ip: geo.ip.clone().into(),
                        isp: geo.isp.clone().into(),
                        location: geo.location.clone().into(),
                        coordinates: geo.coordinates.clone().into(),
                        warp_badge: geo.warp_badge.clone().into(),
                    });
                    let log_message = format!(
                        "[GeoIP] Coordinates synced. IP: {} | ISP: {} ({})",
                        geo.ip, geo.isp, geo.warp_badge
                    );
                    helpers::append_log(&ui, &log_message);
                }
                if let Some(p1) = ping1_to_set {
                    ui.set_ping1(p1);
                }
                if let Some(p2) = ping2_to_set {
                    ui.set_ping2(p2);
                }
            });
        }
    });

    // Loop 1: Network Bandwidth IO speed monitoring
    let tx_speed = tx_msg.clone();
    let mut shutdown_rx1 = shutdown_rx.clone();
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

        let mut last_speed_stats: Option<NetworkSpeed> = None;
        let mut last_dl_ring: Option<std::collections::VecDeque<f32>> = None;
        let mut last_ul_ring: Option<std::collections::VecDeque<f32>> = None;

        loop {
            tokio::select! {
                _ = shutdown_rx1.changed() => {
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(SPEED_POLL_MS)) => {}
            }
            if *shutdown_rx1.borrow() {
                break;
            }

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

                let dl_changed = last_dl_ring.as_ref() != Some(&dl_ring);
                let ul_changed = last_ul_ring.as_ref() != Some(&ul_ring);
                let stats_changed = last_speed_stats.as_ref() != Some(&speed_stats);

                if stats_changed || dl_changed || ul_changed {
                    let dl_vec: Vec<f32> = dl_ring.iter().copied().collect();
                    let ul_vec: Vec<f32> = ul_ring.iter().copied().collect();

                    let _ = tx_speed
                        .send(UiUpdateMsg::Speed {
                            stats: speed_stats.clone(),
                            dl: dl_vec,
                            ul: ul_vec,
                        })
                        .await;

                    last_speed_stats = Some(speed_stats);
                    last_dl_ring = Some(dl_ring.clone());
                    last_ul_ring = Some(ul_ring.clone());
                }
            }
        }
    });

    // Loop 2: Wi-Fi active interface, Cloudflare WARP Daemon status and Mode
    let tx_status = tx_msg.clone();
    let mut shutdown_rx2 = shutdown_rx.clone();
    tokio::spawn(async move {
        let (
            mut last_warp_state,
            mut last_wifi_ssid,
            mut cached_wifi_details,
            initial_warp_mode,
            geo_cache,
        ) = match rx.await {
            Ok(states) => states,
            Err(_) => (
                slint::SharedString::new(),
                slint::SharedString::new(),
                None,
                "DoH".into(),
                None,
            ),
        };

        let shared_state =
            std::sync::Arc::new(tokio::sync::Mutex::new((initial_warp_mode, geo_cache)));

        loop {
            tokio::select! {
                _ = shutdown_rx2.changed() => {
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(STATUS_POLL_MS)) => {}
            }
            if *shutdown_rx2.borrow() {
                break;
            }

            let mut current_wifi_ssid = last_wifi_ssid.clone();
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
                            let new_rate = if !cache.device.is_empty() {
                                wifi::get_realtime_bitrate(cache.device.as_str())
                                    .await
                                    .unwrap_or_else(|| {
                                        active.rate.as_deref().unwrap_or("").to_string()
                                    })
                            } else {
                                active.rate.as_deref().unwrap_or("").to_string()
                            };
                            let rate_changed = cache.rate.as_str() != new_rate.as_str();
                            let signal_changed = cache.signal != active.signal;

                            if rate_changed || signal_changed {
                                cache.signal = active.signal;
                                cache.rate = new_rate.into();

                                active_wifi_to_set = Some(cache.clone());
                                should_update_wifi_ui = true;
                            }
                            current_wifi_ssid = cache.ssid.clone();
                        }
                    } else {
                        // SSID changed or cache is empty, fetch full details with CLI forks
                        if let Ok(Some(active_full)) = wifi::get_active_wifi(true).await {
                            let slint_active = helpers::to_slint_wifi(active_full);
                            cached_wifi_details = Some(slint_active.clone());
                            current_wifi_ssid = slint_active.ssid.clone();
                            active_wifi_to_set = Some(slint_active);
                            should_update_wifi_ui = true;
                        } else {
                            // Fallback if full info query failed
                            let slint_active = helpers::to_slint_wifi(active);
                            current_wifi_ssid = slint_active.ssid.clone();
                            active_wifi_to_set = Some(slint_active);
                            should_update_wifi_ui = true;
                        }
                    }
                }
                Ok(None) => {
                    if cached_wifi_details.is_some() {
                        cached_wifi_details = None;
                        current_wifi_ssid = slint::SharedString::new();
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
            let warp_status_changed = warp_status != last_warp_state;
            let wifi_ssid_changed = current_wifi_ssid != last_wifi_ssid;
            let state_changed = warp_status_changed || wifi_ssid_changed;

            if state_changed {
                last_warp_state = warp_status.clone();
                last_wifi_ssid = current_wifi_ssid.clone();
            }

            if should_update_wifi_ui {
                let _ = tx_status.send(UiUpdateMsg::Wifi(active_wifi_to_set)).await;
            }

            if warp_status_changed {
                let _ = tx_status.send(UiUpdateMsg::WarpStatus(warp_status)).await;
            }

            // Immediate trigger Geolocation refresh and WARP mode check
            if state_changed {
                let tx_status_clone = tx_status.clone();
                let shared_state_clone = shared_state.clone();

                tokio::spawn(async move {
                    // Fetch updated operating mode and GeoIP concurrently
                    let (mode_res, geo_res) =
                        tokio::join!(warp::get_warp_mode(), helpers::query_geoip(),);

                    let new_warp_mode = slint::SharedString::from(mode_res.unwrap_or_else(|e| {
                        eprintln!("[WARN] Failed to get WARP mode on change: {e}");
                        "DoH".to_string()
                    }));

                    let geo_ui = geo_res.ok();

                    let mut state = shared_state_clone.lock().await;
                    let mode_changed = new_warp_mode != state.0;
                    let geo_changed = geo_ui != state.1;

                    if mode_changed || geo_changed {
                        if mode_changed {
                            state.0 = new_warp_mode.clone();
                            let _ = tx_status_clone
                                .send(UiUpdateMsg::WarpMode(new_warp_mode))
                                .await;
                        }
                        if geo_changed {
                            state.1 = geo_ui.clone();
                            let _ = tx_status_clone.send(UiUpdateMsg::GeoIp(geo_ui)).await;
                        }
                    }
                });
            }
        }
    });

    // Loop 3: Ping Diagnostics Latencies
    let tx_ping = tx_msg;
    let mut shutdown_rx3 = shutdown_rx;
    tokio::spawn(async move {
        let mut last_ping1: Option<PingResult> = None;
        let mut last_ping2: Option<PingResult> = None;

        loop {
            if let Ok(results) = net_utils::ping_multiple(&["1.1.1.1", "8.8.8.8"]).await {
                let mut p1_to_set = None;
                let mut p2_to_set = None;

                for res in results {
                    let is_ping1 = res.target == "1.1.1.1";
                    let is_ping2 = res.target == "8.8.8.8";
                    if !is_ping1 && !is_ping2 {
                        continue;
                    }

                    let raw_latency = res.latency.unwrap_or(999.0) as f32;
                    let last = if is_ping1 { &last_ping1 } else { &last_ping2 };

                    let changed = match last {
                        Some(l) => {
                            l.latency != raw_latency || l.status.as_str() != res.status.as_str()
                        }
                        None => true,
                    };

                    if changed {
                        let slint_res = PingResult {
                            target: res.target.into(),
                            latency: raw_latency,
                            status: res.status.into(),
                        };
                        if is_ping1 {
                            p1_to_set = Some(slint_res.clone());
                            last_ping1 = Some(slint_res);
                        } else {
                            p2_to_set = Some(slint_res.clone());
                            last_ping2 = Some(slint_res);
                        }
                    }
                }

                if p1_to_set.is_some() || p2_to_set.is_some() {
                    let _ = tx_ping
                        .send(UiUpdateMsg::Pings {
                            p1: p1_to_set,
                            p2: p2_to_set,
                        })
                        .await;
                }
            }

            tokio::select! {
                _ = shutdown_rx3.changed() => {
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(PING_POLL_MS)) => {}
            }
            if *shutdown_rx3.borrow() {
                break;
            }
        }
    });
}
