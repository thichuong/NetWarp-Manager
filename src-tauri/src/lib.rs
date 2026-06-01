// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::Command;
use std::env;
use std::collections::HashMap;

// Cấu trúc đại diện cho mạng Wi-Fi trả về cho Frontend
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct WifiNetwork {
    ssid: String,
    signal: i32,
    active: bool,
}

/// Lấy danh sách các mạng Wi-Fi khả dụng xung quanh
/// Sử dụng lệnh `nmcli -t -f ACTIVE,SSID,SIGNAL dev wifi list`
#[tauri::command]
async fn get_wifi_list() -> Result<Vec<WifiNetwork>, String> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "ACTIVE,SSID,SIGNAL", "dev", "wifi", "list"])
        .output()
        .map_err(|e| format!("Không thể thực thi lệnh nmcli: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Lỗi hệ thống: {}", err_msg));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut network_map: HashMap<String, (i32, bool)> = HashMap::new();

    // Duyệt qua từng dòng kết quả từ nmcli
    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Tách trường ACTIVE (yes hoặc no) bằng strip_prefix
        let (content, active) = if let Some(stripped) = trimmed.strip_prefix("yes:") {
            (stripped, true)
        } else if let Some(stripped) = trimmed.strip_prefix("no:") {
            (stripped, false)
        } else {
            continue;
        };

        // Tìm dấu hai chấm cuối cùng để phân tách SSID và cột sóng (SIGNAL)
        if let Some(last_colon_idx) = content.rfind(':') {
            let (raw_ssid, signal_str) = content.split_at(last_colon_idx);
            let signal_str = &signal_str[1..]; // Loại bỏ dấu ':' ở đầu

            // Parse cường độ tín hiệu (SIGNAL)
            if let Ok(signal) = signal_str.trim().parse::<i32>() {
                // Unescape ký tự đặc biệt (nmcli escape dấu hai chấm bằng \:)
                let ssid = raw_ssid.replace("\\:", ":").trim().to_string();
                if ssid.is_empty() {
                    continue;
                }

                // Loại bỏ trùng lặp, giữ mạng có tín hiệu mạnh nhất và ưu tiên active
                network_map
                    .entry(ssid)
                    .and_modify(|(s, a)| {
                        if active {
                            *a = true;
                        }
                        if signal > *s {
                            *s = signal;
                        }
                    })
                    .or_insert((signal, active));
            }
        }
    }

    // Chuyển đổi HashMap sang Vec
    let mut wifi_list: Vec<WifiNetwork> = network_map
        .into_iter()
        .map(|(ssid, (signal, active))| WifiNetwork { ssid, signal, active })
        .collect();

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

/// Kết nối vào một mạng Wi-Fi với SSID và Mật khẩu tùy chọn
/// Sử dụng lệnh `nmcli dev wifi connect <ssid> password <password>`
#[tauri::command]
async fn connect_wifi(ssid: String, password: Option<String>) -> Result<String, String> {
    let mut cmd = Command::new("nmcli");
    cmd.arg("dev").arg("wifi").arg("connect").arg(&ssid);

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_wifi_list,
            connect_wifi,
            install_warp,
            warp_toggle,
            get_warp_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
