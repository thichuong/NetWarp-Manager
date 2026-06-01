use std::env;
use std::process::Command;

/// Performs the installation and activation of Cloudflare WARP on Fedora.
/// Consists of 4 sequential steps as required.
#[tauri::command]
pub async fn install_warp() -> Result<String, String> {
    // Get the current directory to download the RPM file
    let cwd =
        env::current_dir().map_err(|e| format!("Failed to determine current directory: {}", e))?;

    // Step 1: Download the RPM file using dnf download
    let dnf_output = Command::new("dnf")
        .args(["download", "cloudflare-warp"])
        .current_dir(&cwd)
        .output()
        .map_err(|e| format!("Error executing dnf download: {}", e))?;

    if !dnf_output.status.success() {
        let err_msg = String::from_utf8_lossy(&dnf_output.stderr).to_string();
        return Err(format!(
            "Error downloading Cloudflare WARP via dnf: {}",
            err_msg
        ));
    }

    // Step 2: Use glob to locate the correct rpm file in the current directory
    let pattern = cwd.join("cloudflare-warp-*.x86_64.rpm");
    let pattern_str = pattern.to_str().ok_or("Invalid RPM search path")?;

    let entries = glob::glob(pattern_str)
        .map_err(|e| format!("Failed to initialize glob folder scanner: {}", e))?;

    let rpm_file = entries.flatten().next().ok_or(
        "No RPM file found matching cloudflare-warp-*.x86_64.rpm in the current directory!",
    )?;
    let rpm_file_str = rpm_file
        .to_str()
        .ok_or("Failed to convert RPM path to string")?;

    // Step 3: Install using pkexec rpm bypassing dependencies (Polkit will display graphical auth box)
    let rpm_output = Command::new("pkexec")
        .args(["rpm", "-ivh", "--nodeps", rpm_file_str])
        .output()
        .map_err(|e| format!("Error executing rpm command: {}", e))?;

    if !rpm_output.status.success() {
        let err_msg = String::from_utf8_lossy(&rpm_output.stderr).to_string();
        // If the package is already installed, we can safely continue
        if !err_msg.contains("already installed") {
            return Err(format!("RPM installation error: {}", err_msg));
        }
    }

    // Step 4: Enable and start warp-svc system service using pkexec
    let systemctl_output = Command::new("pkexec")
        .args(["systemctl", "enable", "--now", "warp-svc"])
        .output()
        .map_err(|e| format!("Error executing systemctl: {}", e))?;

    if !systemctl_output.status.success() {
        let err_msg = String::from_utf8_lossy(&systemctl_output.stderr).to_string();
        return Err(format!(
            "Error activating warp-svc system service: {}",
            err_msg
        ));
    }

    Ok("Cloudflare WARP installed and activated successfully!".to_string())
}

/// Enables or disables Cloudflare WARP.
/// If connect is true -> runs `warp-cli connect`
/// If connect is false -> runs `warp-cli disconnect`
#[tauri::command]
pub async fn warp_toggle(connect: bool) -> Result<String, String> {
    let action = if connect { "connect" } else { "disconnect" };
    let output = Command::new("warp-cli")
        .arg(action)
        .output()
        .map_err(|e| format!("Failed to execute warp-cli command: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("WARP control error: {}", stderr))
    }
}

/// Retrieves the connection status of Cloudflare WARP.
/// Runs `warp-cli status` and parses the output.
#[tauri::command]
pub async fn get_warp_status() -> Result<String, String> {
    let output_result = Command::new("warp-cli").arg("status").output();

    let output = match output_result {
        Ok(o) => o,
        Err(e) => {
            // If warp-cli is not found on the system
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok("Not Installed".to_string());
            }
            return Err(format!("Error invoking warp-cli: {}", e));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Could not get WARP status: {}", stderr));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut status = "Disconnected".to_string(); // Default to disconnected

    for line in stdout_str.lines() {
        let trimmed = line.trim();
        // Parse status from the line starting with "Status update:"
        if trimmed.starts_with("Status update:") {
            status = trimmed.replace("Status update:", "").trim().to_string();
            break;
        }
    }

    Ok(status)
}

/// Retrieves the current operating mode of WARP.
/// Runs `warp-cli settings list` and parses the "Mode:" line.
#[tauri::command]
pub async fn get_warp_mode() -> Result<String, String> {
    let output = Command::new("warp-cli")
        .args(["settings", "list"])
        .output()
        .map_err(|e| format!("Failed to call settings list: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Error fetching WARP settings: {}", err_msg));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Mode:") {
            let lower = trimmed.to_lowercase();
            // Check complex modes containing both Warp and DoH first
            if lower.contains("warp")
                && (lower.contains("doh")
                    || lower.contains("dnsoverhttps")
                    || lower.contains("dns-over-https"))
            {
                return Ok("warp+doh".to_string());
            } else if lower.contains("doh")
                || lower.contains("dnsoverhttps")
                || lower.contains("dns-over-https")
            {
                return Ok("doh".to_string());
            } else if lower.contains("warp") {
                return Ok("warp".to_string());
            } else {
                if let Some(idx) = trimmed.find("Mode:") {
                    if let Some(mode_str) = trimmed.get(idx + 5..) {
                        let mode_part = mode_str.trim().to_string();
                        return Ok(mode_part.to_lowercase());
                    }
                }
            }
        }
    }
    Ok("unknown".to_string())
}

/// Configures a new operating mode for WARP.
/// Runs `warp-cli mode <mode>`
#[tauri::command]
pub async fn set_warp_mode(mode: String) -> Result<String, String> {
    let output = Command::new("warp-cli")
        .args(["mode", &mode])
        .output()
        .map_err(|e| format!("Failed to execute warp-cli command: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("WARP mode setting error: {}", stderr))
    }
}
