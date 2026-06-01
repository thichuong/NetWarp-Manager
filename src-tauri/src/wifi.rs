use std::process::Command;

/// Structure representing a Wi-Fi network returned to the frontend with detailed information.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct WifiNetwork {
    bssid: String,
    ssid: String,
    channel: i32,
    frequency: String,
    band: String,
    signal: i32,
    security: String,
    active: bool,
}

/// Splits a line from the nmcli terse output (-t).
/// Properly supports colon characters escaped with a backslash (`\:`).
fn split_terse_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_c) = chars.peek() {
                if next_c == ':' || next_c == '\\' {
                    current.push(next_c);
                    chars.next(); // Consume the next character
                    continue;
                }
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
#[tauri::command]
pub async fn get_wifi_list() -> Result<Vec<WifiNetwork>, String> {
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
        .map_err(|e| format!("Failed to execute nmcli command: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("System error: {}", err_msg));
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
#[tauri::command]
pub async fn connect_wifi(
    bssid: String,
    _ssid: String,
    password: Option<String>,
    lock_bssid: bool,
) -> Result<String, String> {
    let mut cmd = Command::new("nmcli");
    cmd.arg("dev").arg("wifi").arg("connect").arg(&bssid);

    if let Some(ref pwd) = password {
        if !pwd.trim().is_empty() {
            cmd.arg("password").arg(pwd);
        }
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to invoke connection command: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Wi-Fi connection error: {}", err_msg));
    }

    let success_msg = String::from_utf8_lossy(&output.stdout).to_string();

    // Query active connections to find the UUID of the newly activated connection profile
    let list_output = Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,NAME,UUID,TYPE", "connection", "show"])
        .output()
        .map_err(|e| format!("Failed to query connections list: {}", e))?;

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

            let modify_output = modify_cmd
                .output()
                .map_err(|e| format!("Failed to update connection profile BSSID setting: {}", e))?;

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
                    .output();

                if let Ok(bssid_out) = active_bssid_output {
                    if bssid_out.status.success() {
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
                }

                if let Some(ref active_mac) = current_active_bssid {
                    if active_mac.trim().to_lowercase() != bssid.trim().to_lowercase() {
                        // Reconnection needed because we associated to a different BSSID (e.g. 2.4GHz) initially.
                        let up_output = Command::new("nmcli")
                            .args(["connection", "up", &uuid])
                            .output()
                            .map_err(|e| {
                                format!("Failed to force correct BSSID connection: {}", e)
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
    }

    Ok(format!("Connected successfully! Details: {}", success_msg))
}

/// Retrieves the locked BSSID for a specific connection profile (SSID) if configured.
/// Returns an empty string if there is no lock or if the profile doesn't exist.
#[tauri::command]
pub async fn get_wifi_locked_bssid(ssid: String) -> Result<String, String> {
    let output = Command::new("nmcli")
        .args([
            "-s",
            "-g",
            "802-11-wireless.bssid",
            "connection",
            "show",
            &ssid,
        ])
        .output()
        .map_err(|e| format!("Failed to read connection BSSID info: {}", e))?;

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
#[tauri::command]
pub async fn get_saved_wifi_list() -> Result<Vec<String>, String> {
    let output = Command::new("nmcli")
        .args(["-g", "NAME,TYPE", "connection", "show"])
        .output()
        .map_err(|e| format!("Failed to execute nmcli connection command: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("System error: {}", err_msg));
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
#[tauri::command]
pub async fn get_wifi_password(ssid: String) -> Result<String, String> {
    let output = Command::new("nmcli")
        .args([
            "-s",
            "-g",
            "802-11-wireless-security.psk",
            "connection",
            "show",
            &ssid,
        ])
        .output()
        .map_err(|e| format!("Failed to execute nmcli command for password: {}", e))?;

    if !output.status.success() {
        // Return empty string if password cannot be loaded or is not set yet.
        return Ok(String::new());
    }

    let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(password)
}

/// Retrieves the details of the currently active Wi-Fi connection.
/// Uses a quick nmcli cached query without forcing a hardware scan.
#[tauri::command]
pub async fn get_active_wifi() -> Result<Option<WifiNetwork>, String> {
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
        .map_err(|e| format!("Failed to query active connection: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("System error: {}", err_msg));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);

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

            let display_ssid = if ssid.is_empty() {
                "<Hidden Network>".to_string()
            } else {
                ssid
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
            }));
        }
    }

    Ok(None)
}
