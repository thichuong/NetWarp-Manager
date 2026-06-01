// Cloudflare WARP client controller module
// All comments in English as per user rules

const { invoke } = window.__TAURI__.core;
import { el } from "./dom.js";
import { state } from "./state.js";
import { showToast, logMessage } from "./ui.js";
import { updateNetworkDiagnostics } from "./diagnostics.js";

// Queries and synchronizes current WARP operating mode from backend
export async function syncWarpMode() {
  try {
    const mode = await invoke("get_warp_mode");
    state.currentWarpMode = mode;
    updateWarpModeUI(mode);
    disableModeButtons(false);
  } catch (err) {
    // Silently ignore configuration errors during initialization
  }
}

// Queries warp initial state at launch
export async function getInitialWarpStatus() {
  await getWarpStatus();
  await syncWarpMode();
}

// Periodically updates warp status (called every 5s)
export async function pollWarpStatus() {
  if (state.isFetchingWarpStatus) return;
  state.isFetchingWarpStatus = true;

  try {
    await getWarpStatus();
    // Update network diagnostics metrics alongside the status loop
    await updateNetworkDiagnostics();
  } finally {
    state.isFetchingWarpStatus = false;
    setTimeout(pollWarpStatus, 5000);
  }
}

// Queries WARP daemon connection status and updates dashboard LEDs/badges
export async function getWarpStatus() {
  try {
    const status = await invoke("get_warp_status");
    updateWarpUI(status);
    
    if (status === "Not Installed") {
      disableModeButtons(true);
      if (el.warpModeBadgeContainer) el.warpModeBadgeContainer.classList.add("hidden");
    } else {
      disableModeButtons(false);
    }
  } catch (err) {
    // Fallback status if daemon is sleeping
    updateWarpUI("Disconnected");
    disableModeButtons(true);
    if (el.warpModeBadgeContainer) el.warpModeBadgeContainer.classList.add("hidden");
  }
}

// Renders the overall WARP UI elements depending on current status value
export function updateWarpUI(status) {
  if (!el.ledDot || !el.ledPing || !el.warpStatusText || !el.warpToggle || !el.btnInstall) return;

  if (status === "Connected") {
    el.ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-emerald-500 transition-all duration-300";
    el.ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-500 opacity-20";
    el.warpStatusText.textContent = "Connected";
    el.warpStatusText.className = "text-xs font-bold mt-1 text-emerald-400 uppercase tracking-widest";
    if (el.warpNetworkText) {
      el.warpNetworkText.textContent = "Your network traffic is securely encrypted.";
    }
    
    el.warpToggle.disabled = false;
    if (!state.isTogglingWarp) {
      el.warpToggle.checked = true;
    }
  } else if (status === "Connecting") {
    el.ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-amber-500 transition-all duration-300";
    el.ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-500 opacity-20";
    el.warpStatusText.textContent = "Connecting...";
    el.warpStatusText.className = "text-xs font-bold mt-1 text-amber-400 uppercase tracking-widest";
    if (el.warpNetworkText) {
      el.warpNetworkText.textContent = "Establishing secure connection...";
    }
    
    el.warpToggle.disabled = true;
  } else if (status === "Not Installed") {
    el.ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-slate-600 shadow-none transition-all duration-300";
    el.ledPing.className = "hidden";
    el.warpStatusText.textContent = "Not Installed";
    el.warpStatusText.className = "text-xs font-bold mt-1 text-slate-400 uppercase tracking-widest";
    if (el.warpNetworkText) {
      el.warpNetworkText.textContent = "WARP client daemon not found on this system.";
    }
    
    el.warpToggle.disabled = true;
    el.warpToggle.checked = false;
    el.btnInstall.classList.add("animate-pulse");
  } else {
    // Disconnected state
    el.ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-red-500 transition-all duration-300";
    el.ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-25";
    el.warpStatusText.textContent = "Disconnected";
    el.warpStatusText.className = "text-xs font-bold mt-1 text-red-400 uppercase tracking-widest";
    if (el.warpNetworkText) {
      el.warpNetworkText.textContent = "Your network connection is direct & unprotected.";
    }
    
    el.warpToggle.disabled = false;
    if (!state.isTogglingWarp) {
      el.warpToggle.checked = false;
    }
    el.btnInstall.classList.remove("animate-pulse");
  }
}

// Triggers Cloudflare WARP activation / deactivation command
export async function handleWarpToggle() {
  if (state.isTogglingWarp || !el.warpToggle) return;
  state.isTogglingWarp = true;
  
  const wantConnect = el.warpToggle.checked;
  el.warpToggle.disabled = true;
  
  logMessage(`Triggering Cloudflare WARP ${wantConnect ? "CONNECTION" : "DISCONNECTION"}...`);
  showToast(wantConnect ? "Connecting to WARP network..." : "Disconnecting from WARP...");
  
  try {
    const res = await invoke("warp_toggle", { connect: wantConnect });
    showToast(wantConnect ? "WARP connection request sent!" : "WARP disconnected.");
    logMessage(`WARP toggle feedback: ${res}`);
  } catch (err) {
    showToast(`WARP control error: ${err}`, true);
    logMessage(`WARP error: ${err}`);
    el.warpToggle.checked = !wantConnect; // roll back GUI
  } finally {
    state.isTogglingWarp = false;
    await getWarpStatus();
    await updateNetworkDiagnostics();
  }
}

// Executes warp installation via PKEXEC on Fedora
export async function installWarp() {
  if (!el.btnInstall) return;
  el.btnInstall.classList.add("btn-loading");
  el.btnInstall.disabled = true;
  
  logMessage("Starting Cloudflare WARP installer script...");
  logMessage("Step 1: Refreshing DNF download links for WARP...");
  showToast("Downloading Cloudflare WARP RPM package...");

  try {
    const result = await invoke("install_warp");
    showToast("Cloudflare WARP installed successfully!");
    logMessage(`Install output: ${result}`);
    await getWarpStatus();
    await syncWarpMode();
  } catch (err) {
    showToast(`Installation script failed: ${err}`, true);
    logMessage(`Installation error: ${err}`);
  } finally {
    el.btnInstall.classList.remove("btn-loading");
    el.btnInstall.disabled = false;
  }
}

// Switches WARP mode (DoH DNS / WARP+DoH Tunnel)
export async function handleModeChange(mode) {
  if (state.isSettingMode) return;
  state.isSettingMode = true;

  state.currentWarpMode = mode;
  updateWarpModeUI(mode);
  disableModeButtons(true);

  logMessage(`Requesting WARP mode switch to: ${mode.toUpperCase()}...`);
  showToast(`Switching to ${mode.toUpperCase()} mode...`);

  // Instantly apply amber loading state to indicators
  if (el.warpStatusText) {
    el.warpStatusText.textContent = "Connecting...";
    el.warpStatusText.className = "text-xs font-bold mt-1 text-amber-400 uppercase tracking-widest";
  }
  if (el.ledDot) {
    el.ledDot.className = "relative inline-flex rounded-full h-4 w-4 bg-amber-500 transition-all duration-300";
  }
  if (el.ledPing) {
    el.ledPing.className = "animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-500 opacity-20";
  }
  if (el.traceWarpBadge) {
    el.traceWarpBadge.textContent = "CONNECTING";
    el.traceWarpBadge.className = "bg-amber-500/10 text-amber-400 border border-amber-500/20 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider transition-all duration-300 animate-pulse";
  }

  try {
    const res = await invoke("set_warp_mode", { mode });
    showToast(`WARP mode updated to ${mode.toUpperCase()}!`);
    logMessage(`Mode update feedback: ${res}`);
  } catch (err) {
    showToast(`Mode switch error: ${err}`, true);
    logMessage(`WARP mode error: ${err}`);
    await syncWarpMode(); // revert UI to backend state
  } finally {
    state.isSettingMode = false;
    disableModeButtons(false);
    
    // Give routing table adjustments 1.5s to apply before updating diagnostics
    setTimeout(async () => {
      await updateNetworkDiagnostics();
      await getWarpStatus();
    }, 1500);
  }
}

// Disables or enables mode buttons
export function disableModeButtons(disabled) {
  if (el.btnModeDoh) el.btnModeDoh.disabled = disabled;
  if (el.btnModeWarpDoh) el.btnModeWarpDoh.disabled = disabled;
}

// Updates UI state of WARP mode selection buttons
export function updateWarpModeUI(mode) {
  const inactiveClass = "text-slate-400 hover:text-slate-200 border-transparent hover:bg-slate-900/40";
  const activeDoh = "bg-orange-500 text-white border-transparent shadow-sm scale-[1.01]";
  const activeDual = "bg-emerald-600 text-white border-transparent shadow-sm scale-[1.01]";

  [
    { btn: el.btnModeDoh, key: "doh", activeClass: activeDoh },
    { btn: el.btnModeWarpDoh, key: "warp+doh", activeClass: activeDual }
  ].forEach(({ btn, key, activeClass }) => {
    if (!btn) return;
    if (key === mode) {
      btn.className = `py-2 px-1 rounded-xl text-[9px] font-bold tracking-wide transition-all duration-200 focus:outline-none flex flex-col items-center justify-center gap-1.5 border ${activeClass}`;
    } else {
      btn.className = `py-2 px-1 rounded-xl text-[9px] font-bold tracking-wide transition-all duration-200 focus:outline-none flex flex-col items-center justify-center gap-1.5 border ${inactiveClass}`;
    }
  });

  if (el.warpModeBadge && el.warpModeBadgeContainer) {
    if (mode === "doh") {
      el.warpModeBadge.innerHTML = `
        <svg class="w-3 h-3 animate-pulse text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M13 10V3L4 14h7v7l9-11h-7z"/>
        </svg>
        <span>Mode: DoH DNS Only</span>
      `;
      el.warpModeBadge.className = "inline-flex items-center gap-1.5 px-3 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-orange-500/10 text-orange-400 border border-orange-500/20";
      el.warpModeBadgeContainer.classList.remove("hidden");
    } else if (mode === "warp+doh") {
      el.warpModeBadge.innerHTML = `
        <svg class="w-3 h-3 animate-pulse text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
        </svg>
        <span>Mode: WARP + DoH (Max)</span>
      `;
      el.warpModeBadge.className = "inline-flex items-center gap-1.5 px-3 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-emerald-500/10 text-emerald-400 border border-emerald-500/20";
      el.warpModeBadgeContainer.classList.remove("hidden");
    } else {
      el.warpModeBadge.innerHTML = `
        <svg class="w-3 h-3 animate-pulse text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
        <span>Mode: ${mode.toUpperCase()}</span>
      `;
      el.warpModeBadge.className = "inline-flex items-center gap-1.5 px-3 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-slate-900 text-slate-300 border border-slate-800";
      el.warpModeBadgeContainer.classList.remove("hidden");
    }
  }
}
