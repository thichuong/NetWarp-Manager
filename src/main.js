// WiWarp Entry Point Module
// Orchestrates state, dom references, UI, Wi-Fi, and Cloudflare WARP services
// All comments in English as per user rules

import { loadComponents } from "./js/loader.js";
import { initDOM, el } from "./js/dom.js";
import { scanWifi, connectWifi, startActiveWifiMonitor } from "./js/wifi.js";
import { getInitialWarpStatus, pollWarpStatus, handleWarpToggle, handleModeChange, installWarp } from "./js/warp.js";
import { startNetworkSpeedMonitor, updateNetworkDiagnostics } from "./js/diagnostics.js";
import { closePasswordModal, openWifiListModal, closeWifiListModal } from "./js/ui.js";

// DOMContentLoaded triggers global boot sequence
window.addEventListener("DOMContentLoaded", async () => {
  // 0. Load HTML templates dynamically before mapping DOM bindings
  await loadComponents();

  // 1. Map all HTML DOM Element bindings
  initDOM();

  // 2. Bind event listeners to DOM interactions
  registerEvents();
  
  // 3. Initiate first background scanning for available Wi-Fi
  scanWifi();
  startActiveWifiMonitor();
  
  // 4. Retrieve Cloudflare status and switch modes
  getInitialWarpStatus();

  // 5. Begin real-time network upload/download measuring (1s interval)
  startNetworkSpeedMonitor();

  // 6. Run deep trace check for IP coordinates and latencies
  updateNetworkDiagnostics();
  
  // 7. Establish 5-second polling tick loop for Cloudflare WARP state
  pollWarpStatus();
});

// Map element event handlers to their respective controller methods
function registerEvents() {
  // Change wifi button opens the Wi-Fi Networks Modal
  const btnChangeWifi = document.getElementById("btn-change-wifi");
  if (btnChangeWifi) {
    btnChangeWifi.addEventListener("click", (e) => {
      e.stopPropagation();
      openWifiListModal();
    });
  }

  // Close Wi-Fi Networks Modal button
  if (el.wifiListClose) {
    el.wifiListClose.addEventListener("click", closeWifiListModal);
  }

  // Scanning trigger button inside available list modal
  if (el.btnScan) {
    el.btnScan.addEventListener("click", scanWifi);
  }

  // Dismiss network connecting password modal
  if (el.btnCancel) {
    el.btnCancel.addEventListener("click", closePasswordModal);
  }

  // Connect submission triggers connectWifi client sequence
  if (el.wifiForm) {
    el.wifiForm.addEventListener("submit", (e) => {
      e.preventDefault();
      connectWifi();
    });
  }

  // Cloudflare RPM package installer script trigger
  if (el.btnInstall) {
    el.btnInstall.addEventListener("click", installWarp);
  }

  // Enable/Disable switch trigger for Cloudflare WARP VPN
  if (el.warpToggle) {
    el.warpToggle.addEventListener("change", handleWarpToggle);
  }

  // Mode Selection buttons (DoH / WARP+DoH Tunnel)
  if (el.btnModeDoh) {
    el.btnModeDoh.addEventListener("click", () => handleModeChange("doh"));
  }
  if (el.btnModeWarpDoh) {
    el.btnModeWarpDoh.addEventListener("click", () => handleModeChange("warp+doh"));
  }

  // Toggle password visibility in the modal
  if (el.btnTogglePassword && el.wifiPassword && el.svgEyeIcon) {
    el.btnTogglePassword.addEventListener("click", () => {
      const isPassword = el.wifiPassword.type === "password";
      el.wifiPassword.type = isPassword ? "text" : "password";
      
      // Update SVG Eye Icon
      if (isPassword) {
        // Eye On (showing password)
        el.svgEyeIcon.innerHTML = `
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
        `;
      } else {
        // Eye Off (hidden password)
        el.svgEyeIcon.innerHTML = `
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l18 18" />
        `;
      }
    });
  }
}
