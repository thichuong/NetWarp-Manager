# 🛡️ WiWarp - Wi-Fi & Cloudflare WARP Manager (Slint UI & Rust)

<p align="center">
  <img src="assets/logo.svg" alt="WiWarp Logo" width="100px" height="100px"/>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Slint-v1.9.0-blue?style=flat-square&logo=slint&logoColor=white" alt="Slint UI" />
  <img src="https://img.shields.io/badge/Rust-Latest-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/Tokio-Async-red?style=flat-square&logo=tokio&logoColor=white" alt="Tokio Async" />
  <img src="https://img.shields.io/badge/Fedora-Linux-3C6EB4?style=flat-square&logo=fedora&logoColor=white" alt="Fedora OS" />
</p>

**WiWarp** is a lightweight, high-performance Linux desktop client built using **Slint UI** and asynchronous **Rust** (`tokio`). It features a modern *Cyberpunk Glassmorphism* design, optimized for Fedora, Ubuntu, and Debian.

---

## ⚡ Key Features

### 🛡️ 1. Cloudflare WARP Manager (Bypasses Obsolete `webkit2gtk3` GUI Dependency)
* **Interactive Installer Wizard:** Automatically detects local terminal emulators to download, install, and register Cloudflare WARP without requiring deprecated WebKit dependencies.
* **Auto-clean & Secure Setup:** Temporary setup scripts in `/tmp` use restrictive `0o700` permissions and automatically clean up downloaded files and scripts upon completion or interrupt.
* **Toggle & Tunnel Controls:** 1-click toggle switches with real-time state polling (Connecting/Connected/Disconnected).
* **Mode Selection:** Supports **DNS-only (DoH)**, **WARP (VPN)**, and **WARP + DoH** modes.

### 📶 2. Lock Wi-Fi by BSSID & Band (Wi-Fi 6/6E Ready)
* **BSSID Bounding:** Lock your connection to a specific physical AP's MAC address (BSSID) to prevent unwanted roaming in mesh networks or dual-band setups.
* **Frequency Band Detection:** Auto-categorizes scanning targets into **2.4 GHz**, **5 GHz**, and **6 GHz (Wi-Fi 6/6E)**.
* **Robust Scanner:** Parses structured `nmcli -t` scans safely to prevent connection profile crashes.

### 📊 Network Diagnostic Tools
* **Live Speedometer:** Reads `/proc/net/dev` to track real-time upload/download speeds on a smooth SVG chart.
* **Ping Latency:** Monitors parallel ping response times to Cloudflare (`1.1.1.1`) and Google (`8.8.8.8`).
* **Geo-IP Lookup:** Resolves public IP, ISP, and geo-location using native `reqwest` queries with automatic cooling and state-change triggers.

---

## 🚀 Getting Started

### 1. Install System Dependencies
* **Fedora**:
  ```bash
  sudo dnf groupinstall "Development Tools"
  sudo dnf install fontconfig-devel openssl-devel curl wget
  ```
* **Ubuntu/Debian**:
  ```bash
  sudo apt-get install build-essential libfontconfig1-dev libssl-dev curl wget
  ```

### 2. Development & Build Commands
```bash
# Run in development mode
cargo run

# Build release binary
cargo build --release

# Run release version
./target/release/wiwarp

# Install system-wide (adds menu shortcut and 'wiwarp' CLI command)
sudo ./install.sh
```

---

## 📁 Repository Structure & Documentation

*   [README.md](README.md) - This guide.
*   [architecture.md](architecture.md) - Deep dive into system architecture and async loops.
*   [LICENSE](LICENSE) - MIT License.
*   [install.sh](install.sh) - Installer script.
*   [Cargo.toml](Cargo.toml) - Dependencies and project configurations.
*   [build.rs](build.rs) - Slint build script.

**Source Files:**
*   [src/app.slint](src/app.slint) - UI markup and theme styles.
*   [src/main.rs](src/main.rs) - Runtime entry point.
*   [src/callbacks.rs](src/callbacks.rs) - UI-to-Rust event registration.
*   [src/polling.rs](src/polling.rs) - Periodic background loops.
*   [src/wifi.rs](src/wifi.rs) - `nmcli` wrapper and BSSID locking logic.
*   [src/warp.rs](src/warp.rs) - Cloudflare WARP subprocess execution.
*   [src/net_utils.rs](src/net_utils.rs) - Speed, latency, and Geolocation tasks.
*   [src/error.rs](src/error.rs) - AppError wrapper (`thiserror`).
*   [src/helpers.rs](src/helpers.rs) - Logging and chart helpers.

---

## 🏛️ Detailed Technical Documentation

To dive deeper into the technical inner workings of the system:
*   [**System Architecture Documentation (architecture.md)**](architecture.md): Contains in-depth info on data flows, thread-safety, multi-frequency polling engines, Rust modules, command execution parameters, and shell injection prevention.

---

## 🔒 Security & License
* **Shell Command Safety:** Avoids shell injection by running `std::process::Command` with strictly structured argument vectors.
* **Privilege Separation:** WARP setup executes in a native terminal window. Script tempfiles utilize owner-only (`0700`) permissions and auto-clean via bash `trap`.
* **Zero Panic Codebase:** Strict compiler lints enforce `unsafe_code = "deny"`, `unwrap_used = "deny"`, and `panic = "deny"` to ensure crash-free execution.

Distributed under the [MIT License](LICENSE).
