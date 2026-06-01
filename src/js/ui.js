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
export function openPasswordModal(bssid, ssid, band, frequency) {
  if (!el.passwordModal || !el.modalSsid || !el.wifiPassword) return;
  state.selectedBssid = bssid;
  state.selectedSsid = ssid;
  el.modalSsid.innerHTML = `${ssid} <span class="text-[10px] text-emerald-400 block mt-1 font-mono">BSSID: ${bssid} | Band: ${band} (${frequency})</span>`;
  el.wifiPassword.value = "";
  
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
