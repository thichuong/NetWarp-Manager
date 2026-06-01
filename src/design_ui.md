# 🎨 Quy chuẩn Thiết kế UI WiWarp (src/design_ui.md)

Tài liệu thiết kế giao diện tinh gọn của ứng dụng WiWarp, tập trung tối đa vào sơ đồ bố cục 1600x900 và các quy chuẩn trạng thái trực quan.

---

## 1. Bố Cục Standard Layout 1600x900

Giao diện ứng dụng được khóa cố định ở độ phân giải **1600x900** pixel, sử dụng cấu hình lưới 12 cột (`grid-cols-12`) và ẩn cuộn thanh hệ thống (`overflow-hidden`) để tối ưu hóa trải nghiệm Native Desktop App.

```text
+----------------------------------------------------------------------------------+
|                              HEADER (Logo, Title, Exit)                          |
+------------------------------------------------------+---------------------------+
|                                                      |                           |
|  COLUMN 1: SYSTEM & SPEED MONITOR (8/12)             |  COLUMN 2: SERVICES (4/12)|
|  +------------------------------------------------+  |  +---------------------+  |
|  | [1] Real-time Speed Monitoring Grid            |  |  | [4] Cloudflare WARP |  |
|  |     (Upload/Download Speeds & Graph)           |  |  |     Toggle Control  |  |
|  +------------------------------------------------+  |  +---------------------+  |
|  | [2] IP Geolocation Details                     |  |  | [5] WARP Tunnel Mode|  |
|  |     (IP, City, Country, Org info)              |  |  |     Selection (DoH) |  |
|  +------------------------------------------------+  |  +---------------------+  |
|  | [3] Local Wi-Fi Quick View & Diagnostics       |  |  | [6] System Console  |  |
|  |     (Ping Google/Cloudflare Latency)           |  |  |     Mini Logs       |  |
|  +------------------------------------------------+  |  +---------------------+  |
|                                                      |                           |
+------------------------------------------------------+---------------------------+
|                              FOOTER (App Version, Credits)                       |
+----------------------------------------------------------------------------------+
```

### Chi tiết Phân phối Lưới (12 Cột)
*   **Header & Footer (Chiều ngang 12/12)**: Chứa logo SVG, tên ứng dụng và nhãn thông tin bản quyền tối giản.
*   **Cột Bên Trái (8/12 Cột - Chẩn đoán & Tốc độ Mạng)**:
    1.  *Speedometer Grid*: Đo lượng Upload/Download thời gian thực (Mono font chữ to) kèm biểu đồ chuyển động.
    2.  *IP Geolocation*: Chi tiết thông tin IP công cộng, ISP và vị trí địa lý.
    3.  *Wi-Fi Widget & Ping*: Nút bấm nhanh mở danh sách mạng Wi-Fi và bảng đo độ trễ Ping tới Google/Cloudflare (5s polling).
*   **Cột Bên Phải (4/12 Cột - Quản lý Dịch vụ)**:
    4.  *Cloudflare WARP Toggle*: Công tắc gạt iOS-style bật/tắt chính, bo quanh bằng vòng LED trạng thái phát sáng.
    5.  *Tunnel Mode Switcher*: Chuyển đổi linh hoạt chế độ hoạt động (DNS over DoH / WARP Tunnel / WARP+DoH).
    6.  *Mini Console Logs*: Hiển thị nhật ký log hệ thống thời gian thực với thanh cuộn siêu mỏng.

---

## 2. Hệ Màu & Đèn LED Trạng Thái (Visual States)

Sử dụng phong cách **Cyberpunk Glassmorphism** (nền trong suốt `bg-slate-900/40`, viền `border-slate-800`, hiệu ứng blur `backdrop-blur-md` trên nền tối sâu `slate-950`) kết hợp các đèn LED chỉ thị trạng thái chức năng:
*   🟢 **Green (`#10b981`)**: Đã kết nối thành công (Connected).
*   🟡 **Orange (`#f59e0b`)**: Đang kết nối hoặc đang xử lý tác vụ (Connecting / Pulse animation).
*   🔴 **Red (`#f43f5e`)**: Lỗi hệ thống hoặc đã ngắt kết nối nghiêm trọng (Error).
*   ⚫ **Gray (`#64748b`)**: Trạng thái ngắt kết nối bình thường hoặc chưa cài đặt dịch vụ.

---

## 3. Quy chuẩn Phông Chữ (Typography)
*   **Outfit Font (Sans-serif)**: Dùng cho toàn bộ nhãn hiển thị, thẻ tiêu đề và modal (Hiện đại, công nghệ tương lai).
*   **JetBrains Mono (Monospace)**: Dùng riêng cho các ký tự số (Tốc độ Upload/Download, IP Address, thông số Ping `ms`) và Console Logs giúp hiển thị thẳng hàng hoàn hảo, dễ so sánh số liệu.
