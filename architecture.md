# 🏛️ Tài liệu Kiến trúc WiWarp (NetWarp-Manager)

Tài liệu này mô tả chi tiết kiến trúc kỹ thuật của **WiWarp**, giải thích cách ứng dụng phân tách các thành phần (Modular Architecture) sử dụng **Slint UI framework** và **Rust backend**, cơ chế đồng bộ hóa bất đồng bộ (non-blocking async) và cách quản lý luồng dữ liệu thời gian thực trong một tiến trình duy nhất (Single-Process Architecture).

---

## 1. Tổng Quan Kiến Trúc

WiWarp được thiết kế theo mô hình **Native Single-Process Desktop Application** sử dụng **Slint**:
*   **Giao diện (Slint UI Layer)**: Khai báo giao diện tập trung trong tệp `src/app.slint`. Tại thời điểm biên dịch (Compile-time), trình biên dịch Slint (`slint-build` trong `build.rs`) sẽ tự động chuyển đổi mã khai báo này thành mã Rust tối ưu hóa cao, tích hợp trực tiếp vào tệp thực thi duy nhất (Single Binary) mà không cần dùng đến Webview hay công nghệ Web cồng kềnh.
*   **Mã nguồn chính (Rust Core Layer)**: Chịu trách nhiệm khởi tạo cửa sổ ứng dụng, đăng ký sự kiện, quản lý vòng lặp chẩn đoán mạng, và thực thi các câu lệnh hệ thống Linux (`nmcli`, `warp-cli`, `iw`, `ping`, `curl`) thông qua các module chuyên biệt.
*   **Mô hình Liên kết Sự kiện (Event Binding & Properties)**: Thay vì sử dụng cầu nối IPC Bridge (như Tauri), Slint và Rust chạy chung trong cùng một tiến trình OS. Logic Rust liên kết trực tiếp với UI thông qua các **Callbacks** (sự kiện từ UI gọi sang Rust) và cập nhật giao diện bằng cách thiết lập các **Properties** hoặc thông qua các **Shared Models** (như danh sách mạng Wi-Fi, lịch sử tốc độ).

```mermaid
graph TD
    subgraph Slint UI Layer
        A[src/app.slint] -->|Compile-time Code Gen| B[Generated AppWindow Struct]
        B -->|Properties Binding| C[UI Properties: wifi_list, speed_stats, warp_status, geo_info...]
        B -->|Event Callbacks| D[Callbacks: on_connect_wifi_clicked, on_warp_toggle_clicked...]
    end

    subgraph Rust Core Layer (Backend)
        E[src/main.rs / Orchestrator] -->|1. Instantiate & Run| B
        E -->|2. Register Callbacks| D
        E -->|3. Set/Update Properties| C
        E -->|4. Push Vector Models| F[slint::VecModel]
        F -->|Binds dynamically| C

        E -->|Mod Route| G[src/wifi.rs]
        E -->|Mod Route| H[src/warp.rs]
        E -->|Mod Route| I[src/net_utils.rs]
    end

    subgraph Tokio Async Runtime
        E -->|tokio::spawn background loop| J[Interval Polling Tasks]
        J -->|Measure bandwidth| K[Read /proc/net/dev]
        J -->|Check latency| L[Parallel ping]
        J -->|Fetch Location| M[curl geo-IP]
        J -->|Query CLI state| N[nmcli / warp-cli status]

        J -->|Thread-safe Update| O[Weak AppWindow]
        O -->|upgrade_in_event_loop| B
    end

    subgraph Linux OS Layer & Services
        G -->|Execute CLI commands| P[nmcli / iw]
        H -->|Interactive Installer| Q[Spawn Terminal Emulator & Bash script]
        H -->|Manage connections| R[warp-cli / systemctl]
        I -->|Fetch public IP| S[curl http://ip-api.com/json/]
        I -->|Proc FS| K
        I -->|Networking| L
    end
```

---

## 2. Phân Rã Thành Phần Backend & Logic (Rust)

Mã nguồn Rust nằm trực tiếp trong thư mục `src/` và được chia nhỏ thành các module chức năng độc lập:

*   **[main.rs](file:///home/exblackhole/Desktop/NetWarp-Manager/src/main.rs)**:
    *   Điểm khởi chạy ứng dụng (Bootstrap), thiết lập cửa sổ `AppWindow`.
    *   Đăng ký toàn bộ callback sự kiện tương tác từ Slint UI (như quét Wi-Fi, kết nối Wi-Fi, bật/tắt WARP, chuyển đổi chế độ WARP, cài đặt WARP Daemon).
    *   Khởi chạy các luồng xử lý nền (Background Tasks) bằng Tokio runtime để giám sát tốc độ mạng, ping độ trễ, đồng bộ trạng thái Wi-Fi/WARP và cập nhật thông tin vị trí địa lý.
*   **[wifi.rs](file:///home/exblackhole/Desktop/NetWarp-Manager/src/wifi.rs)**:
    *   **Quét mạng Wi-Fi lân cận**: Gọi `nmcli -t -f ACTIVE,BSSID,SSID,CHAN,FREQ,SIGNAL,SECURITY dev wifi list`, phân tích output Terse an toàn, hỗ trợ bóc tách các ký tự escaped dấu hai chấm (`\:`).
    *   **Khóa cứng BSSID**: Thực hiện cơ chế kết nối bằng địa chỉ MAC (BSSID) thay vì SSID thông qua `nmcli dev wifi connect <BSSID>`. Nếu chọn khóa BSSID, module sẽ tự động cấu hình `802-11-wireless.bssid <BSSID>` cho profile kết nối và kích hoạt lại để ép thiết bị chỉ kết nối với Access Point vật lý đó, ngăn chặn roaming nhầm sang AP yếu hoặc băng tần chậm.
    *   **Truy xuất thông tin bảo mật**: Đọc mật khẩu đã lưu từ NetworkManager keyring qua `nmcli -s -g 802-11-wireless-security.psk connection show <ssid>`.
    *   **Thông tin chi tiết card mạng**: Sử dụng `iw dev <interface> link` kết hợp `nmcli device show` để lấy các thông số chi tiết (tốc độ truyền tx bitrate thực tế, địa chỉ IP, Gateway, DNS chính/phụ).
*   **[warp.rs](file:///home/exblackhole/Desktop/NetWarp-Manager/src/warp.rs)**:
    *   **Trình cài đặt Terminal Tương tác (Interactive Installer)**: Tự động dò tìm các Terminal Emulator có sẵn trên hệ thống Linux (như `gnome-terminal`, `ptyxis`, `konsole`, `xfce4-terminal`, `xterm`). Sau đó, tự động tạo một tệp script bash tạm `/tmp/install_warp_wizard.sh` chứa đầy đủ hướng dẫn từng bước và gọi terminal độc lập chạy script này.
    *   **Cài đặt an toàn bypass dependency**: Script tự động tải gói cài đặt chính thức bằng `dnf download cloudflare-warp` và thực hiện lệnh cài đặt RPM bỏ qua các dependency bị thiếu (như thư viện đồ họa lỗi thời `webkit2gtk3`): `sudo rpm -Uvh --nodeps /tmp/cloudflare-warp-*.rpm`.
    *   **Tự động dọn dẹp hệ thống (Self-cleaning)**: Đăng ký các bẫy tín hiệu `trap` trong Bash để tự động xóa sạch gói RPM tạm và tự xóa chính tệp script `.sh` khi quá trình kết thúc thành công hoặc khi người dùng ngắt bằng `Ctrl+C`.
    *   **Điều khiển daemon**: Bật/Tắt dịch vụ hệ thống `warp-svc` bằng `systemctl` và thực hiện đăng ký client mới qua `warp-cli registration new`.
    *   **Điều khiển kết nối WARP**: Thực hiện bật/tắt kết nối (`warp-cli connect/disconnect`) và cấu hình 3 chế độ hoạt động chính:
        *   `doh` (DNS over HTTPS)
        *   `warp` (VPN định tuyến toàn bộ lưu lượng)
        *   `warp+doh` (Kết hợp bảo mật tối đa)
*   **[net_utils.rs](file:///home/exblackhole/Desktop/NetWarp-Manager/src/net_utils.rs)**:
    *   **Đo tốc độ băng thông thực tế**: Đọc trực tiếp `/proc/net/dev` của Linux theo chu kỳ 1 giây, loại bỏ loopback interface `lo`, tính toán lưu lượng bytes truyền/nhận (rx_bytes/tx_bytes) và chia cho thời gian trôi qua để lấy tốc độ Upload/Download tức thời chính xác.
    *   **Chẩn đoán Ping song song**: Gửi gói tin ping đồng thời (chỉ 1 gói tin, timeout 1s để tối ưu hiệu năng) tới **Google DNS (8.8.8.8)** và **Cloudflare DNS (1.1.1.1)** nhằm giám sát chất lượng đường truyền liên tục.
    *   **IP Geolocation**: Gửi yêu cầu đến API địa phương hóa IP công cộng `http://ip-api.com/json/` thông qua `curl` với các tùy chọn retry thông minh (`--retry 3 --retry-delay 1`) để tránh lỗi tạm thời khi ngắt/kết nối lại đường truyền VPN WARP.

---

## 3. Thành Phần Giao Diện (Slint UI - Declarative Style)

Giao diện đồ họa được thiết kế tập trung trong **[app.slint](file:///home/exblackhole/Desktop/NetWarp-Manager/src/app.slint)** theo phong cách **Cyberpunk Glassmorphism** hiện đại. Khác với các mô hình HTML/CSS thông thường, Slint sử dụng các component khai báo mạnh mẽ:

*   **Header Component**: Hiển thị logo Vector SVG của WiWarp, trạng thái LED xung nhịp hệ thống, và thông tin tóm tắt kết nối hiện tại.
*   **Diagnostics Panel (Ping & IP Info)**: Bảng thông tin IP công cộng, nhà cung cấp ISP, vị trí địa lý, tọa độ và độ trễ Ping thời gian thực của Cloudflare/Google DNS.
*   **Network Speed Monitor (Speedometer)**: Đồng hồ hiển thị tốc độ Download/Upload tức thời, tốc độ đỉnh (peak), tổng lưu lượng phiên sử dụng và đồ thị chuyển động liên tục biểu thị lịch sử tốc độ.
*   **Wi-Fi Access Control Panel**: Danh sách hiển thị mạng Wi-Fi đang kết nối với các thông số chi tiết (MAC, IP, Gateway, DNS, chuẩn bảo mật, băng tần). Có nút chuyển đổi mạng để hiển thị Modal danh sách mạng Wi-Fi xung quanh kèm độ mạnh tín hiệu, băng tần và loại bảo mật.
*   **Cloudflare WARP Control Panel**: Cung cấp công tắc Toggle Switch hiện đại để Bật/Tắt kết nối WARP, bộ lựa chọn tab 3 chế độ hoạt động (DNS over DoH, WARP, WARP + DoH), và nút khởi chạy trình cài đặt tự động.
*   **System Console Log Terminal**: Bảng console nhỏ tích hợp trực tiếp trên UI hiển thị các log hệ thống thời gian thực với tiền tố thời gian chuẩn xác, duy trì tối đa 100 dòng log để tiết kiệm bộ nhớ.
*   **Modals & Dialogs**: Các hộp thoại nhập mật khẩu Wi-Fi (cho phép tùy chọn Khóa BSSID) và hộp thoại danh sách Wi-Fi xuất hiện với hoạt ảnh mượt mà.
*   **Toast Notifications**: Khung thông báo nhỏ gọn xuất hiện ở góc màn hình để thông báo nhanh trạng thái thành công hoặc phát sinh lỗi của hệ thống.

---

## 4. Các Cơ Chế Kỹ Thuật Đặc Thù

### 4.1 Cơ Chế Luồng An Toàn (Thread-Safety) & Cập Nhật Bất Đồng Bộ
Slint UI chạy trên một luồng giao diện chính duy nhất (UI Thread). Mọi thao tác chặn (blocking) như gọi lệnh CLI hoặc chờ phản hồi từ mạng sẽ khiến giao diện bị đóng băng. Để đạt hiệu năng 60 FPS cực kỳ mượt mà:
1.  **Tokio Background Workers**: Mọi câu lệnh CLI hoặc truy vấn mạng đều được Rust đẩy xuống các luồng phụ bất đồng bộ thông qua `tokio::spawn`.
2.  **Upgrade in Event Loop**: Khi luồng phụ nhận được kết quả, nó không thể trực tiếp thay đổi thuộc tính của UI do ràng buộc an toàn luồng. Rust sử dụng con trỏ yếu `Weak<AppWindow>` và gọi phương thức `.upgrade_in_event_loop()` để đẩy một closure cập nhật dữ liệu trở lại UI Thread một cách an toàn và đồng bộ.

### 4.2 Chu Kỳ Polling Đa Tần Số (Multi-Interval Polling Engine)
Ứng dụng sử dụng 4 vòng lặp Polling chạy song song bằng Tokio để đảm bảo tính thời gian thực cao nhất mà vẫn tiết kiệm CPU:
*   **Vòng lặp LED & Radar (500ms)**: Thay đổi trạng thái nhấp nháy đèn LED hệ thống và cập nhật bước quét radar tạo hiệu ứng động.
*   **Vòng lặp Tốc Độ Mạng (1000ms)**: Đọc `/proc/net/dev`, tính toán tốc độ tức thời và đẩy giá trị mới vào đồ thị cuộn (rolling history) hiển thị giao diện.
*   **Vòng lặp Trạng Thái Hệ Thống (1000ms)**: Đồng bộ hóa thông tin Wi-Fi hiện tại (`nmcli`) và daemon WARP (`warp-cli status`, `warp-cli settings list`), đảm bảo UI luôn khớp với thực tế kể cả khi người dùng cấu hình bằng CLI bên ngoài.
*   **Vòng lặp Ping Chẩn Đoán (1000ms)**: Thực hiện các truy vấn ping nhanh song song tới `1.1.1.1` và `8.8.8.8` để đo lường độ trễ mạng liên tục.
*   **Đồng Bộ GeoIP Thông Minh (Smart Cooldown)**: Thông tin vị trí địa lý IP công cộng được kiểm tra định kỳ mỗi 30 giây để tránh làm quá tải máy chủ API. Tuy nhiên, bất kỳ khi nào phát hiện mạng Wi-Fi thay đổi hoặc trạng thái kết nối Cloudflare WARP thay đổi (Bật/Tắt/Đổi chế độ), vòng lặp sẽ **ngay lập tức** bỏ qua thời gian chờ cooldown để cập nhật IP và vị trí mới lên giao diện tức thì.

### 4.3 Thiết Kế Bảo Mật Tuyệt Đối
*   **Hardcoded Shell Commands**: Giao diện UI Slint hoàn toàn không có khả năng thực thi mã lệnh tùy ý. Mọi câu lệnh shell đều được định nghĩa cứng an toàn trong Rust Backend. UI chỉ gửi các tham số chuỗi thuần túy (như SSID, BSSID, Mật khẩu). Rust thực hiện kiểm tra cấu trúc chuỗi trước khi chuyển thành tham số dòng lệnh, triệt tiêu hoàn toàn rủi ro **Shell Injection**.
*   **Minh bạch trong Phân quyền**: Quá trình can thiệp cài đặt hệ thống không chạy ngầm một cách mờ ám dưới quyền root. Thay vào đó, việc mở ra một cửa sổ terminal chuẩn của hệ thống giúp người dùng kiểm soát hoàn toàn việc nhập mật khẩu xác thực `sudo` và theo dõi trực quan tiến trình cài đặt của script.

---

## 5. Quy Chuẩn Phát Triển Code (Coding Standards)

Để duy trì độ ổn định cao và dễ dàng mở rộng dự án:
*   **Strict Rust Lints**: Tuân thủ nghiêm ngặt các quy định cảnh báo và cấm lỗi trong `Cargo.toml`. Tuyệt đối cấm sử dụng mã không an toàn (`unsafe_code = "deny"`), ngăn chặn crash ứng dụng bằng cách cấm sử dụng unwrap/expect/panic trong mã nguồn sản xuất (`unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`). Mọi lỗi tiềm ẩn phải được chuyển đổi thành `Result<T, String>` để xử lý và hiển thị thông báo an toàn ra UI.
*   **Code Formatting & Cleanliness**: Bắt buộc chạy `cargo fmt` để định dạng mã nguồn nhất quán và `cargo clippy` để đảm bảo code tuân thủ đầy đủ chuẩn Rust idiomatic.
*   **Ngôn ngữ Nhất quán**: Mọi chú thích kỹ thuật trực tiếp bên trong mã nguồn (`.rs`, `.slint`) sử dụng **Tiếng Anh** để tối ưu hóa khả năng tương thích công cụ phân tích tĩnh, trong khi toàn bộ tài liệu hướng dẫn vận hành, hướng dẫn kiến trúc và giao tiếp sử dụng **Tiếng Việt**.
