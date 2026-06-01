// UI control helper module (Toasts, Modals, System Logging)
// All comments in English as per user rules

import { el } from "./dom.js";
import { state } from "./state.js";

// Logs a message into the scrolling system logs terminal panel
export function logMessage(message) {
  if (!el.warpLogs) return;
  const time = new Date().toLocaleTimeString();
  const logLine = `[${time}] ${message}\n`;
  el.warpLogs.textContent = logLine + el.warpLogs.textContent;
}

// Displays modern Toast popup notification
export function showToast(message, isError = false) {
  if (!el.toast || !el.toastMessage || !el.toastIcon) return;
  el.toastMessage.textContent = message;
  
  if (isError) {
    el.toast.classList.remove("border-emerald-700/50");
    el.toast.classList.add("border-red-700/50");
    el.toastIcon.innerHTML = `
      <svg class="w-4 h-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
      </svg>
    `;
  } else {
    el.toast.classList.remove("border-red-700/50");
    el.toast.classList.add("border-emerald-700/50");
    el.toastIcon.innerHTML = `
      <svg class="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"/>
      </svg>
    `;
  }

  // Slide up and reveal toast
  el.toast.classList.remove("translate-y-10", "opacity-0", "pointer-events-none");
  el.toast.classList.add("translate-y-0", "opacity-100");

  // Automatically fade out after 4 seconds
  setTimeout(() => {
    el.toast.classList.add("translate-y-10", "opacity-0", "pointer-events-none");
    el.toast.classList.remove("translate-y-0", "opacity-100");
  }, 4000);
}

// Opens the sliding password entry dialog modal
export async function openPasswordModal(bssid, ssid, band, frequency, isSaved = false) {
  if (!el.passwordModal || !el.modalSsid || !el.wifiPassword) return;
  state.selectedBssid = bssid;
  state.selectedSsid = ssid;
  
  const savedLabel = isSaved ? ' <span class="inline-flex items-center ml-2 px-1.5 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-300 border border-emerald-500/20 font-mono animate-pulse">SAVED</span>' : '';
  el.modalSsid.innerHTML = `${ssid}${savedLabel} <span class="text-[10px] text-emerald-400 block mt-1 font-mono">BSSID: ${bssid} | Band: ${band} (${frequency})</span>`;
  
  // Reset password field values and type back to default password
  el.wifiPassword.value = "";
  el.wifiPassword.type = "password";
  if (el.wifiLockBssid) {
    el.wifiLockBssid.checked = true; // Default to checked for new profiles
  }
  if (el.svgEyeIcon) {
    el.svgEyeIcon.innerHTML = `
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l18 18" />
    `;
  }

  // If the profile is saved, fetch the stored password and locked BSSID state
  if (isSaved) {
    try {
      const { invoke } = window.__TAURI__.core;
      const savedPwd = await invoke("get_wifi_password", { ssid });
      if (savedPwd) {
        el.wifiPassword.value = savedPwd;
        logMessage(`Autofilled saved password for "${ssid}".`);
      }
      
      const lockedBssid = await invoke("get_wifi_locked_bssid", { ssid });
      if (el.wifiLockBssid) {
        // If lockedBssid is empty, the connection is free to roam.
        // If lockedBssid matches the current BSSID (case insensitive), we set checked to true.
        if (lockedBssid && lockedBssid.trim().toLowerCase() === bssid.trim().toLowerCase()) {
          el.wifiLockBssid.checked = true;
          logMessage(`BSSID locking is currently active for "${ssid}" targeting BSSID ${bssid}.`);
        } else {
          el.wifiLockBssid.checked = false;
        }
      }
    } catch (err) {
      logMessage(`Failed to read saved profile metadata for "${ssid}": ${err}`);
    }
  }
  
  // Reveal password modal with transition
  el.passwordModal.classList.remove("opacity-0", "pointer-events-none");
  el.passwordModal.classList.add("opacity-100");
  el.passwordModal.firstElementChild.classList.remove("scale-90");
  el.passwordModal.firstElementChild.classList.add("scale-100");
  el.wifiPassword.focus();
}

// Closes the Wi-Fi password modal
export function closePasswordModal() {
  if (!el.passwordModal) return;
  el.passwordModal.classList.add("opacity-0", "pointer-events-none");
  el.passwordModal.classList.remove("opacity-100");
  el.passwordModal.firstElementChild.classList.add("scale-90");
  el.passwordModal.firstElementChild.classList.remove("scale-100");
}

// Opens the Available Wi-Fi networks list modal
export function openWifiListModal() {
  if (!el.wifiListModal) return;
  el.wifiListModal.classList.remove("opacity-0", "pointer-events-none");
  el.wifiListModal.classList.add("opacity-100");
  el.wifiListModal.firstElementChild.classList.remove("scale-95", "-translate-y-4");
  el.wifiListModal.firstElementChild.classList.add("scale-100", "translate-y-0");
}

// Closes the Available Wi-Fi networks list modal
export function closeWifiListModal() {
  if (!el.wifiListModal) return;
  el.wifiListModal.classList.add("opacity-0", "pointer-events-none");
  el.wifiListModal.classList.remove("opacity-100");
  el.wifiListModal.firstElementChild.classList.add("scale-95", "-translate-y-4");
  el.wifiListModal.firstElementChild.classList.remove("scale-100", "translate-y-0");
}
