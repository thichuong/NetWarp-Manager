use std::process::Command;

/// Executes a ping request to the specified target with 4 packets.
/// Uses the system command: `ping -c 4 <target>`
#[tauri::command]
pub async fn ping_target(target: Option<String>) -> Result<String, String> {
    let host = target.unwrap_or_else(|| "1.1.1.1".to_string());
    let clean_host = host.trim();

    if clean_host.is_empty() {
        return Err("Ping target host cannot be empty".to_string());
    }

    let output = Command::new("ping")
        .args(["-c", "4", clean_host])
        .output()
        .map_err(|e| format!("Failed to execute ping command: {}", e))?;

    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout_str)
    } else {
        Err(if stderr_str.trim().is_empty() {
            stdout_str
        } else {
            stderr_str
        })
    }
}

/// Traces the current public IP info using a geo-location JSON API.
/// Uses the system command: `curl -s http://ip-api.com/json/`
/// This bypasses frontend CORS restrictions while providing accurate geo details.
#[tauri::command]
pub async fn trace_ip() -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-s", "http://ip-api.com/json/"])
        .output()
        .map_err(|e| format!("Failed to execute curl command: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Network lookup error: {}", err_msg));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout_str)
}
