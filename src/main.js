// WiWarp Frontend - JavaScript logic
const { invoke } = window.__TAURI__.core;

// DOM Element References
let btnScan, iconScan, wifiContainer, wifiCount, activeWifiNet;
let btnInstall, btnConnect, btnCancel, wifiForm, wifiPassword, passwordModal, modalSsid;
let ledDot, ledPing, warpStatusText, warpNetworkText, warpToggle, warpLogs, toast, toastMessage, toastIcon;
let warpModeBadgeContainer, warpModeBadge;
let btnModeDoh, btnModeWarpDoh;

// Network Dashboard DOM Elements
let speedDownload, speedUpload, speedChart, speedCtx;
let pingCloudflare, pingGoogle;
let traceWarpBadge, traceIpAddress, traceIsp, traceLocation, traceCoords;

// Internal state storage
let currentWarpMode = "";
let isSettingMode = false;
let selectedBssid = "";
let selectedSsid = "";
let isScanning = false;
let isTogglingWarp = false;

// Network speed and sparkline monitoring state
let lastRxBytes = 0;
let lastTxBytes = 0;
let lastSpeedTime = Date.now();
let speedHistory = []; // holds { rx: number, tx: number }
const MAX_HISTORY_POINTS = 30; // sparkline points count

// Launch initialization when DOM is fully loaded
window.addEventListener("DOMContentLoaded", () => {
  initDOMElements();
  registerEvents();
  
  // Initial Wi-Fi scan
  scanWifi();
  
  // Fetch initial WARP status and operating mode
  getInitialStatus();

  // Start real-time network speed monitor (every 1s)
  startNetworkSpeedMonitor();

  // Load first-time network diagnostics
  updateNetworkDiagnostics();
  
  // Start polling Cloudflare WARP connection status every 5 seconds
  pollWarpStatus();
});

// Initialize DOM element references
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

  // Network Dashboard DOM elements
  speedDownload = document.getElementById("speed-download");
  speedUpload = document.getElementById("speed-upload");
  speedChart = document.getElementById("speed-chart");
  if (speedChart) {
    speedCtx = speedChart.getContext("2d");
  }

  pingCloudflare = document.getElementById("ping-cloudflare");
  pingGoogle = document.getElementById("ping-google");

  traceWarpBadge = document.getElementById("trace-warp-badge");
  traceIpAddress = document.getElementById("trace-ip-address");
  traceIsp = document.getElementById("trace-isp");
  traceLocation = document.getElementById("trace-location");
  traceCoords = document.getElementById("trace-coords");
}

// Register interactive element events
function registerEvents() {
  // Wi-Fi Scanning action
  btnScan.addEventListener("click", scanWifi);

  // Close connection password modal
  btnCancel.addEventListener("click", closeModal);

  // Submit password to connect to secured Wi-Fi
  wifiForm.addEventListener("submit", (e) => {
    e.preventDefault();
    connectWifi();
  });

  // Install WARP action
  btnInstall.addEventListener("click", installWarp);

  // Toggle switch to enable/disable Cloudflare WARP
  warpToggle.addEventListener("change", handleWarpToggle);

  // WARP operating mode switch actions
  btnModeDoh.addEventListener("click", () => handleModeChange("doh"));
  btnModeWarpDoh.addEventListener("click", () => handleModeChange("warp+doh"));
}

// Logs messages into the scrolling system logs terminal panel
function logMessage(message) {
  const time = new Date().toLocaleTimeString();
  const logLine = `[${time}] ${message}\n`;
  warpLogs.textContent = logLine + warpLogs.textContent;
}

// Displays modern Toast popup notification
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

  // Slide up and reveal toast
  toast.classList.remove("translate-y-10", "opacity-0", "pointer-events-none");
  toast.classList.add("translate-y-0", "opacity-100");

  // Automatically fade out after 4 seconds
  setTimeout(() => {
    toast.classList.add("translate-y-10", "opacity-0", "pointer-events-none");
    toast.classList.remove("translate-y-0", "opacity-100");
  }, 4000);
}

// 1. WI-FI MANAGER: SCAN FOR IN-RANGE NETWORKS
async function scanWifi() {
  if (isScanning) return;
  isScanning = true;
  
  // Trigger scan icon rotation animation
  iconScan.classList.add("anim-scan");
  btnScan.disabled = true;
  
  logMessage("Scanning for Wi-Fi networks...");
  
  try {
    const list = await invoke("get_wifi_list");
    wifiCount.textContent = list.length;
    renderWifiList(list);
    logMessage(`Scan finished. Found ${list.length} networks in range.`);
  } catch (err) {
    showToast(`Wi-Fi scan failed: ${err}`, true);
    logMessage(`Wi-Fi scan error: ${err}`);
    wifiContainer.innerHTML = `
      <div class="py-12 flex flex-col items-center justify-center space-y-2 text-red-400">
        <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
        </svg>
        <p class="text-sm">Could not scan Wi-Fi. Make sure your network card is enabled.</p>
      </div>
    `;
  } finally {
    iconScan.classList.remove("anim-scan");
    btnScan.disabled = false;
    isScanning = false;
  }
}

// Renders the list of Wi-Fi networks with high-fidelity styles
function renderWifiList(networks) {
  if (networks.length === 0) {
    wifiContainer.innerHTML = `
      <div class="py-12 flex flex-col items-center justify-center text-slate-500">
        <p class="text-sm">No Wi-Fi networks found.</p>
      </div>
    `;
    activeWifiNet.textContent = "No Wi-Fi Connection";
    return;
  }

  let hasActiveConnection = false;
  wifiContainer.innerHTML = "";

  networks.forEach((net) => {
    const item = document.createElement("div");
    
    // Dynamic Wi-Fi signal SVG based on signal strength percentage
    const wifiSvg = getWifiSignalSvg(net.signal);
    
    if (net.active) {
      hasActiveConnection = true;
      activeWifiNet.innerHTML = `Connected: <strong class="text-teal-400">${net.ssid} (${net.band})</strong>`;
      
      // Design card for the active/connected network with glowing green borders
      item.className = "flex items-center justify-between p-3.5 bg-teal-950/20 hover:bg-teal-900/20 border border-teal-500/40 hover:border-teal-400/60 rounded-2xl cursor-pointer transition-all duration-200 group active:scale-[0.99] shadow-[0_0_15px_rgba(20,184,166,0.15)]";
      
      item.innerHTML = `
        <div class="flex items-center space-x-3.5">
          <div class="text-teal-400">
            ${wifiSvg}
          </div>
          <div>
            <h4 class="text-sm font-bold text-teal-300 truncate max-w-[240px]">${net.ssid}</h4>
            <div class="text-[10px] text-teal-500 font-semibold space-y-0.5 mt-0.5">
              <div>MAC: <span class="font-mono text-teal-400/80">${net.bssid}</span> | Band: <span class="text-teal-400/80">${net.band} (${net.frequency})</span></div>
              <div>Signal: ${net.signal}% | Channel: ${net.channel} | Security: ${net.security || "Open"}</div>
            </div>
          </div>
        </div>
        <div class="flex items-center space-x-2 shrink-0">
          <span class="text-[10px] bg-teal-500/10 text-teal-300 px-2.5 py-1 rounded-lg border border-teal-500/20 font-bold flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-teal-400 animate-pulse"></span>
            Connected
          </span>
        </div>
      `;
      
      item.addEventListener("click", () => {
        showToast(`You are already connected to "${net.ssid}".`);
      });
    } else {
      // Standard card design for available networks
      item.className = "flex items-center justify-between p-3.5 bg-slate-950/40 hover:bg-slate-800/40 border border-slate-800/40 hover:border-slate-700/60 rounded-2xl cursor-pointer transition-all duration-200 group active:scale-[0.99]";
      
      item.innerHTML = `
        <div class="flex items-center space-x-3.5">
          <div class="text-slate-400 group-hover:text-teal-400 transition-colors">
            ${wifiSvg}
          </div>
          <div>
            <h4 class="text-sm font-semibold text-slate-200 group-hover:text-white transition-colors truncate max-w-[240px]">${net.ssid}</h4>
            <div class="text-[10px] text-slate-500 group-hover:text-slate-400 font-medium space-y-0.5 mt-0.5 transition-colors">
              <div>MAC: <span class="font-mono text-slate-400/80 group-hover:text-teal-400/60">${net.bssid}</span> | Band: <span class="text-slate-400/80 group-hover:text-teal-400/60">${net.band} (${net.frequency})</span></div>
              <div>Signal: ${net.signal}% | Channel: ${net.channel} | Security: ${net.security || "Open"}</div>
            </div>
          </div>
        </div>
        <div class="flex items-center space-x-2 shrink-0">
          <span class="text-[10px] bg-slate-800/80 group-hover:bg-teal-500/10 text-slate-400 group-hover:text-teal-300 px-2.5 py-1 rounded-lg border border-slate-700/40 group-hover:border-teal-500/20 font-semibold transition-all">Connect</span>
        </div>
      `;
      
      item.addEventListener("click", () => openPasswordModal(net.bssid, net.ssid, net.band, net.frequency));
    }
    
    wifiContainer.appendChild(item);
  });

  if (!hasActiveConnection) {
    activeWifiNet.textContent = "No Wi-Fi Connection";
  }
}

// Maps Wi-Fi signal strength (%) to appropriate SVG wave lines
function getWifiSignalSvg(signal) {
  // Strong signal (>= 75%)
  if (signal >= 75) {
    return `<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M17.778 8.111a10.027 10.027 0 00-11.556 0 1 1 0 101.156 1.632 10.027 10.027 0 0011.556 0zm-2.31 2.31a6.762 6.762 0 00-6.936 0 1 1 0 10.693 1.488 6.762 6.762 0 006.936 0zM10 14a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" clip-rule="evenodd"/></svg>`;
  }
  // Moderate signal (50% - 74%)
  if (signal >= 50) {
    return `<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"><path d="M15.467 10.421a6.762 6.762 0 00-6.936 0 1 1 0 10.693 1.488 6.762 6.762 0 006.936 0zM10 14a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"/></svg>`;
  }
  // Weak signal (25% - 49%)
  if (signal >= 25) {
    return `<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"><path d="M10 14a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"/></svg>`;
  }
  // Very weak signal (< 25%)
  return `<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M12 20h.01"/></svg>`;
}

// Opens the sliding password entry dialog modal
function openPasswordModal(bssid, ssid, band, frequency) {
  selectedBssid = bssid;
  selectedSsid = ssid;
  modalSsid.innerHTML = `${ssid} <span class="text-[10px] text-teal-400 block mt-1 font-mono">BSSID: ${bssid} | Band: ${band} (${frequency})</span>`;
  wifiPassword.value = "";
  
  // Reveal password modal with transition
  passwordModal.classList.remove("opacity-0", "pointer-events-none");
  passwordModal.classList.add("opacity-100");
  passwordModal.firstElementChild.classList.remove("scale-90");
  passwordModal.firstElementChild.classList.add("scale-100");
  wifiPassword.focus();
}

// Closes the Wi-Fi password modal
function closeModal() {
  passwordModal.classList.add("opacity-0", "pointer-events-none");
  passwordModal.classList.remove("opacity-100");
  passwordModal.firstElementChild.classList.add("scale-90");
  passwordModal.firstElementChild.classList.remove("scale-100");
}

// 2. WI-FI MANAGER: EXECUTE NETWORK CONNECTION
async function connectWifi() {
  const password = wifiPassword.value;
  btnConnect.classList.add("btn-loading");
  btnConnect.disabled = true;
  btnCancel.disabled = true;

  logMessage(`Attempting connection to Wi-Fi: "${selectedSsid}" (${selectedBssid})...`);
  showToast(`Connecting to ${selectedSsid}...`);

  try {
    const res = await invoke("connect_wifi", { bssid: selectedBssid, password: password || null });
    showToast("Wi-Fi connected successfully!");
    logMessage(`Connection successful: ${res}`);
    activeWifiNet.innerHTML = `Connected: <strong class="text-teal-400">${selectedSsid}</strong>`;
    closeModal();
    // Refresh list of surrounding networks after 2 seconds
    setTimeout(scanWifi, 2000);
  } catch (err) {
    showToast(`Connection failed: ${err}`, true);
    logMessage(`Wi-Fi connection error: ${err}`);
  } finally {
    btnConnect.classList.remove("btn-loading");
    btnConnect.disabled = false;
    btnCancel.disabled = false;
  }
}

// 3. CLOUDFLARE WARP STATUS POLLING
let isFetchingWarpStatus = false;

// Synchronizes the actual WARP operating mode from backend settings
async function syncWarpMode() {
  try {
    const mode = await invoke("get_warp_mode");
    currentWarpMode = mode;
    updateWarpModeUI(mode);
    disableModeButtons(false);
  } catch (err) {
    // Silently bypass mode query failures during initialization
  }
}

// Retrieve initial system statuses on load
async function getInitialStatus() {
  await getWarpStatus();
  await syncWarpMode();
}

// Recurring status poll for Cloudflare WARP state
async function pollWarpStatus() {
  if (isFetchingWarpStatus) return;
  isFetchingWarpStatus = true;

  try {
    await getWarpStatus();
    // Periodically update diagnostics alongside the 5s status loop
    updateNetworkDiagnostics();
  } finally {
    isFetchingWarpStatus = false;
    // Reschedule next status retrieval cycle in 5 seconds
    setTimeout(pollWarpStatus, 5000);
  }
}

// Queries WARP daemon connection status and updates dashboard LEDs/badges
async function getWarpStatus() {
  try {
    const status = await invoke("get_warp_status");
    updateWarpUI(status);
    
    if (status === "Not Installed") {
      disableModeButtons(true);
      if (warpModeBadgeContainer) warpModeBadgeContainer.classList.add("hidden");
    } else {
      disableModeButtons(false);
    }
  } catch (err) {
    // Avoid flood logs in case backend daemon is temporarily asleep
    updateWarpUI("Disconnected");
    disableModeButtons(true);
    if (warpModeBadgeContainer) warpModeBadgeContainer.classList.add("hidden");
  }
}

// Renders the overall WARP UI elements depending on current status value
function updateWarpUI(status) {
  // Handles: Connected, Disconnected, Connecting, Not Installed
  if (status === "Connected") {
    ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-emerald-500 shadow-[0_0_12px_rgba(16,185,129,0.7)] transition-all duration-300";
    ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75";
    warpStatusText.textContent = "Connected";
    warpStatusText.className = "text-xs font-semibold mt-1 text-emerald-400 uppercase tracking-widest";
    if (warpNetworkText) {
      warpNetworkText.textContent = "Your network traffic is securely encrypted.";
    }
    
    warpToggle.disabled = false;
    if (!isTogglingWarp) {
      warpToggle.checked = true;
    }
  } else if (status === "Connecting") {
    ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-amber-500 shadow-[0_0_12px_rgba(245,158,11,0.7)] transition-all duration-300";
    ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-400 opacity-75";
    warpStatusText.textContent = "Connecting...";
    warpStatusText.className = "text-xs font-semibold mt-1 text-amber-400 uppercase tracking-widest";
    if (warpNetworkText) {
      warpNetworkText.textContent = "Establishing a secure connection...";
    }
    
    warpToggle.disabled = true;
  } else if (status === "Not Installed") {
    ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-slate-500 shadow-none transition-all duration-300";
    ledPing.className = "hidden";
    warpStatusText.textContent = "Not Installed";
    warpStatusText.className = "text-xs font-semibold mt-1 text-slate-400 uppercase tracking-widest";
    if (warpNetworkText) {
      warpNetworkText.textContent = "Could not find Cloudflare WARP client.";
    }
    
    warpToggle.disabled = true;
    warpToggle.checked = false;
    
    // Add pulsing visual cues to the install action button
    btnInstall.classList.add("animate-pulse");
  } else {
    // Disconnected state
    ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-red-500 shadow-[0_0_12px_rgba(239,68,68,0.7)] transition-all duration-300";
    ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75";
    warpStatusText.textContent = "Disconnected";
    warpStatusText.className = "text-xs font-semibold mt-1 text-red-400 uppercase tracking-widest";
    if (warpNetworkText) {
      warpNetworkText.textContent = "Your network connection is direct.";
    }
    
    warpToggle.disabled = false;
    if (!isTogglingWarp) {
      warpToggle.checked = false;
    }
    btnInstall.classList.remove("animate-pulse");
  }
}

// 4. CLOUDFLARE WARP: ENABLE / DISABLE TOGGLE CONTROL
async function handleWarpToggle() {
  if (isTogglingWarp) return;
  isTogglingWarp = true;
  
  const wantConnect = warpToggle.checked;
  warpToggle.disabled = true;
  
  logMessage(`Triggering WARP ${wantConnect ? "CONNECTION" : "DISCONNECTION"}...`);
  showToast(wantConnect ? "Activating WARP protection..." : "Deactivating WARP protection...");
  
  try {
    const res = await invoke("warp_toggle", { connect: wantConnect });
    showToast(wantConnect ? "WARP connection command sent!" : "WARP disconnection command sent.");
    logMessage(`WARP Command Output: ${res}`);
  } catch (err) {
    showToast(`WARP control failed: ${err}`, true);
    logMessage(`WARP control error: ${err}`);
    // Rollback the checkbox GUI toggle switch
    warpToggle.checked = !wantConnect;
  } finally {
    isTogglingWarp = false;
    // Always query and synchronize with the actual daemon status from backend
    await getWarpStatus();
    // Instantly refresh diagnostics to update the new IP and Latency metrics
    updateNetworkDiagnostics();
  }
}

// 5. CLOUDFLARE WARP: SYSTEM INSTALLATION (FEDORA PKEXEC)
async function installWarp() {
  btnInstall.classList.add("btn-loading");
  btnInstall.disabled = true;
  
  logMessage("Initiating Cloudflare WARP install process...");
  logMessage("Step 1: Running 'dnf download cloudflare-warp'...");
  showToast("Installing Cloudflare WARP...");

  try {
    const result = await invoke("install_warp");
    showToast("Cloudflare WARP installed successfully!");
    logMessage(`Installation completed: ${result}`);
    // Instantly trigger sync and refresh
    await getWarpStatus();
    await syncWarpMode();
  } catch (err) {
    showToast(`WARP installation failed: ${err}`, true);
    logMessage(`WARP installation error: ${err}`);
  } finally {
    btnInstall.classList.remove("btn-loading");
    btnInstall.disabled = false;
  }
}

// 6. CLOUDFLARE WARP: CHANGE OPERATING MODE
async function handleModeChange(mode) {
  if (isSettingMode) return;
  isSettingMode = true;

  // Optimistic UI state switch for instantaneous visual feedback
  currentWarpMode = mode;
  updateWarpModeUI(mode);
  disableModeButtons(true);

  logMessage(`Switching WARP mode to: ${mode.toUpperCase()}...`);
  showToast(`Switching to ${mode.toUpperCase()} mode...`);

  // Show visual cue that connection is updating
  if (warpStatusText) {
    warpStatusText.textContent = "Connecting...";
    warpStatusText.className = "text-xs font-semibold mt-1 text-amber-400 uppercase tracking-widest";
  }
  if (ledDot) {
    ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-amber-500 shadow-[0_0_12px_rgba(245,158,11,0.7)] transition-all duration-300";
  }
  if (ledPing) {
    ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-400 opacity-75";
  }
  if (traceWarpBadge) {
    traceWarpBadge.textContent = "CONNECTING";
    traceWarpBadge.className = "bg-amber-500/10 text-amber-400 border border-amber-500/20 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider transition-all duration-300";
  }

  try {
    const res = await invoke("set_warp_mode", { mode });
    showToast(`Mode switched to ${mode.toUpperCase()}!`);
    logMessage(`Mode switch result: ${res}`);
  } catch (err) {
    showToast(`Mode switch failed: ${err}`, true);
    logMessage(`Mode switch error: ${err}`);
    // Rollback to the actual mode fetched from backend
    await syncWarpMode();
  } finally {
    isSettingMode = false;
    disableModeButtons(false);
    // Delay triggering network diagnostics to let the WARP adapter complete routing tables assignment
    setTimeout(() => {
      updateNetworkDiagnostics();
      getWarpStatus();
    }, 1500);
  }
}

// Enforces temporary disable status on toggle buttons
function disableModeButtons(disabled) {
  if (btnModeDoh) btnModeDoh.disabled = disabled;
  if (btnModeWarpDoh) btnModeWarpDoh.disabled = disabled;
}

// Updates background styling properties of mode buttons based on active mode string
function updateWarpModeUI(mode) {
  const activeClass = "bg-gradient-to-r from-orange-500 to-amber-500 text-white border-transparent shadow-[0_0_15px_rgba(249,115,22,0.4)] scale-[1.02]";
  const inactiveClass = "text-slate-400 hover:text-slate-200 border-transparent hover:bg-slate-900/40";

  [
    { btn: btnModeDoh, key: "doh" },
    { btn: btnModeWarpDoh, key: "warp+doh" }
  ].forEach(({ btn, key }) => {
    if (!btn) return;
    if (key === mode) {
      btn.className = `py-2 px-1 rounded-xl text-[9px] font-bold tracking-wide transition-all duration-200 focus:outline-none flex flex-col items-center justify-center gap-1.5 border ${activeClass}`;
    } else {
      btn.className = `py-2 px-1 rounded-xl text-[9px] font-bold tracking-wide transition-all duration-200 focus:outline-none flex flex-col items-center justify-center gap-1.5 border ${inactiveClass}`;
    }
  });

  // Updates status badge text with friendly descriptions
  if (warpModeBadge && warpModeBadgeContainer) {
    if (mode === "doh") {
      warpModeBadge.innerHTML = `
        <svg class="w-3 h-3 animate-pulse text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M13 10V3L4 14h7v7l9-11h-7z"/>
        </svg>
        <span>Mode: DoH DNS Only</span>
      `;
      warpModeBadge.className = "inline-flex items-center gap-1.5 px-3 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-orange-500/10 text-orange-400 border border-orange-500/20 shadow-[0_0_10px_rgba(249,115,22,0.1)]";
      warpModeBadgeContainer.classList.remove("hidden");
    } else if (mode === "warp+doh") {
      warpModeBadge.innerHTML = `
        <svg class="w-3 h-3 animate-pulse text-teal-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
        </svg>
        <span>Mode: WARP + DoH (Max)</span>
      `;
      warpModeBadge.className = "inline-flex items-center gap-1.5 px-3 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-teal-500/10 text-teal-400 border border-teal-500/20 shadow-[0_0_10px_rgba(20,184,166,0.1)]";
      warpModeBadgeContainer.classList.remove("hidden");
    } else {
      warpModeBadge.innerHTML = `
        <svg class="w-3 h-3 animate-pulse text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <span>Mode: ${mode.toUpperCase()}</span>
      `;
      warpModeBadge.className = "inline-flex items-center gap-1.5 px-3 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-slate-500/10 text-slate-400 border border-slate-500/20 shadow-none";
      warpModeBadgeContainer.classList.remove("hidden");
    }
  }
}

// 7. REAL-TIME NETWORK SPEED MONITORING (1S TIMER)
async function startNetworkSpeedMonitor() {
  try {
    const io = await invoke("get_network_io");
    lastRxBytes = io.rx_bytes;
    lastTxBytes = io.tx_bytes;
    lastSpeedTime = Date.now();
  } catch (e) {
    console.error("Failed to read initial network bytes:", e);
  }

  // Monitor network I/O every 1 second
  setInterval(updateNetworkSpeed, 1000);
}

async function updateNetworkSpeed() {
  try {
    const io = await invoke("get_network_io");
    const now = Date.now();
    const timeDiffSec = (now - lastSpeedTime) / 1000;
    if (timeDiffSec <= 0) return;

    // Handle overflow or reset events safely
    const rxDiff = io.rx_bytes > lastRxBytes ? io.rx_bytes - lastRxBytes : 0;
    const txDiff = io.tx_bytes > lastTxBytes ? io.tx_bytes - lastTxBytes : 0;

    const downSpeed = rxDiff / timeDiffSec; // bytes per second
    const upSpeed = txDiff / timeDiffSec; // bytes per second

    // Update frontend text components
    if (speedDownload) speedDownload.textContent = formatSpeed(downSpeed);
    if (speedUpload) speedUpload.textContent = formatSpeed(upSpeed);

    // Save historical points to build smooth visual chart
    speedHistory.push({ rx: downSpeed, tx: upSpeed });
    if (speedHistory.length > MAX_HISTORY_POINTS) {
      speedHistory.shift();
    }

    // Paint dynamic sparklines onto canvas
    drawSparkline();

    // Cache values for the next tick
    lastRxBytes = io.rx_bytes;
    lastTxBytes = io.tx_bytes;
    lastSpeedTime = now;
  } catch (e) {
    // Suppress system errors to keep UI smooth and persistent
  }
}

// Format raw bytes per second to human readable speeds
function formatSpeed(bytesPerSec) {
  if (bytesPerSec < 1024) {
    return `${bytesPerSec.toFixed(1)} B/s`;
  }
  const kb = bytesPerSec / 1024;
  if (kb < 1024) {
    return `${kb.toFixed(2)} KB/s`;
  }
  const mb = kb / 1024;
  return `${mb.toFixed(2)} MB/s`;
}

// Renders an anti-aliased retina-ready double sparkline canvas chart
function drawSparkline() {
  if (!speedCtx || !speedChart || speedHistory.length < 2) return;

  const dpr = window.devicePixelRatio || 1;
  const displayWidth = speedChart.clientWidth;
  const displayHeight = speedChart.clientHeight;

  // Adapt backing store dynamically to ensure pixel-perfect resolution on HiDPI displays
  if (speedChart.width !== displayWidth * dpr || speedChart.height !== displayHeight * dpr) {
    speedChart.width = displayWidth * dpr;
    speedChart.height = displayHeight * dpr;
  }

  // Reset transforms and apply display ratio scaling
  speedCtx.resetTransform();
  speedCtx.scale(dpr, dpr);

  // Clear previous drawings
  speedCtx.clearRect(0, 0, displayWidth, displayHeight);

  // Determine scaling factor by finding the maximum download/upload point
  let maxVal = 10240; // minimum scale threshold of 10 KB/s to look natural on idle networks
  speedHistory.forEach(pt => {
    if (pt.rx > maxVal) maxVal = pt.rx;
    if (pt.tx > maxVal) maxVal = pt.tx;
  });

  const padding = 2;
  const plotWidth = displayWidth - padding * 2;
  const plotHeight = displayHeight - padding * 2;
  const step = plotWidth / (MAX_HISTORY_POINTS - 1);

  // Sub-routine to trace, stroke, and fill paths smoothly
  function drawCurve(dataKey, strokeColor, fillColor) {
    speedCtx.beginPath();
    
    for (let i = 0; i < speedHistory.length; i++) {
      const x = padding + i * step;
      const val = speedHistory[i][dataKey];
      const y = padding + plotHeight - (val / maxVal) * plotHeight;
      
      if (i === 0) {
        speedCtx.moveTo(x, y);
      } else {
        speedCtx.lineTo(x, y);
      }
    }

    // Stroke outline path
    speedCtx.strokeStyle = strokeColor;
    speedCtx.lineWidth = 1.6;
    speedCtx.lineCap = "round";
    speedCtx.lineJoin = "round";
    speedCtx.stroke();

    // Close path boundary down to bottom coordinates to fill area
    speedCtx.lineTo(padding + (speedHistory.length - 1) * step, padding + plotHeight);
    speedCtx.lineTo(padding, padding + plotHeight);
    speedCtx.closePath();

    // Create beautiful transparent gradient under the line
    const grad = speedCtx.createLinearGradient(0, 0, 0, displayHeight);
    grad.addColorStop(0, fillColor);
    grad.addColorStop(1, "rgba(2, 6, 23, 0)"); // Fades cleanly into deep background slate
    
    speedCtx.fillStyle = grad;
    speedCtx.fill();
  }

  // Draw Download curve (Teal styling)
  drawCurve("rx", "#2dd4bf", "rgba(45, 212, 191, 0.12)");

  // Draw Upload curve (Orange styling)
  drawCurve("tx", "#fb923c", "rgba(251, 146, 60, 0.08)");
}

// 8. DIAGNOSTICS & NETWORK DIAGRAM MONITORING (PING / GEOLOCATION)
let isCheckingDiagnostics = false;

async function updateNetworkDiagnostics() {
  if (isCheckingDiagnostics) return;
  isCheckingDiagnostics = true;

  // Run quick latency checks and public IP geolocations asynchronously in parallel
  await Promise.all([
    runQuickPings(),
    runIPTracing()
  ]);

  isCheckingDiagnostics = false;
}

// Runs parallel non-blocking pings to common secure DNS targets
async function runQuickPings() {
  try {
    const results = await invoke("ping_multiple", { targets: ["1.1.1.1", "8.8.8.8"] });
    
    // Check if system is currently switching modes or connecting
    const statusText = warpStatusText ? warpStatusText.textContent.toLowerCase() : "";
    const isConnecting = isSettingMode || statusText.includes("connecting") || statusText.includes("updating");

    results.forEach(res => {
      if (res.target === "1.1.1.1") {
        if (pingCloudflare) {
          if (res.latency !== null) {
            pingCloudflare.textContent = `${res.latency.toFixed(1)} ms`;
            pingCloudflare.className = "text-xs font-black text-teal-400 font-mono";
          } else if (isConnecting) {
            pingCloudflare.textContent = "Connecting...";
            pingCloudflare.className = "text-xs font-bold text-amber-400 font-mono";
          } else {
            pingCloudflare.textContent = "Offline";
            pingCloudflare.className = "text-xs font-black text-red-500 font-mono";
          }
        }
      } else if (res.target === "8.8.8.8") {
        if (pingGoogle) {
          if (res.latency !== null) {
            pingGoogle.textContent = `${res.latency.toFixed(1)} ms`;
            pingGoogle.className = "text-xs font-black text-blue-400 font-mono";
          } else if (isConnecting) {
            pingGoogle.textContent = "Connecting...";
            pingGoogle.className = "text-xs font-bold text-amber-400 font-mono";
          } else {
            pingGoogle.textContent = "Offline";
            pingGoogle.className = "text-xs font-black text-red-500 font-mono";
          }
        }
      }
    });
  } catch (e) {
    console.error("Failed to run quick pings:", e);
    const statusText = warpStatusText ? warpStatusText.textContent.toLowerCase() : "";
    const isConnecting = isSettingMode || statusText.includes("connecting") || statusText.includes("updating");
    
    if (pingCloudflare) {
      pingCloudflare.textContent = isConnecting ? "Connecting..." : "Offline";
      pingCloudflare.className = isConnecting ? "text-xs font-bold text-amber-400 font-mono" : "text-xs font-black text-red-500 font-mono";
    }
    if (pingGoogle) {
      pingGoogle.textContent = isConnecting ? "Connecting..." : "Offline";
      pingGoogle.className = isConnecting ? "text-xs font-bold text-amber-400 font-mono" : "text-xs font-black text-red-500 font-mono";
    }
  }
}

// Query geolocation coordinates and determine VPN / WARP state
async function runIPTracing() {
  try {
    const res = await invoke("trace_ip");
    const info = JSON.parse(res);

    if (info.status === "fail") {
      throw new Error(info.message || "Failed to query GeoIP API");
    }

    // Refresh telemetry components in view
    if (traceIpAddress) traceIpAddress.textContent = info.query || "N/A";
    if (traceIsp) traceIsp.textContent = info.isp || "N/A";
    if (traceLocation) {
      traceLocation.textContent = `${info.city || "N/A"}, ${info.countryCode || info.country || "N/A"}`;
    }
    if (traceCoords) {
      traceCoords.textContent = `Lat ${info.lat?.toFixed(3) || "N/A"}, Lon ${info.lon?.toFixed(3) || "N/A"}`;
    }

    // Detect if IP is routed through Cloudflare to flag WARP state
    const isWarp = (info.isp || "").toLowerCase().includes("cloudflare") || 
                   (info.org || "").toLowerCase().includes("cloudflare") || 
                   (info.as || "").toLowerCase().includes("cloudflare");

    if (traceWarpBadge) {
      if (isWarp) {
        traceWarpBadge.textContent = "WARP ACTIVE";
        traceWarpBadge.className = "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider transition-all duration-300 shadow-[0_0_8px_rgba(16,185,129,0.15)]";
      } else {
        traceWarpBadge.textContent = "DIRECT";
        traceWarpBadge.className = "bg-red-500/10 text-red-400 border border-red-500/20 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider transition-all duration-300";
      }
    }
  } catch (e) {
    console.error("IP trace error:", e);
    
    // Check if system is currently switching modes or connecting
    const statusText = warpStatusText ? warpStatusText.textContent.toLowerCase() : "";
    const isConnecting = isSettingMode || statusText.includes("connecting") || statusText.includes("updating");
    
    if (isConnecting) {
      if (traceIpAddress) traceIpAddress.textContent = "Connecting...";
      if (traceIsp) traceIsp.textContent = "Updating routing...";
      if (traceLocation) traceLocation.textContent = "Locating...";
      if (traceCoords) traceCoords.textContent = "Lat --, Lon --";
      
      if (traceWarpBadge) {
        traceWarpBadge.textContent = "CONNECTING";
        traceWarpBadge.className = "bg-amber-500/10 text-amber-400 border border-amber-500/20 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider transition-all duration-300";
      }
    } else {
      if (traceIpAddress) traceIpAddress.textContent = "Offline/Fail";
      if (traceIsp) traceIsp.textContent = "Connection offline";
      if (traceLocation) traceLocation.textContent = "Unknown location";
      if (traceCoords) traceCoords.textContent = "N/A";
      
      if (traceWarpBadge) {
        traceWarpBadge.textContent = "OFFLINE";
        traceWarpBadge.className = "bg-slate-800 text-slate-500 border border-slate-700 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider";
      }
    }
  }
}
