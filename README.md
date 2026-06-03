# 🛡️ WiWarp - Quản lý Wi-Fi & Cloudflare WARP (Slint UI & Rust)

<p align="center">
  <img src="src/app.slint" alt="WiWarp UI Style" width="120px" height="120px" style="margin-bottom: 20px; border-radius: 8px;"/>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Slint-v1.9.0-blue?style=for-the-badge&logo=slint&logoColor=white" alt="Slint UI" />
  <img src="https://img.shields.io/badge/Rust-Latest-orange?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/Tokio-Async-red?style=for-the-badge&logo=tokio&logoColor=white" alt="Tokio Async" />
  <img src="https://img.shields.io/badge/Fedora-Linux-3C6EB4?style=for-the-badge&logo=fedora&logoColor=white" alt="Fedora OS" />
</p>

**WiWarp** là một ứng dụng Desktop cao cấp, gọn nhẹ và tối ưu hóa cực đỉnh được xây dựng trên nền tảng **Slint UI framework** và ngôn ngữ lập trình **Rust** bất đồng bộ (`tokio`). Ứng dụng cung cấp giao diện người dùng theo phong cách *Cyberpunk Glassmorphism* tuyệt đẹp và mượt mà, giúp người dùng trên hệ điều hành **Fedora Linux** giải quyết triệt để các bài toán kết nối mạng phức tạp, quản lý Wi-Fi cục bộ và điều khiển dịch vụ **Cloudflare WARP (1.1.1.1)** trực quan mà không tốn tài nguyên hệ thống.

---

## ⚡ 2 TÍNH NĂNG TRỌNG TÂM ĐỘC QUYỀN

### 🛡️ 1. Giải Pháp Cloudflare WARP cho Fedora Linux (Vượt Qua Giới Hạn `webkit2gtk3`)

> [!IMPORTANT]
> **Vấn đề kỹ thuật:** Ứng dụng Cloudflare WARP GUI chính thức trên Linux yêu cầu thư viện đồ họa cũ `webkit2gtk3`. Thư viện lỗi thời này đã bị loại bỏ hoàn toàn trên các bản phân phối Fedora hiện đại (như Fedora 39, 40, 41+), khiến người dùng không thể cài đặt hoặc sử dụng bình thường được.

**Cách WiWarp giải quyết triệt để:**
* **Bypass Dependency thông minh:** Sử dụng backend Rust kết hợp với **Trình Cài Đặt Terminal Tương tác cao cấp (Interactive Terminal Installer)**.
  1. Tự động phát hiện các ứng dụng Terminal Emulator (như GNOME Terminal, Ptyxis, Konsole, v.v.) hiện có trên máy người dùng và khởi chạy một cửa sổ dòng lệnh thực tế.
  2. Tự động tải gói cài đặt chính thức bằng `dnf download cloudflare-warp` và thực hiện lệnh cài đặt nâng cấp bỏ qua kiểm tra thư viện bị thiếu thông qua script tạm thời `/tmp/install_warp_wizard_{PID}.sh` được bảo mật bằng quyền hạn nguyên tử `0o700` (chỉ cho phép owner) và chống Symlink Attack bằng `.create_new(true)`: `sudo rpm -Uvh --nodeps /tmp/cloudflare-warp-*.rpm`.
  3. Kích hoạt dịch vụ hệ thống `warp-svc` và hướng dẫn đăng ký client mới (`warp-cli registration new` - xử lý đồng ý các điều khoản sử dụng TOS trực quan trong cửa sổ terminal thật).
* **Tự dọn dẹp hệ thống (Self-cleaning Trap):** Sử dụng các bẫy tín hiệu `trap` trong script Bash để tự động xóa sạch toàn bộ tệp RPM tạm (~220MB) và **tự xóa chính tệp script** `.sh` tạm khi kết thúc thành công hoặc khi người dùng bấm `Ctrl+C` hủy bỏ giữa chừng, kết hợp với cơ chế dọn dẹp chủ động của Backend nếu khởi động terminal thất bại, đảm bảo không lưu lại bất cứ rác hệ thống nào.
* **Bật/Tắt dễ dàng**: Công tắc Toggle Switch phong cách hiện đại để bật/tắt kết nối WARP chỉ với một chạm từ giao diện đồ họa.
* **Thăm dò trạng thái liên tục (Polling)**: Đồng bộ hóa giao diện người dùng theo thời gian thực (mỗi 1 giây) với các trạng thái hoạt động thực tế: *Connecting...*, *Connected*, *Disconnected*, hoặc *Not Installed*.
* **Linh hoạt chuyển đổi 3 chế độ hoạt động (WARP Modes)**:
  * ⚡ **DNS over DoH**: Chỉ thực hiện mã hóa và bảo mật các truy vấn DNS thông qua HTTPS (chế độ DNS-only).
  * 🛡️ **WARP (Cơ bản)**: Định tuyến toàn bộ lưu lượng mạng của bạn qua mạng riêng ảo VPN của Cloudflare.
  * 🔒 **WARP + DoH**: Kết hợp hoàn hảo cả VPN bảo mật lưu lượng lẫn mã hóa các truy vấn DNS an toàn tuyệt đối.

---

### 📶 2. Khóa Mạng Wi-Fi Theo BSSID (MAC Address) & Băng Tần (Hỗ Trợ Wi-Fi 6/6E)

> [!TIP]
> **Vấn đề kỹ thuật:** Trong môi trường nhiều Access Point (AP) phát cùng một tên mạng SSID (hệ thống Mesh gia đình/doanh nghiệp) hoặc Router phát song song nhiều băng tần (2.4 GHz, 5 GHz, 6 GHz) trên cùng một SSID. Hệ điều hành thường tự động chọn AP và nhảy mạng liên tục (roaming), hoặc bị kẹt ở băng tần 2.4 GHz chậm chạp thay vì 5 GHz / 6 GHz tốc độ cao.

**Cách WiWarp giải quyết triệt để:**
* **Bóc tách Terse Output từ `nmcli` an toàn:** Trình điều khiển Wi-Fi viết bằng Rust thực hiện quét mạng chi tiết ở dạng Terse (`nmcli -t`), xử lý thông minh các ký tự đặc biệt được trốn thoát (escaped colon `\:`), đảm bảo độ tin cậy và không bao giờ crash.
* **Nhận diện Băng tần và Wi-Fi 6/6E:** Tự động phân tích tần số hoạt động (Frequency) để xác định chính xác băng tần hoạt động:
  * **2.4 GHz** (2400MHz - 2500MHz)
  * **5 GHz** (4900MHz - 5900MHz)
  * **6 GHz - Wi-Fi 6/6E** (5925MHz - 7125MHz) mang lại tốc độ truyền tải cực cao và độ trễ siêu thấp.
* **Liệt kê tường minh theo BSSID:** Liệt kê riêng biệt từng Access Point vật lý khả dụng với địa chỉ MAC (BSSID) cụ thể, băng tần, kênh, độ mạnh tín hiệu (%) và chuẩn bảo mật kể cả khi chúng có trùng tên SSID.
* **Khóa cứng kết nối theo địa chỉ MAC:** Khi thực hiện kết nối, WiWarp hỗ trợ tùy chọn **Khóa BSSID (Lock BSSID)**. Khi được kích hoạt, Rust Backend sẽ kết nối qua BSSID và cấu hình thuộc tính BSSID cứng của profile kết nối:
  ```bash
  nmcli connection modify <UUID> 802-11-wireless.bssid <BSSID>
  ```
  Điều này ép buộc thiết bị kết nối vào chính xác cột sóng AP và băng tần mong muốn, loại bỏ hoàn toàn hiện tượng roaming nhầm hoặc kết nối vào AP ở xa có tốc độ kém.

---

## ⚙️ CÁC TÍNH NĂNG BỔ TRỢ CAO CẤP

### 📊 Trình Chẩn Đoán & Đo Lường Tốc Độ Thời Gian Thực
* **Đo tốc độ mạng thời gian thực**: Sử dụng cơ chế đọc trực tiếp `/proc/net/dev` của Linux để hiển thị tốc độ Upload/Download thực tế của card mạng theo chu kỳ 1 giây, hiển thị đồ thị lịch sử chuyển động mượt mà.
* **Chẩn đoán Ping liên tục**: Thực hiện ping song song tới **Cloudflare DNS (1.1.1.1)** và **Google DNS (8.8.8.8)** để giám sát độ trễ mạng trực quan.
* **IP Geolocation**: Tự động phát hiện IP công cộng, nhà mạng cung cấp (ISP) và vị trí địa lý của bạn bằng cách gọi API bất đồng bộ thông qua thư viện native `reqwest` thuần Rust (chống nghẽn giao diện và tối ưu hóa vượt bậc so với việc fork tiến trình `curl` bên ngoài). Hỗ trợ cơ chế **thử lại thông minh** (retry 3 lần, delay 1s) khi mạng thay đổi đột ngột và cơ chế **cooldown 30s** để tiết kiệm tài nguyên mạng nhưng sẽ **tự động kích hoạt ngay lập tức** khi có thay đổi trạng thái WARP hoặc mạng Wi-Fi.

### 📋 Console Logs & Toast Thông Báo Cao Cấp
* **Nhật ký hệ thống mini**: Bảng điều khiển logs thu nhỏ tích hợp trực tiếp trên giao diện hiển thị các tiến trình chạy lệnh hệ thống giúp nhà phát triển và người dùng dễ dàng theo dõi, duy trì tối đa 100 dòng log.
* **Hệ thống Toast hiện đại**: Thông báo nhỏ gọn ở góc màn hình xuất hiện mượt mà khi kết nối thành công hoặc phát sinh lỗi kèm biểu tượng tương thích.

---

## 🛠️ CÔNG NGHỆ SỬ DỤNG

* **Frontend**: **Slint UI framework (v1.9.0)** - Khai báo thiết kế giao diện bằng cú pháp Slint mạnh mẽ, tối ưu hóa kích thước, biên dịch tĩnh trực tiếp sang mã Rust để tăng hiệu năng tối đa mà không tốn CPU cho WebView.
* **Backend**: **Rust** & **Tokio runtime** - Quản lý bất đồng bộ các luồng polling và thực thi hệ thống an toàn.
* **Hệ thống thực thi**: Lệnh hệ thống Linux (`nmcli`, `dnf`, `rpm`, `systemctl`, `iw`, `ping`) được gọi bất đồng bộ thông qua `std::process::Command` từ Rust backend, song song với đó các yêu cầu HTTP (như Geo-IP) được thực hiện native bằng thư viện `reqwest` giúp giao diện UI luôn đạt mức 60 FPS mượt mà (non-blocking).

---

## 📋 YÊU CẦU HỆ THỐNG

* **Hệ điều hành**: Fedora Linux (đã được cấu hình `nmcli` và `dnf`).
* **Công cụ**: Cần cài đặt sẵn Rust để phát triển và biên dịch (không cần cài đặt Node.js/npm).
* **Thư viện đồ họa**: Cần cài đặt các thư viện phát triển hệ thống đồ họa cơ bản trên Linux để biên dịch Slint:
  ```bash
  sudo dnf groupinstall "Development Tools"
  sudo dnf install fontconfig-devel
  ```

---

## 🚀 HƯỚNG DẪN CÀI ĐẶT & PHÁT TRIỂN

Dự án sử dụng hoàn toàn hệ sinh thái của Rust, quy trình cực kỳ đơn giản và nhanh gọn:

### 1. Chuẩn bị môi trường
Hãy đảm bảo bạn đã cài đặt đầy đủ các thư viện phát triển hệ thống trên Fedora:
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install fontconfig-devel openssl-devel curl wget
```

### 2. Khởi chạy ứng dụng ở chế độ Phát triển (Dev Mode)
Để khởi chạy ứng dụng trực quan với tính năng Hot-Reload/Compile và theo dõi tức thì:
```bash
cargo run
```

### 3. Biên dịch đóng gói ứng dụng (Build Production)
Để tạo ra file thực thi đã được tối ưu hóa tối đa và cắt bỏ thông tin debug (release binary):
```bash
cargo build --release
```

### 4. Khởi chạy ứng dụng sau khi Build
Sau khi build thành công, file thực thi duy nhất của bạn sẽ nằm trong thư mục `target/release/`. Bạn có thể chạy trực tiếp bằng lệnh:
```bash
./target/release/wiwarp
```

---

## 📁 CẤU TRÚC THƯ MỤC DỰ ÁN

Mã nguồn dự án được tổ chức tối giản và phân rã thành các module nhỏ độc lập phía Backend nhằm nâng cao tính dễ đọc, dễ bảo trì và tối ưu hiệu suất:

```text
wiwarp/
├── src/                          # Mã nguồn chính của ứng dụng
│   ├── app.slint                 # Giao diện người dùng tập trung (Slint UI Declarative)
│   ├── main.rs                   # Điểm khởi chạy tối giản (Bootstrap)
│   ├── helpers.rs                # Tiện ích định dạng, vẽ đồ thị & các tác vụ GeoIP, Ping dùng chung
│   ├── callbacks.rs              # Đăng ký và xử lý tất cả sự kiện tương tác người dùng (UI Callbacks)
│   ├── polling.rs                # Quản lý tập trung các luồng chạy nền giám sát hệ thống
│   ├── wifi.rs                   # Quản lý nmcli quét, kết nối & khóa cứng BSSID Wi-Fi
│   ├── error.rs                  # Định nghĩa custom error enum (AppError) sử dụng thiserror
│   ├── warp.rs                   # Điều phối dịch vụ Cloudflare WARP & installer terminal tương tác
│   └── net_utils.rs              # Đo tốc độ mạng qua proc/net/dev, Ping chẩn đoán & GeoIP (qua reqwest)
├── Cargo.toml                    # Quản lý thư viện phụ thuộc và lints nghiêm ngặt của Rust
├── Cargo.lock                    # Lưu giữ chính xác các phiên bản dependency đã khóa
├── build.rs                      # Build script tự động biên dịch app.slint lúc compile
├── architecture.md               # 🏛️ Chi tiết kiến trúc phân rã, đồng bộ luồng & polling
├── LICENSE                       # 📄 Giấy phép MIT bản quyền phần mềm
└── README.md                     # Tài liệu hướng dẫn sử dụng dự án (Tệp này)
```

---

## 🏛️ TÀI LIỆU KỸ THUẬT CHI TIẾT

Để tìm hiểu sâu hơn về cách ứng dụng hoạt động, bạn có thể tham khảo tài liệu chuyên biệt sau:
*   [**Tài liệu Kiến trúc Hệ thống (architecture.md)**](file:///home/exblackhole/Desktop/NetWarp-Manager/architecture.md): Giải thích chi tiết về luồng đi của dữ liệu, cơ chế đồng bộ luồng an toàn (thread-safety), chu kỳ polling đa tần số (Multi-Interval Polling Engine), các module Rust và cơ chế bảo mật tránh shell injection.

---

## 🔒 GIẤY PHÉP & BẢO MẬT

Dự án sử dụng các cơ chế bảo mật cao cấp của Rust:
* Mọi câu lệnh can thiệp hệ thống (`nmcli`, `dnf`, `rpm`, `systemctl`) đều được đóng gói an toàn phía Backend Rust. Giao diện Slint hoàn toàn không thể trực tiếp chạy lệnh Shell tùy ý, tránh rủi ro bảo mật shell injection.
* Quá trình cài đặt WARP Daemon được khởi chạy minh bạch trên cửa sổ terminal hệ thống. File script cài đặt tạm thời được tạo lập ngẫu nhiên theo PID trong thư mục `/tmp` với quyền hạn nguyên tử `rwx------` (0700) và cơ chế chống ghi đè/symlink `create_new(true)`, triệt tiêu hoàn toàn các lỗ hổng tranh chấp đặc quyền (race condition) trên Linux.
* Áp dụng quy chuẩn lỗi tập trung `AppError` thông qua crate `thiserror` thay thế cho kiểu chuỗi `String` cũ, cải thiện độ chặt chẽ trong phân tích logic và ngăn ngừa mất mát ngữ cảnh lỗi.
* Áp dụng quy chuẩn lint cực kỳ khắt khe của Rust (`unsafe_code = "deny"`, `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"` trong sản xuất), cam kết ứng dụng hoạt động ổn định và không bao giờ crash bất thường.

Dự án được phân phối dưới dạng phần mềm mã nguồn mở theo cơ chế cấp phép [**MIT License**](file:///home/exblackhole/Desktop/NetWarp-Manager/LICENSE).

---

<p align="center">
  Được hoàn thiện với 💖 dành cho cộng đồng người dùng Fedora Linux.
</p>
