use crate::{AppWindow, helpers, wifi};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AppStateCache {
    pub warp_mode: String,
    pub warp_status: String,
    pub wifi_network: Option<wifi::WifiNetwork>,
    pub geo_info: Option<helpers::CachedGeoInfo>,
}

fn get_cache_path() -> std::path::PathBuf {
    #[cfg(test)]
    {
        std::env::temp_dir().join("wiwarp_test_state_cache.json")
    }
    #[cfg(not(test))]
    {
        let cache_dir = std::env::var("XDG_CACHE_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let mut path = std::path::PathBuf::from(home);
                path.push(".cache");
                path
            });
        let mut path = cache_dir;
        path.push("netwarp-manager");
        path.push("state_cache.json");
        path
    }
}

pub fn load_state_cache() -> Option<AppStateCache> {
    let path = get_cache_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_state_cache(cache: &AppStateCache) {
    let path = get_cache_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = serde_json::to_string_pretty(cache) {
        let _ = std::fs::write(path, content);
    }
}

pub fn save_cache_from_ui(ui: &AppWindow) {
    let badge = ui.get_warp_mode_badge().to_string();
    let warp_mode = if badge.starts_with("Mode: ") {
        badge.trim_start_matches("Mode: ").to_string()
    } else {
        badge
    };

    let warp_status = ui.get_warp_status_text().to_string();

    let active_wifi = ui.get_active_wifi();
    let wifi_network = if active_wifi.ssid == "Not Connected" || !active_wifi.active {
        None
    } else {
        Some(wifi::WifiNetwork {
            bssid: active_wifi.bssid.to_string(),
            ssid: active_wifi.ssid.to_string(),
            channel: active_wifi.channel,
            frequency: active_wifi.frequency.to_string(),
            band: active_wifi.band.to_string(),
            signal: active_wifi.signal,
            security: active_wifi.security.to_string(),
            active: active_wifi.active,
            rate: Some(active_wifi.rate.to_string()),
            device: Some(active_wifi.device.to_string()),
            mac: Some(active_wifi.mac.to_string()),
            ip_address: Some(active_wifi.ip_address.to_string()),
            gateway: Some(active_wifi.gateway.to_string()),
            dns_primary: Some(active_wifi.dns_primary.to_string()),
            dns_secondary: Some(active_wifi.dns_secondary.to_string()),
        })
    };

    let geo = ui.get_geo_info();
    let geo_info = if geo.ip.is_empty() || geo.ip == "Unknown" {
        None
    } else {
        Some(helpers::CachedGeoInfo {
            ip: geo.ip.to_string(),
            isp: geo.isp.to_string(),
            location: geo.location.to_string(),
            coordinates: geo.coordinates.to_string(),
            warp_badge: geo.warp_badge.to_string(),
        })
    };

    let cache = AppStateCache {
        warp_mode,
        warp_status,
        wifi_network,
        geo_info,
    };

    save_state_cache(&cache);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_serialization() {
        let geo = helpers::CachedGeoInfo {
            ip: "1.1.1.1".to_string(),
            isp: "Cloudflare".to_string(),
            location: "Austin, Texas".to_string(),
            coordinates: "30.2672, -97.7431".to_string(),
            warp_badge: "WARP".to_string(),
        };
        let wifi = wifi::WifiNetwork {
            bssid: "00:11:22:33:44:55".to_string(),
            ssid: "TestWiFi".to_string(),
            channel: 6,
            frequency: "2437 MHz".to_string(),
            band: "2.4 GHz".to_string(),
            signal: 80,
            security: "WPA2".to_string(),
            active: true,
            rate: Some("150 Mbps".to_string()),
            device: Some("wlan0".to_string()),
            mac: Some("AA:BB:CC:DD:EE:FF".to_string()),
            ip_address: Some("192.168.1.50".to_string()),
            gateway: Some("192.168.1.1".to_string()),
            dns_primary: Some("1.1.1.1".to_string()),
            dns_secondary: Some("8.8.8.8".to_string()),
        };
        let cache = AppStateCache {
            warp_mode: "WARP".to_string(),
            warp_status: "Connected".to_string(),
            wifi_network: Some(wifi),
            geo_info: Some(geo),
        };

        let serialized = serde_json::to_string(&cache).unwrap();
        let deserialized: AppStateCache = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.warp_mode, "WARP");
        assert_eq!(deserialized.warp_status, "Connected");
        assert_eq!(deserialized.wifi_network.as_ref().unwrap().ssid, "TestWiFi");
        assert_eq!(deserialized.geo_info.as_ref().unwrap().ip, "1.1.1.1");
    }

    #[test]
    fn test_save_and_load_cache() {
        let cache = AppStateCache {
            warp_mode: "DoH".to_string(),
            warp_status: "Disconnected".to_string(),
            wifi_network: None,
            geo_info: None,
        };

        // Save state cache (which writes to the test-configured temp file path)
        save_state_cache(&cache);

        // Verify that the file was actually written to the test temp file path
        let cache_file = get_cache_path();
        assert!(cache_file.exists());

        // Load the cache back
        let loaded = load_state_cache().unwrap();
        assert_eq!(loaded.warp_mode, "DoH");
        assert_eq!(loaded.warp_status, "Disconnected");
        assert!(loaded.wifi_network.is_none());
        assert!(loaded.geo_info.is_none());

        // Cleanup
        let _ = std::fs::remove_file(&cache_file);
    }
}
