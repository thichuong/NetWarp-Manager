use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::time::Instant;

/// Struct containing total received and transmitted bytes of the system.
#[derive(serde::Serialize)]
pub struct NetworkIO {
    rx_bytes: u64,
    tx_bytes: u64,
}

/// Struct containing ping results for a specific target.
#[derive(serde::Serialize)]
pub struct PingResult {
    target: String,
    latency: Option<f64>,
    status: String,
}

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
/// Uses the system command: `curl -s --retry 3 --retry-delay 1 --connect-timeout 3 http://ip-api.com/json/`
/// This bypasses frontend CORS restrictions while providing accurate geo details.
/// Added retry parameters to gracefully handle temporary network dropouts during WARP mode switching.
#[tauri::command]
pub async fn trace_ip() -> Result<String, String> {
    let output = Command::new("curl")
        .args([
            "-s",
            "--retry",
            "3",
            "--retry-delay",
            "1",
            "--connect-timeout",
            "3",
            "http://ip-api.com/json/",
        ])
        .output()
        .map_err(|e| format!("Failed to execute curl command: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Network lookup error: {}", err_msg));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout_str)
}

/// Fetches the accumulated download (rx) and upload (tx) bytes across active interfaces.
/// Reads directly from `/proc/net/dev` on Linux systems.
#[tauri::command]
pub async fn get_network_io() -> Result<NetworkIO, String> {
    let file =
        File::open("/proc/net/dev").map_err(|e| format!("Failed to open /proc/net/dev: {}", e))?;
    let reader = BufReader::new(file);

    let mut total_rx = 0;
    let mut total_tx = 0;

    for (idx, line) in reader.lines().enumerate() {
        if idx < 2 {
            continue; // Skip the header lines
        }
        if let Ok(l) = line {
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() >= 10 {
                if let Some(interface_str) = parts.first() {
                    let interface = interface_str.trim_end_matches(':');
                    if interface == "lo" {
                        continue; // Skip loopback
                    }
                }

                // Index 1 contains bytes received (rx_bytes)
                // Index 9 contains bytes transmitted (tx_bytes)
                if let Some(rx_str) = parts.get(1) {
                    if let Ok(rx) = rx_str.parse::<u64>() {
                        total_rx += rx;
                    }
                }
                if let Some(tx_str) = parts.get(9) {
                    if let Ok(tx) = tx_str.parse::<u64>() {
                        total_tx += tx;
                    }
                }
            }
        }
    }

    Ok(NetworkIO {
        rx_bytes: total_rx,
        tx_bytes: total_tx,
    })
}

/// Executes parallel quick pings (1 packet, 1s timeout) to a list of target hosts.
/// Returns their respective RTT latency in milliseconds and online status.
#[tauri::command]
pub async fn ping_multiple(targets: Vec<String>) -> Result<Vec<PingResult>, String> {
    let mut handles = vec![];

    for target in targets {
        let handle = tauri::async_runtime::spawn(async move {
            let start = Instant::now();
            let output = Command::new("ping")
                .args(["-c", "1", "-W", "1", &target])
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let mut latency = elapsed_ms;

                    // Parse the more accurate RTT value from command line output
                    for line in stdout.lines() {
                        if line.contains("time=") {
                            if let Some(idx) = line.find("time=") {
                                if let Some(time_str) = line.get(idx + 5..) {
                                    let parts: Vec<&str> = time_str.split_whitespace().collect();
                                    if let Some(part) = parts.first() {
                                        if let Ok(parsed) = part.parse::<f64>() {
                                            latency = parsed;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    PingResult {
                        target,
                        latency: Some(latency),
                        status: "Online".to_string(),
                    }
                }
                _ => PingResult {
                    target,
                    latency: None,
                    status: "Offline".to_string(),
                },
            }
        });
        handles.push(handle);
    }

    let mut results = vec![];
    for handle in handles {
        if let Ok(res) = handle.await {
            results.push(res);
        }
    }

    Ok(results)
}
