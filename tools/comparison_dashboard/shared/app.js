// ============================================================================
// Benchmark Dashboard — app.js
//
// ES module. Two modes:
//   - Landing page (window.COMPARISONS set): lists comparison sections
//   - Comparison page (window.COMPARISON_SLUG set): shows charts + detail panel
//
// Suite data is pre-loaded via <script> tags that populate
// window.SUITE_DATA before this module runs. Comparison definitions
// are embedded as window.COMPARISONS (landing) or window.COMPARISON
// (detail page).
// ============================================================================

const BASE = "";  // set to "/<repo-name>" for GitHub Pages project sites

// ── Metric display config ──────────────────────────────────────────────────

const METRIC_LABELS = {
  cpu_percentage_normalized_avg: "CPU Average",
  cpu_percentage_normalized_max: "CPU Max",
  ram_mib_avg: "Memory Average",
  ram_mib_max: "Memory Max",
  network_tx_bytes_rate_avg: "Network TX Rate",
  network_rx_bytes_rate_avg: "Network RX Rate",
  dropped_logs_percentage: "Dropped Logs",
  logs_delivery_deviation_percentage: "Logs Delivery Deviation",
  logs_produced_rate: "Offered Load Rate",
  loadgen_logs_sent_rate: "Loadgen Sent Rate",
  logs_received_rate: "Collector Received Rate",
  collector_logs_sent_rate: "Collector Sent Rate",
  backend_logs_received_rate: "Backend Received Rate",
  test_duration: "Test Duration",
};

const METRIC_UNITS = {
  cpu_percentage_normalized_avg: "%",
  cpu_percentage_normalized_max: "%",
  ram_mib_avg: "MiB",
  ram_mib_max: "MiB",
  network_tx_bytes_rate_avg: "bytes/sec",
  network_rx_bytes_rate_avg: "bytes/sec",
  dropped_logs_percentage: "%",
  logs_delivery_deviation_percentage: "%",
  logs_produced_rate: "logs/sec",
  loadgen_logs_sent_rate: "logs/sec",
  logs_received_rate: "logs/sec",
  collector_logs_sent_rate: "logs/sec",
  backend_logs_received_rate: "logs/sec",
  test_duration: "seconds",
};

const DASHBOARD_METRICS = [
  "cpu_percentage_normalized_avg",
  "cpu_percentage_normalized_max",
  "ram_mib_avg",
  "ram_mib_max",
  "network_tx_bytes_rate_avg",
  "network_rx_bytes_rate_avg",
];

const AUTO_COLORS = [
  "#f97316", "#3b82f6", "#22c55e", "#a855f7",
  "#ef4444", "#14b8a6", "#eab308", "#ec4899",
  "#06b6d4", "#84cc16", "#e11d48", "#8b5cf6",
  "#f59e0b", "#0ea5e9", "#10b981", "#d946ef",
];

const COLORBLIND_COLORS = [
  "#0072b2", "#e69f00", "#009e73", "#cc79a7",
  "#56b4e9", "#d55e00", "#f0e442", "#000000",
  "#0099cc", "#994f00", "#006d5b", "#ad5c85",
  "#3a9bd9", "#aa4400", "#c4b832", "#444444",
];

let colorblindMode = localStorage.getItem("colorblindMode") === "true";

function getActivePalette() {
  return colorblindMode ? COLORBLIND_COLORS : AUTO_COLORS;
}

const DATA_LOSS_THRESHOLD = 5;
const RATE_DEVIATION_THRESHOLD = 5;

const FILTER_LABELS = {
  protocols: "Protocol",
  compression: "Compression",
  binary: "Binary",
  signals: "Signal",
};

// ── Data loading ───────────────────────────────────────────────────────────

function loadSuiteData() { return window.SUITE_DATA || {}; }

function getSuiteTests(suiteData, slug) {
  const suite = suiteData[slug];
  return suite ? suite.tests || [] : [];
}

function getTestByName(suiteData, slug, testName) {
  return getSuiteTests(suiteData, slug).find((t) => t.name === testName) || null;
}

function getSuiteMeta(suiteData, slug) {
  const suite = suiteData[slug];
  return suite ? suite.meta || {} : {};
}

// ── Filter infrastructure ──────────────────────────────────────────────────

const perComparisonFilters = new Map();

function collectFilterCategories(suiteData, comparison) {
  const cats = {};
  for (const ref of comparison.suites || []) {
    const meta = getSuiteMeta(suiteData, ref.slug);
    for (const [key, val] of Object.entries(meta)) {
      if (!cats[key]) cats[key] = new Set();
      if (Array.isArray(val)) { for (const v of val) cats[key].add(String(v)); }
      else { cats[key].add(String(val)); }
    }
  }
  const result = {};
  for (const [key, vals] of Object.entries(cats)) {
    if (vals.size > 1) result[key] = [...vals].sort();
  }
  return result;
}

function getFilterState(compSlug, categories) {
  if (!perComparisonFilters.has(compSlug)) {
    const state = new Map();
    for (const [cat, vals] of Object.entries(categories)) {
      state.set(cat, new Set(vals));
    }
    perComparisonFilters.set(compSlug, state);
  }
  return perComparisonFilters.get(compSlug);
}

function suiteMatchesFilters(suiteData, slug, filterState) {
  const meta = getSuiteMeta(suiteData, slug);
  for (const [cat, checked] of filterState) {
    if (checked.size === 0) return false;
    const val = meta[cat];
    if (val === undefined) continue;
    if (Array.isArray(val)) {
      if (!val.some((v) => checked.has(String(v)))) return false;
    } else {
      if (!checked.has(String(val))) return false;
    }
  }
  return true;
}

function filterComparison(comparison, suiteData, filterState) {
  const suites = [];
  const indices = [];
  for (let i = 0; i < (comparison.suites || []).length; i++) {
    if (suiteMatchesFilters(suiteData, comparison.suites[i].slug, filterState)) {
      suites.push(comparison.suites[i]);
      indices.push(i);
    }
  }
  return { ...comparison, suites, _originalIndices: indices };
}

function buildFilterHtml(categories, filterState) {
  const groups = Object.entries(categories).map(([cat, vals]) => {
    const checked = filterState.get(cat) || new Set();
    const opts = vals.map((v) =>
      `<label class="chart-filter-option"><input type="checkbox" data-filter-category="${escapeHtml(cat)}" data-filter-value="${escapeHtml(v)}" ${checked.has(v) ? "checked" : ""}> ${escapeHtml(v)}</label>`
    ).join("");
    const label = FILTER_LABELS[cat] || cat.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
    return `<div class="chart-filter-group"><span class="chart-filter-label">${escapeHtml(label)}:</span>${opts}</div>`;
  }).join("");
  return `<div class="chart-filters">${groups}<button class="filter-reset chart-filter-reset" type="button">Reset</button></div>`;
}

function wireFilters(container, compSlug, categories, onChange) {
  const fs = perComparisonFilters.get(compSlug);
  if (!fs) return;
  for (const cb of container.querySelectorAll("input[data-filter-category]")) {
    cb.onchange = () => {
      const s = fs.get(cb.dataset.filterCategory);
      if (!s) return;
      cb.checked ? s.add(cb.dataset.filterValue) : s.delete(cb.dataset.filterValue);
      onChange();
    };
  }
  const resetBtn = container.querySelector(".chart-filter-reset");
  if (resetBtn) {
    resetBtn.onclick = () => {
      for (const [cat, vals] of Object.entries(categories)) fs.set(cat, new Set(vals));
      for (const cb of container.querySelectorAll("input[data-filter-category]")) cb.checked = true;
      onChange();
    };
  }
}

// ── Chart.js infrastructure ────────────────────────────────────────────────

// Diagonal stripe pattern for missing-data bars
const patternCache = new Map();
function createDiagonalPattern(color) {
  if (patternCache.has(color)) return patternCache.get(color);
  const size = 8;
  const cv = document.createElement("canvas");
  cv.width = size;
  cv.height = size;
  const ctx = cv.getContext("2d");
  ctx.strokeStyle = color;
  ctx.lineWidth = 1.5;
  ctx.globalAlpha = 0.35;
  ctx.beginPath();
  ctx.moveTo(0, size);
  ctx.lineTo(size, 0);
  ctx.moveTo(-size / 2, size / 2);
  ctx.lineTo(size / 2, -size / 2);
  ctx.moveTo(size / 2, size * 1.5);
  ctx.lineTo(size * 1.5, size / 2);
  ctx.stroke();
  const pattern = document.createElement("canvas").getContext("2d").createPattern(cv, "repeat");
  patternCache.set(color, pattern);
  return pattern;
}

const barValueLabelsPlugin = {
  id: "barValueLabels",
  afterDatasetsDraw(chart) {
    const { ctx } = chart;
    const font = '10px "SF Pro Text", "Segoe UI", system-ui, sans-serif';
    const iconFont = '20px "SF Pro Text", "Segoe UI", system-ui, sans-serif';
    ctx.save();
    for (let dsIdx = 0; dsIdx < chart.data.datasets.length; dsIdx++) {
      const ds = chart.data.datasets[dsIdx];
      const meta = chart.getDatasetMeta(dsIdx);
      if (meta.hidden) continue;
      const flags = ds._hasBackpressure || [];
      const missing = ds._missing || [];
      for (let i = 0; i < meta.data.length; i++) {
        if (missing[i]) continue;
        const value = ds.data[i];
        if (value == null) continue;
        const el = meta.data[i];
        const label = formatMetricValue(value, "");
        ctx.font = font;
        ctx.textAlign = "center";
        ctx.textBaseline = "bottom";
        ctx.fillStyle = flags[i] ? "#ef4444" : "#64748b";
        ctx.fillText(label, el.x, el.y - 4);
        if (flags[i]) {
          ctx.font = iconFont;
          ctx.textBaseline = "middle";
          const cy = (el.y + el.base) / 2;
          ctx.lineWidth = 3;
          ctx.strokeStyle = "#ffffff";
          ctx.strokeText("\u26A0", el.x, cy);
          ctx.fillStyle = "#ef4444";
          ctx.fillText("\u26A0", el.x, cy);
        }
      }
    }
    ctx.restore();
  },
};

const activeCharts = new Map();

function getColor(index) { const p = getActivePalette(); return p[index % p.length]; }

function hasBackpressure(metricsArray, loadgenRate) {
  if (!metricsArray) return false;
  const dropped = metricsArray.find((m) => m.name === "dropped_logs_percentage");
  if (dropped && typeof dropped.value === "number" && dropped.value > DATA_LOSS_THRESHOLD) return true;
  if (loadgenRate && loadgenRate > 0) {
    const produced = metricsArray.find((m) => m.name === "logs_produced_rate");
    if (produced && typeof produced.value === "number") {
      if ((loadgenRate - produced.value) / loadgenRate * 100 > RATE_DEVIATION_THRESHOLD) return true;
    }
  }
  return false;
}

function buildComparisonChartData(suiteData, comparison, testNames, selectedMetric) {
  const refs = comparison.suites || [];
  const origIdx = comparison._originalIndices || null;

  // First pass: find the max real value to compute a sentinel height
  let maxVal = 0;
  for (const ref of refs) {
    for (const t of getSuiteTests(suiteData, ref.slug)) {
      if (!t.metrics) continue;
      const m = t.metrics.find((x) => x.name === selectedMetric);
      if (m && typeof m.value === "number" && Number.isFinite(m.value)) maxVal = Math.max(maxVal, Math.abs(m.value));
    }
  }
  const sentinel = Math.max(1, maxVal * 0.03);

  const datasets = refs.map((ref, si) => {
    const colorIdx = origIdx ? origIdx[si] : si;
    const color = getColor(colorIdx);
    const pattern = createDiagonalPattern(color);
    const tests = getSuiteTests(suiteData, ref.slug);
    const data = [], bp = [], missing = [];
    for (const tn of testNames) {
      const t = tests.find((x) => x.name === tn);
      if (!t || !t.metrics) { data.push(sentinel); bp.push(false); missing.push(true); continue; }
      const m = t.metrics.find((x) => x.name === selectedMetric);
      const val = m && typeof m.value === "number" && Number.isFinite(m.value) ? m.value : null;
      if (val === null) { data.push(sentinel); bp.push(false); missing.push(true); }
      else {
        data.push(val);
        const rm = tn.match(/^(\d+)k$/);
        bp.push(hasBackpressure(t.metrics, rm ? parseInt(rm[1]) * 1000 : null));
        missing.push(false);
      }
    }
    return {
      label: ref.short || ref.name, data, _hasBackpressure: bp, _missing: missing,
      backgroundColor: data.map((_, i) => missing[i] ? pattern : color),
      borderColor: data.map((_, i) => missing[i] ? `${color}80` : color),
      borderWidth: 1,
      borderRadius: 4, borderSkipped: "bottom",
    };
  });
  return { labels: testNames, datasets };
}

function chartOptions(onClick) {
  return {
    responsive: true, maintainAspectRatio: false, animation: false,
    layout: { padding: { top: 24 } },
    datasets: { bar: { categoryPercentage: 0.85, barPercentage: 0.9 } },
    scales: {
      x: { grid: { display: false }, border: { display: false }, ticks: { font: { size: 12, weight: "600" }, color: "#64748b" } },
      y: { beginAtZero: true, border: { display: true, color: "#cbd5e1" },
        ticks: { maxTicksLimit: 5, color: "#94a3b8", font: { size: 10 }, callback: (v) => formatMetricValue(v, "") },
        grid: { color: "#e2e8f0" } },
    },
    plugins: {
      legend: { position: "bottom", labels: { boxWidth: 10, boxHeight: 10, borderRadius: 2, useBorderRadius: true, padding: 16, font: { size: 13 }, color: "#0f172a" } },
      tooltip: { backgroundColor: "rgba(15,23,42,0.9)", cornerRadius: 6, padding: 10, titleFont: { size: 12 }, bodyFont: { size: 12 },
        callbacks: { label: (ctx) => {
          const ds = ctx.dataset;
          if ((ds._missing || [])[ctx.dataIndex]) return `${ds.label}: Data missing`;
          return `${ds.label}: ${formatMetricValue(ctx.parsed.y, "")}`;
        } } },
    },
    onClick: onClick || undefined,
  };
}

function createBarChart(canvas, suiteData, comparison, testNames, selectedMetric, onClick) {
  const chart = new Chart(canvas, { type: "bar", data: buildComparisonChartData(suiteData, comparison, testNames, selectedMetric), options: chartOptions(onClick), plugins: [barValueLabelsPlugin] });
  sizeChartContainer(chart, canvas);
  return chart;
}

function sizeChartContainer(chart, canvas, baseHeight = 220) {
  const legendHeight = chart.legend?.height || 0;
  canvas.parentElement.style.height = `${baseHeight + legendHeight}px`;
  chart.resize();
}

function updateBarChartData(chart, suiteData, comparison, testNames, selectedMetric) {
  const d = buildComparisonChartData(suiteData, comparison, testNames, selectedMetric);
  chart.data.labels = d.labels;
  for (let i = 0; i < d.datasets.length; i++) {
    if (chart.data.datasets[i]) {
      const src = d.datasets[i], dst = chart.data.datasets[i];
      dst.data = src.data;
      dst._hasBackpressure = src._hasBackpressure;
      dst._missing = src._missing;
      dst.backgroundColor = src.backgroundColor;
      dst.borderColor = src.borderColor;
    }
  }
  chart.update("none");
}

const TIMESERIES_METRICS = [
  { key: "cpu_percentage_normalized", label: "CPU Normalized", unit: "%", avg: "cpu_percentage_normalized_avg", max: "cpu_percentage_normalized_max" },
  { key: "ram_mib", label: "RAM", unit: "MiB", avg: "ram_mib_avg", max: "ram_mib_max" },
  { key: "network_tx_bytes_rate", label: "Network TX Rate", unit: "bytes/sec", avg: "network_tx_bytes_rate_avg" },
  { key: "network_rx_bytes_rate", label: "Network RX Rate", unit: "bytes/sec", avg: "network_rx_bytes_rate_avg" },
  { key: "logs_produced_rate", label: "Offered Load Rate", unit: "logs/sec", avg: "logs_produced_rate" },
  { key: "logs_received_rate", label: "Backend Received Rate", unit: "logs/sec", avg: "logs_received_rate" },
];

const SCALAR_ONLY_METRICS = [
  { name: "dropped_logs_percentage", label: "Dropped Logs", unit: "%" },
  { name: "test_duration", label: "Test Duration", unit: "seconds" },
];

function tmTitle(tm) { return tm.unit ? `${tm.label} (${tm.unit})` : tm.label; }

function createLineChart(canvas, dataPoints, color) {
  return new Chart(canvas, {
    type: "line",
    data: { labels: dataPoints.map((p) => p.t), datasets: [{ data: dataPoints.map((p) => p.value), borderColor: color, borderWidth: 2, pointRadius: 2.5, pointHitRadius: 6, tension: 0.3, fill: false }] },
    options: {
      responsive: true, maintainAspectRatio: false, animation: false,
      layout: { padding: { top: 4, right: 4 } },
      scales: {
        x: { type: "linear", grid: { display: false }, border: { display: true, color: "#e2e8f0" }, ticks: { maxTicksLimit: 5, color: "#94a3b8", font: { size: 9 }, callback: (v) => `${Math.round(v)}s` } },
        y: { beginAtZero: false, grid: { color: "#f1f5f9" }, border: { display: false }, ticks: { maxTicksLimit: 4, color: "#94a3b8", font: { size: 9 }, callback: (v) => formatMetricValue(v, "") } },
      },
      plugins: { legend: { display: false }, tooltip: { backgroundColor: "rgba(15,23,42,0.9)", cornerRadius: 4, padding: 8, titleFont: { size: 11 }, bodyFont: { size: 11 }, callbacks: { title: (items) => `${Math.round(items[0].parsed.x)}s`, label: (ctx) => formatMetricValue(ctx.parsed.y, "") } } },
    },
  });
}

// ── Utility functions ──────────────────────────────────────────────────────

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = String(text ?? "");
  return div.innerHTML;
}

function formatMetricValue(value, unit) {
  if (value === null || value === undefined || !Number.isFinite(Number(value))) return "-";
  const v = Number(value);
  if (unit === "%") return `${v.toFixed(1)}%`;
  if (unit === "MiB") return `${v.toFixed(1)} MiB`;
  if (unit === "bytes/sec" || unit === "bytes/s") return formatBytes(v) + "/s";
  if (unit === "logs/sec" || unit === "logs/s") return formatCompactInteger(v) + "/s";
  if (unit === "seconds" || unit === "s") return `${v.toFixed(1)}s`;
  if (unit === "ms") return `${v.toFixed(1)}ms`;
  if (Math.abs(v) >= 1000) return formatCompactInteger(v);
  if (Number.isInteger(v)) return String(v);
  return v.toFixed(2);
}

function formatCompactInteger(v) {
  v = Number(v); if (!Number.isFinite(v)) return "-";
  if (Math.abs(v) >= 1e9) return `${(v/1e9).toFixed(1)}B`;
  if (Math.abs(v) >= 1e6) return `${(v/1e6).toFixed(1)}M`;
  if (Math.abs(v) >= 1e3) return `${(v/1e3).toFixed(1)}K`;
  return String(Math.round(v));
}

function formatBytes(v) {
  v = Number(v); if (!Number.isFinite(v)) return "-";
  if (v >= 1e9) return `${(v/1e9).toFixed(1)} GB`;
  if (v >= 1e6) return `${(v/1e6).toFixed(1)} MB`;
  if (v >= 1e3) return `${(v/1e3).toFixed(1)} KB`;
  return `${Math.round(v)} B`;
}

function metricLabel(name) {
  return METRIC_LABELS[name] || name.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

function metricTitle(name) {
  const label = metricLabel(name);
  const unit = METRIC_UNITS[name];
  return unit ? `${label} (${unit})` : label;
}

function collectTestNames(suiteData, comparison) {
  const names = new Set();
  for (const ref of comparison.suites || [])
    for (const t of getSuiteTests(suiteData, ref.slug)) names.add(t.name);
  return [...names].sort((a, b) => (parseInt(a) || 0) - (parseInt(b) || 0));
}

function findAvailableMetrics(suiteData, comparison) {
  return DASHBOARD_METRICS.filter((mn) =>
    (comparison.suites || []).some((ref) =>
      getSuiteTests(suiteData, ref.slug).some((t) =>
        t.metrics && t.metrics.some((m) => m.name === mn && m.value != null))));
}

// ── Syntax highlighting ────────────────────────────────────────────────────

function highlightYaml(text) {
  return String(text || "").replace(/\r\n/g, "\n").split("\n").map((line) => {
    const ci = line.search(/\s#/);
    const base = ci >= 0 ? line.slice(0, ci) : line;
    const comment = ci >= 0 ? line.slice(ci + 1) : "";
    let html = escapeHtml(base);
    html = html.replace(/^(\s*-\s*)?([A-Za-z0-9_.-]+)(\s*:)/, (_, p="", k="", c="") => `${escapeHtml(p)}<span class="yaml-key">${escapeHtml(k)}</span>${escapeHtml(c)}`);
    html = html.replace(/(&quot;[^&]*&quot;|'[^']*')/g, '<span class="yaml-string">$1</span>');
    html = html.replace(/\b(true|false|null)\b/g, '<span class="yaml-bool">$1</span>');
    html = html.replace(/(^|[^\w.-])(-?\d+(?:\.\d+)?)(?=$|[^\w.-])/g, (_, p, n) => `${p}<span class="yaml-number">${n}</span>`);
    if (comment) html += `<span class="yaml-comment">${escapeHtml(`#${comment}`)}</span>`;
    return html || "&nbsp;";
  }).join("\n");
}

function highlightJson(text) {
  return String(text || "").replace(/\r\n/g, "\n").split("\n").map((line) => {
    let html = escapeHtml(line);
    html = html.replace(/^(\s*)(&quot;)((?:[^&]|&(?!quot;))*)(&quot;)(\s*:)/, (_, i, q1, k, q2, c) => `${i}<span class="yaml-key">${q1}${k}${q2}</span>${c}`);
    html = html.replace(/(:\s*|^\s*-?\s*)(&quot;)((?:[^&]|&(?!quot;))*)(&quot;)/g, (m, p, q1, s, q2) => p.includes(":") || p.trim().startsWith("-") ? `${p}<span class="yaml-string">${q1}${s}${q2}</span>` : m);
    html = html.replace(/\b(true|false|null)\b/g, '<span class="yaml-bool">$1</span>');
    html = html.replace(/(:\s*|^\s*-?\s*)(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)(\s*[,\]]?\s*$)/g, (_, p, n, s) => `${p}<span class="yaml-number">${n}</span>${s}`);
    return html || "&nbsp;";
  }).join("\n");
}

function highlightFileContent(name, content) {
  if (name.endsWith(".yaml") || name.endsWith(".yml")) return highlightYaml(content);
  if (name.endsWith(".json")) return highlightJson(content);
  return escapeHtml(content);
}

// ── Landing page ───────────────────────────────────────────────────────────

const perComparisonMetrics = new Map();

function renderColorblindToggle() {
  const label = colorblindMode ? "Standard Colors" : "Colorblind Mode";
  return `<button class="colorblind-toggle${colorblindMode ? " active" : ""}" type="button" title="Toggle colorblind-friendly palette">${label}</button>`;
}

function wireColorblindToggle(container, rerender) {
  const btn = container.querySelector(".colorblind-toggle");
  if (!btn) return;
  btn.onclick = () => {
    colorblindMode = !colorblindMode;
    localStorage.setItem("colorblindMode", String(colorblindMode));
    patternCache.clear();
    rerender();
  };
}

function renderLandingPage() {
  const app = document.getElementById("app");
  const cardsEl = document.getElementById("comparison-cards");
  if (!app) return;
  const suiteData = loadSuiteData();
  const comparisons = window.COMPARISONS || [];
  app.innerHTML = renderColorblindToggle();
  for (const c of activeCharts.values()) c.destroy();
  activeCharts.clear();
  wireColorblindToggle(app, renderLandingPage);
  if (!comparisons.length) { if (cardsEl) cardsEl.innerHTML = '<div class="muted" style="padding:16px">No comparisons defined.</div>'; return; }
  if (cardsEl) {
    cardsEl.innerHTML = comparisons.map((comp) => renderComparisonSection(suiteData, comp)).join("");
    for (const comp of comparisons) wireComparisonSection(suiteData, comp);
  }
}

function renderComparisonSection(suiteData, comparison) {
  const slug = comparison.slug;
  const categories = collectFilterCategories(suiteData, comparison);
  const filterState = getFilterState(slug, categories);
  const filtered = filterComparison(comparison, suiteData, filterState);
  const testNames = collectTestNames(suiteData, filtered);
  const metrics = findAvailableMetrics(suiteData, filtered);
  if (!perComparisonMetrics.has(slug)) perComparisonMetrics.set(slug, metrics.includes("cpu_percentage_normalized_avg") ? "cpu_percentage_normalized_avg" : metrics[0] || "cpu_percentage_normalized_avg");
  const sel = perComparisonMetrics.get(slug);
  const optsHtml = metrics.map((n) => `<option value="${escapeHtml(n)}" ${n === sel ? "selected" : ""}>${escapeHtml(metricLabel(n))}</option>`).join("");
  const hasFilters = Object.keys(categories).length > 0;
  const filterHtml = hasFilters ? buildFilterHtml(categories, filterState) : "";
  const anyBP = (filtered.suites || []).some((r) => getSuiteTests(suiteData, r.slug).some((t) => { const rm = t.name.match(/^(\d+)k$/); return hasBackpressure(t.metrics, rm ? parseInt(rm[1])*1000 : null); }));
  const bpHtml = anyBP ? '<div class="chart-backpressure-legend">\u26A0 Backpressure detected</div>' : "";
  const link = `${BASE}${encodeURIComponent(slug)}/`;
  return `
    <section class="scenario-section" data-comparison-id="${escapeHtml(slug)}">
      <div class="scenario-section-head">
        <a class="scenario-section-title" href="${link}">${escapeHtml(comparison.name || slug)}</a>
        <select class="scenario-metric-select" data-comparison-id="${escapeHtml(slug)}">${optsHtml}</select>
      </div>
      <div class="scenario-section-description">${escapeHtml(comparison.description || "")}</div>
      ${filterHtml}
      <div class="chart-container"><canvas></canvas></div>
      ${bpHtml}
    </section>`;
}

function wireComparisonSection(suiteData, comparison) {
  const slug = comparison.slug;
  const section = document.querySelector(`[data-comparison-id="${slug}"]`);
  if (!section) return;
  const categories = collectFilterCategories(suiteData, comparison);
  const filterState = getFilterState(slug, categories);

  function renderChart() {
    const filtered = filterComparison(comparison, suiteData, filterState);
    const testNames = collectTestNames(suiteData, filtered);
    const sel = perComparisonMetrics.get(slug);
    if (activeCharts.has(slug)) { activeCharts.get(slug).destroy(); activeCharts.delete(slug); }
    const canvas = section.querySelector("canvas");
    if (canvas && filtered.suites.length > 0) {
      activeCharts.set(slug, createBarChart(canvas, suiteData, filtered, testNames, sel));
    }
    const bpEl = section.querySelector(".chart-backpressure-legend");
    if (bpEl) {
      const anyBP = (filtered.suites || []).some((r) => getSuiteTests(suiteData, r.slug).some((t) => { const rm = t.name.match(/^(\d+)k$/); return hasBackpressure(t.metrics, rm ? parseInt(rm[1])*1000 : null); }));
      bpEl.style.display = anyBP ? "" : "none";
    }
  }

  const fc = section.querySelector(".chart-filters");
  if (fc) wireFilters(fc, slug, categories, renderChart);
  const ms = section.querySelector(".scenario-metric-select");
  if (ms) ms.onchange = () => { perComparisonMetrics.set(slug, ms.value); renderChart(); };
  renderChart();
}

// ── Comparison detail page ─────────────────────────────────────────────────

function renderComparisonPage(compSlug) {
  const app = document.getElementById("app");
  if (!app) return;
  const suiteData = loadSuiteData();
  const comparison = window.COMPARISON;
  if (!comparison) { app.innerHTML = '<div class="muted" style="padding:16px">Comparison definition not found.</div>'; return; }

  const categories = collectFilterCategories(suiteData, comparison);
  const filterState = getFilterState(compSlug, categories);
  const hasFilters = Object.keys(categories).length > 0;
  const filterHtml = hasFilters ? buildFilterHtml(categories, filterState) : "";

  app.innerHTML = `
    <div class="scenario-header">
      <a class="back-link" href="${BASE}/">&larr; All Comparisons</a>
      <h1>${escapeHtml(comparison.name || compSlug)}</h1>
      <div class="sub">${escapeHtml(comparison.description || "")}</div>
    </div>
    ${renderColorblindToggle()}
    ${filterHtml}
    <div id="comparison-chart"></div>
    <div id="comparison-detail"></div>`;

  let detailSuiteIdx = 0, detailTestName = "";

  function renderAll() {
    const filtered = filterComparison(comparison, suiteData, filterState);
    const testNames = collectTestNames(suiteData, filtered);
    if (detailSuiteIdx >= filtered.suites.length) detailSuiteIdx = 0;
    if (!testNames.includes(detailTestName)) detailTestName = testNames[0] || "";

    const setDetail = renderComparisonDetail(suiteData, filtered, testNames, detailSuiteIdx, detailTestName, (si, tn) => { detailSuiteIdx = si; detailTestName = tn; });
    renderComparisonChart(suiteData, filtered, testNames, (si, tn) => { detailSuiteIdx = si; detailTestName = tn; setDetail(si, tn); });
  }

  wireColorblindToggle(app, () => renderComparisonPage(compSlug));
  const fc = app.querySelector(".chart-filters");
  if (fc) wireFilters(fc, compSlug, categories, renderAll);
  renderAll();
}

function renderComparisonChart(suiteData, comparison, testNames, onBarClick) {
  const target = document.getElementById("comparison-chart");
  if (!target) return;
  const metrics = findAvailableMetrics(suiteData, comparison);
  let sel = metrics.includes("cpu_percentage_normalized_avg") ? "cpu_percentage_normalized_avg" : metrics[0] || "cpu_percentage_normalized_avg";
  const optsHtml = metrics.map((n) => `<option value="${escapeHtml(n)}" ${n === sel ? "selected" : ""}>${escapeHtml(metricLabel(n))}</option>`).join("");

  const onClick = onBarClick ? (event, elements) => {
    if (!elements.length) return;
    const { datasetIndex, index } = elements[0];
    const ref = (comparison.suites || [])[datasetIndex];
    const tn = testNames[index];
    if (ref && tn) onBarClick(datasetIndex, tn);
  } : null;

  const anyBP = (comparison.suites || []).some((r) => getSuiteTests(suiteData, r.slug).some((t) => { const rm = t.name.match(/^(\d+)k$/); return hasBackpressure(t.metrics, rm ? parseInt(rm[1])*1000 : null); }));
  const bpHtml = anyBP ? '<div class="chart-backpressure-legend">\u26A0 Backpressure detected</div>' : "";

  if (comparison.suites.length === 0) {
    target.innerHTML = '<div class="scenario-section"><div class="muted" style="padding:16px">No suites match the current filters.</div></div>';
    return;
  }

  target.innerHTML = `
    <div class="scenario-section">
      <div class="scenario-section-head">
        <div class="scenario-section-title">${escapeHtml(metricTitle(sel))}</div>
        <select id="metric-select" class="scenario-metric-select">${optsHtml}</select>
      </div>
      <div class="chart-container"><canvas></canvas></div>
      ${bpHtml}
    </div>`;

  const canvas = target.querySelector("canvas");
  let chart = createBarChart(canvas, suiteData, comparison, testNames, sel, onClick);
  const ms = document.getElementById("metric-select");
  if (ms) ms.onchange = () => {
    sel = ms.value;
    updateBarChartData(chart, suiteData, comparison, testNames, sel);
    const t = target.querySelector(".scenario-section-title");
    if (t) t.textContent = metricTitle(sel);
  };
}

// ── Comparison page: test detail panel ─────────────────────────────────────

function renderComparisonDetail(suiteData, comparison, testNames, initialSuiteIdx, initialTestName, onSelectionChange) {
  const target = document.getElementById("comparison-detail");
  if (!target) return () => {};
  const refs = comparison.suites || [];
  const origIdx = comparison._originalIndices || null;
  let selSuite = initialSuiteIdx, selTest = initialTestName;
  let miniCharts = [];

  function setSelection(si, tn) { selSuite = si; selTest = tn; render(); }

  function render() {
    for (const c of miniCharts) c.destroy();
    miniCharts = [];
    if (refs.length === 0) {
      target.innerHTML = '<div class="scenario-section"><div class="scenario-section-head"><div class="scenario-section-title">Test Details</div></div><div class="muted" style="padding:12px 0">No suites match the current filters.</div></div>';
      return;
    }

    const ref = refs[selSuite];
    const test = ref ? getTestByName(suiteData, ref.slug, selTest) : null;
    const metrics = test ? (test.metrics || []) : [];
    const ts = test ? (test.timeseries || null) : null;
    const getAgg = (n) => { const m = metrics.find((x) => x.name === n); return m && typeof m.value === "number" && Number.isFinite(m.value) ? m : null; };

    const pillsHtml = refs.map((r, i) => {
      const ci = origIdx ? origIdx[i] : i;
      return `<button class="detail-pill ${i === selSuite ? "active" : ""}" style="--pill-color: ${getColor(ci)}" data-suite-idx="${i}" type="button">${escapeHtml(r.short || r.name)}</button>`;
    }).join("");

    const testOptsHtml = testNames.map((n) => `<option value="${escapeHtml(n)}" ${n === selTest ? "selected" : ""}>${escapeHtml(n)}</option>`).join("");

    let filesHtml = '<div class="muted">No files available.</div>';
    if (test) {
      const files = [...(test.configFiles || [])].sort();
      if (files.length) filesHtml = `<div class="files-flex">${files.map((f) => `<div class="file-list-item" data-file="${escapeHtml(f)}">${escapeHtml(f)}</div>`).join("")}</div>`;
    }

    const rm = selTest.match(/^(\d+)k$/);
    const lr = rm ? parseInt(rm[1]) * 1000 : null;
    const bpBadge = hasBackpressure(metrics, lr) ? '<div class="detail-backpressure-badge">\u26A0 Backpressure detected</div>' : "";

    let scalarsHtml = "";
    if (test && metrics.length) {
      const cards = SCALAR_ONLY_METRICS.map((sm) => { const m = getAgg(sm.name); if (!m) return ""; const bad = sm.name === "dropped_logs_percentage" && m.value > DATA_LOSS_THRESHOLD;
        return `<div class="metric-scalar-card${bad ? " backpressure" : ""}"><div class="metric-scalar-name">${escapeHtml(sm.label)}</div><div class="metric-scalar-value">${formatMetricValue(m.value, m.unit || sm.unit)}</div></div>`; }).filter(Boolean).join("");
      if (cards) scalarsHtml = `<div class="metric-scalars">${cards}</div>`;
    }

    let chartsHtml = "";
    if (test) {
      const cards = TIMESERIES_METRICS.map((tm) => {
        const parts = [];
        if (tm.avg) { const m = getAgg(tm.avg); if (m) parts.push(`<span>${tm.max ? "Avg: " : ""}${formatMetricValue(m.value, m.unit || tm.unit)}</span>`); }
        if (tm.max) { const m = getAgg(tm.max); if (m) parts.push(`<span>Max: ${formatMetricValue(m.value, m.unit || tm.unit)}</span>`); }
        if (!parts.length) return "";
        const hasSeries = ts && ts[tm.key] && ts[tm.key].length > 1;
        return `<div class="metric-chart-card" data-ts-key="${escapeHtml(tm.key)}"><div class="metric-chart-header"><div class="metric-chart-name">${escapeHtml(tmTitle(tm))}</div><div class="metric-chart-values">${parts.join("")}</div></div>${hasSeries ? '<div class="metric-chart-body"><canvas></canvas></div>' : '<div class="muted" style="font-size:12px">No time-series data available.</div>'}</div>`;
      }).filter(Boolean).join("");
      if (cards) chartsHtml = `<div class="metric-chart-grid">${cards}</div>`;
    }

    if (!test) {
      target.innerHTML = `<div class="scenario-section"><div class="scenario-section-head"><div class="scenario-section-title">Test Details</div></div><div class="detail-controls"><div class="detail-pills">${pillsHtml}</div><select class="detail-test-select">${testOptsHtml}</select></div><div class="muted" style="padding:12px 0">No data available for this selection.</div></div>`;
    } else {
      target.innerHTML = `<div class="scenario-section"><div class="scenario-section-head"><div class="scenario-section-title">Test Details</div></div><div class="detail-controls"><div class="detail-pills">${pillsHtml}</div><select class="detail-test-select">${testOptsHtml}</select></div>${bpBadge}<div class="files-section"><div class="detail-pane-title">Files</div>${filesHtml}</div><div class="detail-pane-title" style="margin-top:16px">Metrics</div>${scalarsHtml}${chartsHtml || '<div class="muted">No metrics available.</div>'}</div>`;
    }

    for (const pill of target.querySelectorAll(".detail-pill")) pill.onclick = () => { selSuite = Number(pill.dataset.suiteIdx); if (onSelectionChange) onSelectionChange(selSuite, selTest); render(); };
    const ts2 = target.querySelector(".detail-test-select");
    if (ts2) ts2.onchange = () => { selTest = ts2.value; if (onSelectionChange) onSelectionChange(selSuite, selTest); render(); };
    if (test && ref) for (const item of target.querySelectorAll(".file-list-item")) item.onclick = () => openFileModal(ref.slug, selTest, item.dataset.file);
    if (test && ts) {
      const ci = origIdx ? origIdx[selSuite] : selSuite;
      const color = getColor(ci);
      for (const card of target.querySelectorAll(".metric-chart-card[data-ts-key]")) {
        const series = ts[card.dataset.tsKey];
        if (!series || series.length < 2) continue;
        const cv = card.querySelector("canvas");
        if (cv) miniCharts.push(createLineChart(cv, series, color));
      }
    }
  }

  render();
  return setSelection;
}

// ── File viewer modal ──────────────────────────────────────────────────────

async function loadConfigFile(suiteSlug, testName, fileName) {
  const url = `${BASE}/data/suite/${encodeURIComponent(suiteSlug)}/${encodeURIComponent(testName)}/${encodeURIComponent(fileName)}`;
  const resp = await fetch(url);
  if (!resp.ok) throw new Error(`Failed to load ${fileName}: ${resp.status}`);
  return resp.text();
}

async function openFileModal(suiteSlug, testName, fileName) {
  const modal = document.getElementById("run-detail-modal");
  const body = document.getElementById("run-detail-body");
  const title = document.getElementById("run-detail-title");
  if (!modal || !body || !title) return;
  title.textContent = fileName;
  modal.hidden = false;
  body.innerHTML = '<pre class="config-full-code"><code id="file-modal-content">Loading...</code></pre>';
  try {
    const content = await loadConfigFile(suiteSlug, testName, fileName);
    const el = document.getElementById("file-modal-content");
    if (el) el.innerHTML = highlightFileContent(fileName, content);
  } catch (e) {
    const el = document.getElementById("file-modal-content");
    if (el) el.textContent = `Error loading file: ${e.message}`;
  }
}

// ── Modal init + Bootstrap ─────────────────────────────────────────────────

function initModal() {
  const modal = document.getElementById("run-detail-modal");
  const closeBtn = document.getElementById("run-detail-close");
  if (!modal || !closeBtn) return;
  closeBtn.onclick = () => { modal.hidden = true; };
  modal.addEventListener("click", (evt) => { if (evt.target === modal) modal.hidden = true; });
  document.addEventListener("keydown", (evt) => { if (evt.key === "Escape" && !modal.hidden) modal.hidden = true; });
}

function main() {
  initModal();
  if (window.COMPARISON_SLUG) renderComparisonPage(window.COMPARISON_SLUG);
  else if (window.COMPARISONS) renderLandingPage();
  else { const app = document.getElementById("app"); if (app) app.innerHTML = '<div class="muted" style="padding:16px">No data loaded. Run build.py to generate dashboard data.</div>'; }
}

try { main(); } catch (err) {
  const app = document.getElementById("app");
  if (app) app.innerHTML = `<pre style="padding:16px;color:red">Failed to load dashboard: ${escapeHtml(String(err))}</pre>`;
  console.error(err);
}
