use crate::{AppWindow, IPGeolocatorInfo, PingResult, net_utils};
use slint::Model;
use std::rc::Rc;

/// Generates SVG path commands for a history array mapping into a physical pixel dimension coordinate space.
pub fn generate_svg_path(
    history: &[f32],
    max_val: f32,
    chart_w: f32,
    chart_h: f32,
    is_area: bool,
) -> String {
    if history.is_empty() {
        return String::new();
    }

    let len = history.len();
    let x_step = chart_w / (len - 1) as f32;

    let get_y = |val: f32| -> f32 {
        let ratio = if max_val > 0.0 { val / max_val } else { 0.0 };
        // Map 0 to 95% of height, and max to 5% of height (keep 5% padding from boundaries)
        chart_h - ratio * (chart_h * 0.9) - (chart_h * 0.05)
    };

    let mut commands = String::new();

    if is_area {
        // Start at bottom-left corner of the chart area in physical pixels
        commands.push_str(&format!(
            "M 0.0 {:.2} L 0.0 {:.2} ",
            chart_h,
            get_y(history[0])
        ));
    } else {
        commands.push_str(&format!("M 0.0 {:.2} ", get_y(history[0])));
    }

    for (i, &val) in history.iter().enumerate().skip(1) {
        let x = i as f32 * x_step;
        let y = get_y(val);
        commands.push_str(&format!("L {:.2} {:.2} ", x, y));
    }

    if is_area {
        // Go down to bottom-right corner and close the shape
        commands.push_str(&format!("L {:.2} {:.2} Z", chart_w, chart_h));
    }

    commands
}

/// Converts a size in bytes to a human-readable string (KB, MB, GB, etc.)
pub fn format_bytes(bytes: u64) -> String {
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
pub fn format_speed(kb_s: f64) -> String {
    if kb_s >= 1024.0 {
        format!("{:.2} MB/s", kb_s / 1024.0)
    } else {
        format!("{:.2} KB/s", kb_s)
    }
}

/// Helper function to append a system log to the UI logs terminal
pub fn append_log(ui: &AppWindow, message: &str) {
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

/// Fetches the public IP and Geolocation details asynchronously and updates the UI.
/// This single function consolidates geolocation updates throughout the application.
pub async fn refresh_geoip(ui_weak: slint::Weak<AppWindow>) {
    if let Ok(raw_json) = net_utils::trace_ip().await
        && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw_json)
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
            ip: ip.clone().into(),
            isp: isp.clone().into(),
            location: location.into(),
            coordinates: coords.into(),
            warp_badge: badge.into(),
        };

        let _ = ui_weak.upgrade_in_event_loop(move |ui| {
            ui.set_geo_info(slint_geo);
            append_log(
                &ui,
                &format!(
                    "[GeoIP] Coordinates synced. IP: {} | ISP: {} ({})",
                    ip, isp, badge
                ),
            );
        });
    }
}

/// Refreshes target latency pings (1.1.1.1 and 8.8.8.8) and updates diagnostic widgets.
pub async fn refresh_ping(ui_weak: slint::Weak<AppWindow>) {
    let targets = vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()];
    if let Ok(results) = net_utils::ping_multiple(targets).await {
        let _ = ui_weak.upgrade_in_event_loop(move |ui| {
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
}
