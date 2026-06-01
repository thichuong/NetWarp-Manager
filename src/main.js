// WiWarp Frontend - JavaScript logic
const { invoke } = window.__TAURI__.core;

// Tham chiếu các thành phần DOM
let btnScan, iconScan, wifiContainer, wifiCount, activeWifiNet;
let btnInstall, btnConnect, btnCancel, wifiForm, wifiPassword, passwordModal, modalSsid;
let ledDot, ledPing, warpStatusText, warpNetworkText, warpToggle, warpLogs, toast, toastMessage, toastIcon;
let warpModeBadgeContainer, warpModeBadge;
let btnModeDoh, btnModeWarpDoh;
let currentWarpMode = "";
let isSettingMode = false;

// Lưu trữ SSID đang được chọn để kết nối
let selectedSsid = "";
let isScanning = false;
let isTogglingWarp = false;

// Khởi chạy khi DOM đã load hoàn chỉnh
window.addEventListener("DOMContentLoaded", () => {
  initDOMElements();
  registerEvents();
  
  // Quét Wi-Fi lần đầu
  scanWifi();
  
  // Bắt đầu chu kỳ thăm dò (polling) trạng thái Cloudflare WARP định kỳ mỗi 3 giây
  pollWarpStatus();
});

// Khởi tạo các tham chiếu DOM
function initDOMElements() {
  btnScan = document.getElementById("btn-scan");
  iconScan = document.getElementById("icon-scan");
  wifiContainer = document.getElementById("wifi-container");
  wifiCount = document.getElementById("wifi-count");
  activeWifiNet = document.getElementById("active-wifi-net");

  btnInstall = document.getElementById("btn-install");
  btnConnect = document.getElementById("btn-connect");
  btnCancel = document.getElementById("btn-cancel");
  wifiForm = document.getElementById("wifi-form");
  wifiPassword = document.getElementById("wifi-password");
  passwordModal = document.getElementById("password-modal");
  modalSsid = document.getElementById("modal-ssid");

  ledDot = document.getElementById("led-dot");
  ledPing = document.getElementById("led-ping");
  warpStatusText = document.getElementById("warp-status-text");
  warpNetworkText = document.getElementById("warp-network-text");
  warpToggle = document.getElementById("warp-toggle");
  warpLogs = document.getElementById("warp-logs");

  toast = document.getElementById("toast");
  toastMessage = document.getElementById("toast-message");
  toastIcon = document.getElementById("toast-icon");

  warpModeBadgeContainer = document.getElementById("warp-mode-badge-container");
  warpModeBadge = document.getElementById("warp-mode-badge");

  btnModeDoh = document.getElementById("mode-doh");
  btnModeWarpDoh = document.getElementById("mode-warpdoh");
}

// Đăng ký các sự kiện tương tác
function registerEvents() {
  // Sự kiện quét Wi-Fi
  btnScan.addEventListener("click", scanWifi);

  // Mở modal kết nối khi click mạng trong danh sách được quản lý động
  btnCancel.addEventListener("click", closeModal);

  // Form submit mật khẩu Wi-Fi
  wifiForm.addEventListener("submit", (e) => {
    e.preventDefault();
    connectWifi();
  });

  // Nút cài đặt WARP
  btnInstall.addEventListener("click", installWarp);

  // Switch Toggle bật/tắt WARP
  warpToggle.addEventListener("change", handleWarpToggle);

  // Sự kiện chuyển đổi chế độ hoạt động WARP
  btnModeDoh.addEventListener("click", () => handleModeChange("doh"));
  btnModeWarpDoh.addEventListener("click", () => handleModeChange("warp+doh"));
}

// Hàm ghi log vào bảng console nhỏ trên UI
function logMessage(message) {
  const time = new Date().toLocaleTimeString();
  const logLine = `[${time}] ${message}\n`;
  warpLogs.textContent = logLine + warpLogs.textContent;
}

// Hiển thị Toast thông báo hiện đại
function showToast(message, isError = false) {
  toastMessage.textContent = message;
  
  if (isError) {
    toast.classList.remove("border-teal-700/50");
    toast.classList.add("border-red-700/50");
    toastIcon.innerHTML = `
      <svg class="w-4 h-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
      </svg>
    `;
  } else {
    toast.classList.remove("border-red-700/50");
    toast.classList.add("border-teal-700/50");
    toastIcon.innerHTML = `
      <svg class="w-4 h-4 text-teal-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"/>
      </svg>
    `;
  }

  // Slide up và mờ
  toast.classList.remove("translate-y-10", "opacity-0", "pointer-events-none");
  toast.classList.add("translate-y-0", "opacity-100");

  // Tự ẩn sau 4 giây
  setTimeout(() => {
    toast.classList.add("translate-y-10", "opacity-0", "pointer-events-none");
    toast.classList.remove("translate-y-0", "opacity-100");
  }, 4000);
}

// 1. QUẢN LÝ WI-FI: QUÉT DANH SÁCH MẠNG
async function scanWifi() {
  if (isScanning) return;
  isScanning = true;
  
  // Hiệu ứng quay icon scan
  iconScan.classList.add("anim-scan");
  btnScan.disabled = true;
  
  logMessage("Đang quét các mạng Wi-Fi...");
  
  try {
    const list = await invoke("get_wifi_list");
    wifiCount.textContent = list.length;
    renderWifiList(list);
    logMessage(`Đã quét xong. Tìm thấy ${list.length} mạng khả dụng.`);
  } catch (err) {
    showToast(`Quét Wi-Fi thất bại: ${err}`, true);
    logMessage(`Lỗi quét Wi-Fi: ${err}`);
    wifiContainer.innerHTML = `
      <div class="py-12 flex flex-col items-center justify-center space-y-2 text-red-400">
        <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
        </svg>
        <p class="text-sm">Không thể quét Wi-Fi. Hãy đảm bảo card mạng đã bật.</p>
      </div>
    `;
  } finally {
    iconScan.classList.remove("anim-scan");
    btnScan.disabled = false;
    isScanning = false;
  }
}

// Hàm render danh sách Wi-Fi với CSS Tailwind cao cấp
function renderWifiList(networks) {
  if (networks.length === 0) {
    wifiContainer.innerHTML = `
      <div class="py-12 flex flex-col items-center justify-center text-slate-500">
        <p class="text-sm">Không tìm thấy mạng Wi-Fi nào.</p>
      </div>
    `;
    activeWifiNet.textContent = "Chưa kết nối Wi-Fi";
    return;
  }

  // Khởi tạo trạng thái mặc định
  let hasActiveConnection = false;
  wifiContainer.innerHTML = "";

  networks.forEach((net) => {
    const item = document.createElement("div");
    
    // Tạo sóng Wi-Fi SVG động dựa trên signal
    const wifiSvg = getWifiSignalSvg(net.signal);
    
    if (net.active) {
      hasActiveConnection = true;
      activeWifiNet.innerHTML = `Đang kết nối: <strong class="text-teal-400">${net.ssid}</strong>`;
      
      // Thiết kế card active với viền xanh lục phát sáng
      item.className = "flex items-center justify-between p-3.5 bg-teal-950/20 hover:bg-teal-900/20 border border-teal-500/40 hover:border-teal-400/60 rounded-2xl cursor-pointer transition-all duration-200 group active:scale-[0.99] shadow-[0_0_15px_rgba(20,184,166,0.15)]";
      
      item.innerHTML = `
        <div class="flex items-center space-x-3.5">
          <div class="text-teal-400">
            ${wifiSvg}
          </div>
          <div>
            <h4 class="text-sm font-bold text-teal-300 truncate max-w-[200px]">${net.ssid}</h4>
            <span class="text-[10px] text-teal-500 font-semibold">Tín hiệu: ${net.signal}% (Tốt nhất)</span>
          </div>
        </div>
        <div class="flex items-center space-x-2">
          <span class="text-[10px] bg-teal-500/10 text-teal-300 px-2.5 py-1 rounded-lg border border-teal-500/20 font-bold flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-teal-400 animate-pulse"></span>
            Đã kết nối
          </span>
        </div>
      `;
      
      // Nhấp vào mạng đang kết nối thì thông báo
      item.addEventListener("click", () => {
        showToast(`Bạn đang ở trong kết nối mạng "${net.ssid}" rồi.`);
      });
    } else {
      // Thiết kế card mạng khả dụng bình thường
      item.className = "flex items-center justify-between p-3.5 bg-slate-950/40 hover:bg-slate-800/40 border border-slate-800/40 hover:border-slate-700/60 rounded-2xl cursor-pointer transition-all duration-200 group active:scale-[0.99]";
      
      item.innerHTML = `
        <div class="flex items-center space-x-3.5">
          <div class="text-slate-400 group-hover:text-teal-400 transition-colors">
            ${wifiSvg}
          </div>
          <div>
            <h4 class="text-sm font-semibold text-slate-200 group-hover:text-white transition-colors truncate max-w-[200px]">${net.ssid}</h4>
            <span class="text-[10px] text-slate-500 font-medium">Tín hiệu: ${net.signal}%</span>
          </div>
        </div>
        <div class="flex items-center space-x-2">
          <span class="text-[10px] bg-slate-800/80 group-hover:bg-teal-500/10 text-slate-400 group-hover:text-teal-300 px-2.5 py-1 rounded-lg border border-slate-700/40 group-hover:border-teal-500/20 font-semibold transition-all">Kết nối</span>
        </div>
      `;
      
      // Bắt sự kiện click mở modal nhập pass
      item.addEventListener("click", () => openPasswordModal(net.ssid));
    }
    
    wifiContainer.appendChild(item);
  });

  if (!hasActiveConnection) {
    activeWifiNet.textContent = "Chưa kết nối Wi-Fi";
  }
}

// Trả về SVG sóng Wi-Fi theo cường độ sóng (%)
function getWifiSignalSvg(signal) {
  // Sóng cực mạnh (>= 75%)
  if (signal >= 75) {
    return `<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M17.778 8.111a10.027 10.027 0 00-11.556 0 1 1 0 101.156 1.632 10.027 10.027 0 0011.556 0zm-2.31 2.31a6.762 6.762 0 00-6.936 0 1 1 0 10.693 1.488 6.762 6.762 0 006.936 0zM10 14a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" clip-rule="evenodd"/></svg>`;
  }
  // Sóng khá (50% - 74%)
  if (signal >= 50) {
    return `<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"><path d="M15.467 10.421a6.762 6.762 0 00-6.936 0 1 1 0 10.693 1.488 6.762 6.762 0 006.936 0zM10 14a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"/></svg>`;
  }
  // Sóng trung bình (25% - 49%)
  if (signal >= 25) {
    return `<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"><path d="M10 14a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"/></svg>`;
  }
  // Sóng yếu (< 25%)
  return `<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M12 20h.01"/></svg>`;
}

// Mở modal nhập mật khẩu Wi-Fi
function openPasswordModal(ssid) {
  selectedSsid = ssid;
  modalSsid.textContent = ssid;
  wifiPassword.value = "";
  
  // Trực quan hoá Modal mượt mà
  passwordModal.classList.remove("opacity-0", "pointer-events-none");
  passwordModal.classList.add("opacity-100");
  passwordModal.firstElementChild.classList.remove("scale-90");
  passwordModal.firstElementChild.classList.add("scale-100");
  wifiPassword.focus();
}

// Đóng modal
function closeModal() {
  passwordModal.classList.add("opacity-0", "pointer-events-none");
  passwordModal.classList.remove("opacity-100");
  passwordModal.firstElementChild.classList.add("scale-90");
  passwordModal.firstElementChild.classList.remove("scale-100");
}

// 2. KẾT NỐI VÀO WI-FI
async function connectWifi() {
  const password = wifiPassword.value;
  btnConnect.classList.add("btn-loading");
  btnConnect.disabled = true;
  btnCancel.disabled = true;

  logMessage(`Đang cố gắng kết nối tới Wi-Fi: "${selectedSsid}"...`);
  showToast(`Đang kết nối tới ${selectedSsid}...`);

  try {
    const res = await invoke("connect_wifi", { ssid: selectedSsid, password: password || null });
    showToast("Kết nối Wi-Fi thành công!");
    logMessage(`Kết nối thành công: ${res}`);
    activeWifiNet.innerHTML = `Đang kết nối: <strong class="text-teal-400">${selectedSsid}</strong>`;
    closeModal();
    // Refresh lại danh sách mạng
    setTimeout(scanWifi, 2000);
  } catch (err) {
    showToast(`Kết nối thất bại: ${err}`, true);
    logMessage(`Lỗi kết nối Wi-Fi: ${err}`);
  } finally {
    btnConnect.classList.remove("btn-loading");
    btnConnect.disabled = false;
    btnCancel.disabled = false;
  }
}

// 3. THĂM DÒ (POLLING) TRẠNG THÁI CLOUDFLARE WARP
async function pollWarpStatus() {
  await getWarpStatus();
  // Lặp lại mỗi 3 giây
  setInterval(getWarpStatus, 3000);
}

// Lấy trạng thái của WARP và đồng bộ hóa giao diện người dùng
async function getWarpStatus() {
  try {
    const status = await invoke("get_warp_status");
    updateWarpUI(status);
    
    // Nếu WARP đã được cài đặt, lấy thêm chế độ hoạt động hiện tại
    if (status !== "Not Installed" && !isSettingMode) {
      try {
        const mode = await invoke("get_warp_mode");
        currentWarpMode = mode;
        updateWarpModeUI(mode);
        disableModeButtons(false);
      } catch (err) {
        // Bỏ qua lỗi lấy mode âm thầm
      }
    } else if (status === "Not Installed") {
      disableModeButtons(true);
      if (warpModeBadgeContainer) warpModeBadgeContainer.classList.add("hidden");
    }
  } catch (err) {
    // Không log liên tục lỗi định kỳ để tránh làm ngập log panel
    updateWarpUI("Disconnected");
    disableModeButtons(true);
    if (warpModeBadgeContainer) warpModeBadgeContainer.classList.add("hidden");
  }
}

// Cập nhật giao diện tương ứng với trạng thái WARP
function updateWarpUI(status) {
  // Xử lý các trạng thái: Connected, Disconnected, Connecting, Not Installed
  if (status === "Connected") {
    ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-emerald-500 shadow-[0_0_12px_rgba(16,185,129,0.7)] transition-all duration-300";
    ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75";
    warpStatusText.textContent = "Đã Kết Nối";
    warpStatusText.className = "text-sm font-semibold mt-1 text-emerald-400 uppercase tracking-widest";
    warpNetworkText.textContent = "Dữ liệu của bạn hiện đang được bảo vệ và tối ưu hoá.";
    
    warpToggle.disabled = false;
    if (!isTogglingWarp) {
      warpToggle.checked = true;
    }
  } else if (status === "Connecting") {
    ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-amber-500 shadow-[0_0_12px_rgba(245,158,11,0.7)] transition-all duration-300";
    ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-400 opacity-75";
    warpStatusText.textContent = "Đang kết nối...";
    warpStatusText.className = "text-sm font-semibold mt-1 text-amber-400 uppercase tracking-widest";
    warpNetworkText.textContent = "Đang thiết lập kênh truyền an toàn...";
    
    warpToggle.disabled = true;
  } else if (status === "Not Installed") {
    ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-slate-500 shadow-none transition-all duration-300";
    ledPing.className = "hidden";
    warpStatusText.textContent = "Chưa Cài Đặt";
    warpStatusText.className = "text-sm font-semibold mt-1 text-slate-400 uppercase tracking-widest";
    warpNetworkText.textContent = "Không tìm thấy Cloudflare WARP trên Fedora.";
    
    warpToggle.disabled = true;
    warpToggle.checked = false;
    
    // Thêm hiệu ứng nhấp nháy thu hút sự chú ý vào nút Cài đặt nếu chưa cài
    btnInstall.classList.add("animate-pulse");
  } else {
    // Trạng thái Disconnected
    ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-red-500 shadow-[0_0_12px_rgba(239,68,68,0.7)] transition-all duration-300";
    ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75";
    warpStatusText.textContent = "Đã Ngắt Kết Nối";
    warpStatusText.className = "text-sm font-semibold mt-1 text-red-400 uppercase tracking-widest";
    warpNetworkText.textContent = "Hệ thống mạng an toàn và riêng tư.";
    
    warpToggle.disabled = false;
    if (!isTogglingWarp) {
      warpToggle.checked = false;
    }
    btnInstall.classList.remove("animate-pulse");
  }
}

// 4. BẬT / TẮT CLOUDFLARE WARP
async function handleWarpToggle() {
  if (isTogglingWarp) return;
  isTogglingWarp = true;
  
  const wantConnect = warpToggle.checked;
  warpToggle.disabled = true;
  
  logMessage(`Đang thực hiện ${wantConnect ? "BẬT" : "TẮT"} Cloudflare WARP...`);
  
  try {
    const res = await invoke("warp_toggle", { connect: wantConnect });
    showToast(wantConnect ? "Đang bật bảo vệ WARP!" : "Đã ngắt bảo vệ WARP.");
    logMessage(`WARP Command Output: ${res}`);
    // Đợi 1 giây rồi cập nhật trạng thái ngay
    setTimeout(getWarpStatus, 1000);
  } catch (err) {
    showToast(`Điều khiển WARP thất bại: ${err}`, true);
    logMessage(`Lỗi điều khiển WARP: ${err}`);
    // Rollback lại toggle switch trên UI
    warpToggle.checked = !wantConnect;
  } finally {
    isTogglingWarp = false;
    warpToggle.disabled = false;
  }
}

// 5. CÀI ĐẶT CLOUDFLARE WARP TRÊN FEDORA
async function installWarp() {
  btnInstall.classList.add("btn-loading");
  btnInstall.disabled = true;
  
  logMessage("Đang tiến hành tải và cài đặt Cloudflare WARP...");
  logMessage("Bước 1: Gọi lệnh 'dnf download cloudflare-warp'...");
  showToast("Bắt đầu cài đặt WARP...");

  try {
    const result = await invoke("install_warp");
    showToast("Cài đặt Cloudflare WARP thành công!");
    logMessage(`Cài đặt hoàn tất: ${result}`);
    // Cập nhật lại UI lập tức
    getWarpStatus();
  } catch (err) {
    showToast(`Cài đặt WARP thất bại: ${err}`, true);
    logMessage(`LỖI cài đặt WARP: ${err}`);
  } finally {
    btnInstall.classList.remove("btn-loading");
    btnInstall.disabled = false;
  }
}

// 6. THAY ĐỔI CHẾ ĐỘ HOẠT ĐỘNG CLOUDFLARE WARP
async function handleModeChange(mode) {
  if (isSettingMode) return;
  isSettingMode = true;

  // Cập nhật UI ngay lập tức (Optimistic Update) giúp nút sáng lên ngay lập tức khi click!
  currentWarpMode = mode;
  updateWarpModeUI(mode);
  disableModeButtons(true);

  logMessage(`Đang chuyển chế độ WARP sang: ${mode.toUpperCase()}...`);
  showToast(`Đang chuyển sang chế độ ${mode.toUpperCase()}...`);

  try {
    const res = await invoke("set_warp_mode", { mode });
    showToast(`Đã chuyển sang chế độ ${mode.toUpperCase()}!`);
    logMessage(`Kết quả chuyển chế độ: ${res}`);
  } catch (err) {
    showToast(`Lỗi chuyển chế độ: ${err}`, true);
    logMessage(`Lỗi chuyển chế độ: ${err}`);
    // Load lại chế độ thực tế để đồng bộ lại
    getWarpStatus();
  } finally {
    isSettingMode = false;
    disableModeButtons(false);
  }
}

function disableModeButtons(disabled) {
  if (btnModeDoh) btnModeDoh.disabled = disabled;
  if (btnModeWarpDoh) btnModeWarpDoh.disabled = disabled;
}

function updateWarpModeUI(mode) {
  const activeClass = "bg-gradient-to-r from-orange-500 to-amber-500 text-white border-transparent shadow-[0_0_15px_rgba(249,115,22,0.4)] scale-[1.02]";
  const inactiveClass = "text-slate-400 hover:text-slate-200 border-transparent hover:bg-slate-900/40";

  [
    { btn: btnModeDoh, key: "doh" },
    { btn: btnModeWarpDoh, key: "warp+doh" }
  ].forEach(({ btn, key }) => {
    if (!btn) return;
    if (key === mode) {
      btn.className = `py-2.5 px-1 rounded-xl text-[10px] font-bold tracking-wide transition-all duration-200 focus:outline-none flex flex-col items-center justify-center gap-1.5 border ${activeClass}`;
    } else {
      btn.className = `py-2.5 px-1 rounded-xl text-[10px] font-bold tracking-wide transition-all duration-200 focus:outline-none flex flex-col items-center justify-center gap-1.5 border ${inactiveClass}`;
    }
  });

  // Cập nhật nhãn/badge trạng thái hiển thị rõ ràng chế độ hiện tại
  if (warpModeBadge && warpModeBadgeContainer) {
    if (mode === "doh") {
      warpModeBadge.innerHTML = `
        <svg class="w-3 h-3 animate-pulse text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M13 10V3L4 14h7v7l9-11h-7z"/>
        </svg>
        <span>Chế độ: DoH (Chỉ DNS)</span>
      `;
      warpModeBadge.className = "inline-flex items-center gap-1.5 px-3.5 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider bg-orange-500/10 text-orange-400 border border-orange-500/20 shadow-[0_0_10px_rgba(249,115,22,0.1)]";
      warpModeBadgeContainer.classList.remove("hidden");
    } else if (mode === "warp+doh") {
      warpModeBadge.innerHTML = `
        <svg class="w-3 h-3 animate-pulse text-teal-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
        </svg>
        <span>Chế độ: WARP + DoH (Tối đa)</span>
      `;
      warpModeBadge.className = "inline-flex items-center gap-1.5 px-3.5 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider bg-teal-500/10 text-teal-400 border border-teal-500/20 shadow-[0_0_10px_rgba(20,184,166,0.1)]";
      warpModeBadgeContainer.classList.remove("hidden");
    } else {
      warpModeBadge.innerHTML = `
        <svg class="w-3 h-3 animate-pulse text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <span>Chế độ: ${mode.toUpperCase()}</span>
      `;
      warpModeBadge.className = "inline-flex items-center gap-1.5 px-3.5 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider bg-slate-500/10 text-slate-400 border border-slate-500/20 shadow-none";
      warpModeBadgeContainer.classList.remove("hidden");
    }
  }
}

