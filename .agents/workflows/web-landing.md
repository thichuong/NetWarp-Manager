---
description: Hướng dẫn quy trình cập nhật, sửa đổi trang giới thiệu sản phẩm (Web Landing Page) trong public/index.html của dự án NetWarp-Manager (WiWarp).
---

# Quy trình Cập nhật & Phát triển Web Landing Page

Tài liệu này hướng dẫn cách tiếp cận, chỉnh sửa và kiểm tra các thay đổi trên trang Landing Page (`public/index.html`) của dự án. Vì trang web này hoạt động độc lập với phần mềm Rust Desktop Client, quy trình làm việc cần được tuân thủ riêng biệt.

---

## 🛠️ BƯỚC 1: Xác định Yêu cầu & Thiết kế
1. **Định hướng thiết kế**:
   - Trang web sử dụng thiết kế tối hiện đại (Premium Dark Theme), kết hợp hiệu ứng kính (Glassmorphism), dải màu chuyển sắc (Gradients) và viền phát sáng (Glow border).
   - Mọi thay đổi về mặt giao diện phải đảm bảo tính đồng nhất thẩm mỹ với các token CSS hiện tại (`--bg-dark`, `--accent-cyan`, `--accent-blue`, etc.).
2. **Định hướng SEO**:
   - Giữ các thẻ tiêu đề `<title>` và mô tả `<meta name="description">` rõ ràng, chứa từ khóa chính (`WiWarp`, `Fedora Linux`, `WARP`, `Slint`).

---

## 📐 BƯỚC 2: Thực hiện chỉnh sửa mã nguồn HTML/CSS
Agent cần áp dụng các quy tắc kỹ thuật sau:
1. **Sửa các khối lệnh (Terminal Box)**:
   - Khi chỉnh sửa các bước cài đặt ở `Quick Start`, bắt buộc tuân thủ quy tắc chia dòng bằng thẻ `<div class="terminal-line">` để tránh lỗi khoảng trắng thụt đầu dòng do `white-space: pre`.
   - Cú pháp chuẩn cho một dòng lệnh:
     ```html
     <div class="terminal-line"><span class="prompt">$</span> <span class="cmd">câu_lệnh_ở_đây</span></div>
     ```
2. **Kiểm tra chức năng Copy**:
   - Chắc chắn rằng nút "Copy" liên kết đúng ID của khối terminal (ví dụ: `onclick="copyCommand('cmd-stepX', 'btn-copy-stepX')"`).
   - Đảm bảo thẻ `div` chứa code có đúng `id` tương ứng để JS tìm thấy.

---

## 📈 BƯỚC 3: Quy trình Kiểm tra & Xác minh
Khi thay đổi trang Landing Page, Agent cần chạy kiểm tra các phần sau:

1. **Kiểm tra hiển thị (HTML/CSS)**:
   - Các khối hộp terminal có bị lệch lề trái/phải hay không.
   - Layout trên thiết bị di động (Responsive) có bị tràn màn hình hoặc chồng chéo chữ hay không.
2. **Kiểm tra Javascript (Copy Clipboard)**:
   - Nhấp vào nút "Copy" trên từng khối lệnh để đảm bảo:
     - Hiển thị Toast thông báo "Command copied to clipboard!".
     - Biểu tượng nút chuyển sang trạng thái "Copied" (dấu tích xanh) thành công.
     - Kiểm tra dữ liệu thực tế được copy vào bộ nhớ đệm: Chỉ chứa các câu lệnh thực thi (nằm trong `span.cmd`), loại bỏ hoàn toàn các ký tự dấu nhắc lệnh `$` và chú thích `#`.
