// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Live log-stream controller for the embedded admin UI.
//
// Connects to GET /api/v1/telemetry/logs/stream over WebSocket and renders
// a bounded ring buffer of log entries into the Logs panel.
//
// Server-side pause/resume: when the user pauses, a `pause` message is sent
// so the server stops forwarding events to this client.  The browser buffer is
// not affected — it retains the entries already received.

export const MAX_LOG_ROWS = 500; // browser-side bounded ring buffer
const FILTER_DEBOUNCE_MS = 400; // delay before sending setFilter to server
const RECONNECT_DELAY_MS = 3000; // wait before automatic reconnect

// Level badge colours (dark-theme safe).
export const LEVEL_CLASSES = {
  ERROR: "log-level-error",
  WARN: "log-level-warn",
  INFO: "log-level-info",
  DEBUG: "log-level-debug",
  TRACE: "log-level-trace",
};

/**
 * Map a log-level string (any case) to its CSS badge class.
 * Unknown levels fall back to "log-level-trace".
 * @param {string|undefined} levelStr
 * @returns {string}
 */
export function resolveLevelClass(levelStr) {
  const upper = (levelStr || "").toUpperCase();
  return LEVEL_CLASSES[upper] || "log-level-trace";
}

/**
 * Format an RFC 3339 timestamp string as a short local time string.
 * Returns the original string unchanged if parsing fails.
 * @param {string} rfc3339
 * @returns {string}
 */
export function formatTimestamp(rfc3339) {
  try {
    const d = new Date(rfc3339);
    if (isNaN(d.getTime())) return rfc3339;
    return d.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      fractionalSecondDigits: 3,
    });
  } catch {
    return rfc3339;
  }
}

/**
 * Append entries to a mutable ring buffer, evicting the oldest when the
 * buffer exceeds maxRows.
 * @param {Array} buffer   Mutable log-entry array (newest at the end).
 * @param {Array} entries  New entries to append.
 * @param {number} maxRows Maximum buffer size.
 */
export function appendToBuffer(buffer, entries, maxRows) {
  for (const entry of entries) {
    buffer.push(entry);
  }
  if (buffer.length > maxRows) {
    buffer.splice(0, buffer.length - maxRows);
  }
}

/**
 * Create and manage the live log-stream controller.
 *
 * @param {object} opts
 * @param {HTMLElement} opts.containerEl  Root element that contains all the
 *   log-panel DOM nodes (see index.html #logs-section).
 * @returns {{ destroy: Function }} Handle to shut the controller down.
 */
export function createLogsController({ containerEl }) {
  // -------------------------------------------------------------------------
  // DOM refs (all within containerEl so the controller is self-contained).
  // -------------------------------------------------------------------------
  const connectBtn = containerEl.querySelector("#logs-connect-btn");
  const pauseBtn = containerEl.querySelector("#logs-pause-btn");
  const clearBtn = containerEl.querySelector("#logs-clear-btn");
  const backfillBtn = containerEl.querySelector("#logs-backfill-btn");
  const filterInput = containerEl.querySelector("#logs-filter-input");
  const statusEl = containerEl.querySelector("#logs-status");
  const dropBannerEl = containerEl.querySelector("#logs-drop-banner");
  const tableBody = containerEl.querySelector("#logs-tbody");

  // -------------------------------------------------------------------------
  // Internal state
  // -------------------------------------------------------------------------
  let ws = null;
  /** @type {boolean} */
  let connected = false;
  /** @type {boolean} */
  let paused = false;
  /** last sequence number the server confirmed (used for backfill) */
  let serverCursor = 0;
  /**
   * When the server reports a lag gap, this holds the cursor value from just
   * before the gap so that Resync can fetch the dropped region rather than
   * starting from the (already-advanced) serverCursor.  Cleared after use.
   * @type {number|null}
   */
  let lagBeforeSeq = null;
  /** Browser-side ring buffer: newest at the end, bounded to MAX_LOG_ROWS. */
  const logBuffer = [];
  let filterDebounceTimer = null;
  let reconnectTimer = null;

  // -------------------------------------------------------------------------
  // Public API wired up at the end of this function.
  // -------------------------------------------------------------------------

  function buildWsUrl() {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    return `${proto}//${location.host}/api/v1/telemetry/logs/stream`;
  }

  // ---- WebSocket lifecycle -------------------------------------------------

  function connect() {
    if (ws) return;
    const url = buildWsUrl();
    try {
      ws = new WebSocket(url);
    } catch (e) {
      setStatus(`Connect failed: ${e.message}`, true);
      return;
    }

    ws.addEventListener("open", onOpen);
    ws.addEventListener("message", onMessage);
    ws.addEventListener("close", onClose);
    ws.addEventListener("error", onError);

    setStatus("Connecting…", false);
    updateButtons();
  }

  function disconnect() {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
    if (!ws) return;
    // Remove auto-reconnect listeners before closing.
    ws.removeEventListener("close", onClose);
    ws.close(1000, "user disconnect");
    ws = null;
    connected = false;
    paused = false;
    setStatus("Disconnected", false);
    updateButtons();
  }

  function onOpen() {
    connected = true;
    setStatus("Connected — subscribing…", false);
    // On the very first connect serverCursor is 0, so we send `after: null`
    // to get the latest retained snapshot.  On reconnect serverCursor holds
    // the last seq we received, so we request only newer events to avoid
    // re-appending entries that are already in the browser buffer.
    sendJson({
      type: "subscribe",
      after: serverCursor > 0 ? serverCursor : null,
      limit: 100,
      searchQuery: filterInput.value.trim() || null,
      minimumTimestamp: null,
      paused: false,
    });
    updateButtons();
  }

  function onClose(evt) {
    ws = null;
    connected = false;
    paused = false;
    const reason = evt.wasClean ? "server closed" : "lost connection";
    setStatus(`Disconnected (${reason}) — reconnecting in ${RECONNECT_DELAY_MS / 1000}s…`, true);
    updateButtons();
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null;
      if (!ws) connect();
    }, RECONNECT_DELAY_MS);
  }

  function onError() {
    setStatus("WebSocket error", true);
  }

  // ---- Message handling ----------------------------------------------------

  function onMessage(evt) {
    let msg;
    try {
      msg = JSON.parse(evt.data);
    } catch {
      return;
    }
    switch (msg.type) {
      case "snapshot":
        hideDropBanner();
        appendEntries(msg.logs || []);
        serverCursor = msg.next_seq ?? serverCursor;
        setStatus(`Live — ${logBuffer.length} entries`, false);
        break;
      case "log":
        appendEntries([msg]);
        serverCursor = msg.seq;
        if (!paused) {
          setStatus(`Live — ${logBuffer.length} entries`, false);
        }
        break;
      case "state":
        paused = msg.paused ?? paused;
        serverCursor = msg.next_seq ?? serverCursor;
        if (paused) {
          setStatus(`Paused — cursor ${serverCursor}`, false);
        } else {
          setStatus(`Live — ${logBuffer.length} entries`, false);
        }
        updateButtons();
        break;
      case "error":
        // If the server reports a lag gap it includes the cursor from just
        // before the dropped region so Resync can fetch the missing events.
        if (msg.lag_before_seq != null) {
          lagBeforeSeq = msg.lag_before_seq;
        }
        showDropBanner(msg.message || "server error");
        break;
      case "pong":
        break;
      default:
        break;
    }
  }

  // ---- Buffer and rendering ------------------------------------------------

  /**
   * Append new entries to the ring buffer and re-render the table.
   * Keeps the buffer bounded to MAX_LOG_ROWS by evicting the oldest entries.
   * @param {Array<object>} entries
   */
  function appendEntries(entries) {
    appendToBuffer(logBuffer, entries, MAX_LOG_ROWS);
    renderTable();
  }

  function renderTable() {
    // Replace entire tbody content.  For up to 500 rows this is fast enough.
    const fragment = document.createDocumentFragment();
    for (const entry of logBuffer) {
      const tr = buildRow(entry);
      fragment.appendChild(tr);
    }
    tableBody.textContent = "";
    tableBody.appendChild(fragment);
    // Auto-scroll to the bottom unless the user has scrolled up.
    const viewport = tableBody.closest(".logs-viewport");
    if (viewport) {
      const atBottom =
        viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < 60;
      if (atBottom) {
        viewport.scrollTop = viewport.scrollHeight;
      }
    }
  }

  function buildRow(entry) {
    const tr = document.createElement("tr");
    tr.className = "log-row";

    // Timestamp (short form for readability).
    const ts = entry.timestamp ? formatTimestamp(entry.timestamp) : "—";
    td(tr, ts, "log-ts");

    // Level badge.
    const levelCell = document.createElement("td");
    levelCell.className = "log-level-cell";
    const levelSpan = document.createElement("span");
    const levelUpper = (entry.level || "").toUpperCase();
    levelSpan.className = `log-level-badge ${resolveLevelClass(entry.level)}`;
    levelSpan.textContent = levelUpper || "—";
    levelCell.appendChild(levelSpan);
    tr.appendChild(levelCell);

    // Target.
    td(tr, entry.target || "—", "log-target");

    // Rendered message (main content).
    const msgCell = document.createElement("td");
    msgCell.className = "log-rendered";
    msgCell.textContent = entry.rendered || entry.event_name || "—";
    tr.appendChild(msgCell);

    return tr;
  }

  function td(tr, text, className) {
    const cell = document.createElement("td");
    cell.className = className;
    cell.textContent = text;
    tr.appendChild(cell);
  }

  // ---- Server-side controls -----------------------------------------------

  function sendJson(obj) {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(obj));
    }
  }

  function sendPause() {
    paused = true;
    sendJson({ type: "pause" });
    setStatus(`Paused — cursor ${serverCursor}`, false);
    updateButtons();
  }

  function sendResume() {
    paused = false;
    sendJson({ type: "resume" });
    setStatus(`Live — ${logBuffer.length} entries`, false);
    updateButtons();
  }

  function sendFilter() {
    const q = filterInput.value.trim() || null;
    sendJson({ type: "setFilter", searchQuery: q, minimumTimestamp: null });
  }

  function requestBackfill() {
    if (lagBeforeSeq !== null) {
      // A lag gap was reported: fetch from just before the gap so the dropped
      // events are recovered.  Clear the browser buffer first so that events
      // already received after the gap do not create duplicates when the
      // snapshot arrives.
      const after = lagBeforeSeq;
      lagBeforeSeq = null;
      logBuffer.length = 0;
      sendJson({ type: "backfill", after, limit: 100 });
    } else {
      sendJson({ type: "backfill", after: serverCursor, limit: 100 });
    }
    hideDropBanner();
  }

  // ---- Drop / lag notifications --------------------------------------------

  function showDropBanner(msg) {
    dropBannerEl.textContent = `⚠ ${msg}`;
    dropBannerEl.classList.remove("hidden");
  }

  function hideDropBanner() {
    dropBannerEl.classList.add("hidden");
  }

  // ---- Status and button state ---------------------------------------------

  function setStatus(text, isError) {
    statusEl.textContent = text;
    statusEl.classList.toggle("text-red-400", isError);
    statusEl.classList.toggle("text-slate-400", !isError);
  }

  function updateButtons() {
    const isConnected = !!ws && connected;
    connectBtn.textContent = isConnected ? "Disconnect" : "Connect";
    pauseBtn.disabled = !isConnected;
    pauseBtn.textContent = paused ? "Resume" : "Pause";
    backfillBtn.disabled = !isConnected;
  }

  // ---- Event wiring --------------------------------------------------------

  connectBtn.addEventListener("click", () => {
    if (ws && connected) {
      disconnect();
    } else {
      connect();
    }
  });

  pauseBtn.addEventListener("click", () => {
    if (!connected) return;
    if (paused) {
      sendResume();
    } else {
      sendPause();
    }
  });

  clearBtn.addEventListener("click", () => {
    logBuffer.length = 0;
    tableBody.textContent = "";
    setStatus(
      connected ? (paused ? `Paused — cursor ${serverCursor}` : `Live — 0 entries`) : "Disconnected",
      false
    );
  });

  backfillBtn.addEventListener("click", () => {
    if (connected) requestBackfill();
  });

  filterInput.addEventListener("input", () => {
    clearTimeout(filterDebounceTimer);
    filterDebounceTimer = setTimeout(() => {
      if (connected) sendFilter();
    }, FILTER_DEBOUNCE_MS);
  });

  // ---- Initial button state ------------------------------------------------
  updateButtons();
  setStatus("Disconnected", false);

  // ---- Exported teardown ---------------------------------------------------
  function destroy() {
    disconnect();
    clearTimeout(filterDebounceTimer);
  }

  return { destroy };
}
