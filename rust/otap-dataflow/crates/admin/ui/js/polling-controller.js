// Polling loop primitives for metrics, health probes, and runtime status.
// Enforces single in-flight requests per endpoint family and callback-driven
// state updates.
import { fetchMetricsFromCandidates } from "./metrics-api.js";

const DELTA_STATE_RETENTION_MS = 30 * 60 * 1000;
const CHANNEL_SENDER_DELTA_METRICS = new Set([
  "send.count",
  "send.error_full",
  "send.error_closed",
]);
const CHANNEL_RECEIVER_DELTA_METRICS = new Set([
  "recv.count",
  "recv.error_empty",
  "recv.error_closed",
]);

function nowMs() {
  return typeof performance !== "undefined" && typeof performance.now === "function"
    ? performance.now()
    : Date.now();
}

function normalizeAttributes(attrs) {
  const out = {};
  if (!attrs || typeof attrs !== "object") return out;
  for (const [key, value] of Object.entries(attrs)) {
    if (value == null) continue;
    if (typeof value === "object") {
      const entry = Object.entries(value)[0];
      if (entry) {
        out[key] = String(entry[1]);
      }
      continue;
    }
    out[key] = String(value);
  }
  return out;
}

function buildMetricSeriesKey(setName, attrs, metricName) {
  const attrEntries = Object.entries(normalizeAttributes(attrs || {})).sort(([a], [b]) =>
    a.localeCompare(b)
  );
  return `${String(setName || "")}::${metricName}::${JSON.stringify(attrEntries)}`;
}

function shouldDeriveClientDelta(setName, metricName, metric) {
  if (!metric || !Number.isFinite(metric.value)) return false;
  const name = String(metricName || "");
  const instrument = String(metric.instrument || "").toLowerCase();
  const temporality = String(metric.temporality || "").toLowerCase();
  if (temporality === "delta") return true;
  if (instrument === "delta_counter" || instrument.includes("delta_counter")) return true;
  if (name === "cpu.time" || name.endsWith(".delta")) return true;
  if (setName === "channel.sender" && CHANNEL_SENDER_DELTA_METRICS.has(name)) return true;
  if (setName === "channel.receiver" && CHANNEL_RECEIVER_DELTA_METRICS.has(name)) return true;
  return false;
}

function pruneDeltaState(state, nowTsMs) {
  const prevBySeries = state.clientDeltaPrevBySeries;
  if (!(prevBySeries instanceof Map)) return;
  const cutoff = nowTsMs - DELTA_STATE_RETENTION_MS;
  for (const [key, prev] of prevBySeries) {
    if (!prev || !Number.isFinite(prev.tsMs) || prev.tsMs < cutoff) {
      prevBySeries.delete(key);
    }
  }
}

export function deriveClientDeltas(metricSets, state, nowTsMs) {
  if (!(state.clientDeltaPrevBySeries instanceof Map)) {
    state.clientDeltaPrevBySeries = new Map();
  }

  const prevBySeries = state.clientDeltaPrevBySeries;
  const transformed = (metricSets || []).map((set) => {
    const setName = String(set?.name || "");
    const attrs = set?.attributes || {};
    const metrics = (set?.metrics || []).map((metric) => {
      const metricName = metric?.name;
      if (!metricName || !shouldDeriveClientDelta(setName, metricName, metric)) {
        return metric;
      }

      const key = buildMetricSeriesKey(setName, attrs, metricName);
      const prev = prevBySeries.get(key);
      const current = metric.value;
      let delta = 0;
      if (prev && Number.isFinite(prev.value)) {
        const rawDelta = current - prev.value;
        if (Number.isFinite(rawDelta) && rawDelta >= 0) {
          delta = rawDelta;
        }
      }
      prevBySeries.set(key, { value: current, tsMs: nowTsMs });
      return { ...metric, value: delta };
    });
    return { ...set, metrics };
  });

  pruneDeltaState(state, nowTsMs);
  return transformed;
}

export function scheduleNextTimer(timerId, delayMs, callback) {
  if (timerId != null) {
    window.clearTimeout(timerId);
  }
  return window.setTimeout(() => {
    void callback();
  }, delayMs);
}

export function parseSnapshotTimestamp(value) {
  if (value == null) return null;
  const ts = new Date(value);
  if (!Number.isFinite(ts.getTime())) {
    return null;
  }
  return ts;
}

async function fetchMetricsSnapshot(state, metricsUrlCandidates, signal) {
  const { data, resolvedUrl } = await fetchMetricsFromCandidates(
    metricsUrlCandidates,
    state.resolvedMetricsUrl,
    { signal }
  );
  state.resolvedMetricsUrl = resolvedUrl;
  return data;
}

export async function runMetricsPoll({
  state,
  metricsUrlCandidates,
  getLastSampleTs,
  onConnected,
  onDisconnected,
  onHideError,
  onShowError,
  onSampleAccepted,
  scheduleNext,
}) {
  if (state.fetchInFlight) return;
  state.fetchInFlight = true;
  const requestId = ++state.latestFetchRequestId;
  const controller = new AbortController();
  state.activeFetchController = controller;
  try {
    const data = await fetchMetricsSnapshot(state, metricsUrlCandidates, controller.signal);
    if (requestId < state.latestAppliedFetchRequestId) {
      return;
    }
    state.latestAppliedFetchRequestId = requestId;
    onConnected();
    onHideError();

    const ts = parseSnapshotTimestamp(data.timestamp);
    if (!ts) {
      onShowError("Received metrics snapshot with invalid timestamp; sample ignored.");
      return;
    }

    const tsMs = ts.getTime();
    const prevTs = getLastSampleTs();
    const prevTsMs = prevTs ? prevTs.getTime() : null;
    if (prevTsMs != null && tsMs <= prevTsMs) {
      return;
    }

    const sampleSeconds = prevTsMs == null ? null : (tsMs - prevTsMs) / 1000;
    const metricSets = deriveClientDeltas(data.metric_sets || [], state, tsMs);
    onSampleAccepted({
      ts,
      sampleSeconds,
      metricSets,
    });
  } catch (error) {
    if (error?.name === "AbortError") {
      return;
    }
    onDisconnected();
    onShowError(error?.message || "Failed to load metrics.");
  } finally {
    if (state.activeFetchController === controller) {
      state.activeFetchController = null;
    }
    state.fetchInFlight = false;
    scheduleNext();
  }
}

export async function probeHealthEndpoint(path, timeoutMs) {
  const controller = new AbortController();
  const started = nowMs();
  const timeoutId = window.setTimeout(() => {
    controller.abort();
  }, timeoutMs);
  try {
    const response = await fetch(path, {
      method: "GET",
      cache: "no-store",
      signal: controller.signal,
    });
    const ended = nowMs();
    return {
      state: response.ok ? "up" : "down",
      status: response.status,
      latencyMs: ended - started,
    };
  } catch (error) {
    const ended = nowMs();
    return {
      state: "unknown",
      latencyMs: ended - started,
      error:
        error?.name === "AbortError"
          ? `timeout>${timeoutMs}ms`
          : error?.message || "request failed",
    };
  } finally {
    window.clearTimeout(timeoutId);
  }
}

export async function runHealthPoll({
  state,
  healthRequestTimeoutMs,
  onProbeResult,
  scheduleNext,
}) {
  if (state.healthFetchInFlight) return;
  state.healthFetchInFlight = true;
  const checkedAt = Date.now();
  try {
    const [livezProbe, readyzProbe] = await Promise.all([
      probeHealthEndpoint("/livez", healthRequestTimeoutMs),
      probeHealthEndpoint("/readyz", healthRequestTimeoutMs),
    ]);
    livezProbe.checkedAt = checkedAt;
    readyzProbe.checkedAt = checkedAt;
    onProbeResult(livezProbe, readyzProbe);
  } finally {
    state.healthFetchInFlight = false;
    scheduleNext();
  }
}

export async function runStatusPoll({
  state,
  statusRequestTimeoutMs,
  buildStatusSnapshot,
  onSnapshotReady,
  onProbeUpdate,
  onRefreshDecorations,
  isOverlayOpen,
  renderOverlay,
  scheduleNext,
}) {
  if (state.statusFetchInFlight) return;
  state.statusFetchInFlight = true;
  const checkedAt = Date.now();
  const controller = new AbortController();
  const started = nowMs();
  const timeoutId = window.setTimeout(() => {
    controller.abort();
  }, statusRequestTimeoutMs);

  try {
    const response = await fetch("/status", {
      method: "GET",
      cache: "no-store",
      signal: controller.signal,
    });
    const ended = nowMs();
    const probe = {
      status: response.status,
      latencyMs: ended - started,
      checkedAt,
    };

    if (!response.ok) {
      state.statusLastProbe = {
        ...probe,
        error: `status endpoint returned HTTP ${response.status}`,
      };
      onProbeUpdate(state.statusLastProbe);
      onRefreshDecorations();
      if (isOverlayOpen()) {
        renderOverlay();
      }
      return;
    }

    const payload = await response.json();
    const snapshot = buildStatusSnapshot(payload);
    state.statusLastCheckedAtMs = checkedAt;
    state.statusLastProbe = probe;
    onSnapshotReady(snapshot);
    onProbeUpdate(probe);
    onRefreshDecorations();
    if (isOverlayOpen()) {
      renderOverlay();
    }
  } catch (error) {
    const ended = nowMs();
    state.statusLastProbe = {
      latencyMs: ended - started,
      checkedAt,
      error:
        error?.name === "AbortError"
          ? `timeout>${statusRequestTimeoutMs}ms`
          : error?.message || "status request failed",
    };
    onProbeUpdate(state.statusLastProbe);
    onRefreshDecorations();
    if (isOverlayOpen()) {
      renderOverlay();
    }
  } finally {
    window.clearTimeout(timeoutId);
    state.statusFetchInFlight = false;
    scheduleNext();
  }
}
