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
    if (el.wifiCount) el.wifiCount.textContent = list.length;
    renderWifiList(list);
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
export function renderWifiList(networks) {
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
    
    if (net.active) {
      activeNet = net;
      
      // Beautiful card design for active/connected network
      item.className = "flex items-center justify-between p-3.5 bg-emerald-950/20 hover:bg-emerald-900/20 border border-emerald-500/40 hover:border-emerald-400/60 rounded-2xl cursor-pointer transition-all duration-200 group active:scale-[0.99] shadow-[0_0_15px_rgba(16,185,129,0.15)]";
      item.innerHTML = `
        <div class="flex items-center space-x-3.5">
          <div class="text-emerald-400">
            ${wifiSvg}
          </div>
          <div>
            <h4 class="text-sm font-bold text-emerald-300 truncate max-w-[200px]">${net.ssid}</h4>
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
        showToast(`You are already connected to "${net.ssid}".`);
      });
    } else {
      // Sleek available networks card
      item.className = "flex items-center justify-between p-3.5 bg-slate-950/40 hover:bg-slate-800/40 border border-slate-800/40 hover:border-slate-700/60 rounded-2xl cursor-pointer transition-all duration-200 group active:scale-[0.99]";
      item.innerHTML = `
        <div class="flex items-center space-x-3.5">
          <div class="text-slate-400 group-hover:text-emerald-400 transition-colors">
            ${wifiSvg}
          </div>
          <div>
            <h4 class="text-sm font-semibold text-slate-200 group-hover:text-white transition-colors truncate max-w-[200px]">${net.ssid}</h4>
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
      
      item.addEventListener("click", () => openPasswordModal(net.bssid, net.ssid, net.band, net.frequency));
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
    el.activeWifiNet.innerHTML = `Connected | Band: <span class="text-emerald-400 font-mono">${activeNet.band} (${activeNet.frequency})</span> | Sig: <span class="text-emerald-400 font-mono">${activeNet.signal}%</span>`;
    
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
    el.wifiSignalIcon.className = "text-slate-500 w-6 h-6";
    el.wifiSignalIcon.innerHTML = `<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 5.636a9 9 0 010 12.728m0 0l-2.829-2.829m2.829 2.829L21 21M15.536 8.464a5 5 0 010 7.072m0 0l-2.829-2.829m-4.243 2.829a4.978 4.978 0 01-1.414-3.536 5 5 0 011.414-3.536m0 0L5.636 5.636M1.414 1.414L3 3m0 0L21 21M8.464 15.536L5.636 18.364m0 0l-1.414-1.414m1.414 1.414L1 21"/></svg>`;
    
    const activeWifiLabel = document.getElementById("active-wifi-net-label");
    if (activeWifiLabel) {
      activeWifiLabel.textContent = "No Active Connection";
    }
  }
}

// Submits the connection password and connects to a network
export async function connectWifi() {
  if (!el.wifiPassword || !el.btnConnect || !el.btnCancel) return;
  const password = el.wifiPassword.value;
  el.btnConnect.classList.add("btn-loading");
  el.btnConnect.disabled = true;
  el.btnCancel.disabled = true;

  logMessage(`Connecting to network "${state.selectedSsid}" (${state.selectedBssid})...`);
  showToast(`Initiating connection to ${state.selectedSsid}...`);

  try {
    const res = await invoke("connect_wifi", { bssid: state.selectedBssid, password: password || null });
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
