# 🏛️ Tài liệu Kiến trúc WiWarp (NetWarp-Manager)

Tài liệu này mô tả chi tiết kiến trúc kỹ thuật của **WiWarp**, giải thích cách ứng dụng phân tách các thành phần (Modular Architecture) ở cả Frontend lẫn Backend, cơ chế giao tiếp IPC bảo mật thông qua Tauri v2, và cách quản lý luồng dữ liệu thời gian thực.

---

## 1. Tổng Quan Kiến Trúc

WiWarp được thiết kế theo mô hình **Hybrid Desktop Application** sử dụng **Tauri v2**:
*   **Frontend (UI Layer)**: Chạy trên Webview gọn nhẹ sử dụng Vanilla HTML/JS/CSS kết hợp Tailwind CSS. Giao diện được thiết kế dạng modular với các thành phần HTML được tải động.
*   **Backend (Core Layer)**: Viết bằng ngôn ngữ **Rust**, chịu trách nhiệm gọi các lệnh hệ thống Linux (`nmcli`, `systemctl`, `warp-cli`, `rpm`, `pkexec`), đảm bảo an toàn bộ nhớ và hiệu năng tối đa.
*   **IPC Bridge**: Kênh truyền thông điệp bất đồng bộ (Asynchronous Command Dispatcher) kết nối Frontend và Backend thông qua giao thức `tauri::command` (Invoke).

```mermaid
graph TD
    subgraph Frontend (WebView)
        A[index.html] -->|Load| B[loader.js]
        B -->|Inject Components| C[HTML Templates]
        C --> D[main.js / Entry Point]
        D -->|Event Listeners| E[JS Modules: wifi, warp, diagnostics, ui, state]
    end

    subgraph Tauri IPC Bridge
        E -->|invoke_handler| F[Tauri IPC Bridge]
    end

    subgraph Backend (Rust Core)
        F -->|Route Commands| G[lib.rs]
        G --> H[wifi.rs Module]
        G --> I[warp.rs Module]
        G --> J[net_utils.rs Module]
    end

    subgraph Linux System Services
        H -->|Execute non-blocking| K[nmcli CLI]
        I -->|Execute via Polkit/Command| L[warp-cli / pkexec rpm / systemctl]
        J -->|Network Diagnostic commands| M[ping / tracepath / proc_net_dev]
    end
```

---

## 2. Phân Rã Thành Phần Backend (Rust - Modular)

Mã nguồn Rust phía backend nằm trong thư mục `src-tauri/src` và được cấu trúc chặt chẽ thành các module chuyên biệt:

*   **[main.rs](file:///home/exblackhole/Desktop/NetWarp-Manager/src-tauri/src/main.rs)**: Điểm khởi chạy tối giản. Chỉ đóng vai trò gọi hàm `run()` từ thư viện `lib`.
*   **[lib.rs](file:///home/exblackhole/Desktop/NetWarp-Manager/src-tauri/src/lib.rs)**: Đóng vai trò là "Tổng đài điều phối" (Orchestrator). Đăng ký các plugins, khởi tạo cấu hình Tauri và thiết lập cổng API giao tiếp `tauri::generate_handler!`.
*   **[wifi.rs](file:///home/exblackhole/Desktop/NetWarp-Manager/src-tauri/src/wifi.rs)**:
    *   Thực hiện quét các mạng Wi-Fi lân cận qua `nmcli device wifi list`.
    *   Điều khiển kết nối/ngắt kết nối Wi-Fi thông qua CLI `nmcli`.
    *   Truy xuất danh sách mạng đã lưu và mật khẩu (yêu cầu bảo mật an toàn).
*   **[warp.rs](file:///home/exblackhole/Desktop/NetWarp-Manager/src-tauri/src/warp.rs)**:
    *   Tích hợp cài đặt RPM tự động: Tải và cài đặt thông qua Polkit `pkexec` (yêu cầu quyền quản trị bằng giao diện đồ họa an toàn).
    *   Bật/tắt kết nối Cloudflare WARP thông qua `warp-cli connect/disconnect`.
    *   Cấu hình các chế độ hoạt động (DNS over DoH, WARP, WARP + DoH).
    *   Quản lý dịch vụ hệ thống `warp-svc` qua `systemctl`.
*   **[net_utils.rs](file:///home/exblackhole/Desktop/NetWarp-Manager/src-tauri/src/net_utils.rs)**:
    *   Đo lường IO mạng thời gian thực: Đọc file hệ thống `/proc/net/dev` để tính toán chính xác tốc độ Upload/Download mà không cần cài đặt thêm thư viện bên ngoài.
    *   Thực hiện Ping đa mục tiêu (Google DNS, Cloudflare DNS) và IP Tracing (định vị địa lý IP).

---

## 3. Phân Rã Thành Phần Frontend (HTML/JS - Modular)

Giao diện được tách biệt hoàn toàn khỏi logic điều khiển, cho phép dễ dàng bảo trì và tối ưu giao diện:

### 3.1 Cấu trúc HTML & Components (`src/components/`)
Thay vì viết một file HTML khổng lồ, ứng dụng chia nhỏ thành các mảnh giao diện:
*   `header.html`: Thanh công cụ, tiêu đề, và hiển thị logo WiWarp.
*   `speed_wifi_section.html`: Khung giám sát tốc độ mạng, danh sách mạng Wi-Fi và bảng Diagnostics (Ping & IP Location).
*   `warp_control_section.html`: Công tắc Toggle, bộ lựa chọn chế độ WARP và bảng điều khiển Console logs hệ thống.
*   `wifi_modal.html` & `password_modal.html`: Các hộp thoại nhập mật khẩu và danh sách chi tiết Wi-Fi ẩn/hiện mượt mà.
*   `toast.html`: Thông báo góc màn hình nhanh chóng (Toaster).
*   `footer.html`: Tín chỉ bản quyền tối giản.

### 3.2 Cấu trúc Logic JS (`src/js/`)
*   **[loader.js](file:///home/exblackhole/Desktop/NetWarp-Manager/src/js/loader.js)**: Chịu trách nhiệm thực hiện Fetch bất đồng bộ tất cả các tệp component HTML và tiêm (inject) vào đúng vị trí tương ứng trong `index.html` trước khi ứng dụng bắt đầu khởi tạo.
*   **[dom.js](file:///home/exblackhole/Desktop/NetWarp-Manager/src/js/dom.js)**: Lưu trữ và ánh xạ tập trung toàn bộ các selector DOM (`document.getElementById`, vv.) giúp tránh trùng lặp code và dễ dàng cập nhật khi thay đổi ID HTML.
*   **[state.js](file:///home/exblackhole/Desktop/NetWarp-Manager/src/js/state.js)**: Quản lý trạng thái chia sẻ (Global Shared State) như: Trạng thái WARP hiện tại, danh sách Wi-Fi đã quét, các trạng thái loading của tiến trình.
*   **[ui.js](file:///home/exblackhole/Desktop/NetWarp-Manager/src/js/ui.js)**: Thực hiện các hiệu ứng hiển thị, ẩn/hiện Modal, vẽ danh sách Wi-Fi động lên UI và kích hoạt Toast thông báo.
*   **[wifi.js](file:///home/exblackhole/Desktop/NetWarp-Manager/src/js/wifi.js)**: Giao tiếp với Rust Backend để gọi quét Wi-Fi, yêu cầu kết nối mạng mới và xử lý sự kiện kết nối thành công/thất bại.
*   **[warp.js](file:///home/exblackhole/Desktop/NetWarp-Manager/src/js/warp.js)**: Điều phối hoạt động chuyển đổi chế độ Cloudflare WARP, cập nhật giao diện switch và xử lý luồng cài đặt tự động.
*   **[diagnostics.js](file:///home/exblackhole/Desktop/NetWarp-Manager/src/js/diagnostics.js)**: Quản lý vòng lặp interval đo lường tốc độ Upload/Download (1s/lần) và Ping chẩn đoán mạng (5s/lần).

---

## 4. Các Cơ Chế Kỹ Thuật Đặc Thù

### 4.1 Cơ Chế Non-Blocking và Polling Chu kỳ
Để đảm bảo giao diện luôn đạt hiệu năng 60 FPS mượt mà:
1.  **Non-Blocking Commands**: Phía backend Rust thực thi các câu lệnh shell bằng cách khởi tạo tiến trình con (`std::process::Command`) một cách bất đồng bộ hoặc thông qua các thread riêng biệt, không làm nghẽn Main Thread của Tauri.
2.  **Double-Interval Polling**:
    *   **Mạng IO (1 giây)**: Một vòng lặp 1000ms gửi yêu cầu tới Rust để đọc `/proc/net/dev` và cập nhật tức thì biểu đồ/đồng hồ đo tốc độ.
    *   **Trạng thái WARP (5 giây)**: Một vòng lặp 5000ms gửi tín hiệu chẩn đoán tới daemon `warp-svc` để cập nhật trạng thái kết nối lên UI, đảm bảo đồng bộ hóa ngay cả khi người dùng thay đổi trạng thái WARP từ terminal bên ngoài.

### 4.2 Thiết Kế Bảo Mật Tối Đa
*   **Không chạy Shell trực tiếp từ Frontend**: Mọi lệnh Command Line đều được cấu hình cứng trong code Rust Backend. Frontend chỉ gửi các tham số an toàn (như tên Wi-Fi, mật khẩu) qua IPC Command. Điều này triệt tiêu hoàn toàn nguy cơ chấn thương bảo mật Shell Injection.
*   **Phân Quyền Minh Bạch Polkit**: Khi cần cài đặt gói Cloudflare WARP `.rpm`, thay vì chạy ngầm dưới quyền root nguy hiểm, ứng dụng gọi lệnh thông qua `pkexec`. Hệ thống Fedora sẽ hiển thị hộp thoại xác thực mật khẩu chuẩn của hệ điều hành, đảm bảo tính minh bạch và bảo mật tuyệt đối cho người dùng cuối.

---

## 5. Quy Chuẩn Phát Triển Code (Coding Standards)

Để dự án duy trì được độ ổn định và tính mở rộng cao, các quy tắc sau được áp dụng bắt buộc:
*   **Rust Idioms**:
    *   Tận dụng kiểu dữ liệu `Result<T, E>` để bắt và trả lỗi rõ ràng về Frontend dưới dạng String.
    *   Định dạng code bằng `cargo fmt` trước mỗi lần commit.
    *   Đảm bảo không có cảnh báo nghiêm trọng từ `cargo clippy`.
*   **Phân rã tệp tin**: Bất kỳ tính năng mới nào (ví dụ: VPN bên thứ ba khác) đều phải được tách thành module độc lập `.rs` ở backend và tệp `.js` điều khiển tương ứng ở frontend.
*   **Chú thích tiếng Anh**: Mọi dòng chú thích kỹ thuật trực tiếp trong file code (`.js`, `.rs`) đều sử dụng **Tiếng Anh** chuẩn để tối ưu hóa sự tương thích công cụ phân tích tĩnh, trong khi tài liệu hướng dẫn vận hành sử dụng **Tiếng Việt**.
