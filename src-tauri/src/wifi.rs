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

    // Sort: Active connection first, remaining networks sorted by signal strength descending
    wifi_list.sort_by(|a, b| {
        if a.active && !b.active {
            std::cmp::Ordering::Less
        } else if !a.active && b.active {
            std::cmp::Ordering::Greater
        } else {
            b.signal.cmp(&a.signal)
        }
    });

    Ok(wifi_list)
}

/// Connects to a Wi-Fi network using BSSID (MAC Address) and an optional password.
/// Uses the command: `nmcli dev wifi connect <bssid> password <password>`
#[tauri::command]
pub async fn connect_wifi(bssid: String, password: Option<String>) -> Result<String, String> {
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

    if output.status.success() {
        let success_msg = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(format!("Connected successfully! Details: {}", success_msg))
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Wi-Fi connection error: {}", err_msg))
    }
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
