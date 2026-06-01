// Wi-Fi networking client module
// All comments in English as per user rules

const { invoke } = window.__TAURI__.core;
import { el } from "./dom.js";
import { state } from "./state.js";
import { showToast, logMessage, openPasswordModal, closePasswordModal } from "./ui.js";

// Triggers scanning for Wi-Fi networks in range
export async function scanWifi() {
  if (state.isScanning) return;
  state.isScanning = true;
  
  if (el.iconScan) el.iconScan.classList.add("anim-scan");
  if (el.btnScan) el.btnScan.disabled = true;
  
  logMessage("Scanning for surrounding Wi-Fi networks...");
  
  // Play the radar animation container in the modal if it exists
  const radar = document.getElementById("wifi-radar-container");
  if (radar) radar.classList.remove("hidden");
  
  try {
    const list = await invoke("get_wifi_list");
    let savedList = [];
    try {
      savedList = await invoke("get_saved_wifi_list");
    } catch (e) {
      logMessage(`Failed to fetch saved connections: ${e}`);
    }
    if (el.wifiCount) el.wifiCount.textContent = list.length;
    renderWifiList(list, savedList);
    logMessage(`Wi-Fi scanning completed. Found ${list.length} networks.`);
  } catch (err) {
    showToast(`Wi-Fi scan failed: ${err}`, true);
    logMessage(`Wi-Fi scan error: ${err}`);
    if (el.wifiContainer) {
      el.wifiContainer.innerHTML = `
        <div class="py-12 flex flex-col items-center justify-center space-y-3 text-red-400">
          <svg class="w-8 h-8 text-red-500 animate-bounce" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
          </svg>
          <p class="text-sm font-semibold">Wi-Fi adapter offline. Check hardware configuration.</p>
        </div>
      `;
    }
  } finally {
    if (el.iconScan) el.iconScan.classList.remove("anim-scan");
    if (el.btnScan) el.btnScan.disabled = false;
    state.isScanning = false;
    if (radar) radar.classList.add("hidden");
  }
}

// Renders the list of Wi-Fi networks in the available Wi-Fi Modal
export function renderWifiList(networks, savedList = []) {
  if (!el.wifiContainer) return;
  
  if (networks.length === 0) {
    el.wifiContainer.innerHTML = `
      <div class="py-12 flex flex-col items-center justify-center text-slate-500">
        <p class="text-sm font-semibold">No Wi-Fi networks found.</p>
      </div>
    `;
    updateActiveWifiDisplay(null);
    return;
  }

  let activeNet = null;
  el.wifiContainer.innerHTML = "";

  networks.forEach((net) => {
    const item = document.createElement("div");
    const wifiSvg = getWifiSignalSvg(net.signal);
    const isSaved = savedList.includes(net.ssid);
    
    // Create highly premium Cyberpunk styled band badges
    let bandBadge = "";
    if (net.band.includes("6 GHz")) {
      bandBadge = '<span class="text-[9px] bg-fuchsia-500/10 text-fuchsia-400 px-1.5 py-0.5 rounded border border-fuchsia-500/20 font-bold ml-1.5 shadow-[0_0_8px_rgba(217,70,239,0.15)] font-mono animate-pulse">6 GHz</span>';
    } else if (net.band.includes("5 GHz")) {
      bandBadge = '<span class="text-[9px] bg-cyan-500/10 text-cyan-400 px-1.5 py-0.5 rounded border border-cyan-500/20 font-bold ml-1.5 shadow-[0_0_8px_rgba(6,182,212,0.15)] font-mono">5 GHz</span>';
    } else if (net.band.includes("2.4 GHz")) {
      bandBadge = '<span class="text-[9px] bg-amber-500/10 text-amber-400 px-1.5 py-0.5 rounded border border-amber-500/20 font-bold ml-1.5 font-mono">2.4G</span>';
    }
    
    if (net.active) {
      activeNet = net;
      
      const savedBadge = isSaved ? '<span class="text-[9px] bg-emerald-500/10 text-emerald-400 px-1.5 py-0.5 rounded border border-emerald-500/20 font-bold ml-1.5">Saved</span>' : '';
      
      // Beautiful card design for active/connected network
      item.className = "flex items-center justify-between p-3.5 bg-emerald-950/20 hover:bg-emerald-900/20 border border-emerald-500/40 hover:border-emerald-400/60 rounded-2xl cursor-pointer transition-all duration-200 group active:scale-[0.99] shadow-[0_0_15px_rgba(16,185,129,0.15)]";
      item.innerHTML = `
        <div class="flex items-center space-x-3.5">
          <div class="text-emerald-400">
            ${wifiSvg}
          </div>
          <div>
            <h4 class="text-sm font-bold text-emerald-300 truncate max-w-[200px] flex items-center">${net.ssid}${savedBadge}${bandBadge}</h4>
            <div class="text-[10px] text-emerald-500 font-semibold space-y-0.5 mt-0.5">
              <div>MAC: <span class="font-mono text-emerald-400/80">${net.bssid}</span> | Band: <span class="text-emerald-400/80">${net.band} (${net.frequency})</span></div>
              <div>Signal: ${net.signal}% | Channel: ${net.channel} | Security: ${net.security || "Open"}</div>
            </div>
          </div>
        </div>
        <div class="flex items-center space-x-2 shrink-0">
          <span class="text-[10px] bg-emerald-500/10 text-emerald-300 px-2.5 py-1 rounded-lg border border-emerald-500/20 font-bold flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
            Connected
          </span>
        </div>
      `;
      
      item.addEventListener("click", () => {
        openPasswordModal(net.bssid, net.ssid, net.band, net.frequency, isSaved);
      });
    } else {
      // Sleek available networks card
      const savedBadge = isSaved ? '<span class="text-[9px] bg-slate-800/80 text-emerald-400 px-1.5 py-0.5 rounded border border-slate-700/60 font-bold ml-1.5 group-hover:border-emerald-500/20 transition-all">Saved</span>' : '';
      
      item.className = "flex items-center justify-between p-3.5 bg-slate-950/40 hover:bg-slate-800/40 border border-slate-800/40 hover:border-slate-700/60 rounded-2xl cursor-pointer transition-all duration-200 group active:scale-[0.99]";
      item.innerHTML = `
        <div class="flex items-center space-x-3.5">
          <div class="text-slate-400 group-hover:text-emerald-400 transition-colors">
            ${wifiSvg}
          </div>
          <div>
            <h4 class="text-sm font-semibold text-slate-200 group-hover:text-white transition-colors truncate max-w-[200px] flex items-center">${net.ssid}${savedBadge}${bandBadge}</h4>
            <div class="text-[10px] text-slate-500 group-hover:text-slate-400 font-medium space-y-0.5 mt-0.5 transition-colors">
              <div>MAC: <span class="font-mono text-slate-400/80 group-hover:text-emerald-400/60">${net.bssid}</span> | Band: <span class="text-slate-400/80 group-hover:text-emerald-400/60">${net.band} (${net.frequency})</span></div>
              <div>Signal: ${net.signal}% | Channel: ${net.channel} | Security: ${net.security || "Open"}</div>
            </div>
          </div>
        </div>
        <div class="flex items-center space-x-2 shrink-0">
          <span class="text-[10px] bg-slate-800/80 group-hover:bg-emerald-500/10 text-slate-400 group-hover:text-emerald-300 px-2.5 py-1 rounded-lg border border-slate-700/40 group-hover:border-emerald-500/20 font-semibold transition-all">Connect</span>
        </div>
      `;
      
      item.addEventListener("click", () => openPasswordModal(net.bssid, net.ssid, net.band, net.frequency, isSaved));
    }
    
    el.wifiContainer.appendChild(item);
  });

  updateActiveWifiDisplay(activeNet);
}

// Updates the Widget Wi-Fi in the main Panel 1
export function updateActiveWifiDisplay(activeNet) {
  if (!el.activeWifiSSID || !el.activeWifiNet || !el.wifiSignalIcon) return;

  if (activeNet) {
    el.activeWifiSSID.textContent = activeNet.ssid;
    el.activeWifiNet.innerHTML = `Connected | Band: <span class="text-emerald-400 font-mono">${activeNet.band} (${activeNet.frequency})</span>`;
    
    // Update Glowing Signal Badge
    if (el.activeWifiSignalBadge && el.activeWifiSignalText) {
      el.activeWifiSignalBadge.classList.remove("hidden");
      el.activeWifiSignalText.textContent = `${activeNet.signal}%`;

      // Apply dynamic colors depending on signal level
      let badgeClasses = "";
      if (activeNet.signal >= 75) {
        badgeClasses = "bg-emerald-500/10 text-emerald-400 border-emerald-500/20";
      } else if (activeNet.signal >= 50) {
        badgeClasses = "bg-cyan-500/10 text-cyan-400 border-cyan-500/20";
      } else if (activeNet.signal >= 25) {
        badgeClasses = "bg-orange-500/10 text-orange-400 border-orange-500/20";
      } else {
        badgeClasses = "bg-red-500/10 text-red-400 border-red-500/20 animate-pulse";
      }
      el.activeWifiSignalBadge.className = `text-[10px] font-mono font-bold px-2.5 py-1 rounded-lg border flex items-center gap-1 transition-all duration-300 ${badgeClasses}`;
    }

    // Wave Wi-Fi Signal SVG color green
    el.wifiSignalIcon.className = "text-emerald-400 w-6 h-6 animate-pulse";
    el.wifiSignalIcon.innerHTML = getWifiSignalSvg(activeNet.signal);
    
    // Update dashboard title
    const activeWifiLabel = document.getElementById("active-wifi-net-label");
    if (activeWifiLabel) {
      activeWifiLabel.innerHTML = `Connected to: <strong class="text-emerald-400">${activeNet.ssid}</strong>`;
    }
  } else {
    el.activeWifiSSID.textContent = "No Connection";
    el.activeWifiNet.textContent = "Click to scan and connect to a Wi-Fi network";
    
    // Hide Signal Badge
    if (el.activeWifiSignalBadge) {
      el.activeWifiSignalBadge.classList.add("hidden");
    }

    el.wifiSignalIcon.className = "text-slate-500 w-6 h-6";
    el.wifiSignalIcon.innerHTML = `<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 5.636a9 9 0 010 12.728m0 0l-2.829-2.829m2.829 2.829L21 21M15.536 8.464a5 5 0 010 7.072m0 0l-2.829-2.829m-4.243 2.829a4.978 4.978 0 01-1.414-3.536 5 5 0 011.414-3.536m0 0L5.636 5.636M1.414 1.414L3 3m0 0L21 21M8.464 15.536L5.636 18.364m0 0l-1.414-1.414m1.414 1.414L1 21"/></svg>`;
    
    const activeWifiLabel = document.getElementById("active-wifi-net-label");
    if (activeWifiLabel) {
      activeWifiLabel.textContent = "No Active Connection";
    }
  }
}

// Starts the dynamic, low-overhead background polling monitor for the active connection (1s interval)
export function startActiveWifiMonitor() {
  setInterval(updateActiveWifiSignal, 1000);
}

// Fetches the active connection details asynchronously and updates display
async function updateActiveWifiSignal() {
  try {
    const activeNet = await invoke("get_active_wifi");
    updateActiveWifiDisplay(activeNet);
  } catch (err) {
    // Suppress background errors to guarantee seamless UX
  }
}

// Submits the connection password and connects to a network
export async function connectWifi() {
  if (!el.wifiPassword || !el.btnConnect || !el.btnCancel || !el.wifiLockBssid) return;
  const password = el.wifiPassword.value;
  const lockBssid = el.wifiLockBssid.checked;
  el.btnConnect.classList.add("btn-loading");
  el.btnConnect.disabled = true;
  el.btnCancel.disabled = true;

  logMessage(`Connecting to network "${state.selectedSsid}" (${state.selectedBssid}) [Lock BSSID: ${lockBssid}]...`);
  showToast(`Initiating connection to ${state.selectedSsid}...`);

  try {
    const res = await invoke("connect_wifi", {
      bssid: state.selectedBssid,
      ssid: state.selectedSsid,
      password: password || null,
      lockBssid: lockBssid
    });
    showToast("Wi-Fi connected successfully!");
    logMessage(`Connected successfully to ${state.selectedSsid}.`);
    
    if (el.activeWifiSSID) el.activeWifiSSID.textContent = state.selectedSsid;
    
    closePasswordModal();
    
    // Close the list modal automatically after successful connection
    setTimeout(() => {
      const closeBtn = document.getElementById("wifi-list-close");
      if (closeBtn) closeBtn.click();
    }, 800);
    
    // Refresh Wi-Fi status after 2 seconds
    setTimeout(scanWifi, 2000);
  } catch (err) {
    showToast(`Wi-Fi connection failed: ${err}`, true);
    logMessage(`Wi-Fi connection error: ${err}`);
  } finally {
    el.btnConnect.classList.remove("btn-loading");
    el.btnConnect.disabled = false;
    el.btnCancel.disabled = false;
  }
}

// Maps Wi-Fi signal strength (%) to appropriate SVG wave lines
export function getWifiSignalSvg(signal) {
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
