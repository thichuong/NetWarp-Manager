use crate::AppError;
use std::env;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::process::Command;

/// Helper function to detect available terminal emulators on Linux.
/// Returns the terminal name and the appropriate flag to execute a script.
fn find_terminal() -> Option<(String, Vec<String>)> {
    let terminals = vec![
        ("gnome-terminal", vec!["--".to_string()]),
        ("ptyxis", vec!["--".to_string()]),
        ("kgx", vec!["-e".to_string()]),
        ("konsole", vec!["-e".to_string()]),
        ("xfce4-terminal", vec!["-e".to_string()]),
        ("mate-terminal", vec!["-e".to_string()]),
        ("lxterminal", vec!["-e".to_string()]),
        ("xterm", vec!["-e".to_string()]),
    ];

    for (term, args) in terminals {
        if Command::new("which")
            .arg(term)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some((term.to_string(), args));
        }
    }
    None
}

/// Creates an interactive terminal bash script to guide the user through installing Cloudflare WARP,
/// then opens it in an available terminal emulator on Fedora/Linux.
pub async fn install_warp() -> Result<String, AppError> {
    println!("[WARP Installer] Starting Cloudflare WARP interactive installer process...");

    let script_content = r#"#!/bin/bash
# Interactive Cloudflare WARP installer wizard for NetWarp-Manager

# ANSI color escape codes for high-quality visual presentation
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Cleanup handler to ensure downloaded RPM packages and the script itself are removed on script exit or interruption
cleanup() {
    rm -f /tmp/cloudflare-warp-*.rpm 2>/dev/null
    rm -f "$0" 2>/dev/null
}

# Interrupt handler to safely clean up and terminate the shell immediately on Ctrl+C (SIGINT) or SIGTERM
on_interrupt() {
    cleanup
    exit 1
}

trap cleanup EXIT
trap on_interrupt INT TERM

clear
echo -e "${CYAN}============================================================${NC}"
echo -e "${GREEN}${BOLD}      HƯỚNG DẪN CÀI ĐẶT CLOUDFLARE WARP (NETWARP-MANAGER)    ${NC}"
echo -e "${CYAN}============================================================${NC}"
echo ""
echo -e "Trình hướng dẫn này giúp bạn cài đặt Cloudflare WARP an toàn trên Fedora."
echo -e "Các lệnh yêu cầu quyền quản trị sẽ chạy thông qua ${YELLOW}sudo${NC}."
echo -e "Vui lòng nhập mật khẩu hệ thống của bạn khi được yêu cầu."
echo ""

# Step 1: Add Cloudflare repository configuration
echo -e "${YELLOW}${BOLD}[Bước 1/5] Thêm kho lưu trữ Cloudflare WARP...${NC}"
echo -e "Lệnh: ${BLUE}curl -fsSL https://pkg.cloudflareclient.com/cloudflare-warp-ascii.repo | sudo tee /etc/yum.repos.d/cloudflare-warp.repo${NC}"
echo -n "Nhấn ENTER để bắt đầu thực hiện bước 1 (hoặc Ctrl+C để hủy)... "
read -r
if curl -fsSL https://pkg.cloudflareclient.com/cloudflare-warp-ascii.repo | sudo tee /etc/yum.repos.d/cloudflare-warp.repo; then
    echo -e "${GREEN}-> Thành công! Thư viện Cloudflare đã được thêm.${NC}"
else
    echo -e "${RED}-> Thất bại khi tải/ghi file cấu hình!${NC}"
    echo -n "Nhấn ENTER để tiếp tục các bước tiếp theo hoặc Ctrl+C để thoát... "
    read -r
fi
echo ""

# Step 2: Update repository DNF cache
echo -e "${YELLOW}${BOLD}[Bước 2/5] Cập nhật bộ nhớ cache kho lưu trữ DNF...${NC}"
echo -e "Lệnh: ${BLUE}sudo dnf makecache${NC}"
echo -n "Nhấn ENTER để thực hiện... "
read -r
if sudo dnf makecache; then
    echo -e "${GREEN}-> Thành công! DNF cache đã được làm mới.${NC}"
else
    echo -e "${RED}-> Thất bại khi cập nhật cache!${NC}"
    echo -n "Nhấn ENTER để tiếp tục... "
    read -r
fi
echo ""

# Step 3: Install Cloudflare WARP package
echo -e "${YELLOW}${BOLD}[Bước 3/5] Tải và cài đặt gói Cloudflare WARP...${NC}"
echo -e "Lệnh 1 (Tải gói RPM): ${BLUE}dnf download cloudflare-warp --destdir=/tmp${NC}"
echo -e "Lệnh 2 (Cài đặt bỏ qua dependency): ${BLUE}sudo rpm -Uvh --nodeps /tmp/cloudflare-warp-*\$(uname -m).rpm${NC}"

SKIP_INSTALL=false
if command -v warp-cli &>/dev/null; then
    echo -e "${GREEN}Phát hiện Cloudflare WARP đã được cài đặt trước đó trên hệ thống.${NC}"
    echo -n "Bạn có muốn BỎ QUA bước cài đặt này không? (y/n - Mặc định là 'y'): "
    read -r choice
    if [ "$choice" != "n" ] && [ "$choice" != "N" ]; then
        echo -e "${GREEN}-> Đã bỏ qua cài đặt gói.${NC}"
        SKIP_INSTALL=true
    fi
fi

if [ "$SKIP_INSTALL" != "true" ]; then
    echo -n "Nhấn ENTER để tải gói Cloudflare WARP về /tmp... "
    read -r
    # Clean up old RPM files to prevent globbing confusion
    rm -f /tmp/cloudflare-warp-*.rpm
    
    if dnf download cloudflare-warp --destdir=/tmp; then
        echo -e "${GREEN}-> Đã tải xong gói RPM.${NC}"
        echo ""
        echo -e "Tiến hành cài đặt gói RPM và bỏ qua kiểm tra thư viện bị thiếu (như webkit2gtk3)..."
        echo -n "Nhấn ENTER để bắt đầu cài đặt... "
        read -r
        
        # Locate the downloaded RPM file for the current system architecture
        RPM_FILE=$(ls /tmp/cloudflare-warp-*.$(uname -m).rpm 2>/dev/null | head -n 1)
        if [ -n "$RPM_FILE" ] && sudo rpm -Uvh --nodeps "$RPM_FILE"; then
            echo -e "${GREEN}-> Thành công! Gói Cloudflare WARP đã được cài đặt vào hệ thống.${NC}"
            rm -f "$RPM_FILE"
        else
            echo -e "${RED}-> Thất bại khi chạy lệnh cài đặt RPM!${NC}"
            echo -n "Bạn có muốn BỎ QUA lỗi này để chạy tiếp các bước sau không? (y/n): "
            read -r choice
            if [ "$choice" != "y" ] && [ "$choice" != "Y" ]; then
                echo -e "${RED}-> Đã hủy bỏ quá trình cài đặt.${NC}"
                rm -f /tmp/cloudflare-warp-*.rpm
                exit 1
            fi
        fi
    else
        echo -e "${RED}-> Thất bại khi tải gói RPM qua DNF!${NC}"
        echo -n "Bạn có muốn BỎ QUA lỗi này để tiếp tục không? (y/n): "
        read -r choice
        if [ "$choice" != "y" ] && [ "$choice" != "Y" ]; then
            echo -e "${RED}-> Đã hủy bỏ quá trình cài đặt.${NC}"
            exit 1
        fi
    fi
fi
echo ""

# Step 4: Enable warp-svc system service
echo -e "${YELLOW}${BOLD}[Bước 4/5] Kích hoạt dịch vụ hệ thống (warp-svc)...${NC}"
echo -e "Lệnh: ${BLUE}sudo systemctl enable --now warp-svc${NC}"
echo -n "Nhấn ENTER để kích hoạt... "
read -r
if sudo systemctl enable --now warp-svc; then
    echo -e "${GREEN}-> Thành công! Dịch vụ warp-svc đã được kích hoạt và chạy nền.${NC}"
else
    echo -e "${RED}-> Thất bại khi kích hoạt dịch vụ!${NC}"
    echo -n "Nhấn ENTER để tiếp tục... "
    read -r
fi
echo ""

# Step 5: Register a new WARP client (handles interactive TOS accept)
echo -e "${YELLOW}${BOLD}[Bước 5/5] Đăng ký Client WARP mới...${NC}"
echo -e "Lệnh: ${BLUE}warp-cli registration new${NC}"
echo -e "${CYAN}Lưu ý:${NC} Thao tác này sẽ hiển thị các điều khoản của Cloudflare. Hãy làm theo hướng dẫn trên màn hình."
echo -n "Nhấn ENTER để thực hiện đăng ký... "
read -r
if warp-cli registration new; then
    echo -e "${GREEN}-> Đăng ký Client WARP thành công!${NC}"
else
    echo -e "${RED}-> Không thể đăng ký mới (có thể bạn đã đăng ký trước đó hoặc có lỗi).${NC}"
fi
echo ""

echo -e "${CYAN}============================================================${NC}"
echo -e "${GREEN}${BOLD}               HOÀN THÀNH QUÁ TRÌNH CÀI ĐẶT!                  ${NC}"
echo -e "${CYAN}============================================================${NC}"
echo -e "Bạn đã hoàn tất hướng dẫn cài đặt Cloudflare WARP."
echo -e "Hãy đóng cửa sổ này và quay lại giao diện đồ họa NetWarp-Manager."
echo ""
echo -n "Nhấn ENTER để thoát cửa sổ này... "
read -r
"#;

    // Use temp_dir safely to write our shell script with a unique name using PID
    let temp_dir = env::temp_dir();
    let pid = std::process::id();
    let script_name = format!("install_warp_wizard_{}.sh", pid);
    let script_path = temp_dir.join(&script_name);
    let script_path_str = script_path.to_str().ok_or_else(|| {
        AppError::WarpInstaller("Failed to construct script path string".to_string())
    })?;

    // Create the interactive script file securely with atomic owner-only permissions (0700)
    // and fail if the file/symlink already exists to prevent symlink attacks.
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o700)
        .open(&script_path)
        .map_err(|e| {
            AppError::WarpInstaller(format!(
                "Failed to create secure script file in temp: {}",
                e
            ))
        })?;

    file.write_all(script_content.as_bytes()).map_err(|e| {
        AppError::WarpInstaller(format!("Failed to write interactive script: {}", e))
    })?;

    // Detect terminal emulator and run the script
    if let Some((term, mut args)) = find_terminal() {
        println!(
            "[WARP Installer] Found terminal emulator '{}'. Spawning installer window...",
            term
        );
        args.push(script_path_str.to_string());
        if let Err(e) = Command::new(&term).args(args).spawn() {
            let _ = std::fs::remove_file(&script_path); // Cleanup on spawn failure
            return Err(AppError::WarpInstaller(format!(
                "Failed to launch terminal '{}': {}",
                term, e
            )));
        }

        Ok(
            "Terminal opened successfully. Please complete the steps in the new window."
                .to_string(),
        )
    } else {
        let _ = std::fs::remove_file(&script_path); // Cleanup on no terminal
        let err_msg = "No suitable terminal emulator (gnome-terminal, konsole, xterm, etc.) was found on your system!".to_string();
        eprintln!("[WARP Installer] Error: {}", err_msg);
        Err(AppError::WarpInstaller(err_msg))
    }
}

/// Enables or disables Cloudflare WARP.
/// If connect is true -> runs `warp-cli connect`
/// If connect is false -> runs `warp-cli disconnect`
pub async fn warp_toggle(connect: bool) -> Result<String, AppError> {
    let action = if connect { "connect" } else { "disconnect" };
    let output = Command::new("warp-cli")
        .arg(action)
        .output()
        .map_err(|e| AppError::WarpControl(format!("Failed to execute warp-cli command: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::WarpControl(format!(
            "WARP control error: {}",
            stderr
        )))
    }
}

/// Retrieves the connection status of Cloudflare WARP.
/// Runs `warp-cli status` and parses the output.
pub async fn get_warp_status() -> Result<String, AppError> {
    let output_result = Command::new("warp-cli").arg("status").output();

    let output = match output_result {
        Ok(o) => o,
        Err(e) => {
            // If warp-cli is not found on the system
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok("Not Installed".to_string());
            }
            return Err(AppError::WarpStatus(format!(
                "Error invoking warp-cli: {}",
                e
            )));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::WarpStatus(format!(
            "Could not get WARP status: {}",
            stderr
        )));
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
pub async fn get_warp_mode() -> Result<String, AppError> {
    let output = Command::new("warp-cli")
        .args(["settings", "list"])
        .output()
        .map_err(|e| AppError::WarpStatus(format!("Failed to call settings list: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::WarpStatus(format!(
            "Error fetching WARP settings: {}",
            err_msg
        )));
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
                if let Some(idx) = trimmed.find("Mode:")
                    && let Some(mode_str) = trimmed.get(idx + 5..)
                {
                    let mode_part = mode_str.trim().to_string();
                    return Ok(mode_part.to_lowercase());
                }
            }
        }
    }
    Ok("unknown".to_string())
}

/// Configures a new operating mode for WARP.
/// Runs `warp-cli mode <mode>`
pub async fn set_warp_mode(mode: String) -> Result<String, AppError> {
    let output = Command::new("warp-cli")
        .args(["mode", &mode])
        .output()
        .map_err(|e| AppError::WarpControl(format!("Failed to execute warp-cli command: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::WarpControl(format!(
            "WARP mode setting error: {}",
            stderr
        )))
    }
}
