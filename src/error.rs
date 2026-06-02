use thiserror::Error;

/// Custom Error enumeration for NetWarp-Manager backend operations.
/// Consolidates all system, Wi-Fi, WARP client, network interface, and HTTP failures.
#[derive(Error, Debug)]
pub enum AppError {
    /// Wrapper for standard input/output errors.
    #[error("System I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Wrapper for standard HTTP client network requests.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Represents failure during wireless interface active AP list retrieval.
    #[error("Wi-Fi network scan failed: {0}")]
    WifiScan(String),

    /// Represents failure during active network profile association.
    #[error("Wi-Fi network connection failed: {0}")]
    WifiConnect(String),

    /// Represents errors retrieving Wi-Fi configuration/credentials.
    #[error("Wi-Fi settings access failed: {0}")]
    WifiSettings(String),

    /// Represents installer terminal wrapper wizard errors.
    #[error("WARP installer wizard failed: {0}")]
    WarpInstaller(String),

    /// Represents failures querying Cloudflare WARP client status.
    #[error("WARP status query failed: {0}")]
    WarpStatus(String),

    /// Represents failures triggering WARP connections/toggles.
    #[error("WARP control failed: {0}")]
    WarpControl(String),

    /// Represents errors running ping metrics and parsing raw command outputs.
    #[error("Diagnostics ping failed: {0}")]
    Ping(String),

    /// Represents errors in public IP geolocation lookups.
    #[error("Geo-IP lookup failed: {0}")]
    GeoIp(String),

    /// Represents errors parsing interface metrics in `/proc/net/dev`.
    #[error("Network monitoring query failed: {0}")]
    NetworkIO(String),

    /// Represents generic failure executing standalone OS binaries.
    #[error("Command execution failed: {0}")]
    Command(String),

    /// Represents serialization/deserialization or custom layout failures.
    #[error("Parsing error: {0}")]
    Parse(String),

    /// Represents all other unclassified runtime errors.
    #[error("General system error: {0}")]
    System(String),
}
