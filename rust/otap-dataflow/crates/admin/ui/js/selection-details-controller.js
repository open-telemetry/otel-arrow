function findMetric(metrics, name) {
  if (!metrics) return null;
  return metrics.find((metric) => metric.name === name) || null;
}

function buildChannelLegendRows(point) {
  if (!point) return [];
  return [
    { label: "send.count", value: point.sendRate, color: "rgba(34,197,94,0.9)" },
    { label: "recv.count", value: point.recvRate, color: "rgba(56,189,248,0.9)" },
    { label: "send.error_full", value: point.sendErrorFullRate, color: "rgba(248,113,113,0.95)" },
    { label: "send.error_closed", value: point.sendErrorClosedRate, color: "rgba(239,68,68,0.9)" },
    { label: "recv.error_empty", value: point.recvErrorEmptyRate, color: "rgba(244,63,94,0.9)" },
    { label: "recv.error_closed", value: point.recvErrorClosedRate, color: "rgba(225,29,72,0.9)" },
  ];
}

function buildCommonChannelAttributes(senderAttrs, receiverAttrs) {
  const out = {};
  for (const [key, value] of Object.entries(senderAttrs || {})) {
    if (!key.startsWith("channel.")) continue;
    if (receiverAttrs && receiverAttrs[key] === value) {
      out[key] = value;
    }
  }
  return out;
}

export function createSelectionDetailsController({
  selectionTitle,
  edgeDetailMeta,
  edgeDetailBody,
  channelSeries,
  pipelineHoverPlugin,
  getChartThemeColors,
  getWindowEndMs,
  getWindowMs,
  getSeriesWindow,
  getPointAtTime,
  getDisplayTimeMs,
  getChannelPoint,
  getFreezeActive,
  formatRate,
  formatRateWithUnit,
  formatWindowLabel,
  formatValueWithUnit,
  renderAttributes,
  renderMetricTable,
  renderNodeMetricTable,
  metricMap,
  calcRate,
  buildNodeSummary,
  escapeHtml,
  setGlobalHover,
  destroyNodeCharts,
  initNodeRateCharts,
  getLastSampleSeconds,
  getLastEdgeRates,
  getGlobalHoverTs,
  getChannelChart,
  setChannelChart,
  getChannelChartId,
  setChannelChartId,
}) {
  function getChart() {
    return getChannelChart ? getChannelChart() : null;
  }

  function getChartId() {
    return getChannelChartId ? getChannelChartId() : null;
  }

  function renderChannelLegend(channelId, pointOverride) {
    const legend = document.getElementById("channelChartLegend");
    if (!legend) return;
    if (!channelId) {
      legend.classList.add("hidden");
      legend.innerHTML = "";
      return;
    }

    const series = channelSeries.get(channelId);
    const endMs = getWindowEndMs();
    const startMs = endMs - getWindowMs();
    const points = series ? getSeriesWindow(series.points, startMs, endMs) : [];
    const defaultPoint = pointOverride
      ? pointOverride
      : getFreezeActive()
        ? getPointAtTime(points, getDisplayTimeMs())
        : points[points.length - 1];
    const latest = defaultPoint;
    if (!latest) {
      legend.classList.remove("hidden");
      legend.innerHTML = '<div class="text-slate-400">No recent activity.</div>';
      return;
    }

    const rows = buildChannelLegendRows(latest);
    const formatLegendRate = (value) => {
      const rate = formatRate(value);
      return rate === "n/a" ? rate : rate.replace("/s", " msg/s");
    };

    legend.classList.remove("hidden");
    legend.innerHTML = rows
      .map((row) => {
        const safeLabel = escapeHtml(row.label);
        return `
          <div class="channel-chart-row">
            <span class="channel-chart-label">
              <span class="channel-chart-dot" style="color:${row.color}; background:${row.color};"></span>
              ${safeLabel}
            </span>
            <span class="font-mono text-slate-100">${formatLegendRate(row.value)}</span>
          </div>`;
      })
      .join("");
  }

  function clearChannelChart() {
    const chart = getChart();
    if (chart) {
      if (chart._legendHandlers) {
        const { move, leave } = chart._legendHandlers;
        chart.canvas.removeEventListener("mousemove", move);
        chart.canvas.removeEventListener("mouseleave", leave);
        chart._legendHandlers = null;
      }
      chart.destroy();
      setChannelChart(null);
      setChannelChartId(null);
    }
    renderChannelLegend(null);
  }

  function attachLegendInteraction(chart) {
    if (!chart || !chart.canvas) return;
    if (chart._legendHandlers) {
      const { move, leave } = chart._legendHandlers;
      chart.canvas.removeEventListener("mousemove", move);
      chart.canvas.removeEventListener("mouseleave", leave);
    }

    const move = (event) => {
      const points = chart.getElementsAtEventForMode(
        event,
        "index",
        { intersect: false },
        false
      );
      if (!points.length) return;
      const index = points[0].index;
      const ts = chart._tsLabels ? chart._tsLabels[index] : null;
      if (!Number.isFinite(ts)) return;
      setGlobalHover(ts);
    };

    const leave = () => {
      setGlobalHover(null);
    };

    chart.canvas.addEventListener("mousemove", move);
    chart.canvas.addEventListener("mouseleave", leave);
    chart._legendHandlers = { move, leave };
  }

  function renderChannelChart(channelId) {
    if (!channelId) {
      clearChannelChart();
      return;
    }

    const canvas = document.getElementById("channelChart");
    renderChannelLegend(channelId);
    if (!canvas || !window.Chart) return;

    const series = channelSeries.get(channelId);
    const rawPoints = series ? series.points : [];
    const endMs = getWindowEndMs();
    const startMs = endMs - getWindowMs();
    const points = getSeriesWindow(rawPoints, startMs, endMs);
    const labels = points.map((point) => new Date(point.ts).toLocaleTimeString());
    const sendData = points.map((point) => point.sendRate);
    const recvData = points.map((point) => point.recvRate);
    const sendErrorFullData = points.map((point) => point.sendErrorFullRate);
    const sendErrorClosedData = points.map((point) => point.sendErrorClosedRate);
    const recvErrorEmptyData = points.map((point) => point.recvErrorEmptyRate);
    const recvErrorClosedData = points.map((point) => point.recvErrorClosedRate);

    const currentChart = getChart();
    if (currentChart && currentChart.canvas !== canvas) {
      clearChannelChart();
    }

    let chart = getChart();
    const currentChannelChartId = getChartId();
    const theme = getChartThemeColors();
    if (!chart || currentChannelChartId !== channelId) {
      clearChannelChart();
      setChannelChartId(channelId);
      chart = new Chart(canvas.getContext("2d"), {
        type: "line",
        data: {
          labels,
          datasets: [
            {
              label: "send.count msg/s",
              data: sendData,
              borderWidth: 2,
              tension: 0.25,
              borderColor: "rgba(34,197,94,0.9)",
              pointRadius: 0,
            },
            {
              label: "recv.count msg/s",
              data: recvData,
              borderWidth: 2,
              tension: 0.25,
              borderColor: "rgba(56,189,248,0.9)",
              pointRadius: 0,
            },
            {
              label: "send.error_full msg/s",
              data: sendErrorFullData,
              borderWidth: 2,
              tension: 0.25,
              borderColor: "rgba(248,113,113,0.95)",
              borderDash: [4, 2],
              pointRadius: 0,
            },
            {
              label: "send.error_closed msg/s",
              data: sendErrorClosedData,
              borderWidth: 2,
              tension: 0.25,
              borderColor: "rgba(239,68,68,0.9)",
              borderDash: [2, 2],
              pointRadius: 0,
            },
            {
              label: "recv.error_empty msg/s",
              data: recvErrorEmptyData,
              borderWidth: 2,
              tension: 0.25,
              borderColor: "rgba(244,63,94,0.9)",
              borderDash: [6, 2],
              pointRadius: 0,
            },
            {
              label: "recv.error_closed msg/s",
              data: recvErrorClosedData,
              borderWidth: 2,
              tension: 0.25,
              borderColor: "rgba(225,29,72,0.9)",
              borderDash: [1, 2],
              pointRadius: 0,
            },
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          animation: false,
          plugins: {
            legend: {
              display: false,
            },
            tooltip: {
              enabled: false,
            },
          },
          interaction: {
            mode: "index",
            intersect: false,
          },
          scales: {
            x: {
              ticks: { color: theme.tick, maxTicksLimit: 6 },
              grid: { color: theme.grid },
            },
            y: {
              beginAtZero: true,
              ticks: { color: theme.tick },
              grid: { color: theme.grid },
            },
          },
        },
        plugins: [pipelineHoverPlugin],
      });
      chart._tsLabels = points.map((point) => point.ts);
      attachLegendInteraction(chart);
      setChannelChart(chart);
      renderChannelLegend(channelId);
      return;
    }

    chart.data.labels = labels;
    chart.data.datasets[0].data = sendData;
    chart.data.datasets[1].data = recvData;
    chart.data.datasets[2].data = sendErrorFullData;
    chart.data.datasets[3].data = sendErrorClosedData;
    chart.data.datasets[4].data = recvErrorEmptyData;
    chart.data.datasets[5].data = recvErrorClosedData;
    chart.options.scales.x.ticks.color = theme.tick;
    chart.options.scales.y.ticks.color = theme.tick;
    chart.options.scales.x.grid.color = theme.grid;
    chart.options.scales.y.grid.color = theme.grid;
    chart.update("none");
    chart._tsLabels = points.map((point) => point.ts);
    if (getGlobalHoverTs() != null) {
      setGlobalHover(getGlobalHoverTs());
    }
    renderChannelLegend(channelId);
  }

  function renderSelectionNone() {
    if (selectionTitle) {
      selectionTitle.textContent = "Selection Details";
    }
    edgeDetailMeta.textContent = "None selected";
    edgeDetailBody.innerHTML = "Click a node or edge to show details.";
    clearChannelChart();
    destroyNodeCharts();
  }

  function renderEdgeDetails(edge) {
    if (!edge) {
      renderSelectionNone();
      return;
    }

    if (selectionTitle) {
      selectionTitle.textContent = "Selection Details - Channel";
    }
    destroyNodeCharts();

    const channel = edge.data;
    const senderAttrs = channel.sender?.attrs || {};
    const receiverAttrs = channel.receiver?.attrs || {};
    const channelAttrs = buildCommonChannelAttributes(senderAttrs, receiverAttrs);
    const channelType = channelAttrs["channel.type"] || "channel";
    const channelKind = channelAttrs["channel.kind"] || channel.kind || "";
    const channelModeValue = channelAttrs["channel.mode"] || "n/a";
    const channelImpl = channelAttrs["channel.impl"] || "n/a";
    const channelTitle = `${String(channelType).toUpperCase()} channel (${channelModeValue}, ${channelImpl})`;
    const channelAttrsFiltered = Object.fromEntries(
      Object.entries(channelAttrs).filter(
        ([key]) => key !== "channel.type" && key !== "channel.mode" && key !== "channel.impl"
      )
    );

    const capacityMetric =
      findMetric(channel.receiver?.metrics || [], "capacity") ||
      findMetric(channel.sender?.metrics || [], "capacity");
    const capacityValue = capacityMetric
      ? formatValueWithUnit(capacityMetric.value, capacityMetric.unit)
      : "n/a";

    const queueDepthMetric = findMetric(channel.receiver?.metrics || [], "queue.depth");
    const queueDepthValue = queueDepthMetric
      ? formatValueWithUnit(queueDepthMetric.value, queueDepthMetric.unit)
      : "n/a";

    const capacityNumber =
      capacityMetric && typeof capacityMetric.value === "number"
        ? capacityMetric.value
        : null;
    const queueDepthNumber =
      queueDepthMetric && typeof queueDepthMetric.value === "number"
        ? queueDepthMetric.value
        : null;
    const queueRatio =
      Number.isFinite(capacityNumber) && capacityNumber > 0 && Number.isFinite(queueDepthNumber)
        ? Math.min(Math.max(queueDepthNumber / capacityNumber, 0), 1)
        : null;
    const queuePercent = queueRatio == null ? "n/a" : `${Math.round(queueRatio * 100)}%`;
    const queueColor =
      queueRatio == null
        ? "rgba(148,163,184,0.5)"
        : queueRatio >= 0.8
          ? "rgba(248,113,113,0.9)"
          : queueRatio >= 0.5
            ? "rgba(251,191,36,0.9)"
            : "rgba(52,211,153,0.9)";

    const senderMetricsMap = metricMap(channel.sender?.metrics || []);
    const receiverMetricsMap = metricMap(channel.receiver?.metrics || []);
    const channelId = edge.channelId || channel?.id || edge.id;
    const channelDisplayId = edge.channelDisplayId || channel?.displayId || channelId;
    const sourceDisplayId = edge.sourceDisplayId || senderAttrs["node.id"] || edge.source;
    const targetDisplayId = edge.targetDisplayId || receiverAttrs["node.id"] || edge.target;

    const useChannelSeries = !(channel?.multiSender || channel?.multiReceiver);
    const seriesPoint = useChannelSeries ? getChannelPoint(channelId, getDisplayTimeMs()) : null;
    const edgeRates = getLastEdgeRates().get(edge.id);
    const lastSampleSeconds = getLastSampleSeconds();

    const sendRate =
      seriesPoint?.sendRate ??
      edgeRates?.sendRate ??
      calcRate(senderMetricsMap["send.count"] ?? 0, lastSampleSeconds);
    const recvRate =
      seriesPoint?.recvRate ??
      edgeRates?.recvRate ??
      calcRate(receiverMetricsMap["recv.count"] ?? 0, lastSampleSeconds);
    const sendErrRate =
      seriesPoint
        ? (seriesPoint.sendErrorFullRate || 0) + (seriesPoint.sendErrorClosedRate || 0)
        : edgeRates?.sendErrorRate ??
          calcRate(
            (senderMetricsMap["send.error_full"] ?? 0) +
              (senderMetricsMap["send.error_closed"] ?? 0),
            lastSampleSeconds
          );
    const recvErrRate =
      seriesPoint
        ? (seriesPoint.recvErrorEmptyRate || 0) + (seriesPoint.recvErrorClosedRate || 0)
        : edgeRates?.recvErrorRate ??
          calcRate(
            (receiverMetricsMap["recv.error_empty"] ?? 0) +
              (receiverMetricsMap["recv.error_closed"] ?? 0),
            lastSampleSeconds
          );

    const edgeMetaParts = [`${sourceDisplayId} -> ${targetDisplayId}`];
    if (channelDisplayId) edgeMetaParts.push(channelDisplayId);
    if (edge.port) edgeMetaParts.push(`port ${edge.port}`);
    edgeDetailMeta.textContent = edgeMetaParts.join(" | ");

    const senderName = channelKind === "control" ? "Pipeline controller" : sourceDisplayId;
    const senderType = channelKind === "control" ? "controller" : senderAttrs["node.type"] || "node";

    const safeSenderName = escapeHtml(senderName);
    const safeSenderType = escapeHtml(senderType);
    const safeSendRate = escapeHtml(formatRateWithUnit(sendRate, "message"));
    const safeSendErrRate = escapeHtml(formatRateWithUnit(sendErrRate, "error"));
    const safeChannelTitle = escapeHtml(channelTitle);
    const safeCapacityValue = escapeHtml(capacityValue);
    const safeQueuePercent = escapeHtml(queuePercent);
    const safeQueueDepthValue = escapeHtml(queueDepthValue);
    const safeTargetDisplayId = escapeHtml(targetDisplayId);
    const safeReceiverType = escapeHtml(receiverAttrs["node.type"] || "node");
    const safeRecvRate = escapeHtml(formatRateWithUnit(recvRate, "message"));
    const safeRecvErrRate = escapeHtml(formatRateWithUnit(recvErrRate, "error"));
    const safeWindowLabel = escapeHtml(formatWindowLabel());

    edgeDetailBody.innerHTML = `
      <div class="channel-rail">
        <div class="channel-end">
          <div class="channel-end-label">Sender</div>
          <div class="channel-end-id">${safeSenderName} <span class="text-slate-400 text-xs">(${safeSenderType})</span></div>
          <div class="mt-2 text-xs text-slate-400">Rate: <span class="font-mono text-slate-200">${safeSendRate}</span></div>
          <div class="mt-1 text-xs text-slate-400">Errors: <span class="font-mono text-slate-200">${safeSendErrRate}</span></div>
        </div>
        <div class="channel-mid">
          <div class="channel-mid-title">${safeChannelTitle}</div>
          <div class="channel-mid-body">${renderAttributes(channelAttrsFiltered)}</div>
          <div class="channel-capacity">
            <span>Capacity</span>
            <span class="font-mono">${safeCapacityValue}</span>
          </div>
          <div class="channel-util">
            <div class="flex items-center justify-between">
              <span>Queue utilization</span>
              <span class="font-mono">${safeQueuePercent}${queuePercent !== "n/a" ? ` (${safeQueueDepthValue} / ${safeCapacityValue})` : ""}</span>
            </div>
            <div class="channel-util-bar">
              <div class="channel-util-fill" style="width:${queueRatio == null ? 0 : queueRatio * 100}%; background:${queueColor};"></div>
            </div>
          </div>
        </div>
        <div class="channel-end channel-end-right">
          <div class="channel-end-label">Receiver</div>
          <div class="channel-end-id">${safeTargetDisplayId} <span class="text-slate-400 text-xs">(${safeReceiverType})</span></div>
          <div class="mt-2 text-xs text-slate-400">Rate: <span class="font-mono text-slate-200">${safeRecvRate}</span></div>
          <div class="mt-1 text-xs text-slate-400">Errors: <span class="font-mono text-slate-200">${safeRecvErrRate}</span></div>
        </div>
      </div>
      <div class="mt-6 grid gap-6 md:grid-cols-[1fr_0.9fr_1fr]">
        <div>
          <div class="text-xs uppercase tracking-wide text-slate-400">Metrics</div>
          <div class="mt-2 text-xs">${renderMetricTable(
            (channel.sender?.metrics || []).filter(
              (metric) =>
                metric.name !== "send.error_full" && metric.name !== "send.error_closed"
            )
          )}</div>
        </div>
        <div></div>
        <div class="channel-metrics-right">
          <div class="text-xs uppercase tracking-wide text-slate-400">Metrics</div>
          <div class="mt-2 text-xs">${renderMetricTable(
            (channel.receiver?.metrics || []).filter(
              (metric) =>
                metric.name !== "capacity" &&
                metric.name !== "recv.error_empty" &&
                metric.name !== "recv.error_closed"
            )
          )}</div>
        </div>
      </div>
      <div class="mt-6">
        <div class="text-xs uppercase tracking-wide text-slate-400">Activity (last ${safeWindowLabel})</div>
        <div class="mt-3 channel-chart-wrap">
          <div class="channel-chart-canvas">
            <canvas id="channelChart"></canvas>
          </div>
          <div id="channelChartLegend" class="channel-chart-legend hidden"></div>
        </div>
      </div>
    `;

    renderChannelChart(channelId);
  }

  function renderNodeDetails(node) {
    if (!node) {
      renderSelectionNone();
      return;
    }

    if (selectionTitle) {
      selectionTitle.textContent = "Selection Details - Node";
    }
    clearChannelChart();
    destroyNodeCharts();
    const type = node.attrs["node.type"] || "node";
    edgeDetailMeta.textContent = `${node.displayId || node.id} (${type})`;

    const summary = buildNodeSummary(node.id);
    const safeInRate = escapeHtml(formatRateWithUnit(summary.inRate, "msg"));
    const safeOutRate = escapeHtml(formatRateWithUnit(summary.outRate, "msg"));
    const safeErrorRate = escapeHtml(formatRateWithUnit(summary.errorRate, "error"));
    const summaryHtml = `
      <div class="mt-4 grid gap-3 sm:grid-cols-3">
        <div class="card rounded-xl p-3">
          <div class="text-[0.6rem] uppercase tracking-wide text-slate-400">In rate</div>
          <div class="text-lg font-semibold text-slate-200">${safeInRate}</div>
        </div>
        <div class="card rounded-xl p-3">
          <div class="text-[0.6rem] uppercase tracking-wide text-slate-400">Out rate</div>
          <div class="text-lg font-semibold text-slate-200">${safeOutRate}</div>
        </div>
        <div class="card rounded-xl p-3">
          <div class="text-[0.6rem] uppercase tracking-wide text-slate-400">Errors</div>
          <div class="text-lg font-semibold text-slate-200">${safeErrorRate}</div>
        </div>
      </div>
    `;

    const nodeAttrs = node.displayAttrs || {};
    const metricBlocks = node.metricSets
      .map(
        (set) => `
          <div class="mt-4">
            <div class="text-xs uppercase tracking-wide text-slate-400">${escapeHtml(set.name)}</div>
            <div class="mt-2 text-xs">${renderNodeMetricTable(set.metrics, node.id, set.name)}</div>
          </div>`
      )
      .join("");

    edgeDetailBody.innerHTML = `
      <div>
        <div class="text-xs uppercase tracking-wide text-slate-400">Attributes</div>
        <div class="mt-2 space-y-1 text-xs">${renderAttributes(nodeAttrs)}</div>
      </div>
      ${summaryHtml}
      <div class="mt-4">
        ${metricBlocks || '<div class="mt-2 text-slate-400">No node metrics.</div>'}
      </div>
    `;
    initNodeRateCharts();
  }

  function setChannelHover(ts) {
    const chart = getChart();
    const channelId = getChartId();
    if (!chart || !channelId) {
      return;
    }
    if (!Number.isFinite(ts)) {
      chart._hoverIndex = null;
      chart.draw();
      renderChannelLegend(channelId);
      return;
    }
    const labels = chart._tsLabels || [];
    if (!labels.length) {
      renderChannelLegend(channelId);
      return;
    }

    let idx = null;
    if (ts <= labels[0]) {
      idx = 0;
    } else if (ts >= labels[labels.length - 1]) {
      idx = labels.length - 1;
    } else {
      let low = 0;
      let high = labels.length - 1;
      while (low <= high) {
        const mid = Math.floor((low + high) / 2);
        const value = labels[mid];
        if (value === ts) {
          idx = mid;
          break;
        }
        if (value < ts) {
          low = mid + 1;
        } else {
          high = mid - 1;
        }
      }
      if (idx == null) {
        const lowIdx = Math.max(0, high);
        const highIdx = Math.min(labels.length - 1, low);
        idx =
          Math.abs(labels[highIdx] - ts) < Math.abs(ts - labels[lowIdx])
            ? highIdx
            : lowIdx;
      }
    }

    chart._hoverIndex = idx;
    chart.draw();
    const point = getChannelPoint(channelId, labels[idx]);
    renderChannelLegend(channelId, point || undefined);
  }

  function applyChannelChartTheme(theme) {
    const chart = getChart();
    if (!chart) return;
    chart.options.scales.x.ticks.color = theme.tick;
    chart.options.scales.y.ticks.color = theme.tick;
    chart.options.scales.x.grid.color = theme.grid;
    chart.options.scales.y.grid.color = theme.grid;
    chart.update("none");
  }

  return {
    clearChannelChart,
    renderChannelChart,
    renderSelectionNone,
    renderEdgeDetails,
    renderNodeDetails,
    setChannelHover,
    applyChannelChartTheme,
  };
}
