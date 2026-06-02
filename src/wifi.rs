use crate::AppError;
use tokio::process::Command;

/// Structure representing a Wi-Fi network returned to the frontend with detailed information.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct WifiNetwork {
    pub bssid: String,
    pub ssid: String,
    pub channel: i32,
    pub frequency: String,
    pub band: String,
    pub signal: i32,
    pub security: String,
    pub active: bool,
    // Detailed active connection parameters
    pub rate: Option<String>,
    pub device: Option<String>,
    pub mac: Option<String>,
    pub ip_address: Option<String>,
    pub gateway: Option<String>,
    pub dns_primary: Option<String>,
    pub dns_secondary: Option<String>,
}

/// Splits a line from the nmcli terse output (-t).
/// Properly supports colon characters escaped with a backslash (`\:`).
fn split_terse_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_c) = chars.peek()
                && (next_c == ':' || next_c == '\\')
            {
                current.push(next_c);
                chars.next(); // Consume the next character
                continue;
            }
            current.push(c);
        } else if c == ':' {
            parts.push(current);
            current = String::new();
        } else {
            current.push(c);
        }
    }
    parts.push(current);
    parts
}

/// Identifies the operating frequency band based on the frequency string (e.g., "5180 MHz").
fn get_wifi_band(freq_str: &str) -> String {
    let freq_num = freq_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<i32>().ok());

    match freq_num {
        Some(f) if (2400..=2500).contains(&f) => "2.4 GHz".to_string(),
        Some(f) if (4900..=5900).contains(&f) => "5 GHz".to_string(),
        Some(f) if (5925..=7125).contains(&f) => "6 GHz".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Retrieves a list of available Wi-Fi networks in range.
/// Uses the command: `nmcli -t -f ACTIVE,BSSID,SSID,CHAN,FREQ,SIGNAL,SECURITY dev wifi list`
pub async fn get_wifi_list() -> Result<Vec<WifiNetwork>, AppError> {
    let output = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "ACTIVE,BSSID,SSID,CHAN,FREQ,SIGNAL,SECURITY",
            "dev",
            "wifi",
            "list",
        ])
        .output()
        .await
        .map_err(|e| AppError::WifiScan(format!("Failed to execute nmcli command: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::WifiScan(format!("System error: {}", err_msg)));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut wifi_list: Vec<WifiNetwork> = Vec::new();

    // Iterate through each output line from nmcli
    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts = split_terse_line(trimmed);
        if parts.len() < 7 {
            continue;
        }

        let active = parts.first().is_some_and(|s| s.trim() == "yes");
        let bssid = parts
            .get(1)
            .map_or_else(String::new, |s| s.trim().to_string());
        let ssid = parts
            .get(2)
            .map_or_else(String::new, |s| s.trim().to_string());
        let channel = parts
            .get(3)
            .map_or(0, |s| s.trim().parse::<i32>().unwrap_or(0));
        let frequency = parts
            .get(4)
            .map_or_else(String::new, |s| s.trim().to_string());
        let band = get_wifi_band(&frequency);
        let signal = parts
            .get(5)
            .map_or(0, |s| s.trim().parse::<i32>().unwrap_or(0));
        let security = parts
            .get(6)
            .map_or_else(String::new, |s| s.trim().to_string());

        // Ignore networks without an SSID, unless it's a hidden network
        let display_ssid = if ssid.is_empty() {
            "<Hidden Network>".to_string()
        } else {
            ssid
        };

        wifi_list.push(WifiNetwork {
            bssid,
            ssid: display_ssid,
            channel,
            frequency,
            band,
            signal,
            security,
            active,
            rate: None,
            device: None,
            mac: None,
            ip_address: None,
            gateway: None,
            dns_primary: None,
            dns_secondary: None,
        });
    }

    // Sort: Active connection first, remaining networks sorted by band priority then signal strength descending
    wifi_list.sort_by(|a, b| {
        if a.active && !b.active {
            std::cmp::Ordering::Less
        } else if !a.active && b.active {
            std::cmp::Ordering::Greater
        } else {
            // Sort by frequency band priority (6 GHz > 5 GHz > 2.4 GHz > Unknown)
            let band_priority = |band: &str| -> i32 {
                if band.contains("6 GHz") {
                    3
                } else if band.contains("5 GHz") {
                    2
                } else if band.contains("2.4 GHz") {
                    1
                } else {
                    0
                }
            };
            let a_priority = band_priority(&a.band);
            let b_priority = band_priority(&b.band);

            if a_priority != b_priority {
                b_priority.cmp(&a_priority) // Descending
            } else {
                b.signal.cmp(&a.signal) // Same band, sort by signal strength descending
            }
        }
    });

    Ok(wifi_list)
}

/// Connects to a Wi-Fi network using BSSID (MAC Address), SSID, and an optional password.
/// Optionally locks the connection profile to this specific BSSID to prevent roaming.
/// Uses the command: `nmcli dev wifi connect <bssid> password <password>`
/// and `nmcli connection modify <uuid> 802-11-wireless.bssid <bssid>` for BSSID locking.
pub async fn connect_wifi(
    bssid: String,
    ssid: String,
    password: Option<String>,
    lock_bssid: bool,
) -> Result<String, AppError> {
    // Check if a connection profile for this SSID already exists
    let profile_exists = Command::new("nmcli")
        .args(["connection", "show", &ssid])
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);

    let use_password = if profile_exists {
        // Fetch the saved password for comparison
        let saved_pwd = get_wifi_password(&ssid).await.unwrap_or_default();
        let input_pwd = password.as_deref().unwrap_or("").trim();

        if input_pwd != saved_pwd.trim() {
            // User entered a new password. To avoid the "key-mgmt: property is missing" bug
            // when connecting to an existing profile with a new password, we delete the existing
            // profile and let nmcli create a clean new one with the new password.
            let _ = Command::new("nmcli")
                .args(["connection", "delete", &ssid])
                .output()
                .await;
            true
        } else {
            false // Password matches the saved one, connect without password arg to avoid the bug
        }
    } else {
        true // New network, use the provided password if any
    };

    let mut cmd = Command::new("nmcli");
    cmd.arg("dev").arg("wifi").arg("connect").arg(&bssid);

    if use_password
        && let Some(ref pwd) = password
        && !pwd.trim().is_empty()
    {
        cmd.arg("password").arg(pwd);
    }

    let output = cmd.output().await.map_err(|e| {
        AppError::WifiConnect(format!("Failed to invoke connection command: {}", e))
    })?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::WifiConnect(format!(
            "Wi-Fi connection error: {}",
            err_msg
        )));
    }

    let success_msg = String::from_utf8_lossy(&output.stdout).to_string();

    // Query active connections to find the UUID of the newly activated connection profile
    let list_output = Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,NAME,UUID,TYPE", "connection", "show"])
        .output()
        .await
        .map_err(|e| AppError::WifiSettings(format!("Failed to query connections list: {}", e)))?;

    if list_output.status.success() {
        let list_stdout = String::from_utf8_lossy(&list_output.stdout);
        let mut active_uuid = None;

        for line in list_stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parts = split_terse_line(trimmed);
            if parts.len() < 4 {
                continue;
            }
            let active = parts.first().is_some_and(|s| s.trim() == "yes");
            let uuid = parts.get(2).map(|s| s.trim().to_string());
            let conn_type = parts.get(3).map(|s| s.trim());

            if active && conn_type == Some("802-11-wireless") {
                active_uuid = uuid;
                break;
            }
        }

        if let Some(uuid) = active_uuid {
            let mut modify_cmd = Command::new("nmcli");
            modify_cmd.arg("connection").arg("modify").arg(&uuid);

            if lock_bssid {
                modify_cmd.arg("802-11-wireless.bssid").arg(&bssid);
            } else {
                modify_cmd.arg("802-11-wireless.bssid").arg("");
            }

            let modify_output = modify_cmd.output().await.map_err(|e| {
                AppError::WifiSettings(format!(
                    "Failed to update connection profile BSSID setting: {}",
                    e
                ))
            })?;

            if !modify_output.status.success() {
                let modify_err = String::from_utf8_lossy(&modify_output.stderr).to_string();
                return Ok(format!(
                    "Connected successfully, but failed to update BSSID configuration: {}",
                    modify_err
                ));
            }

            // Fix for the first-time connection roaming bug:
            // If the active BSSID is different from the target BSSID, and we want to lock the BSSID,
            // we trigger an immediate reconnection with the modified locked profile.
            if lock_bssid {
                let mut current_active_bssid = None;
                let active_bssid_output = Command::new("nmcli")
                    .args(["-t", "-f", "ACTIVE,BSSID", "device", "wifi", "list"])
                    .output()
                    .await;

                if let Ok(bssid_out) = active_bssid_output
                    && bssid_out.status.success()
                {
                    let bssid_stdout = String::from_utf8_lossy(&bssid_out.stdout);
                    for line in bssid_stdout.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        let parts = split_terse_line(trimmed);
                        if parts.len() < 2 {
                            continue;
                        }
                        let active = parts.first().is_some_and(|s| s.trim() == "yes");
                        let raw_mac = parts.get(1).map(|s| s.trim().to_string());
                        if active {
                            current_active_bssid = raw_mac.map(|m| m.replace("\\:", ":"));
                            break;
                        }
                    }
                }

                if let Some(ref active_mac) = current_active_bssid
                    && active_mac.trim().to_lowercase() != bssid.trim().to_lowercase()
                {
                    // Reconnection needed because we associated to a different BSSID (e.g. 2.4GHz) initially.
                    let up_output = Command::new("nmcli")
                        .args(["connection", "up", &uuid])
                        .output()
                        .await
                        .map_err(|e| {
                            AppError::WifiConnect(format!(
                                "Failed to force correct BSSID connection: {}",
                                e
                            ))
                        })?;

                    if !up_output.status.success() {
                        let up_err = String::from_utf8_lossy(&up_output.stderr).to_string();
                        // We return Ok since the first association succeeded, but warn that lock failed.
                        return Ok(format!(
                            "Connected successfully, but failed to force association with target BSSID: {}",
                            up_err
                        ));
                    }
                }
            }
        }
    }

    Ok(format!("Connected successfully! Details: {}", success_msg))
}

/// Retrieves the locked BSSID for a specific connection profile (SSID) if configured.
/// Returns an empty string if there is no lock or if the profile doesn't exist.
pub async fn get_wifi_locked_bssid(ssid: &str) -> Result<String, AppError> {
    let output = Command::new("nmcli")
        .args([
            "-s",
            "-g",
            "802-11-wireless.bssid",
            "connection",
            "show",
            ssid,
        ])
        .output()
        .await
        .map_err(|e| {
            AppError::WifiSettings(format!("Failed to read connection BSSID info: {}", e))
        })?;

    if !output.status.success() {
        return Ok(String::new());
    }

    // NetworkManager returns escaped colons, e.g., "00\:11\:22\:33\:44\:55". Unescape it.
    let raw_bssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let clean_bssid = raw_bssid.replace("\\:", ":");
    Ok(clean_bssid)
}

/// Retrieves a list of saved Wi-Fi connections (SSIDs) on the system.
/// Uses the command: `nmcli -g NAME,TYPE connection show` and filters for `802-11-wireless`.
#[allow(dead_code)]
pub async fn get_saved_wifi_list() -> Result<Vec<String>, AppError> {
    let output = Command::new("nmcli")
        .args(["-g", "NAME,TYPE", "connection", "show"])
        .output()
        .await
        .map_err(|e| {
            AppError::WifiSettings(format!("Failed to execute nmcli connection command: {}", e))
        })?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::WifiSettings(format!("System error: {}", err_msg)));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut saved_list: Vec<String> = Vec::new();

    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse line formatted as "NAME:TYPE"
        let parts = split_terse_line(trimmed);
        if parts.len() < 2 {
            continue;
        }

        let name = match parts.first() {
            Some(n) => n.trim().to_string(),
            None => continue,
        };
        let conn_type = match parts.get(1) {
            Some(t) => t.trim(),
            None => continue,
        };

        if conn_type == "802-11-wireless" {
            saved_list.push(name);
        }
    }

    Ok(saved_list)
}

/// Retrieves the saved WPA/WEP password for a specific Wi-Fi connection.
/// Uses the command: `nmcli -s -g 802-11-wireless-security.psk connection show <ssid>`
pub async fn get_wifi_password(ssid: &str) -> Result<String, AppError> {
    let output = Command::new("nmcli")
        .args([
            "-s",
            "-g",
            "802-11-wireless-security.psk",
            "connection",
            "show",
            ssid,
        ])
        .output()
        .await
        .map_err(|e| {
            AppError::WifiSettings(format!(
                "Failed to execute nmcli command for password: {}",
                e
            ))
        })?;

    if !output.status.success() {
        // Return empty string if password cannot be loaded or is not set yet.
        return Ok(String::new());
    }

    let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(password)
}

/// Structure holding details about the active network interface.
#[derive(Debug, Default)]
struct ActiveDeviceDetails {
    device: Option<String>,
    mac: Option<String>,
    ip_address: Option<String>,
    gateway: Option<String>,
    dns_primary: Option<String>,
    dns_secondary: Option<String>,
    realtime_rate: Option<String>,
}

/// Retrieves additional network interface configuration details for a connected Wi-Fi device
async fn get_active_device_details() -> ActiveDeviceDetails {
    let mut details = ActiveDeviceDetails::default();

    // 1. Find active Wi-Fi device interface name
    let dev_output = match Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,TYPE,STATE", "device"])
        .output()
        .await
    {
        Ok(out) => out,
        Err(_) => return details,
    };

    if !dev_output.status.success() {
        return details;
    }

    let dev_stdout = String::from_utf8_lossy(&dev_output.stdout);
    let mut active_interface = None;

    for line in dev_stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts = split_terse_line(trimmed);
        if parts.len() < 3 {
            continue;
        }
        let device = parts.first().map(|s| s.trim().to_string());
        let dev_type = parts.get(1).map(|s| s.trim());
        let state = parts.get(2).map(|s| s.trim());

        if dev_type == Some("wifi") && state == Some("connected") {
            active_interface = device;
            break;
        }
    }

    let interface = match active_interface {
        Some(iface) => iface,
        None => return details,
    };

    details.device = Some(interface.clone());

    // 2. Query detailed information for this active device interface
    let show_output = match Command::new("nmcli")
        .args(["device", "show", &interface])
        .output()
        .await
    {
        Ok(out) => out,
        Err(_) => return details,
    };

    if show_output.status.success() {
        let show_stdout = String::from_utf8_lossy(&show_output.stdout);

        for line in show_stdout.lines() {
            let trimmed = line.trim();
            let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
            if parts.len() < 2 {
                continue;
            }
            let key = parts.first().map(|s| s.trim()).unwrap_or("");
            let val = parts.get(1).map(|s| s.trim()).unwrap_or("");

            if val == "--" || val.is_empty() {
                continue;
            }

            match key {
                "GENERAL.HWADDR" => details.mac = Some(val.to_string()),
                "IP4.ADDRESS[1]" => {
                    let ip = val.split('/').next().unwrap_or(val).trim().to_string();
                    details.ip_address = Some(ip);
                }
                "IP4.GATEWAY" => details.gateway = Some(val.to_string()),
                "IP4.DNS[1]" => details.dns_primary = Some(val.to_string()),
                "IP4.DNS[2]" => details.dns_secondary = Some(val.to_string()),
                _ => {}
            }
        }
    }

    // 3. Query real-time link bitrate via iw command
    if let Ok(iw_output) = Command::new("iw")
        .args(["dev", &interface, "link"])
        .output()
        .await
        && iw_output.status.success()
    {
        let iw_stdout = String::from_utf8_lossy(&iw_output.stdout);
        for line in iw_stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("tx bitrate:") {
                if let Some(part) = trimmed.strip_prefix("tx bitrate:") {
                    let words: Vec<&str> = part.split_whitespace().collect();
                    if words.len() >= 2 {
                        let speed = words.first().unwrap_or(&"");
                        let unit = words.get(1).unwrap_or(&"");
                        details.realtime_rate = Some(format!("{} {}", speed, unit));
                    }
                }
                break;
            }
        }
    }

    details
}

/// Retrieves the details of the currently active Wi-Fi connection.
/// Uses a quick nmcli cached query without forcing a hardware scan.
pub async fn get_active_wifi(full_details: bool) -> Result<Option<WifiNetwork>, AppError> {
    let output = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "ACTIVE,BSSID,SSID,CHAN,FREQ,SIGNAL,SECURITY,RATE",
            "dev",
            "wifi",
            "list",
        ])
        .output()
        .await
        .map_err(|e| AppError::WifiScan(format!("Failed to query active connection: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::WifiScan(format!("System error: {}", err_msg)));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);

    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts = split_terse_line(trimmed);
        if parts.len() < 8 {
            continue;
        }

        let active = parts.first().is_some_and(|s| s.trim() == "yes");
        if active {
            let bssid = parts
                .get(1)
                .map_or_else(String::new, |s| s.trim().to_string());
            let ssid = parts
                .get(2)
                .map_or_else(String::new, |s| s.trim().to_string());
            let channel = parts
                .get(3)
                .map_or(0, |s| s.trim().parse::<i32>().unwrap_or(0));
            let frequency = parts
                .get(4)
                .map_or_else(String::new, |s| s.trim().to_string());
            let band = get_wifi_band(&frequency);
            let signal = parts
                .get(5)
                .map_or(0, |s| s.trim().parse::<i32>().unwrap_or(0));
            let security = parts
                .get(6)
                .map_or_else(String::new, |s| s.trim().to_string());
            let rate = parts
                .get(7)
                .map_or_else(String::new, |s| s.trim().to_string());

            let display_ssid = if ssid.is_empty() {
                "<Hidden Network>".to_string()
            } else {
                ssid
            };

            let (rate_val, device, mac, ip_address, gateway, dns_primary, dns_secondary) =
                if full_details {
                    let details = get_active_device_details().await;
                    (
                        details.realtime_rate.unwrap_or(rate),
                        details.device,
                        details.mac,
                        details.ip_address,
                        details.gateway,
                        details.dns_primary,
                        details.dns_secondary,
                    )
                } else {
                    (rate, None, None, None, None, None, None)
                };

            return Ok(Some(WifiNetwork {
                bssid,
                ssid: display_ssid,
                channel,
                frequency,
                band,
                signal,
                security,
                active,
                rate: Some(rate_val),
                device,
                mac,
                ip_address,
                gateway,
                dns_primary,
                dns_secondary,
            }));
        }
    }

    Ok(None)
}
