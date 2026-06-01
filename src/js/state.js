// Global state management for WiWarp application
// All comments in English as per user rules

export const state = {
  currentWarpMode: "",
  isSettingMode: false,
  selectedBssid: "",
  selectedSsid: "",
  isScanning: false,
  isTogglingWarp: false,
  
  // Traffic speed states
  lastRxBytes: 0,
  lastTxBytes: 0,
  lastSpeedTime: Date.now(),
  speedHistory: [],
  maxHistoryPoints: 40, // Increased for a wider and more detailed chart in 1600x900
  
  // Statistical speed states
  peakDownload: 0,
  peakUpload: 0,
  sessionTotalRx: 0,
  sessionTotalTx: 0,
  
  // Diagnostic flags
  isCheckingDiagnostics: false,
  isFetchingWarpStatus: false
};
