# 🛡️ WiWarp - Quản lý Wi-Fi & Cloudflare WARP (Tauri v2)

<p align="center">
  <img src="src/assets/tauri.svg" alt="WiWarp Logo" width="120px" height="120px" style="margin-bottom: 20px;"/>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-v2.0-blue?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri v2" />
  <img src="https://img.shields.io/badge/Rust-Latest-orange?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/Tailwind_CSS-3.x-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white" alt="Tailwind CSS" />
  <img src="https://img.shields.io/badge/Fedora-Linux-3C6EB4?style=for-the-badge&logo=fedora&logoColor=white" alt="Fedora OS" />
</p>

**WiWarp** là một ứng dụng Desktop cao cấp, gọn nhẹ được xây dựng trên nền tảng **Tauri v2**, **Rust** và **Vanilla HTML/CSS/JS** kết hợp **Tailwind CSS**. Ứng dụng cung cấp giao diện người dùng theo phong cách *Cyberpunk Glassmorphism* tuyệt đẹp, giúp người dùng trên hệ điều hành **Fedora Linux** quản lý kết nối Wi-Fi cục bộ và tích hợp, điều khiển dịch vụ **Cloudflare WARP (1.1.1.1)** một cách dễ dàng và trực quan.

---

## ✨ Tính Năng Nổi Bật

### 📶 1. Trình Quản Lý Wi-Fi Thông Minh
*   **Quét mạng thời gian thực**: Sử dụng `nmcli` quét và tìm kiếm nhanh chóng toàn bộ các mạng Wi-Fi khả dụng xung quanh.
*   **Chỉ số sóng trực quan**: Phân cấp tín hiệu mạng động (Sóng cực mạnh, sóng khá, trung bình, yếu) hiển thị qua các biểu tượng SVG tương ứng.
*   **Kết nối an toàn**: Hỗ trợ kết nối Wi-Fi có mật khẩu hoặc không có mật khẩu thông qua hộp thoại nhập mật khẩu (Modal) trượt mượt mà.
*   **Ưu tiên mạng hiện tại**: Tự động đưa Wi-Fi đang kết nối lên đầu danh sách với huy hiệu (Badge) **"Đã kết nối"** và hiệu ứng LED xanh dịu mắt.

### 🛡️ 2. Tích Hợp Sâu Cloudflare WARP (1.1.1.1)
*   **Cài đặt 1-Click tự động**:
    1.  Tải gói cài đặt `cloudflare-warp` RPM chính thức bằng `dnf download`.
    2.  Sử dụng cơ chế `glob` để quét và định vị chính xác tệp `.rpm` cục bộ.
    3.  Thực hiện cài đặt thông qua `pkexec rpm -ivh --nodeps` (hộp thoại đồ họa Polkit tự động hiện lên yêu cầu xác thực bảo mật).
    4.  Kích hoạt và khởi động dịch vụ hệ thống `warp-svc` tức thì qua `systemctl`.
*   **Bật/Tắt dễ dàng**: Công tắc Toggle Switch phong cách iOS hiện đại để bật/tắt kết nối WARP chỉ với một chạm.
*   **Thăm dò trạng thái liên tục (Polling)**: Đồng bộ hóa giao diện người dùng theo thời gian thực (mỗi 3 giây) với các trạng thái: *Đang kết nối...*, *Đã kết nối*, *Đã ngắt kết nối*, hoặc *Chưa cài đặt*.
*   **Hệ thống đèn LED Cyberpunk**: Đèn LED chỉ báo trạng thái kết nối màu Đỏ/Vàng/Xanh/Xám có hiệu ứng nhấp nháy (LED Pulse) cực kỳ bắt mắt.
*   **Linh hoạt chuyển đổi 3 chế độ hoạt động (WARP Modes)**:
    *   ⚡ **DNS over DoH**: Chỉ thực hiện mã hóa và bảo mật các truy vấn DNS thông qua HTTPS.
    *   🛡️ **WARP (Cơ bản)**: Định tuyến toàn bộ lưu lượng mạng của bạn qua mạng riêng ảo VPN của Cloudflare.
    *   🔒 **WARP + DoH**: Kết hợp hoàn hảo cả VPN bảo mật lưu lượng lẫn mã hóa các truy vấn DNS an toàn tuyệt đối.

### 📋 3. Console Logs & Toast Thông Báo Cao Cấp
*   **Nhật ký hệ thống mini**: Bảng điều khiển logs thu nhỏ tích hợp trực tiếp trên giao diện hiển thị các tiến trình chạy lệnh hệ thống giúp nhà phát triển và người dùng dễ dàng theo dõi.
*   **Hệ thống Toast hiện đại**: Thông báo nhỏ gọn ở góc màn hình xuất hiện mượt mà khi kết nối thành công hoặc phát sinh lỗi kèm biểu tượng tương thích.

---

## 🛠️ Công Nghệ Sử Dụng

*   **Frontend**: Vanilla HTML5, JavaScript (ES6+), CSS3 (Custom Scrollbars, animations), Tailwind CSS CDN (với cấu hình Inter font cao cấp).
*   **Backend**: Rust, Tauri v2 (API giao tiếp IPC hiệu suất cao, an toàn).
*   **Hệ thống thực thi**: Lệnh hệ thống Linux (`nmcli`, `dnf`, `rpm`, `systemctl`, `pkexec`) được gọi bất đồng bộ từ Rust giúp giao diện UI không bao giờ bị đóng băng (non-blocking).

---

## 📋 Yêu Cầu Hệ Thống

*   **Hệ điều hành**: Fedora Linux (đã được cấu hình `nmcli` và `dnf`).
*   **Công cụ**: Cần cài đặt sẵn Rust và Node.js trên máy để phát triển/biên dịch.
*   **Quyền hệ thống**: Yêu cầu quyền quản trị (Sudo/Polkit) để cài đặt ứng dụng RPM và kích hoạt dịch vụ hệ thống.

---

## 🚀 Hướng Dẫn Cài Đặt & Phát Triển

### 1. Chuẩn bị môi trường
Hãy đảm bảo bạn đã cài đặt đầy đủ các thư viện phát triển hệ thống trên Fedora:
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget
```

### 2. Cài đặt các gói phụ thuộc NodeJS
Di chuyển vào thư mục dự án và chạy lệnh cài đặt:
```bash
npm install
```

### 3. Khởi chạy ứng dụng ở chế độ Phát triển (Dev Mode)
Để khởi chạy ứng dụng trực quan với tính năng Hot-Reload (tự động cập nhật giao diện khi sửa code):
```bash
npm run tauri dev
```

### 4. Biên dịch đóng gói ứng dụng (Build Production)
Để tạo ra gói cài đặt chính thức cho Linux:
```bash
npm run tauri build
```

---

## 📁 Cấu Trúc Thư Mục Dự Án

```text
NetWarp-Manager/
├── src/                      # Giao diện người dùng (Frontend)
│   ├── assets/               # Hình ảnh, tài nguyên tĩnh
│   ├── index.html            # Cấu trúc giao diện ứng dụng (Tailwind CSS)
│   ├── main.js               # Logic điều khiển, gọi lệnh Tauri và cập nhật DOM
│   └── styles.css            # Tùy biến scrollbar và các hiệu ứng animation nâng cao
├── src-tauri/                # Mã nguồn Backend (Rust & Tauri)
│   ├── src/
│   │   ├── lib.rs            # Định nghĩa các tauri::command (nmcli, warp-cli, rpm, systemctl)
│   │   └── main.rs           # Điểm khởi chạy ứng dụng Tauri
│   ├── Cargo.toml            # Quản lý thư viện phụ thuộc của Rust
│   └── tauri.conf.json       # Tệp cấu hình ứng dụng Tauri v2
├── package.json              # Quản lý script và thư viện Node.js
└── README.md                 # Tài liệu hướng dẫn dự án
```

---

## 🔒 Giấy Phép & Bảo Mật

Dự án sử dụng cơ chế bảo mật cao cấp của Tauri v2:
*   Mọi câu lệnh can thiệp hệ thống (`nmcli`, `dnf`, `rpm`, `systemctl`) đều được đóng gói an toàn phía Backend Rust. Giao diện Frontend hoàn toàn không thể trực tiếp chạy lệnh Shell tùy ý, tránh rủi ro bảo mật injection.
*   Việc cài đặt dịch vụ hệ thống được phân quyền minh bạch thông qua cơ chế Polkit của Linux (`pkexec`), yêu cầu người dùng nhập mật khẩu xác thực đồ họa chuẩn của hệ điều hành Fedora.

---

<p align="center">
  Được hoàn thiện với 💖 dành cho cộng đồng người dùng Fedora Linux.
</p>
