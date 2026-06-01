// Network diagnostics, real-time speed monitoring, and sparkline graphics module
// All comments in English as per user rules

const { invoke } = window.__TAURI__.core;
import { el } from "./dom.js";
import { state } from "./state.js";

// Configures initial network speeds at application boot
export async function startNetworkSpeedMonitor() {
  try {
    const io = await invoke("get_network_io");
    state.lastRxBytes = io.rx_bytes;
    state.lastTxBytes = io.tx_bytes;
    state.lastSpeedTime = Date.now();
  } catch (e) {
    console.error("Failed to read initial network bytes:", e);
  }

  // Trigger network tick every 1 second
  setInterval(updateNetworkSpeed, 1000);
}

// Tick update measuring current traffic and painting sparkline canvas
async function updateNetworkSpeed() {
  try {
    const io = await invoke("get_network_io");
    const now = Date.now();
    const timeDiffSec = (now - state.lastSpeedTime) / 1000;
    if (timeDiffSec <= 0) return;

    // Handle network reset or bytes wrap-around
    const rxDiff = io.rx_bytes > state.lastRxBytes ? io.rx_bytes - state.lastRxBytes : 0;
    const txDiff = io.tx_bytes > state.lastTxBytes ? io.tx_bytes - state.lastTxBytes : 0;

    const downSpeed = rxDiff / timeDiffSec; // Bytes per second
    const upSpeed = txDiff / timeDiffSec; // Bytes per second

    // Update Peaks
    if (downSpeed > state.peakDownload) {
      state.peakDownload = downSpeed;
      if (el.peakDownload) el.peakDownload.textContent = formatSpeed(state.peakDownload);
    }
    if (upSpeed > state.peakUpload) {
      state.peakUpload = upSpeed;
      if (el.peakUpload) el.peakUpload.textContent = formatSpeed(state.peakUpload);
    }

    // Accumulate total usage inside this UI session
    state.sessionTotalRx += rxDiff;
    state.sessionTotalTx += txDiff;
    if (el.totalUsage) {
      el.totalUsage.textContent = formatTotalBytes(state.sessionTotalRx + state.sessionTotalTx);
    }

    // Update real-time textual speeds
    if (el.speedDownload) el.speedDownload.textContent = formatSpeed(downSpeed);
    if (el.speedUpload) el.speedUpload.textContent = formatSpeed(upSpeed);

    // Save history point
    state.speedHistory.push({ rx: downSpeed, tx: upSpeed });
    if (state.speedHistory.length > state.maxHistoryPoints) {
      state.speedHistory.shift();
    }

    // Dynamic high-end canvas painting
    drawSparkline();

    // Cache current stats for the next loop
    state.lastRxBytes = io.rx_bytes;
    state.lastTxBytes = io.tx_bytes;
    state.lastSpeedTime = now;
  } catch (e) {
    // Suppress background errors to guarantee smooth UX
  }
}

// Format speeds into human readable forms
export function formatSpeed(bytesPerSec) {
  if (bytesPerSec < 1024) {
    return `${bytesPerSec.toFixed(1)} B/s`;
  }
  const kb = bytesPerSec / 1024;
  if (kb < 1024) {
    return `${kb.toFixed(1)} KB/s`;
  }
  const mb = kb / 1024;
  return `${mb.toFixed(2)} MB/s`;
}

// Format total usage bytes in session
function formatTotalBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  const mb = kb / 1024;
  if (mb < 1024) return `${mb.toFixed(1)} MB`;
  const gb = mb / 1024;
  return `${gb.toFixed(2)} GB`;
}

// Paints dynamic, premium retina-ready speed curves with neon shadow glow
export function drawSparkline() {
  if (!el.speedCtx || !el.speedChart || state.speedHistory.length < 2) return;

  const dpr = window.devicePixelRatio || 1;
  const displayWidth = el.speedChart.clientWidth;
  const displayHeight = el.speedChart.clientHeight;

  // Scale backing canvas memory dynamically for razor sharp visual rendering on HiDPI/Retina screens
  if (el.speedChart.width !== displayWidth * dpr || el.speedChart.height !== displayHeight * dpr) {
    el.speedChart.width = displayWidth * dpr;
    el.speedChart.height = displayHeight * dpr;
  }

  el.speedCtx.resetTransform();
  el.speedCtx.scale(dpr, dpr);
  el.speedCtx.clearRect(0, 0, displayWidth, displayHeight);

  // Compute maximum peak point dynamically (ensure 10 KB/s minimum height baseline)
  let maxVal = 10240;
  state.speedHistory.forEach(pt => {
    if (pt.rx > maxVal) maxVal = pt.rx;
    if (pt.tx > maxVal) maxVal = pt.tx;
  });

  const padding = 4;
  const plotWidth = displayWidth - padding * 2;
  const plotHeight = displayHeight - padding * 2;
  const step = plotWidth / (state.maxHistoryPoints - 1);

  // Helper routine to render glowing canvas lines and matching transparent gradient fills
  function drawCurve(dataKey, strokeColor, fillColor) {
    if (!el.speedCtx) return;
    el.speedCtx.beginPath();
    
    for (let i = 0; i < state.speedHistory.length; i++) {
      const x = padding + i * step;
      const val = state.speedHistory[i][dataKey];
      const y = padding + plotHeight - (val / maxVal) * plotHeight;
      
      if (i === 0) {
        el.speedCtx.moveTo(x, y);
      } else {
        el.speedCtx.lineTo(x, y);
      }
    }

    // 1. Draw glowing neon path outline
    el.speedCtx.strokeStyle = strokeColor;
    el.speedCtx.lineWidth = 2.5; // Slightly thicker line for premium look
    el.speedCtx.lineCap = "round";
    el.speedCtx.lineJoin = "round";
    
    // Configure Neon Glow Shadow Effects
    el.speedCtx.shadowColor = strokeColor;
    el.speedCtx.shadowBlur = 10;
    el.speedCtx.shadowOffsetX = 0;
    el.speedCtx.shadowOffsetY = 0;
    
    el.speedCtx.stroke();

    // 2. Clear neon shadow attributes prior to rendering background gradients
    el.speedCtx.shadowColor = "transparent";
    el.speedCtx.shadowBlur = 0;

    // 3. Connect path down to base bounds to fill background gradient
    el.speedCtx.lineTo(padding + (state.speedHistory.length - 1) * step, padding + plotHeight);
    el.speedCtx.lineTo(padding, padding + plotHeight);
    el.speedCtx.closePath();

    const grad = el.speedCtx.createLinearGradient(0, 0, 0, displayHeight);
    grad.addColorStop(0, fillColor);
    grad.addColorStop(1, "rgba(2, 6, 23, 0)"); // clean gradient fade to background
    
    el.speedCtx.fillStyle = grad;
    el.speedCtx.fill();
  }

  // Draw Download curve (Emerald Green styling)
  drawCurve("rx", "#10b981", "rgba(16, 185, 129, 0.15)");

  // Draw Upload curve (Orange styling)
  drawCurve("tx", "#f97316", "rgba(249, 115, 22, 0.08)");
}

// Executes diagnostic tasks asynchronously in parallel
export async function updateNetworkDiagnostics() {
  if (state.isCheckingDiagnostics) return;
  state.isCheckingDiagnostics = true;

  await Promise.all([
    runQuickPings(),
    runIPTracing()
  ]);

  state.isCheckingDiagnostics = false;
}

// Runs parallel non-blocking pings to DNS servers
export async function runQuickPings() {
  try {
    const results = await invoke("ping_multiple", { targets: ["1.1.1.1", "8.8.8.8"] });
    
    const statusTextElement = document.getElementById("warp-status-text");
    const statusText = statusTextElement ? statusTextElement.textContent.toLowerCase() : "";
    const isConnecting = state.isSettingMode || statusText.includes("connecting") || statusText.includes("updating");

    results.forEach(res => {
      if (res.target === "1.1.1.1") {
        if (!el.pingCloudflare) return;
        if (res.latency !== null) {
          el.pingCloudflare.textContent = `${res.latency.toFixed(1)} ms`;
          el.pingCloudflare.className = "text-xs font-black text-emerald-400 font-mono";
          
          // Update ping graph bars if available
          updatePingVisualBar("ping-bar-cloudflare", res.latency, "#10b981");
        } else if (isConnecting) {
          el.pingCloudflare.textContent = "Connecting...";
          el.pingCloudflare.className = "text-xs font-semibold text-amber-400 font-mono";
        } else {
          el.pingCloudflare.textContent = "Offline";
          el.pingCloudflare.className = "text-xs font-black text-red-500 font-mono";
        }
      } else if (res.target === "8.8.8.8") {
        if (!el.pingGoogle) return;
        if (res.latency !== null) {
          el.pingGoogle.textContent = `${res.latency.toFixed(1)} ms`;
          el.pingGoogle.className = "text-xs font-black text-blue-400 font-mono";
          
          updatePingVisualBar("ping-bar-google", res.latency, "#3b82f6");
        } else if (isConnecting) {
          el.pingGoogle.textContent = "Connecting...";
          el.pingGoogle.className = "text-xs font-semibold text-amber-400 font-mono";
        } else {
          el.pingGoogle.textContent = "Offline";
          el.pingGoogle.className = "text-xs font-black text-red-500 font-mono";
        }
      }
    });
  } catch (e) {
    console.error("Failed to run quick pings:", e);
    const statusTextElement = document.getElementById("warp-status-text");
    const statusText = statusTextElement ? statusTextElement.textContent.toLowerCase() : "";
    const isConnecting = state.isSettingMode || statusText.includes("connecting") || statusText.includes("updating");
    
    if (el.pingCloudflare) {
      el.pingCloudflare.textContent = isConnecting ? "Connecting..." : "Offline";
      el.pingCloudflare.className = isConnecting ? "text-xs font-semibold text-amber-400 font-mono" : "text-xs font-black text-red-500 font-mono";
    }
    if (el.pingGoogle) {
      el.pingGoogle.textContent = isConnecting ? "Connecting..." : "Offline";
      el.pingGoogle.className = isConnecting ? "text-xs font-semibold text-amber-400 font-mono" : "text-xs font-black text-red-500 font-mono";
    }
  }
}

// Adjusts the visual width of a ping latency bar widget
function updatePingVisualBar(barId, latency, colorHex) {
  const bar = document.getElementById(barId);
  if (!bar) return;
  // Let 150ms be 100% of the bar width
  const percent = Math.min(100, Math.max(8, (latency / 150) * 100));
  bar.style.width = `${percent}%`;
  bar.style.backgroundColor = colorHex;
}

// Query geolocation coordinates and determine VPN / WARP state
export async function runIPTracing() {
  try {
    const res = await invoke("trace_ip");
    const info = JSON.parse(res);

    if (info.status === "fail") {
      throw new Error(info.message || "Failed to query GeoIP API");
    }

    if (el.traceIpAddress) el.traceIpAddress.textContent = info.query || "N/A";
    if (el.traceIsp) el.traceIsp.textContent = info.isp || "N/A";
    if (el.traceLocation) {
      el.traceLocation.textContent = `${info.city || "N/A"}, ${info.countryCode || info.country || "N/A"}`;
    }
    if (el.traceCoords) {
      el.traceCoords.textContent = `Lat ${info.lat?.toFixed(3) || "N/A"}, Lon ${info.lon?.toFixed(3) || "N/A"}`;
    }

    // Detect if IP is routed through Cloudflare to flag WARP state
    const isWarp = (info.isp || "").toLowerCase().includes("cloudflare") || 
                   (info.org || "").toLowerCase().includes("cloudflare") || 
                   (info.as || "").toLowerCase().includes("cloudflare");

    if (el.traceWarpBadge) {
      if (isWarp) {
        el.traceWarpBadge.textContent = "WARP ACTIVE";
        el.traceWarpBadge.className = "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider transition-all duration-300 shadow-[0_0_8px_rgba(16,185,129,0.15)] animate-pulse";
      } else {
        el.traceWarpBadge.textContent = "DIRECT";
        el.traceWarpBadge.className = "bg-red-500/10 text-red-400 border border-red-500/20 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider transition-all duration-300";
      }
    }
  } catch (e) {
    console.error("IP trace error:", e);
    
    const statusTextElement = document.getElementById("warp-status-text");
    const statusText = statusTextElement ? statusTextElement.textContent.toLowerCase() : "";
    const isConnecting = state.isSettingMode || statusText.includes("connecting") || statusText.includes("updating");
    
    if (isConnecting) {
      if (el.traceIpAddress) el.traceIpAddress.textContent = "Connecting...";
      if (el.traceIsp) el.traceIsp.textContent = "Updating routing table...";
      if (el.traceLocation) el.traceLocation.textContent = "Locating...";
      if (el.traceCoords) el.traceCoords.textContent = "Lat --, Lon --";
      
      if (el.traceWarpBadge) {
        el.traceWarpBadge.textContent = "CONNECTING";
        el.traceWarpBadge.className = "bg-amber-500/10 text-amber-400 border border-amber-500/20 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider transition-all duration-300";
      }
    } else {
      if (el.traceIpAddress) el.traceIpAddress.textContent = "Offline/Fail";
      if (el.traceIsp) el.traceIsp.textContent = "Connection offline";
      if (el.traceLocation) el.traceLocation.textContent = "Unknown location";
      if (el.traceCoords) el.traceCoords.textContent = "N/A";
      
      if (el.traceWarpBadge) {
        el.traceWarpBadge.textContent = "OFFLINE";
        el.traceWarpBadge.className = "bg-slate-800 text-slate-500 border border-slate-700 px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider";
      }
    }
  }
}
