// Controller for pipeline metric card charts.
// Maintains bounded in-memory series, keeps chart hover synchronized with the
// global timeline, and delegates rendering to the injected Chart runtime.
function getClosestIndex(labels, ts) {
  if (!labels.length || !Number.isFinite(ts)) return null;
  if (ts <= labels[0]) return 0;
  if (ts >= labels[labels.length - 1]) return labels.length - 1;
  let low = 0;
  let high = labels.length - 1;
  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    const value = labels[mid];
    if (value === ts) return mid;
    if (value < ts) {
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }
  const lowIdx = Math.max(0, high);
  const highIdx = Math.min(labels.length - 1, low);
  return Math.abs(labels[highIdx] - ts) < Math.abs(ts - labels[lowIdx])
    ? highIdx
    : lowIdx;
}

export function createPipelineChartsController({
  pipelineSeries,
  pipelineCharts,
  maxWindowMs,
  pipelineChartConfig,
  pipelineMetricDisplay,
  getWindowEndMs,
  getWindowMs,
  getDisplayTimeMs,
  getSeriesWindow,
  getPointAtTime,
  getChartThemeColors,
  pipelineHoverPlugin,
  onGlobalHover,
  getGlobalHoverTs,
}) {
  let hoverTs = null;

  function recordMetric(metricKey, value, ts) {
    if (!ts || !Number.isFinite(value)) return;
    const nowMs = ts.getTime();
    const cutoff = nowMs - maxWindowMs;
    const entry = pipelineSeries.get(metricKey) || { points: [] };
    entry.points.push({ ts: nowMs, value });
    entry.points = entry.points.filter((point) => point.ts >= cutoff);
    pipelineSeries.set(metricKey, entry);
  }

  function getMetricPointAtTime(metricKey, ts) {
    const series = pipelineSeries.get(metricKey);
    if (!series || !series.points.length) return null;
    const endMs = getWindowEndMs();
    const startMs = endMs - getWindowMs();
    const points = getSeriesWindow(series.points, startMs, endMs);
    if (!points.length) return null;
    const targetTs = Number.isFinite(ts) ? ts : getDisplayTimeMs();
    return getPointAtTime(points, targetTs) || points[points.length - 1];
  }

  function applyMetricValues(ts) {
    Object.entries(pipelineMetricDisplay).forEach(([key, cfg]) => {
      if (!cfg.el || !cfg.format) return;
      const point = getMetricPointAtTime(key, ts);
      cfg.el.textContent = cfg.format(point ? point.value : null);
    });
  }

  function getMetricSeriesWindow(metricKey) {
    const endMs = getWindowEndMs();
    const startMs = endMs - getWindowMs();
    const series = pipelineSeries.get(metricKey);
    if (!series) return [];
    return getSeriesWindow(series.points, startMs, endMs);
  }

  function setHover(ts) {
    hoverTs = Number.isFinite(ts) ? ts : null;
    applyMetricValues(hoverTs);
    pipelineCharts.forEach((chart) => {
      const labels = chart._tsLabels || [];
      if (!labels.length || hoverTs == null) {
        chart._hoverIndex = null;
        chart.draw();
        return;
      }
      const idx = getClosestIndex(labels, hoverTs);
      chart._hoverIndex = idx;
      chart.draw();
    });
    return hoverTs;
  }

  function updateCharts() {
    const theme = getChartThemeColors();
    const ChartCtor = typeof window !== "undefined" ? window.Chart : null;
    Object.values(pipelineChartConfig).forEach((config) => {
      const canvas = document.getElementById(config.canvasId);
      if (!canvas || !ChartCtor) return;
      const labelSet = new Set();
      const seriesMaps = new Map();
      config.metrics.forEach((metric) => {
        const points = getMetricSeriesWindow(metric.key);
        const map = new Map(points.map((point) => [point.ts, point.value]));
        seriesMaps.set(metric.key, map);
        points.forEach((point) => labelSet.add(point.ts));
      });
      const labels = Array.from(labelSet).sort((a, b) => a - b);
      const labelStrings = labels.map((ts) => new Date(ts).toLocaleTimeString());
      const datasets = config.metrics.map((metric) => {
        const map = seriesMaps.get(metric.key) || new Map();
        return {
          data: labels.map((ts) => map.get(ts) ?? null),
          borderWidth: 2,
          tension: 0.25,
          borderColor: metric.color,
          pointRadius: 0,
          spanGaps: true,
        };
      });

      const existing = pipelineCharts.get(config.canvasId);
      if (!existing) {
        const chart = new ChartCtor(canvas.getContext("2d"), {
          type: "line",
          data: { labels: labelStrings, datasets },
          plugins: [pipelineHoverPlugin],
          options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: false,
            interaction: { mode: "index", intersect: false },
            plugins: { legend: { display: false }, tooltip: { enabled: false } },
            scales: {
              x: {
                ticks: { color: theme.tick, maxTicksLimit: 6 },
                grid: { color: theme.grid },
              },
              y: {
                beginAtZero: false,
                ticks: { color: theme.tick },
                grid: { color: theme.grid },
              },
            },
          },
        });
        chart._tsLabels = labels;
        const move = (event) => {
          const elements = chart.getElementsAtEventForMode(
            event,
            "index",
            { intersect: false },
            false
          );
          if (!elements.length) return;
          const index = elements[0].index;
          const ts = chart._tsLabels ? chart._tsLabels[index] : null;
          if (!Number.isFinite(ts)) return;
          if (hoverTs !== ts) {
            onGlobalHover(ts);
          }
        };
        const leave = () => {
          onGlobalHover(null);
        };
        chart.canvas.addEventListener("mousemove", move);
        chart.canvas.addEventListener("mouseleave", leave);
        chart._pipelineHoverHandlers = { move, leave, canvas: chart.canvas };
        pipelineCharts.set(config.canvasId, chart);
        return;
      }

      existing.data.labels = labelStrings;
      existing.data.datasets = datasets;
      existing._tsLabels = labels;
      existing.options.scales.x.ticks.color = theme.tick;
      existing.options.scales.y.ticks.color = theme.tick;
      existing.options.scales.x.grid.color = theme.grid;
      existing.options.scales.y.grid.color = theme.grid;
      existing.update("none");
      if (getGlobalHoverTs() != null) {
        onGlobalHover(getGlobalHoverTs());
      }
    });
  }

  function updateMetricLegends(show) {
    const metricEls = document.querySelectorAll(".metric-legend");
    metricEls.forEach((el) => {
      const key = el.dataset.metric;
      const color =
        Object.values(pipelineChartConfig)
          .flatMap((cfg) => cfg.metrics)
          .find((metric) => metric.key === key)?.color || null;
      if (show && color) {
        el.style.setProperty("--legend-color", color);
        el.classList.add("metric-legend-active");
      } else {
        el.classList.remove("metric-legend-active");
        el.style.removeProperty("--legend-color");
      }
    });
  }

  function toggleCharts(show) {
    document
      .querySelectorAll(".metric-card-chart")
      .forEach((el) => el.classList.toggle("hidden", !show));
    updateMetricLegends(show);
    if (show) {
      updateCharts();
    }
  }

  function destroyCharts() {
    pipelineCharts.forEach((chart) => {
      if (chart._pipelineHoverHandlers) {
        const { move, leave, canvas } = chart._pipelineHoverHandlers;
        canvas.removeEventListener("mousemove", move);
        canvas.removeEventListener("mouseleave", leave);
        chart._pipelineHoverHandlers = null;
      }
      chart.destroy();
    });
    pipelineCharts.clear();
    hoverTs = null;
    applyMetricValues(null);
  }

  function applyTheme(theme) {
    pipelineCharts.forEach((chart) => {
      chart.options.scales.x.ticks.color = theme.tick;
      chart.options.scales.y.ticks.color = theme.tick;
      chart.options.scales.x.grid.color = theme.grid;
      chart.options.scales.y.grid.color = theme.grid;
      chart.update("none");
    });
  }

  function clearSeries() {
    pipelineSeries.clear();
  }

  return {
    recordMetric,
    getMetricPointAtTime,
    applyMetricValues,
    setHover,
    getHoverTs() {
      return hoverTs;
    },
    updateCharts,
    toggleCharts,
    destroyCharts,
    applyTheme,
    clearSeries,
  };
}
