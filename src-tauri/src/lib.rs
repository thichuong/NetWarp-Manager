// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::Command;
use std::env;

// Cấu trúc đại diện cho mạng Wi-Fi trả về cho Frontend với đầy đủ thông tin chi tiết
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

/// Phân tách dòng kết quả ở chế độ terse (-t) của nmcli
/// Hỗ trợ chuẩn xác các ký tự hai chấm bị escape bằng dấu gạch chéo ngược (`\:`)
fn split_terse_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_c) = chars.peek() {
                if next_c == ':' || next_c == '\\' {
                    current.push(next_c);
                    chars.next(); // tiêu thụ ký tự tiếp theo
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

/// Xác định băng tần hoạt động dựa trên chuỗi tần số (ví dụ: "5180 MHz")
fn get_wifi_band(freq_str: &str) -> String {
    let freq_num = freq_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<i32>().ok());

    match freq_num {
        Some(f) if (2400..=2500).contains(&f) => "2.4 GHz".to_string(),
        Some(f) if (4900..=5900).contains(&f) => "5 GHz".to_string(),
        Some(f) if (5925..=7125).contains(&f) => "6 GHz".to_string(),
        _ => "Không rõ".to_string(),
    }
}

/// Lấy danh sách các mạng Wi-Fi khả dụng xung quanh
/// Sử dụng lệnh `nmcli -t -f ACTIVE,BSSID,SSID,CHAN,FREQ,SIGNAL,SECURITY dev wifi list`
#[tauri::command]
async fn get_wifi_list() -> Result<Vec<WifiNetwork>, String> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,BSSID,SSID,CHAN,FREQ,SIGNAL,SECURITY", "dev", "wifi", "list"])
        .output()
        .map_err(|e| format!("Không thể thực thi lệnh nmcli: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Lỗi hệ thống: {}", err_msg));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut wifi_list: Vec<WifiNetwork> = Vec::new();

    // Duyệt qua từng dòng kết quả từ nmcli
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
        let bssid = parts.get(1).map_or_else(String::new, |s| s.trim().to_string());
        let ssid = parts.get(2).map_or_else(String::new, |s| s.trim().to_string());
        let channel = parts.get(3).map_or(0, |s| s.trim().parse::<i32>().unwrap_or(0));
        let frequency = parts.get(4).map_or_else(String::new, |s| s.trim().to_string());
        let band = get_wifi_band(&frequency);
        let signal = parts.get(5).map_or(0, |s| s.trim().parse::<i32>().unwrap_or(0));
        let security = parts.get(6).map_or_else(String::new, |s| s.trim().to_string());

        // Bỏ qua các mạng không có SSID (trừ khi là mạng ẩn nhưng nmcli thường để SSID trống)
        let display_ssid = if ssid.is_empty() {
            "<Mạng ẩn>".to_string()
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

    // Sắp xếp: Mạng đang kết nối lên đầu tiên, các mạng còn lại sắp xếp theo tín hiệu giảm dần
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

/// Kết nối vào một mạng Wi-Fi bằng BSSID (địa chỉ MAC) và Mật khẩu tùy chọn
/// Sử dụng lệnh `nmcli dev wifi connect <bssid> password <password>`
#[tauri::command]
async fn connect_wifi(bssid: String, password: Option<String>) -> Result<String, String> {
    let mut cmd = Command::new("nmcli");
    cmd.arg("dev").arg("wifi").arg("connect").arg(&bssid);

    if let Some(ref pwd) = password {
        if !pwd.trim().is_empty() {
            cmd.arg("password").arg(pwd);
        }
    }

    let output = cmd.output()
        .map_err(|e| format!("Không thể gọi lệnh kết nối: {}", e))?;

    if output.status.success() {
        let success_msg = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(format!("Kết nối thành công! Chi tiết: {}", success_msg))
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Lỗi kết nối Wi-Fi: {}", err_msg))
    }
}

/// Thực hiện cài đặt và kích hoạt Cloudflare WARP trên Fedora
/// Bao gồm 4 bước tuần tự như yêu cầu
#[tauri::command]
async fn install_warp() -> Result<String, String> {
    // Lấy thư mục hiện tại để tải tệp RPM về
    let cwd = env::current_dir().map_err(|e| format!("Không thể xác định thư mục hiện tại: {}", e))?;

    // Bước 1: Tải về file RPM bằng dnf download
    let dnf_output = Command::new("dnf")
        .args(["download", "cloudflare-warp"])
        .current_dir(&cwd)
        .output()
        .map_err(|e| format!("Lỗi thực thi dnf download: {}", e))?;

    if !dnf_output.status.success() {
        let err_msg = String::from_utf8_lossy(&dnf_output.stderr).to_string();
        return Err(format!("Lỗi khi tải Cloudflare WARP qua dnf: {}", err_msg));
    }

    // Bước 2: Sử dụng glob để quét tìm file rpm đúng định dạng trong thư mục hiện hành
    let pattern = cwd.join("cloudflare-warp-*.x86_64.rpm");
    let pattern_str = pattern.to_str().ok_or("Đường dẫn tìm kiếm RPM không hợp lệ")?;
    
    let entries = glob::glob(pattern_str)
        .map_err(|e| format!("Lỗi khởi tạo bộ quét thư mục glob: {}", e))?;

    let rpm_file = entries.flatten().next()
        .ok_or("Không tìm thấy tệp RPM nào có định dạng cloudflare-warp-*.x86_64.rpm trong thư mục hiện tại!")?;
    let rpm_file_str = rpm_file.to_str().ok_or("Không thể chuyển đổi đường dẫn RPM thành chuỗi ký tự")?;

    // Bước 3: Cài đặt bằng pkexec rpm bỏ qua phụ thuộc (Polkit hiển thị hộp thoại xác thực đồ hoạ)
    let rpm_output = Command::new("pkexec")
        .args(["rpm", "-ivh", "--nodeps", rpm_file_str])
        .output()
        .map_err(|e| format!("Lỗi khi thực thi lệnh rpm: {}", e))?;

    if !rpm_output.status.success() {
        let err_msg = String::from_utf8_lossy(&rpm_output.stderr).to_string();
        // Nếu gói đã được cài đặt từ trước, chúng ta có thể tiếp tục
        if !err_msg.contains("already installed") {
            return Err(format!("Lỗi cài đặt RPM: {}", err_msg));
        }
    }

    // Bước 4: Kích hoạt dịch vụ hệ thống warp-svc bằng pkexec
    let systemctl_output = Command::new("pkexec")
        .args(["systemctl", "enable", "--now", "warp-svc"])
        .output()
        .map_err(|e| format!("Lỗi khi thực thi systemctl: {}", e))?;

    if !systemctl_output.status.success() {
        let err_msg = String::from_utf8_lossy(&systemctl_output.stderr).to_string();
        return Err(format!("Lỗi kích hoạt dịch vụ hệ thống warp-svc: {}", err_msg));
    }

    Ok("Cài đặt và kích hoạt Cloudflare WARP thành công!".to_string())
}

/// Bật hoặc Tắt Cloudflare WARP
/// Nếu connect là true -> chạy `warp-cli connect`
/// Nếu connect là false -> chạy `warp-cli disconnect`
#[tauri::command]
async fn warp_toggle(connect: bool) -> Result<String, String> {
    let action = if connect { "connect" } else { "disconnect" };
    let output = Command::new("warp-cli")
        .arg(action)
        .output()
        .map_err(|e| format!("Không thể thực thi lệnh warp-cli: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Lỗi điều khiển WARP: {}", stderr))
    }
}

/// Lấy trạng thái kết nối Cloudflare WARP
/// Chạy lệnh `warp-cli status` và bóc tách kết quả
#[tauri::command]
async fn get_warp_status() -> Result<String, String> {
    let output_result = Command::new("warp-cli")
        .arg("status")
        .output();

    let output = match output_result {
        Ok(o) => o,
        Err(e) => {
            // Nếu không tìm thấy warp-cli trên hệ thống
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok("Not Installed".to_string());
            }
            return Err(format!("Lỗi khi gọi warp-cli: {}", e));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Không thể lấy trạng thái WARP: {}", stderr));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut status = "Disconnected".to_string(); // Mặc định là đã ngắt kết nối

    for line in stdout_str.lines() {
        let trimmed = line.trim();
        // Bóc tách trạng thái từ dòng bắt đầu bằng "Status update:"
        if trimmed.starts_with("Status update:") {
            status = trimmed.replace("Status update:", "").trim().to_string();
            break;
        }
    }

    Ok(status)
}

/// Lấy chế độ hoạt động hiện tại của WARP
/// Chạy lệnh `warp-cli settings list` và bóc tách "Mode:"
#[tauri::command]
async fn get_warp_mode() -> Result<String, String> {
    let output = Command::new("warp-cli")
        .args(["settings", "list"])
        .output()
        .map_err(|e| format!("Không thể gọi settings list: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Lỗi khi lấy cài đặt WARP: {}", err_msg));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Mode:") {
            let lower = trimmed.to_lowercase();
            // Kiểm tra các chế độ phức hợp chứa cả Warp và DoH trước
            if lower.contains("warp") && (lower.contains("doh") || lower.contains("dnsoverhttps") || lower.contains("dns-over-https")) {
                return Ok("warp+doh".to_string());
            } else if lower.contains("doh") || lower.contains("dnsoverhttps") || lower.contains("dns-over-https") {
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

/// Thiết lập chế độ hoạt động mới cho WARP
/// Chạy lệnh `warp-cli mode <mode>`
#[tauri::command]
async fn set_warp_mode(mode: String) -> Result<String, String> {
    let output = Command::new("warp-cli")
        .args(["mode", &mode])
        .output()
        .map_err(|e| format!("Không thể thực thi lệnh warp-cli: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Lỗi đặt chế độ WARP: {}", stderr))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_wifi_list,
            connect_wifi,
            install_warp,
            warp_toggle,
            get_warp_status,
            get_warp_mode,
            set_warp_mode
        ])
        .run(tauri::generate_context!())
    {
        eprintln!("error while running tauri application: {}", e);
    }
}
