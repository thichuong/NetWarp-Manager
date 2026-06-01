// WiWarp Entry Point Module
// Orchestrates state, dom references, UI, Wi-Fi, and Cloudflare WARP services
// All comments in English as per user rules

import { loadComponents } from "./js/loader.js";
import { initDOM, el } from "./js/dom.js";
import { scanWifi, connectWifi } from "./js/wifi.js";
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
  // Widget Wifi Click opens the Wi-Fi Networks Modal
  if (el.wifiWidget) {
    el.wifiWidget.addEventListener("click", openWifiListModal);
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
}
