use crate::AppError;
use std::time::Instant;
use tokio::process::Command;

/// Struct containing total received and transmitted bytes of the system.
#[derive(serde::Serialize)]
pub struct NetworkIO {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

/// Struct containing ping results for a specific target.
#[derive(serde::Serialize)]
pub struct PingResult {
    pub target: String,
    pub latency: Option<f64>,
    pub status: String,
}

/// Executes a ping request to the specified target with 4 packets.
/// Uses the system command: `ping -c 4 <target>`
#[allow(dead_code)]
pub async fn ping_target(target: Option<&str>) -> Result<String, AppError> {
    let host = target.unwrap_or("1.1.1.1");
    let clean_host = host.trim();

    if clean_host.is_empty() {
        return Err(AppError::Ping(
            "Ping target host cannot be empty".to_string(),
        ));
    }

    let output = Command::new("ping")
        .args(["-c", "4", clean_host])
        .output()
        .await
        .map_err(|e| AppError::Ping(format!("Failed to execute ping command: {}", e)))?;

    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout_str)
    } else {
        Err(AppError::Ping(if stderr_str.trim().is_empty() {
            stdout_str
        } else {
            stderr_str
        }))
    }
}

/// Traces the current public IP info using a geo-location JSON API.
/// Uses native HTTP crate (reqwest) instead of curl.
/// Implements a 3-retry fallback with 1s delay and 3s connection timeout to gracefully
/// handle temporary network dropouts during WARP mode switching.
pub async fn trace_ip() -> Result<String, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .user_agent("wiwarp/0.2.0")
        .build()
        .map_err(|e| AppError::GeoIp(format!("Failed to build HTTP client: {}", e)))?;

    let mut last_err = None;
    for attempt in 1..=4 {
        match client.get("https://ipwho.is/").send().await {
            Ok(res) => {
                let body = res.text().await.map_err(|e| {
                    AppError::GeoIp(format!("Failed to read geo response body: {}", e))
                })?;
                return Ok(body);
            }
            Err(e) => {
                last_err = Some(e);
                if attempt < 4 {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    Err(AppError::GeoIp(format!(
        "Network lookup failed after 3 retries. Last error: {:?}",
        last_err
    )))
}

/// Fetches the accumulated download (rx) and upload (tx) bytes across active interfaces.
/// Reads directly from `/proc/net/dev` on Linux systems asynchronously.
pub async fn get_network_io() -> Result<NetworkIO, AppError> {
    let content = tokio::fs::read_to_string("/proc/net/dev")
        .await
        .map_err(|e| AppError::NetworkIO(format!("Failed to read /proc/net/dev: {}", e)))?;

    let mut total_rx = 0;
    let mut total_tx = 0;

    for line in content.lines().skip(2) {
        let mut tokens = line.split_whitespace();
        if let Some(interface_str) = tokens.next() {
            let interface = interface_str.trim_end_matches(':');
            if interface == "lo" {
                continue; // Skip loopback
            }

            // Index 1 contains bytes received (rx_bytes)
            if let Some(rx_str) = tokens.next()
                && let Ok(rx) = rx_str.parse::<u64>()
            {
                total_rx += rx;
            }

            // Index 9 contains bytes transmitted (tx_bytes)
            // Skip 7 elements to go from index 1 to index 9
            if let Some(tx_str) = tokens.nth(7)
                && let Ok(tx) = tx_str.parse::<u64>()
            {
                total_tx += tx;
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
pub async fn ping_multiple(targets: &[&str]) -> Result<Vec<PingResult>, AppError> {
    let mut handles = vec![];

    for &target in targets {
        let target_string = target.to_string();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            let output = Command::new("ping")
                .args(["-c", "1", "-W", "1", &target_string])
                .output()
                .await;

            match output {
                Ok(out) if out.status.success() => {
                    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let mut latency = elapsed_ms;

                    for line in stdout.lines() {
                        if let Some(idx) = line.find("time=")
                            && let Some(time_str) = line.get(idx + 5..)
                            && let Some(part) = time_str.split_whitespace().next()
                            && let Ok(parsed) = part.parse::<f64>()
                        {
                            latency = parsed;
                            break;
                        }
                    }

                    PingResult {
                        target: target_string,
                        latency: Some(latency),
                        status: "Online".to_string(),
                    }
                }
                _ => PingResult {
                    target: target_string,
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
