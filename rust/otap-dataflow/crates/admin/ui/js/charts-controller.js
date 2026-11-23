// Time-series helpers for channel/edge activity views.
// This module only transforms metric samples and does not read or mutate DOM.
// Callers inject scoping and lookup functions to keep behavior deterministic.

export function getSeriesWindow(points, startMs, endMs) {
  const list = Array.isArray(points) ? points : [];
  return list.filter(
    (point) => point && Number.isFinite(point.ts) && point.ts >= startMs && point.ts <= endMs
  );
}

export function getPointAtTime(points, ts) {
  if (!Array.isArray(points) || !points.length) return null;
  let chosen = null;
  for (const point of points) {
    if (!point || !Number.isFinite(point.ts)) continue;
    if (point.ts <= ts) {
      chosen = point;
    } else {
      break;
    }
  }
  return chosen || points[0] || null;
}

export function getChannelPoint({
  channelSeries,
  channelId,
  ts,
  endMs,
  windowMs,
  displayTimeMs,
  getSeriesWindowFn = getSeriesWindow,
  getPointAtTimeFn = getPointAtTime,
}) {
  if (!channelSeries || !channelId) return null;
  const series = channelSeries.get(channelId);
  if (!series || !Array.isArray(series.points) || !series.points.length) return null;
  const startMs = endMs - windowMs;
  const points = getSeriesWindowFn(series.points, startMs, endMs);
  if (!points.length) return null;
  const targetTs = Number.isFinite(ts) ? ts : displayTimeMs;
  return getPointAtTimeFn(points, targetTs) || points[points.length - 1];
}

export function updateNodeSeries({
  metricSets,
  sampleSeconds,
  ts,
  dagScope = null,
  nodeSeries,
  maxWindowMs,
  shouldShowNodeRate,
  resolveScopedNodeId,
  normalizeAttributes,
  buildNodeMetricKey,
  perfStart = null,
  perfEnd = null,
}) {
  const perfMs = typeof perfStart === "function" ? perfStart() : null;
  try {
    if (!Number.isFinite(sampleSeconds) || sampleSeconds <= 0) return;
    if (!ts || typeof ts.getTime !== "function") return;
    const nowMs = ts.getTime();
    const cutoff = nowMs - maxWindowMs;
    const scopeByPipeline = dagScope?.scopeByPipeline === true;

    (metricSets || []).forEach((set) => {
      if (
        set.name === "channel.sender" ||
        set.name === "channel.receiver" ||
        set.name === "pipeline" ||
        set.name === "tokio.runtime"
      ) {
        return;
      }
      const attrs = normalizeAttributes(set.attributes || {});
      const nodeId = resolveScopedNodeId(attrs, scopeByPipeline);
      if (!nodeId) return;
      const entry = nodeSeries.get(nodeId) || { metrics: new Map() };
      (set.metrics || []).forEach((metric) => {
        if (!shouldShowNodeRate(metric)) return;
        if (!Number.isFinite(metric.value)) return;
        const rate = metric.value / sampleSeconds;
        const metricKey = buildNodeMetricKey(set.name, metric.name);
        const series = entry.metrics.get(metricKey) || { points: [] };
        series.points.push({ ts: nowMs, value: rate });
        series.points = series.points.filter((point) => point.ts >= cutoff);
        entry.metrics.set(metricKey, series);
      });
      nodeSeries.set(nodeId, entry);
    });
  } finally {
    if (typeof perfEnd === "function") {
      perfEnd("updateNodeSeries", perfMs, { sets: metricSets?.length || 0 });
    }
  }
}

function normalizeDataPointAttributes(attrs) {
  const out = {};
  for (const [key, value] of Object.entries(attrs || {})) {
    if (value == null) continue;
    if (typeof value === "object") {
      const entry = Object.entries(value)[0];
      if (entry) out[key] = String(entry[1]);
    } else {
      out[key] = String(value);
    }
  }
  return out;
}

export function summarizeChannelSenderMetrics(metrics) {
  let send = 0;
  let sendErrorFull = 0;
  let sendErrorClosed = 0;
  for (const metric of metrics || []) {
    if (!metric || typeof metric.name !== "string") continue;
    const value = Number.isFinite(metric.value) ? metric.value : 0;
    const attrs = normalizeDataPointAttributes(metric.attributes);
    switch (metric.name) {
      case "messages":
        if (attrs.outcome === "success") send += value;
        break;
      case "failures":
        if (attrs["error.type"] === "full") sendErrorFull += value;
        if (attrs["error.type"] === "closed") sendErrorClosed += value;
        break;
      case "send.count":
        send += value;
        break;
      case "send.error_full":
        sendErrorFull += value;
        break;
      case "send.error_closed":
        sendErrorClosed += value;
        break;
      default:
        break;
    }
  }
  return { send, sendErrorFull, sendErrorClosed };
}

export function summarizeChannelReceiverMetrics(metrics) {
  let recv = 0;
  for (const metric of metrics || []) {
    if (!metric || typeof metric.name !== "string") continue;
    const value = Number.isFinite(metric.value) ? metric.value : 0;
    switch (metric.name) {
      case "messages":
        recv += value;
        break;
      case "recv.count":
        recv += value;
        break;
      default:
        break;
    }
  }
  return { recv };
}

export function updateChannelSeries({
  metricSets,
  sampleSeconds,
  ts,
  dagScope = null,
  channelSeries,
  maxWindowMs,
  resolveScopedChannelId,
  normalizeAttributes,
  perfStart = null,
  perfEnd = null,
}) {
  const perfMs = typeof perfStart === "function" ? perfStart() : null;
  try {
    if (!Number.isFinite(sampleSeconds) || sampleSeconds <= 0) return;
    if (!ts || typeof ts.getTime !== "function") return;
    const scopeByPipeline = dagScope?.scopeByPipeline === true;

    const perChannel = new Map();
    const ensureChannel = (id) => {
      if (!perChannel.has(id)) {
        perChannel.set(id, {
          send: 0,
          recv: 0,
          sendErrorFull: 0,
          sendErrorClosed: 0,
        });
      }
      return perChannel.get(id);
    };

    (metricSets || []).forEach((set) => {
      if (set.name !== "channel.sender" && set.name !== "channel.receiver") {
        return;
      }
      const attrs = normalizeAttributes(set.attributes || {});
      const channelId = resolveScopedChannelId(attrs, scopeByPipeline);
      if (!channelId) return;

      const channelEntry = ensureChannel(channelId);
      if (set.name === "channel.sender") {
        const sender = summarizeChannelSenderMetrics(set.metrics || []);
        channelEntry.send += sender.send;
        channelEntry.sendErrorFull += sender.sendErrorFull;
        channelEntry.sendErrorClosed += sender.sendErrorClosed;
      } else {
        const receiver = summarizeChannelReceiverMetrics(set.metrics || []);
        channelEntry.recv += receiver.recv;
      }
    });

    const nowMs = ts.getTime();
    const cutoff = nowMs - maxWindowMs;

    perChannel.forEach((counts, channelId) => {
      const sendRate = counts.send / sampleSeconds;
      const recvRate = counts.recv / sampleSeconds;
      const sendErrorFullRate = counts.sendErrorFull / sampleSeconds;
      const sendErrorClosedRate = counts.sendErrorClosed / sampleSeconds;
      const series = channelSeries.get(channelId) || { points: [] };
      series.points.push({
        ts: nowMs,
        sendRate,
        recvRate,
        sendErrorFullRate,
        sendErrorClosedRate,
      });
      series.points = series.points.filter((point) => point.ts >= cutoff);
      channelSeries.set(channelId, series);
    });
  } finally {
    if (typeof perfEnd === "function") {
      perfEnd("updateChannelSeries", perfMs, { sets: metricSets?.length || 0 });
    }
  }
}

export function computeEdgeRates({
  edges,
  displayTimeMs,
  sampleSeconds,
  channelSeries,
  getWindowEndMs,
  getWindowMs,
  getDisplayTimeMs,
  calcRate,
  getSeriesWindowFn = getSeriesWindow,
  getPointAtTimeFn = getPointAtTime,
}) {
  const rates = new Map();
  const endMs = getWindowEndMs();
  const windowMs = getWindowMs();
  const fallbackDisplayTimeMs = getDisplayTimeMs();

  (edges || []).forEach((edge) => {
    const senderMetrics = summarizeChannelSenderMetrics(edge.data.sender?.metrics || []);
    const receiverMetrics = summarizeChannelReceiverMetrics(edge.data.receiver?.metrics || []);
    const channelId = edge.channelId || edge.data?.id || edge.id;
    const useChannelSeries = !(edge.data?.multiSender || edge.data?.multiReceiver);
    const point = useChannelSeries
      ? getChannelPoint({
          channelSeries,
          channelId,
          ts: displayTimeMs,
          endMs,
          windowMs,
          displayTimeMs: fallbackDisplayTimeMs,
          getSeriesWindowFn,
          getPointAtTimeFn,
        })
      : null;

    const fallbackSendRate = calcRate(senderMetrics.send, sampleSeconds) ?? 0;
    const fallbackRecvRate = calcRate(receiverMetrics.recv, sampleSeconds) ?? 0;
    const fallbackSendErrRate =
      calcRate(
        senderMetrics.sendErrorFull + senderMetrics.sendErrorClosed,
        sampleSeconds
      ) ?? 0;

    const sendRate = point?.sendRate ?? fallbackSendRate;
    const recvRate = point?.recvRate ?? fallbackRecvRate;
    const sendErrorRate =
      point == null
        ? fallbackSendErrRate
        : (point.sendErrorFullRate || 0) + (point.sendErrorClosedRate || 0);
    const recvErrorRate = 0;

    const errorRate = sendErrorRate + recvErrorRate;
    rates.set(edge.id, {
      sendRate,
      recvRate,
      sendErrorRate,
      recvErrorRate,
      errorRate,
      active: sendRate > 0 || recvRate > 0,
      errorActive: errorRate > 0,
    });
  });

  return rates;
}
