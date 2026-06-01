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
Để tạo ra gói cài đặt chính thức cho Linux (`.deb`, `.rpm`, `.AppImage`):
```bash
npm run tauri build
```

> [!NOTE]  
> Trên các bản phân phối Linux hiện đại (như Fedora 40+, Ubuntu 22.04+, Debian 12+), công cụ `strip` tích hợp trong `linuxdeploy` có thể bị lỗi khi xử lý định dạng phân đoạn `.relr.dyn` mới của hệ thống (lỗi `failed to run linuxdeploy`). 
> Dự án đã được tự động cấu hình tích hợp sẵn biến môi trường `NO_STRIP=1` vào script `tauri` trong `package.json` để bỏ qua tiến trình này và hoàn tất biên dịch thành công.

### 5. Khởi chạy ứng dụng sau khi Build

Sau khi build thành công, bạn có thể khởi chạy ứng dụng bằng một trong hai cách dưới đây:

#### Cách A: Chạy trực tiếp File thực thi (Binary)
Chạy trực tiếp file đã biên dịch bằng lệnh:
```bash
env WEBKIT_DISABLE_COMPOSITING_MODE=1 ./src-tauri/target/release/tauri-app
```

#### Cách B: Chạy gói AppImage di động
Cấp quyền thực thi và khởi chạy gói AppImage đã đóng gói:
```bash
chmod +x ./src-tauri/target/release/bundle/appimage/wiwarp_0.1.0_amd64.AppImage
env WEBKIT_DISABLE_COMPOSITING_MODE=1 ./src-tauri/target/release/bundle/appimage/wiwarp_0.1.0_amd64.AppImage
```

> [!TIP]  
> Biến `env WEBKIT_DISABLE_COMPOSITING_MODE=1` là bắt buộc trên nhiều cấu hình đồ họa Linux (đặc biệt là Wayland) để tránh lỗi giao thức đồ họa/crash hiển thị của WebKit. Nếu ứng dụng vẫn gặp lỗi hiển thị, bạn có thể thử thêm biến `WEBKIT_DISABLE_DMABUF_RENDERER=1`:
> ```bash
> env WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1 ./src-tauri/target/release/tauri-app
> ```

---

## 📁 Cấu Trúc Thư Mục Dự Án

Dự án được phân rã thành các module nhỏ độc lập ở cả hai phía Frontend và Backend nhằm nâng cao tính dễ đọc, dễ bảo trì và tối ưu hiệu suất:

```text
NetWarp-Manager/
├── src/                          # Giao diện người dùng (Frontend)
│   ├── assets/                   # Biểu tượng SVG, logo ứng dụng
│   ├── components/               # Các mảnh giao diện HTML động (Template Components)
│   │   ├── header.html           # Thanh tiêu đề chính và logo
│   │   ├── footer.html           # Thanh chân trang thông tin phiên bản
│   │   ├── speed_wifi_section.html # Biểu đồ tốc độ, chẩn đoán Ping & Wi-Fi
│   │   ├── warp_control_section.html # Toggle Cloudflare WARP, chế độ & logs console
│   │   ├── wifi_modal.html       # Hộp thoại hiển thị danh sách mạng Wi-Fi
│   │   ├── password_modal.html   # Hộp thoại nhập mật khẩu Wi-Fi
│   │   └── toast.html            # Khung hiển thị thông báo góc màn hình
│   ├── js/                       # Logic JavaScript phân rã dạng Module
│   │   ├── loader.js             # Tải động các component HTML vào index.html
│   │   ├── dom.js                # Ánh xạ và quản lý tập trung các phần tử DOM
│   │   ├── state.js              # Quản lý trạng thái chia sẻ toàn cục (Global State)
│   │   ├── ui.js                 # Điều khiển ẩn hiện modal, cập nhật giao diện mạng
│   │   ├── wifi.js               # Quét và gửi tín hiệu kết nối Wi-Fi qua Rust IPC
│   │   ├── warp.js               # Quản lý toggle trạng thái và chế độ Cloudflare WARP
│   │   └── diagnostics.js        # Vòng lặp đo lường tốc độ mạng IO & Ping chẩn đoán
│   ├── index.html                # Điểm neo cấu trúc và liên kết CSS/JS chính
│   ├── main.js                   # Điểm khởi chạy (Entry Point) của Frontend
│   ├── styles.css                # Định nghĩa scrollbar, hoạt ảnh LED và hiệu ứng kính
│   └── design_ui.md              # 🎨 Chi tiết triết lý và quy chuẩn thiết kế UI
├── src-tauri/                    # Mã nguồn Backend (Rust & Tauri)
│   ├── src/
│   │   ├── main.rs               # Điểm khởi chạy ứng dụng (Bootstrap)
│   │   ├── lib.rs                # Đăng ký handler, plugin và cấu hình Tauri IPC
│   │   ├── wifi.rs               # Điều khiển nmcli quét và quản lý kết nối Wi-Fi
│   │   ├── warp.rs               # Điều phối dịch vụ Cloudflare WARP & Polkit installer
│   │   └── net_utils.rs          # Đo tốc độ mạng qua proc/net/dev và Ping chẩn đoán
│   ├── Cargo.toml                # Quản lý thư viện phụ thuộc của Rust
│   └── tauri.conf.json           # File cấu hình ứng dụng Tauri v2
├── architecture.md               # 🏛️ Chi tiết kiến trúc phân rã & luồng dữ liệu IPC
├── LICENSE                       # 📄 Giấy phép MIT bản quyền phần mềm
├── package.json                  # Quản lý script biên dịch và gói node_modules
└── README.md                     # Tài liệu hướng dẫn sử dụng dự án
```

---

## 🏛️ Tài Liệu Kỹ Thuật Chi Tiết

Để tìm hiểu sâu hơn về cách ứng dụng hoạt động, bạn có thể tham khảo các tài liệu chuyên biệt sau:
*   [**Tài liệu Kiến trúc Hệ thống (architecture.md)**](file:///home/exblackhole/Desktop/NetWarp-Manager/architecture.md): Giải thích chi tiết về luồng đi của dữ liệu, cơ chế giao tiếp IPC qua Tauri Bridge, cách hoạt động bất đồng bộ phi chặn (non-blocking thread) và bảo mật Polkit.
*   [**Tài liệu Thiết kế Giao diện (src/design_ui.md)**](file:///home/exblackhole/Desktop/NetWarp-Manager/src/design_ui.md): Mô tả triết lý thiết kế Cyberpunk Glassmorphism kết hợp đơn sắc Monochrome tinh tế, bộ màu sắc chỉ thị trạng thái chức năng, quy chuẩn phông chữ và tối ưu hóa layout 1600x900.

---

## 🔒 Giấy Phép & Bảo Mật

Dự án sử dụng cơ chế bảo mật cao cấp của Tauri v2:
*   Mọi câu lệnh can thiệp hệ thống (`nmcli`, `dnf`, `rpm`, `systemctl`) đều được đóng gói an toàn phía Backend Rust. Giao diện Frontend hoàn toàn không thể trực tiếp chạy lệnh Shell tùy ý, tránh rủi ro bảo mật injection.
*   Việc cài đặt dịch vụ hệ thống được phân quyền minh bạch thông qua cơ chế Polkit của Linux (`pkexec`), yêu cầu người dùng nhập mật khẩu xác thực đồ họa chuẩn của hệ điều hành Fedora.

Dự án được phân phối dưới dạng phần mềm mã nguồn mở theo cơ chế cấp phép kép [**MIT License hoặc Apache License, Version 2.0**](file:///home/exblackhole/Desktop/NetWarp-Manager/LICENSE) (tùy bạn lựa chọn).

---

<p align="center">
  Được hoàn thiện với 💖 dành cho cộng đồng người dùng Fedora Linux.
</p>

