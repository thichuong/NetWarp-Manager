use crate::AppError;
use socket2::{Domain, Protocol, Socket, Type};
use std::sync::LazyLock;
use std::time::Instant;
struct Command;
impl Command {
    fn new(program: &str) -> tokio::process::Command {
        crate::helpers::new_tokio_command(program)
    }
}

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
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .user_agent("wiwarp/0.2.0")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
});

pub async fn trace_ip() -> Result<String, AppError> {
    let mut last_err = None;
    for attempt in 1..=4 {
        match HTTP_CLIENT.get("https://ipwho.is/").send().await {
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
/// Reads directly from `/proc/net/dev` on Linux systems synchronously.
pub fn get_network_io_sync() -> Result<NetworkIO, AppError> {
    use std::io::{BufRead, BufReader};
    let file = std::fs::File::open("/proc/net/dev")
        .map_err(|e| AppError::NetworkIO(format!("Failed to read /proc/net/dev: {}", e)))?;
    let reader = BufReader::new(file);

    let mut total_rx = 0;
    let mut total_tx = 0;

    for line in reader.lines().skip(2).flatten() {
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

#[allow(clippy::indexing_slicing)]
fn checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut chunks = data.chunks_exact(2);
    for chunk in &mut chunks {
        let val = u16::from_be_bytes([chunk[0], chunk[1]]);
        sum += val as u32;
    }
    if let Some(&remainder) = chunks.remainder().first() {
        let val = u16::from_be_bytes([remainder, 0]);
        sum += val as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

#[allow(clippy::indexing_slicing)]
fn build_icmp_request(identifier: u16, seq: u16) -> Vec<u8> {
    let mut packet = vec![0u8; 16];
    packet[0] = 8; // Type: Echo Request
    packet[1] = 0; // Code: 0
    packet[4..6].copy_from_slice(&identifier.to_be_bytes());
    packet[6..8].copy_from_slice(&seq.to_be_bytes());
    packet[8..16].copy_from_slice(b"wiwarp!!");

    let cs = checksum(&packet);
    packet[2..4].copy_from_slice(&cs.to_be_bytes());
    packet
}

#[allow(clippy::indexing_slicing)]
async fn raw_icmp_ping(
    target: std::net::IpAddr,
    timeout_duration: std::time::Duration,
) -> Result<f64, AppError> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4))
        .map_err(|e| AppError::Ping(format!("Failed to create ICMP socket: {}", e)))?;

    socket
        .set_nonblocking(true)
        .map_err(|e| AppError::Ping(format!("Failed to set nonblocking: {}", e)))?;

    let std_sock: std::net::UdpSocket = socket.into();
    let tokio_sock = tokio::net::UdpSocket::from_std(std_sock)
        .map_err(|e| AppError::Ping(format!("Failed to wrap tokio socket: {}", e)))?;

    let identifier = std::process::id() as u16;
    let seq = 1u16;
    let request_packet = build_icmp_request(identifier, seq);
    let dest = std::net::SocketAddr::new(target, 0);

    let start = Instant::now();
    tokio_sock
        .send_to(&request_packet, dest)
        .await
        .map_err(|e| AppError::Ping(format!("Failed to send ICMP packet: {}", e)))?;

    let mut recv_buf = [0u8; 256];

    loop {
        let recv_future = tokio_sock.recv_from(&mut recv_buf);
        match tokio::time::timeout(timeout_duration, recv_future).await {
            Ok(Ok((len, _from_addr))) => {
                if len >= 8 {
                    let icmp_type = recv_buf[0];
                    let icmp_code = recv_buf[1];
                    let rcv_seq = u16::from_be_bytes([recv_buf[6], recv_buf[7]]);

                    if icmp_type == 0 && icmp_code == 0 && rcv_seq == seq {
                        let rtt = start.elapsed().as_secs_f64() * 1000.0;
                        return Ok(rtt);
                    }
                }
            }
            Ok(Err(e)) => {
                return Err(AppError::Ping(format!("Socket recv error: {}", e)));
            }
            Err(_) => {
                return Err(AppError::Ping("ICMP request timed out".to_string()));
            }
        }
    }
}

async fn ping_single_target(target_str: &str, timeout_duration: std::time::Duration) -> PingResult {
    let target_ip: Result<std::net::IpAddr, _> = target_str.parse();

    match target_ip {
        Ok(ip) => match raw_icmp_ping(ip, timeout_duration).await {
            Ok(rtt) => PingResult {
                target: target_str.to_string(),
                latency: Some(rtt),
                status: "Online".to_string(),
            },
            Err(e) => {
                eprintln!(
                    "[INFO] ICMP ping to {} failed: {}. Falling back to TCP connect...",
                    target_str, e
                );
                let start = Instant::now();
                let addr_443 = std::net::SocketAddr::new(ip, 443);
                match tokio::time::timeout(
                    timeout_duration,
                    tokio::net::TcpStream::connect(addr_443),
                )
                .await
                {
                    Ok(Ok(_)) => PingResult {
                        target: target_str.to_string(),
                        latency: Some(start.elapsed().as_secs_f64() * 1000.0),
                        status: "Online".to_string(),
                    },
                    _ => {
                        let start_80 = Instant::now();
                        let addr_80 = std::net::SocketAddr::new(ip, 80);
                        match tokio::time::timeout(
                            timeout_duration,
                            tokio::net::TcpStream::connect(addr_80),
                        )
                        .await
                        {
                            Ok(Ok(_)) => PingResult {
                                target: target_str.to_string(),
                                latency: Some(start_80.elapsed().as_secs_f64() * 1000.0),
                                status: "Online".to_string(),
                            },
                            _ => PingResult {
                                target: target_str.to_string(),
                                latency: None,
                                status: "Offline".to_string(),
                            },
                        }
                    }
                }
            }
        },
        Err(_) => PingResult {
            target: target_str.to_string(),
            latency: None,
            status: "Offline".to_string(),
        },
    }
}

/// Executes parallel quick pings (1 packet, 1s timeout) to a list of target hosts.
/// Returns their respective RTT latency in milliseconds and online status.
pub async fn ping_multiple(targets: &[&str]) -> Result<Vec<PingResult>, AppError> {
    let mut handles = vec![];

    for &target in targets {
        let target_string = target.to_string();
        let handle = tokio::spawn(async move {
            ping_single_target(&target_string, std::time::Duration::from_secs(1)).await
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
