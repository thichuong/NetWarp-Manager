use crate::{AppWindow, IPGeolocatorInfo, PingResult, WifiNetwork, net_utils, wifi};
use slint::Model;
use std::rc::Rc;

/// Generates SVG path commands for a history array mapping into a physical pixel dimension coordinate space.
/// Returns a tuple containing (line_path, area_path) generated in a single pass.
pub fn generate_svg_paths(
    history: &[f32],
    max_val: f32,
    chart_w: f32,
    chart_h: f32,
) -> (String, String) {
    if history.is_empty() {
        return (String::new(), String::new());
    }

    let len = history.len();
    let x_step = chart_w / (len - 1) as f32;

    let get_y = |val: f32| -> f32 {
        let ratio = if max_val > 0.0 { val / max_val } else { 0.0 };
        // Map 0 to 95% of height, and max to 5% of height (keep 5% padding from boundaries)
        chart_h - ratio * (chart_h * 0.9) - (chart_h * 0.05)
    };

    use std::fmt::Write;
    let estimated_capacity = history.len() * 24 + 32;
    let mut line = String::with_capacity(estimated_capacity);
    let mut area = String::with_capacity(estimated_capacity + 32);

    let first_val = history.first().copied().unwrap_or(0.0);
    let y0 = get_y(first_val);

    let _ = write!(line, "M 0.0 {:.2} ", y0);
    let _ = write!(area, "M 0.0 {:.2} L 0.0 {:.2} ", chart_h, y0);

    for (i, &val) in history.iter().enumerate().skip(1) {
        let x = i as f32 * x_step;
        let y = get_y(val);
        let _ = write!(line, "L {:.2} {:.2} ", x, y);
        let _ = write!(area, "L {:.2} {:.2} ", x, y);
    }

    let _ = write!(area, "L {:.2} {:.2} Z", chart_w, chart_h);

    (line, area)
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

use std::cell::RefCell;

thread_local! {
    static LOGS_MODEL: RefCell<Option<Rc<slint::VecModel<slint::SharedString>>>> = const { RefCell::new(None) };
    pub static DOWNLOAD_HISTORY_MODEL: RefCell<Option<Rc<slint::VecModel<f32>>>> = const { RefCell::new(None) };
    pub static UPLOAD_HISTORY_MODEL: RefCell<Option<Rc<slint::VecModel<f32>>>> = const { RefCell::new(None) };
}

/// Initializes the thread-local reference to the console logs model for O(1) logging.
pub fn init_logs_model(model: Rc<slint::VecModel<slint::SharedString>>) {
    LOGS_MODEL.with(|m| {
        *m.borrow_mut() = Some(model);
    });
}

/// Initializes the thread-local reference to the download and upload history models.
pub fn init_history_models(dl: Rc<slint::VecModel<f32>>, ul: Rc<slint::VecModel<f32>>) {
    DOWNLOAD_HISTORY_MODEL.with(|m| {
        *m.borrow_mut() = Some(dl);
    });
    UPLOAD_HISTORY_MODEL.with(|m| {
        *m.borrow_mut() = Some(ul);
    });
}

/// Helper function to append a system log to the UI logs terminal
pub fn append_log(ui: &AppWindow, message: &str) {
    let local_time = chrono::Local::now();
    let time_str = local_time.format("[%H:%M:%S]").to_string();
    let formatted_message = format!("{} {}", time_str, message);

    let mut success = false;
    LOGS_MODEL.with(|m| {
        if let Some(model) = m.borrow().as_ref() {
            model.push(slint::SharedString::from(&formatted_message));
            if model.row_count() > 100 {
                model.remove(0);
            }
            success = true;
        }
    });

    if !success {
        // Fallback for tests or uninitialized model - O(n) allocation
        let logs_rc = ui.get_console_logs();
        let mut logs: Vec<String> = logs_rc.iter().map(|s| s.to_string()).collect();
        logs.push(formatted_message);
        if logs.len() > 100 {
            logs.remove(0);
        }
        let slint_logs: Vec<slint::SharedString> =
            logs.into_iter().map(slint::SharedString::from).collect();
        ui.set_console_logs(Rc::new(slint::VecModel::from(slint_logs)).into());
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct CachedGeoInfo {
    pub ip: String,
    pub isp: String,
    pub location: String,
    pub coordinates: String,
    pub warp_badge: String,
}

/// Queries the public IP and Geolocation details asynchronously.
pub async fn query_geoip() -> Result<CachedGeoInfo, crate::AppError> {
    let raw_json = net_utils::trace_ip().await?;
    let parsed: serde_json::Value = serde_json::from_str(&raw_json)
        .map_err(|e| crate::AppError::GeoIp(format!("Failed to parse geo response: {}", e)))?;

    let ip = parsed
        .get("ip")
        .or_else(|| parsed.get("query"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let isp = parsed
        .get("connection")
        .and_then(|c| c.get("isp"))
        .or_else(|| parsed.get("org"))
        .or_else(|| parsed.get("isp"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let city = parsed
        .get("city")
        .or_else(|| parsed.get("cityName"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let country = parsed
        .get("country")
        .or_else(|| parsed.get("country_name"))
        .or_else(|| parsed.get("countryName"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let location = if city.is_empty() {
        country.clone()
    } else {
        format!("{}, {}", city, country)
    };

    let lat = parsed
        .get("latitude")
        .or_else(|| parsed.get("lat"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let lon = parsed
        .get("longitude")
        .or_else(|| parsed.get("lon"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let coords = format!("{:.4}, {:.4}", lat, lon);

    let is_warp = isp.to_lowercase().contains("cloudflare");
    let badge = if is_warp { "WARP" } else { "DIRECT" }.to_string();

    Ok(CachedGeoInfo {
        ip,
        isp,
        location,
        coordinates: coords,
        warp_badge: badge,
    })
}

/// Fetches the public IP and Geolocation details asynchronously and updates the UI.
/// This single function consolidates geolocation updates throughout the application.
pub async fn refresh_geoip(ui_weak: slint::Weak<AppWindow>) {
    if let Ok(info) = query_geoip().await {
        let slint_geo = IPGeolocatorInfo {
            ip: info.ip.clone().into(),
            isp: info.isp.clone().into(),
            location: info.location.clone().into(),
            coordinates: info.coordinates.clone().into(),
            warp_badge: info.warp_badge.clone().into(),
        };

        let log_message = format!(
            "[GeoIP] Coordinates synced. IP: {} | ISP: {} ({})",
            info.ip, info.isp, info.warp_badge
        );

        let _ = ui_weak.upgrade_in_event_loop(move |ui| {
            ui.set_geo_info(slint_geo);
            append_log(&ui, &log_message);
        });
    }
}

/// Refreshes target latency pings (1.1.1.1 and 8.8.8.8) and updates diagnostic widgets.
pub async fn refresh_ping(ui_weak: slint::Weak<AppWindow>) {
    if let Ok(results) = net_utils::ping_multiple(&["1.1.1.1", "8.8.8.8"]).await {
        let _ = ui_weak.upgrade_in_event_loop(move |ui| {
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
            append_log(&ui, "[Diagnostics] Latency diagnostics refreshed.");
        });
    }
}

/// Dynamically detects the host OS name from `/etc/os-release`.
/// Returns the name in uppercase (e.g., "FEDORA LINUX" or "UBUNTU"),
/// defaulting to "LINUX SYSTEM" on failure.
pub fn detect_os_name() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(name_val) = line.strip_prefix("NAME=") {
                let name = name_val.trim_matches('"');
                return name.to_uppercase();
            }
        }
    }
    "LINUX SYSTEM".to_string()
}

/// Converts a backend `wifi::WifiNetwork` to a Slint `WifiNetwork` UI struct.
pub fn to_slint_wifi(net: wifi::WifiNetwork) -> WifiNetwork {
    WifiNetwork {
        bssid: net.bssid.into(),
        ssid: net.ssid.into(),
        channel: net.channel,
        frequency: net.frequency.into(),
        band: net.band.into(),
        signal: net.signal,
        security: net.security.into(),
        active: net.active,
        rate: net
            .rate
            .map(slint::SharedString::from)
            .unwrap_or_else(|| "--".into()),
        device: net
            .device
            .map(slint::SharedString::from)
            .unwrap_or_else(|| "--".into()),
        mac: net
            .mac
            .map(slint::SharedString::from)
            .unwrap_or_else(|| "--".into()),
        ip_address: net
            .ip_address
            .map(slint::SharedString::from)
            .unwrap_or_else(|| "--".into()),
        gateway: net
            .gateway
            .map(slint::SharedString::from)
            .unwrap_or_else(|| "--".into()),
        dns_primary: net
            .dns_primary
            .map(slint::SharedString::from)
            .unwrap_or_else(|| "--".into()),
        dns_secondary: net
            .dns_secondary
            .map(slint::SharedString::from)
            .unwrap_or_else(|| "--".into()),
    }
}

/// Returns a default Slint `WifiNetwork` struct representing a disconnected state.
pub fn disconnected_wifi() -> WifiNetwork {
    WifiNetwork {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1500 * 1024 * 1024), "1.46 GB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(0.0), "0.00 KB/s");
        assert_eq!(format_speed(512.5), "512.50 KB/s");
        assert_eq!(format_speed(1024.0), "1.00 MB/s");
        assert_eq!(format_speed(1536.0), "1.50 MB/s");
    }

    #[test]
    fn test_generate_svg_paths() {
        let history = vec![10.0, 20.0, 30.0];
        let (path_line, path_area) = generate_svg_paths(&history, 100.0, 200.0, 100.0);
        assert!(!path_line.is_empty());
        assert!(path_line.starts_with("M 0.0"));
        assert!(!path_area.is_empty());
        assert!(path_area.starts_with("M 0.0"));
        assert!(path_area.ends_with('Z'));
    }
}
