// DOM elements references and initialization for WiWarp
// All comments in English as per user rules

export const el = {
  // Integrated Wi-Fi Widget & Toggles
  wifiWidget: null,
  wifiSignalIcon: null,
  activeWifiSSID: null,
  activeWifiNet: null,
  
  // Real-time Speed elements
  speedDownload: null,
  speedUpload: null,
  speedChart: null,
  speedCtx: null,
  peakDownload: null,
  peakUpload: null,
  totalUsage: null,

  // Available Wi-Fi List Modal (Popup Modal)
  wifiListModal: null,
  wifiListClose: null,
  btnScan: null,
  iconScan: null,
  wifiContainer: null,
  wifiCount: null,
  
  // Connection Password Modal
  passwordModal: null,
  wifiPassword: null,
  modalSsid: null,
  wifiForm: null,
  btnConnect: null,
  btnCancel: null,
  btnTogglePassword: null,
  svgEyeIcon: null,

  // Cloudflare WARP
  ledDot: null,
  ledPing: null,
  warpStatusText: null,
  warpNetworkText: null,
  warpToggle: null,
  warpLogs: null,
  btnInstall: null,
  warpModeBadgeContainer: null,
  warpModeBadge: null,
  btnModeDoh: null,
  btnModeWarpDoh: null,

  // Network Diagnostics
  pingCloudflare: null,
  pingGoogle: null,
  traceWarpBadge: null,
  traceIpAddress: null,
  traceIsp: null,
  traceLocation: null,
  traceCoords: null,

  // UI Toast notification
  toast: null,
  toastMessage: null,
  toastIcon: null
};

// Initialize references once the DOM is fully parsed
export function initDOM() {
  el.wifiWidget = document.getElementById("wifi-widget");
  el.wifiSignalIcon = document.getElementById("wifi-signal-icon");
  el.activeWifiSSID = document.getElementById("active-wifi-ssid");
  el.activeWifiNet = document.getElementById("active-wifi-net");
  
  el.speedDownload = document.getElementById("speed-download");
  el.speedUpload = document.getElementById("speed-upload");
  el.speedChart = document.getElementById("speed-chart");
  if (el.speedChart) {
    el.speedCtx = el.speedChart.getContext("2d");
  }
  el.peakDownload = document.getElementById("peak-download");
  el.peakUpload = document.getElementById("peak-upload");
  el.totalUsage = document.getElementById("total-usage");

  el.wifiListModal = document.getElementById("wifi-list-modal");
  el.wifiListClose = document.getElementById("wifi-list-close");
  el.btnScan = document.getElementById("btn-scan");
  el.iconScan = document.getElementById("icon-scan");
  el.wifiContainer = document.getElementById("wifi-container");
  el.wifiCount = document.getElementById("wifi-count");

  el.passwordModal = document.getElementById("password-modal");
  el.wifiPassword = document.getElementById("wifi-password");
  el.modalSsid = document.getElementById("modal-ssid");
  el.wifiForm = document.getElementById("wifi-form");
  el.btnConnect = document.getElementById("btn-connect");
  el.btnCancel = document.getElementById("btn-cancel");
  el.btnTogglePassword = document.getElementById("btn-toggle-password");
  el.svgEyeIcon = document.getElementById("svg-eye-icon");

  el.ledDot = document.getElementById("led-dot");
  el.ledPing = document.getElementById("led-ping");
  el.warpStatusText = document.getElementById("warp-status-text");
  el.warpNetworkText = document.getElementById("warp-network-text");
  el.warpToggle = document.getElementById("warp-toggle");
  el.warpLogs = document.getElementById("warp-logs");
  el.btnInstall = document.getElementById("btn-install");
  el.warpModeBadgeContainer = document.getElementById("warp-mode-badge-container");
  el.warpModeBadge = document.getElementById("warp-mode-badge");
  el.btnModeDoh = document.getElementById("mode-doh");
  el.btnModeWarpDoh = document.getElementById("mode-warpdoh");

  el.pingCloudflare = document.getElementById("ping-cloudflare");
  el.pingGoogle = document.getElementById("ping-google");
  el.traceWarpBadge = document.getElementById("trace-warp-badge");
  el.traceIpAddress = document.getElementById("trace-ip-address");
  el.traceIsp = document.getElementById("trace-isp");
  el.traceLocation = document.getElementById("trace-location");
  el.traceCoords = document.getElementById("trace-coords");

  el.toast = document.getElementById("toast");
  el.toastMessage = document.getElementById("toast-message");
  el.toastIcon = document.getElementById("toast-icon");
}
