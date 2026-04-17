  // Main single-page controller:
  // - polls metrics snapshots
  // - derives card/chart data
  // - builds and renders topology
  // - wires all user interactions
  import {
    buildPipelineOptgroupElement,
    buildPipelineOptionElement,
    setToggleVisualState,
  } from "./control-utils.js";
  import { buildMetricsCandidates } from "./metrics-api.js";
  import {
    formatPipelineGroupLabel,
    getPipelineGroupId,
    getPipelineId,
    getPipelineSelectionKeyFromAttrs,
    makePipelineSelectionKey,
    normalizeAttributes,
    resolveSelectedCoreId,
  } from "./pipeline-utils.js";
  import { deriveEngineCardValues, extractEngineSummary } from "./engine-metrics.js";
  import { escapeAttr, escapeHtml } from "./dom-safety.js";
  import {
    averageCoreUsage,
    buildCoreUsage,
    refreshPipelineSelectorStatusDecorations as refreshPipelineSelectorDecorations,
    renderCoreOverlay as renderCoreOverlayUi,
    updateCoreSelectionDisplay as updateCoreSelectionDisplayUi,
    updatePipelineSelectionDisplay as updatePipelineSelectionDisplayUi,
  } from "./selectors-ui.js";
  import {
    buildGraph,
    buildScopedMetricId,
    resolveScopedChannelId,
    resolveScopedNodeId,
  } from "./graph-model.js";
  import {
    computeEdgeRates as computeEdgeRatesFromSeries,
    getChannelPoint as getChannelPointFromSeries,
    getPointAtTime as getPointAtTimeFromSeries,
    getSeriesWindow as getSeriesWindowFromRange,
    updateChannelSeries as updateChannelSeriesFromMetrics,
    updateNodeSeries as updateNodeSeriesFromMetrics,
  } from "./charts-controller.js";
  import { createPipelineChartsController } from "./pipeline-charts-controller.js";
  import { createSelectionDetailsController } from "./selection-details-controller.js";
  import { createDagInteractionController } from "./dag-interaction-controller.js";
  import {
    getNodeOutputAnchorY as getLayoutNodeOutputAnchorY,
    layoutGraph as computeLayoutGraph,
  } from "./graph-layout.js";
  import { renderGraphFrame } from "./graph-renderer.js";
  import {
    filterMetricSets as filterMetricSetsBySelection,
    getDagMetricSets as getDagMetricSetsBySelection,
    isDeltaCounterMetric,
  } from "./metric-filters.js";
  import {
    runHealthPoll,
    runMetricsPoll,
    runStatusPoll,
    scheduleNextTimer,
  } from "./polling-controller.js";
  import { buildStatusSnapshot, getStatusSeverity } from "./status-runtime.js";
  import {
    buildInterPipelineTopology,
    createEmptyInterPipelineTopology,
    findMostCentralPipelineKey,
    getPipelineInterconnect,
    getTransitivelyConnectedPipelineKeys,
  } from "./inter-pipeline-topology.js";
  import { createLogsController } from "./logs-controller.js";

  // Query params tune metrics query behavior.
  const urlParams = new URLSearchParams(window.location.search);

  // Metrics endpoint strategy: read cumulative snapshots (no server-side reset),
  // compute deltas client-side, and keep-all-zeroes is configurable.
  const keepAllZeroesParam = urlParams.get("keep_all_zeroes");
  const keepAllZeroes =
    keepAllZeroesParam == null ? true : keepAllZeroesParam === "true";
  const METRICS_URL_CANDIDATES = buildMetricsCandidates({
    query: `format=json&reset=false&keep_all_zeroes=${keepAllZeroes ? "true" : "false"}`,
  });
  const HOLD_LAST_ENGINE_VALUES = true;
  const SKIP_ENGINE_ALL_ZERO_SNAPSHOTS = true;
  const POLL_INTERVAL_MS = 2000;
  const HEALTH_POLL_INTERVAL_MS = 5000;
  const HEALTH_REQUEST_TIMEOUT_MS = 1200;
  const STATUS_POLL_INTERVAL_MS = 10000;
  const STATUS_REQUEST_TIMEOUT_MS = 1800;
  const PIPELINE_STATUS_COLORS = {
    up: "#34d399",
    down: "#f87171",
    unknown: "#facc15",
  };
  const PERF_ENABLED = urlParams.get("perf") === "true";
  const PERF_LOG_EVERY = 30;
  const PERF_SLOW_MS = 16;
  const perfStats = new Map();

  // DAG sizing/layout constants.
  const NODE_WIDTH = 210;
  const NODE_HEADER_HEIGHT = 38;
  const NODE_PADDING_Y = 6;
  const PORT_ROW_HEIGHT = 20;
  const NODE_FOOTER_HEIGHT = 12;
  const LEVEL_GAP = 140;
  const ROW_GAP = 40;
  const MARGIN = 48;
  const TOP_PADDING = 18;
  const EDGE_INSET = 6;
  const MAX_WINDOW_MS = 60 * 60 * 1000;
  const PIPELINE_NAV_ATTACH_GAP = 10;
  const PIPELINE_NAV_ROW_GAP = 24;
  const PIPELINE_NAV_LINE_WIDTH = 14;
  const PIPELINE_NAV_EDGE_PADDING = 18;
  const PIPELINE_NAV_MIN_WIDTH = 96;
  const PIPELINE_NAV_MAX_WIDTH = 260;
  const PIPELINE_NAV_CHAR_PX = 7.2;
  const ZOOM_MIN = 0.2;
  const ZOOM_MAX = 2.0;
  const ZOOM_FIT_PADDING = 18;
  const ZOOM_BUTTON_STEP = 0.00175;
  const WHEEL_ZOOM_SENSITIVITY = 0.000175;
  const DAG_DRAG_THRESHOLD_PX = 3;
  const MULTI_PIPELINE_COLUMN_GAP = 120;
  const MULTI_PIPELINE_ROW_GAP = 5;
  const DAG_SCOPE_SINGLE = "single";
  const DAG_SCOPE_CONNECTED = "connected";
  const GRAPH_LAYOUT_CONSTANTS = {
    NODE_WIDTH,
    NODE_HEADER_HEIGHT,
    NODE_PADDING_Y,
    PORT_ROW_HEIGHT,
    NODE_FOOTER_HEIGHT,
    LEVEL_GAP,
    ROW_GAP,
    MARGIN,
    TOP_PADDING,
    EDGE_INSET,
    MULTI_PIPELINE_COLUMN_GAP,
    MULTI_PIPELINE_ROW_GAP,
  };

  // DOM references: top-level status/toggles.
  const connectionDot = document.getElementById("connection-dot");
  const connectionStatus = document.getElementById("connection-status");
  const healthLivez = document.getElementById("health-livez");
  const healthReadyz = document.getElementById("health-readyz");
  const lastUpdateEl = document.getElementById("last-update");
  const errorBanner = document.getElementById("error-banner");
  const errorText = document.getElementById("error-text");
  const topPanels = document.getElementById("top-panels");
  const selectionTitle = document.getElementById("selection-title");
  const zeroToggleWrap = document.getElementById("toggle-zero-wrap");
  const zeroToggleTrack = document.getElementById("toggle-zero-track");
  const controlToggleWrap = document.getElementById("toggle-control-wrap");
  const controlToggleTrack = document.getElementById("toggle-control-track");
  const controlToggle = document.getElementById("toggle-control");
  const pipelineChartToggleWrap = document.getElementById("pipeline-chart-toggle-wrap");
  const pipelineChartToggleTrack = document.getElementById("pipeline-chart-toggle-track");
  const pipelineChartToggle = document.getElementById("pipeline-chart-toggle");
  const controlToggleText = document.getElementById("control-toggle-text");
  const themeToggleWrap = document.getElementById("theme-toggle-wrap");
  const themeToggleTrack = document.getElementById("theme-toggle-track");
  const themeToggle = document.getElementById("theme-toggle");
  const themeToggleLabel = document.getElementById("theme-toggle-label");

  document.title = "Rust Dataflow Engine";

  const THEME_STORAGE_KEY = "ogdp-theme";

  function perfStart() {
    if (!PERF_ENABLED) return null;
    if (typeof performance !== "undefined" && typeof performance.now === "function") {
      return performance.now();
    }
    return Date.now();
  }

  function perfEnd(label, startMs, fields = null) {
    if (!PERF_ENABLED || !Number.isFinite(startMs)) return;
    const now =
      typeof performance !== "undefined" && typeof performance.now === "function"
        ? performance.now()
        : Date.now();
    const duration = now - startMs;
    const entry = perfStats.get(label) || { count: 0, totalMs: 0, maxMs: 0 };
    entry.count += 1;
    entry.totalMs += duration;
    entry.maxMs = Math.max(entry.maxMs, duration);
    perfStats.set(label, entry);

    const shouldLog =
      duration >= PERF_SLOW_MS || entry.count % PERF_LOG_EVERY === 0;
    if (!shouldLog) return;

    const avgMs = entry.totalMs / entry.count;
    const fieldSuffix =
      fields && typeof fields === "object"
        ? ` ${Object.entries(fields)
            .map(([k, v]) => `${k}=${v}`)
            .join(" ")}`
        : "";
    console.info(
      `[ui perf] ${label} last=${duration.toFixed(2)}ms avg=${avgMs.toFixed(2)}ms max=${entry.maxMs.toFixed(2)}ms count=${entry.count}${fieldSuffix}`
    );
  }

  // Applies the visual theme and updates chart colors to match.
  function applyTheme(theme) {
    const isDay = theme === "day";
    document.body.classList.toggle("day-mode", isDay);
    if (themeToggle) {
      themeToggle.checked = isDay;
    }
    if (themeToggleTrack) {
      themeToggleTrack.classList.toggle("toggle-track-active", isDay);
    }
    if (themeToggleWrap) {
      themeToggleWrap.classList.toggle("text-slate-200", isDay);
      themeToggleWrap.classList.toggle("text-slate-300", !isDay);
    }
    if (themeToggleLabel) {
      themeToggleLabel.textContent = isDay ? "Day" : "Night";
      themeToggleLabel.classList.toggle("text-slate-200", isDay);
      themeToggleLabel.classList.toggle("text-slate-400", !isDay);
    }
    applyChartTheme();
  }

  const dagCanvas = document.getElementById("dag-canvas");
  const dagZoom = document.getElementById("dag-zoom");
  const dagViewport = document.getElementById("dag-viewport");
  const dagEdges = document.getElementById("dag-edges");
  const dagLanes = document.getElementById("dag-lanes");
  const dagNodes = document.getElementById("dag-nodes");
  const dagEmpty = document.getElementById("dag-empty");
  const edgeDetailMeta = document.getElementById("edge-detail-meta");
  const edgeDetailBody = document.getElementById("edge-detail-body");
  const pipelineSelector = document.getElementById("pipeline-selector");
  const pipelineSelectBtn = document.getElementById("pipeline-select-btn");
  const pipelineSelectIcon = document.getElementById("pipeline-select-icon");
  const pipelineSelectValue = document.getElementById("pipeline-select-value");
  const pipelineOverlay = document.getElementById("pipeline-overlay");
  const pipelineSelect = document.getElementById("pipeline-select");
  const coreSelector = document.getElementById("core-selector");
  const coreSelectBtn = document.getElementById("core-select-btn");
  const coreSelectValue = document.getElementById("core-select-value");
  const coreSelectSwatch = document.getElementById("core-select-swatch");
  const coreOverlay = document.getElementById("core-overlay");
  const zeroToggle = document.getElementById("toggle-zero");

  const zoomOutBtn = document.getElementById("zoom-out");
  const zoomInBtn = document.getElementById("zoom-in");
  const zoomResetBtn = document.getElementById("zoom-reset");
  const zoomValueEl = document.getElementById("zoom-value");
  const fullscreenBtn = document.getElementById("toggle-fullscreen");
  const dagScopeBtn = document.getElementById("toggle-dag-scope");
  const dagSearch = document.getElementById("dag-search");
  const viewSelect = document.getElementById("view-select");
  const modeSelect = document.getElementById("mode-select");
  const windowSelect = document.getElementById("window-select");
  const scrubToggle = document.getElementById("scrub-toggle");
  const scrubSlider = document.getElementById("scrub-slider");
  const scrubLabel = document.getElementById("scrub-label");
  const pipelineOptgroupTemplate = document.getElementById("pipeline-optgroup-template");
  const pipelineOptionTemplate = document.getElementById("pipeline-option-template");
  const runtimeStatusAnchor = document.getElementById("runtime-status-anchor");
  const runtimeStatusOverlay = document.getElementById("runtime-status-overlay");
  const runtimeStatusPanel = document.getElementById("runtime-status-panel");
  const runtimeStatusCloseBtn = document.getElementById("runtime-status-close");
  const runtimeStatusRefreshBtn = document.getElementById("runtime-status-refresh");
  const runtimeStatusMeta = document.getElementById("runtime-status-meta");
  const runtimeStatusTbody = document.getElementById("runtime-status-tbody");

  const engineCpuUtilEl = document.getElementById("engine-cpu-util");
  const engineMemoryRssEl = document.getElementById("engine-memory-rss");
  const engineGroupCountEl = document.getElementById("engine-group-count");
  const enginePipelineCountEl = document.getElementById("engine-pipeline-count");
  const engineCoreCountEl = document.getElementById("engine-core-count");
  const engineUptimeEl = document.getElementById("engine-uptime");

  const pipeCpuUtilEl = document.getElementById("pipe-cpu-util");
  const pipeCpuCoresEl = document.getElementById("pipe-cpu-cores");
  const pipeCpuTimeEl = document.getElementById("pipe-cpu-time-60");
  const pipeUptimeEl = document.getElementById("pipe-uptime");
  const pipeInstancesEl = document.getElementById("pipe-instances");

  const pipeMemUsageEl = document.getElementById("pipe-mem-usage");
  const pipeAllocRateEl = document.getElementById("pipe-alloc-rate-60");
  const pipeFreeRateEl = document.getElementById("pipe-free-rate-60");
  const pipeNetAllocRateEl = document.getElementById("pipe-net-alloc-rate-60");

  const pipeCtxVolRateEl = document.getElementById("pipe-ctx-vol-rate-60");
  const pipeCtxInvolRateEl = document.getElementById("pipe-ctx-invol-rate-60");
  const pipeFaultMinorRateEl = document.getElementById("pipe-fault-minor-rate-60");
  const pipeFaultMajorRateEl = document.getElementById("pipe-fault-major-rate-60");

  const tokioWorkerCountEl = document.getElementById("tokio-worker-count");
  const tokioBusyRateEl = document.getElementById("tokio-busy-rate");
  const tokioInstanceCountEl = document.getElementById("tokio-instance-count");
  const tokioActiveTasksEl = document.getElementById("tokio-active-tasks");
  const tokioQueueSizeEl = document.getElementById("tokio-queue-size");
  const tokioParkRateEl = document.getElementById("tokio-park-rate");
  const tokioUnparkRateEl = document.getElementById("tokio-unpark-rate");

  const tabPanelGeneral = document.getElementById("tab-panel-general");
  const tabPanelControl = document.getElementById("tab-panel-control");
  const tabPanelTokio = document.getElementById("tab-panel-tokio");

  const controlDrainStateEl = document.getElementById("control-drain-state");
  const controlDrainPendingReceiversEl = document.getElementById(
    "control-drain-pending-receivers"
  );
  const controlDrainInstanceCountEl = document.getElementById(
    "control-drain-instance-count"
  );
  const controlDrainDeadlineForcedEl = document.getElementById(
    "control-drain-deadline-forced"
  );
  const controlRuntimePendingSendsEl = document.getElementById(
    "control-runtime-pending-sends"
  );
  const controlCompletionPendingSendsEl = document.getElementById(
    "control-completion-pending-sends"
  );
  const controlDelayedDataQueuedEl = document.getElementById(
    "control-delayed-data-queued"
  );
  const controlTimersActiveEl = document.getElementById("control-timers-active");
  const controlDelayedDataSentRateEl = document.getElementById(
    "control-delayed-data-sent-rate"
  );
  const controlTimerTickRateEl = document.getElementById("control-timer-tick-rate");
  const controlCollectTelemetryRateEl = document.getElementById(
    "control-collect-telemetry-rate"
  );
  const controlDelayReturnRateEl = document.getElementById("control-delay-return-rate");
  const controlDeliverAckRateEl = document.getElementById("control-deliver-ack-rate");
  const controlDeliverNackRateEl = document.getElementById("control-deliver-nack-rate");
  const controlAckAttemptRateEl = document.getElementById("control-ack-attempt-rate");
  const controlNackAttemptRateEl = document.getElementById("control-nack-attempt-rate");
  const controlAckDeliveredRateEl = document.getElementById(
    "control-ack-delivered-rate"
  );
  const controlNackDeliveredRateEl = document.getElementById(
    "control-nack-delivered-rate"
  );
  const controlAckDroppedEl = document.getElementById("control-ack-dropped");
  const controlNackDroppedEl = document.getElementById("control-nack-dropped");
  const controlEventsTitleEl = document.getElementById("control-events-title");
  const controlEventsMetaEl = document.getElementById("control-events-meta");
  const controlEventsListEl = document.getElementById("control-events-list");

  // Runtime state: selection, filtering, history caches, chart instances, and layout/zoom.
  let layoutSize = { width: 0, height: 0 };
  let lastSampleTs = null;
  let pipelinePrev = null;
  let tokioPrev = null;
  let selectedEdgeId = null;
  let selectedNodeId = null;
  let selectedEdgeData = null;
  let selectedNodeData = null;
  let lastSampleSeconds = null;
  let selectedPipelineKey = null;
  let selectedCoreId = null;
  let lastMetricSets = [];
  let interPipelineTopology = createEmptyInterPipelineTopology();
  // Derived inter-pipeline state for the currently selected pipeline (future side-panel usage).
  let selectedPipelineInterconnect = null;
  let lastEngineCpuUtilPercent = null;
  let lastEngineMemoryRssMiB = null;
  let lastEngineUptimeSeconds = null;
  let lastCoreUsage = new Map();
  let hideZeroActivity = false;
  let showControlChannels = false;
  let showPipelineCharts = false;
  let pipelineHoverTs = null;
  let globalHoverTs = null;
  let nodeHoverTs = null;
  let dagSearchQuery = "";
  let metricMode = "throughput";
  let lastGraph = null;
  let lastDataGraph = null;
  let lastControlGraph = null;
  let lastRenderedNodes = [];
  let lastRenderedEdges = [];
  let lastRenderedControlEdges = [];
  let lastRenderedSampleSeconds = null;
  let lastRenderedStructureSignature = null;
  let lastRenderedNodesById = new Map();
  let lastRenderedEdgesById = new Map();
  let lastRenderedControlEdgesById = new Map();
  let lastCoreUsageAvg = null;
  let lastCoreIds = [];
  let stickyPanelsObserver = null;
  let dagPipelineScopeMode = DAG_SCOPE_SINGLE;
  const pollingState = {
    pollTimer: null,
    healthPollTimer: null,
    statusPollTimer: null,
    fetchInFlight: false,
    healthFetchInFlight: false,
    statusFetchInFlight: false,
    statusLastCheckedAtMs: null,
    statusLastProbe: null,
    resolvedMetricsUrl: null,
    activeFetchController: null,
    latestFetchRequestId: 0,
    latestAppliedFetchRequestId: 0,
    clientDeltaPrevBySeries: new Map(),
  };
  let statusSnapshot = null;
  let controlPlanePrev = null;
  let pipelineOptionLabelByKey = new Map();
  const CORE_ALL = "__all__";
  let windowMinutes = 5;
  let freezeActive = false;
  let freezeAnchorMs = null;
  let freezeTimeMs = null;
  let lastEdgeRates = new Map();
  let channelChart = null;
  let channelChartId = null;
  const channelSeries = new Map();
  const nodeSeries = new Map();
  const nodeCharts = new Map();
  const pipelineSeries = new Map();
  const pipelineCharts = new Map();
  const dagNodeElements = new Map();
  const dagNodePortDotsByNode = new Map();
  const dagControlIndicatorsByNode = new Map();
  const dagEdgeElements = new Map();
  let dagEdgeDefsElement = null;
  let pipelineChartsController = null;
  let selectionDetailsController = null;
  let dagInteractionController = null;

  const PIPELINE_CHART_CONFIG = {
    engineCpu: {
      canvasId: "engineChartCpu",
      metrics: [
        { key: "engine.cpu.utilization", color: "rgba(16,185,129,0.9)" },
      ],
    },
    engineMemory: {
      canvasId: "engineChartMemory",
      metrics: [
        { key: "engine.memory.rss", color: "rgba(59,130,246,0.9)" },
      ],
    },
    cpu: {
      canvasId: "pipeChartCpu",
      metrics: [
        { key: "cpu.utilization", color: "rgba(34,197,94,0.9)" },
      ],
    },
    memory: {
      canvasId: "pipeChartMemory",
      metrics: [
        { key: "memory.usage", color: "rgba(99,102,241,0.9)" },
      ],
    },
    scheduling: {
      canvasId: "pipeChartScheduling",
      metrics: [
        { key: "context.switches.involuntary", color: "rgba(248,113,113,0.9)" },
        { key: "context.switches.voluntary", color: "rgba(167,139,250,0.9)" },
        { key: "page.faults.minor", color: "rgba(250,204,21,0.9)" },
        { key: "page.faults.major", color: "rgba(239,68,68,0.9)" },
      ],
    },
    controlDrain: {
      canvasId: "pipeChartControlDrain",
      metrics: [
        { key: "control.drain.active", color: "rgba(14,165,233,0.9)" },
        { key: "control.drain.pending_receivers", color: "rgba(250,204,21,0.9)" },
      ],
    },
    controlBacklog: {
      canvasId: "pipeChartControlBacklog",
      metrics: [
        { key: "control.runtime.pending_sends.buffered", color: "rgba(248,113,113,0.9)" },
        { key: "control.completion.pending_sends.buffered", color: "rgba(167,139,250,0.9)" },
        { key: "control.delayed_data.queued", color: "rgba(52,211,153,0.9)" },
      ],
    },
    controlRuntimeFlow: {
      canvasId: "pipeChartControlRuntimeFlow",
      metrics: [
        { key: "control.runtime.delayed_data.sent.rate", color: "rgba(34,197,94,0.9)" },
        { key: "control.runtime.timer_tick.sent.rate", color: "rgba(56,189,248,0.9)" },
        { key: "control.runtime.collect_telemetry.sent.rate", color: "rgba(249,115,22,0.9)" },
      ],
    },
    controlCompletionIngress: {
      canvasId: "pipeChartCompletionIngress",
      metrics: [
        { key: "control.completion.deliver_ack.received.rate", color: "rgba(34,197,94,0.9)" },
        { key: "control.completion.deliver_nack.received.rate", color: "rgba(248,113,113,0.9)" },
      ],
    },
    controlCompletionDelivery: {
      canvasId: "pipeChartCompletionDelivery",
      metrics: [
        { key: "control.completion.ack.delivered.rate", color: "rgba(34,197,94,0.9)" },
        { key: "control.completion.nack.delivered.rate", color: "rgba(248,113,113,0.9)" },
      ],
    },
  };

  // --- View mode and connection status helpers ---
  function setActiveTab(tab) {
    const isGeneral = tab === "general";
    const isControl = tab === "control";
    const isTokio = tab === "tokio";
    tabPanelGeneral.classList.toggle("hidden", !isGeneral);
    tabPanelControl.classList.toggle("hidden", !isControl);
    tabPanelTokio.classList.toggle("hidden", !isTokio);
    if (viewSelect) {
      viewSelect.value = isGeneral ? "general" : isControl ? "control" : "tokio";
    }
  }

  function setMetricMode(mode) {
    metricMode = mode;
    if (modeSelect) {
      modeSelect.value = mode;
    }
    applyFilteredView(lastMetricSets, false);
  }

  function navigateToPipeline(pipelineKey) {
    closePipelineOverlay();
    if (!pipelineKey || pipelineKey === selectedPipelineKey) {
      return;
    }
    selectedPipelineKey = pipelineKey;
    ensureRecentStatusSnapshot();
    selectedCoreId = null;
    dagInteractionController?.resetZoomOverride();
    resetVisualizationStateForFilterChange();
    updateFilterSelectors(lastMetricSets);
    applyFilteredView(lastMetricSets, false);
  }

  function isDagFullscreenActive() {
    return document.body.classList.contains("dag-fullscreen");
  }

  function getConnectedScopePipelineKeys() {
    if (!selectedPipelineKey) return [];
    return getTransitivelyConnectedPipelineKeys(interPipelineTopology, selectedPipelineKey);
  }

  function getDagRenderScope() {
    const connectedScopeActive =
      dagPipelineScopeMode === DAG_SCOPE_CONNECTED && isDagFullscreenActive();
    if (!connectedScopeActive) {
      return {
        mode: DAG_SCOPE_SINGLE,
        pipelineKeys: null,
        scopeByPipeline: false,
      };
    }

    const connectedKeys = getConnectedScopePipelineKeys();
    if (connectedKeys.length <= 1) {
      return {
        mode: DAG_SCOPE_SINGLE,
        pipelineKeys: null,
        scopeByPipeline: false,
      };
    }

    return {
      mode: DAG_SCOPE_CONNECTED,
      pipelineKeys: new Set(connectedKeys),
      scopeByPipeline: true,
    };
  }

  function updateDagScopeButtonState() {
    if (!dagScopeBtn) return;
    const connectedKeys = getConnectedScopePipelineKeys();
    const connectedCount = connectedKeys.length;
    const canUseConnectedScope = connectedCount > 1;
    if (!canUseConnectedScope && dagPipelineScopeMode === DAG_SCOPE_CONNECTED) {
      dagPipelineScopeMode = DAG_SCOPE_SINGLE;
    }

    const connectedActive =
      dagPipelineScopeMode === DAG_SCOPE_CONNECTED && isDagFullscreenActive();
    dagScopeBtn.disabled = !canUseConnectedScope;
    dagScopeBtn.classList.toggle("opacity-50", !canUseConnectedScope);
    dagScopeBtn.classList.toggle("cursor-not-allowed", !canUseConnectedScope);
    dagScopeBtn.textContent = connectedActive
      ? "Single pipeline view"
      : `Connected view (${connectedCount})`;
    dagScopeBtn.title = canUseConnectedScope
      ? connectedActive
        ? "Show only the selected pipeline DAG."
        : "Show all pipelines transitively connected through topics."
      : "No connected pipelines found for this selection.";
  }

  function setDagPipelineScopeMode(nextMode, options = {}) {
    const rerender = options.rerender !== false;
    const normalized =
      nextMode === DAG_SCOPE_CONNECTED ? DAG_SCOPE_CONNECTED : DAG_SCOPE_SINGLE;
    if (normalized === dagPipelineScopeMode) {
      updateDagScopeButtonState();
      return;
    }

    dagPipelineScopeMode = normalized;
    if (dagPipelineScopeMode === DAG_SCOPE_CONNECTED) {
      const connectedKeys = getConnectedScopePipelineKeys();
      if (connectedKeys.length <= 1) {
        dagPipelineScopeMode = DAG_SCOPE_SINGLE;
        updateDagScopeButtonState();
        return;
      }
    }

    dagInteractionController?.resetZoomOverride();
    resetVisualizationStateForFilterChange();
    clearSelection();
    updateDagScopeButtonState();
    if (rerender) {
      applyFilteredView(lastMetricSets, false);
    }
  }

  function estimatePipelineDagChipWidth(text) {
    const label = String(text || "(unknown)");
    const estimated = Math.ceil(label.length * PIPELINE_NAV_CHAR_PX) + 26;
    return Math.max(PIPELINE_NAV_MIN_WIDTH, Math.min(PIPELINE_NAV_MAX_WIDTH, estimated));
  }

  function computePipelineDagNavLayout(anchors) {
    let upstreamWidth = 0;
    let downstreamWidth = 0;
    (anchors || []).forEach((anchor) => {
      const width = estimatePipelineDagChipWidth(anchor.label);
      if (anchor.side === "upstream") {
        upstreamWidth = Math.max(upstreamWidth, width);
      } else if (anchor.side === "downstream") {
        downstreamWidth = Math.max(downstreamWidth, width);
      }
    });

    const leftGutter =
      upstreamWidth > 0
        ? Math.max(
            0,
            upstreamWidth +
              PIPELINE_NAV_LINE_WIDTH +
              PIPELINE_NAV_ATTACH_GAP +
              PIPELINE_NAV_EDGE_PADDING -
              MARGIN
          )
        : 0;
    const rightGutter =
      downstreamWidth > 0
        ? Math.max(
            0,
            downstreamWidth +
              PIPELINE_NAV_LINE_WIDTH +
              PIPELINE_NAV_ATTACH_GAP +
              PIPELINE_NAV_EDGE_PADDING -
              MARGIN
          )
        : 0;

    return {
      upstreamWidth,
      downstreamWidth,
      leftGutter,
      rightGutter,
    };
  }

  function createPipelineDagNavButton(anchor, widthPx) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "pipeline-dag-nav-chip";
    if (anchor.kind === "topic") {
      button.classList.add("pipeline-dag-nav-chip-topic");
    }
    button.textContent = anchor.label || "(unknown)";
    if (widthPx > 0) {
      button.style.width = `${widthPx}px`;
    }
    button.title = anchor.title || anchor.label || "";
    if (anchor.pipelineKey) {
      button.dataset.pipelineKey = anchor.pipelineKey;
      button.setAttribute("aria-label", `Open pipeline ${anchor.label || "(unknown)"}`);
      button.addEventListener("click", (event) => {
        event.stopPropagation();
        navigateToPipeline(anchor.pipelineKey);
      });
    } else {
      button.setAttribute("aria-label", `Topic ${anchor.label || "(unknown)"}`);
      button.tabIndex = -1;
    }
    return button;
  }

  function collectPipelineDagAnchors(nodeMap) {
    if (!nodeMap || !nodeMap.size) {
      return [];
    }

    const anchors = [];
    const seen = new Set();

    const addPipelineAnchors = (
      hostPipelineKey,
      neighbors,
      side,
      nodeIdKey,
      scopeByPipeline
    ) => {
      (neighbors || []).forEach((neighbor) => {
        if (!neighbor?.key) return;
        const nodeIds = new Set(
          (neighbor.edges || []).map((edge) => edge[nodeIdKey]).filter((nodeId) => nodeId)
        );
        nodeIds.forEach((nodeId) => {
          const scopedNodeId = buildScopedMetricId(
            nodeId,
            hostPipelineKey,
            scopeByPipeline
          );
          if (!nodeMap.has(scopedNodeId)) return;
          const id = `${side}\u0000${hostPipelineKey}\u0000${neighbor.key}\u0000${scopedNodeId}`;
          if (seen.has(id)) return;
          seen.add(id);
          const pipelineLabel = neighbor.pipelineId || "(unknown)";
          const groupLabel = formatPipelineGroupLabel(neighbor.groupId || "");
          anchors.push({
            side,
            nodeId: scopedNodeId,
            pipelineKey: neighbor.key,
            groupId: neighbor.groupId || "",
            pipelineId: pipelineLabel,
            label: pipelineLabel,
            title: `${pipelineLabel} | ${groupLabel}`,
            kind: "pipeline",
          });
        });
      });
    };

    if (!selectedPipelineInterconnect || !selectedPipelineKey) {
      return anchors;
    }

    addPipelineAnchors(
      selectedPipelineKey,
      selectedPipelineInterconnect.upstream,
      "upstream",
      "toNodeId",
      false
    );
    addPipelineAnchors(
      selectedPipelineKey,
      selectedPipelineInterconnect.downstream,
      "downstream",
      "fromNodeId",
      false
    );
    return anchors;
  }

  function buildConnectedTopicLinks(nodeMap, dagScope) {
    if (
      !nodeMap ||
      !nodeMap.size ||
      !dagScope ||
      dagScope.mode !== DAG_SCOPE_CONNECTED ||
      !(dagScope.pipelineKeys instanceof Set)
    ) {
      return [];
    }

    const grouped = new Map();
    const scopeGroupIds = new Set();
    dagScope.pipelineKeys.forEach((pipelineKey) => {
      const pipeline = interPipelineTopology?.pipelineByKey?.get(pipelineKey);
      if (pipeline?.groupId) {
        scopeGroupIds.add(pipeline.groupId);
      }
    });
    const edges = interPipelineTopology?.edges || [];
    edges.forEach((edge) => {
      const fromKey = edge?.from?.key;
      const toKey = edge?.to?.key;
      if (!fromKey || !toKey) return;
      if (!dagScope.pipelineKeys.has(fromKey) || !dagScope.pipelineKeys.has(toKey)) {
        return;
      }

      const sourceNodeId = buildScopedMetricId(edge.fromNodeId, fromKey, true);
      const targetNodeId = buildScopedMetricId(edge.toNodeId, toKey, true);
      if (!nodeMap.has(sourceNodeId) || !nodeMap.has(targetNodeId)) return;

      const topic = edge.topic || "(topic)";
      const processInstanceId = edge.processInstanceId || "";
      const groupKey = `${processInstanceId}\u0000${topic}`;
      let item = grouped.get(groupKey);
      if (!item) {
        item = {
          processInstanceId,
          topic,
          sourceNodeIds: new Set(),
          targetNodeIds: new Set(),
          groupIds: new Set(),
        };
        grouped.set(groupKey, item);
      }
      item.sourceNodeIds.add(sourceNodeId);
      item.targetNodeIds.add(targetNodeId);
      if (edge.from?.groupId) item.groupIds.add(edge.from.groupId);
      if (edge.to?.groupId) item.groupIds.add(edge.to.groupId);
    });

    const compactTopicLabel = (topic) => {
      const raw = String(topic || "").trim();
      if (!raw) return "(topic)";
      const splitClean = (value, separator) =>
        value
          .split(separator)
          .map((part) => part.trim())
          .filter((part) => part);

      let compact = raw;
      ["::", "/", ":"].forEach((separator) => {
        if (!compact.includes(separator)) return;
        const parts = splitClean(compact, separator);
        if (parts.length > 1) {
          compact = parts[parts.length - 1];
        }
      });

      // Fallback for very long dotted names where the suffix is most meaningful.
      if (compact.length > 42 && compact.includes(".")) {
        const parts = splitClean(compact, ".");
        if (parts.length > 1) {
          compact = parts[parts.length - 1];
        }
      }

      return compact || raw;
    };

    const links = [];
    grouped.forEach((groupedItem, groupKey) => {
      const sourceNodeIds = Array.from(groupedItem.sourceNodeIds).sort();
      const targetNodeIds = Array.from(groupedItem.targetNodeIds).sort();
      if (!sourceNodeIds.length || !targetNodeIds.length) return;
      const groupIds = Array.from(groupedItem.groupIds).sort((a, b) =>
        String(a).localeCompare(String(b), undefined, { numeric: true, sensitivity: "base" })
      );
      const label =
        scopeGroupIds.size <= 1
          ? compactTopicLabel(groupedItem.topic)
          : groupIds.length <= 1
            ? `${groupIds[0] || "group"}/${compactTopicLabel(groupedItem.topic)}`
            : `${groupIds.join("->")}/${compactTopicLabel(groupedItem.topic)}`;

      const sourcePoints = sourceNodeIds
        .map((nodeId) => {
          const node = nodeMap.get(nodeId);
          if (!node) return null;
          return {
            x: node.x + node.width + PIPELINE_NAV_ATTACH_GAP,
            y: node.y + node.height / 2,
          };
        })
        .filter(Boolean);
      const targetPoints = targetNodeIds
        .map((nodeId) => {
          const node = nodeMap.get(nodeId);
          if (!node) return null;
          return {
            x: node.x - PIPELINE_NAV_ATTACH_GAP,
            y: node.y + node.height / 2,
          };
        })
        .filter(Boolean);
      if (!sourcePoints.length || !targetPoints.length) return;

      const avg = (values) =>
        values.reduce((sum, value) => sum + value, 0) / Math.max(values.length, 1);
      const sourceAvgX = avg(sourcePoints.map((point) => point.x));
      const targetAvgX = avg(targetPoints.map((point) => point.x));
      const centerX = (sourceAvgX + targetAvgX) / 2;
      const centerY = avg(
        [...sourcePoints, ...targetPoints].map((point) => point.y)
      );

      links.push({
        key: groupKey,
        processInstanceId: groupedItem.processInstanceId,
        sourcePoints,
        targetPoints,
        label,
        title:
          `Topic: ${groupedItem.topic}` +
          (groupedItem.processInstanceId
            ? ` | process: ${groupedItem.processInstanceId}`
            : ""),
        centerX,
        centerY,
      });
    });

    links.sort((a, b) =>
      `${a.centerY}:${a.centerX}:${a.label}`.localeCompare(
        `${b.centerY}:${b.centerX}:${b.label}`,
        undefined,
        { numeric: true, sensitivity: "base" }
      )
    );
    return links;
  }

  function renderConnectedTopicNavigation(nodeMap, dagScope) {
    const topicLinks = buildConnectedTopicLinks(nodeMap, dagScope);
    if (!topicLinks.length) return;

    // Keep topic chips readable by enforcing a minimum vertical gap.
    const minGap = Math.max(16, Math.round(PIPELINE_NAV_ROW_GAP * 0.72));
    let previousY = Number.NEGATIVE_INFINITY;
    topicLinks.forEach((link) => {
      if (link.centerY - previousY < minGap) {
        link.centerY = previousY + minGap;
      }
      previousY = link.centerY;
    });

    topicLinks.forEach((link) => {
      const centerX = link.centerX;
      const centerY = link.centerY;
      const widthPx = estimatePipelineDagChipWidth(link.label);
      const halfWidth = widthPx / 2;

      const drawTopicLine = (point) => {
        const attachX = point.x <= centerX ? centerX - halfWidth : centerX + halfWidth;
        const direction = attachX >= point.x ? 1 : -1;
        const dx = Math.abs(attachX - point.x);
        const curvature = Math.min(90, Math.max(28, dx * 0.42));
        const c1x = point.x + curvature * direction;
        const c2x = attachX - curvature * direction;
        const pathData = `M ${point.x} ${point.y} C ${c1x} ${point.y}, ${c2x} ${centerY}, ${attachX} ${centerY}`;
        const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
        path.setAttribute("d", pathData);
        path.setAttribute("class", "dag-topic-link");
        dagEdges.appendChild(path);
      };

      link.sourcePoints.forEach(drawTopicLine);
      link.targetPoints.forEach(drawTopicLine);

      const nav = document.createElement("div");
      nav.className = "pipeline-dag-nav pipeline-dag-nav-topic";
      nav.style.left = `${centerX}px`;
      nav.style.top = `${centerY}px`;
      const button = createPipelineDagNavButton(
        { label: link.label, title: link.title, kind: "topic" },
        widthPx
      );
      nav.appendChild(button);
      dagNodes.appendChild(nav);
    });
  }

  function renderPipelineDagNavigation(nodeMap, anchors, navLayout) {
    if (!anchors.length) return;

    const groups = new Map();
    anchors.forEach((anchor) => {
      const key = `${anchor.side}\u0000${anchor.nodeId}`;
      if (!groups.has(key)) {
        groups.set(key, []);
      }
      groups.get(key).push(anchor);
    });

    groups.forEach((groupAnchors, groupKey) => {
      const [side, nodeId] = groupKey.split("\u0000");
      const node = nodeMap.get(nodeId);
      if (!node) return;

      groupAnchors.sort((a, b) =>
        (a.label || "").localeCompare(b.label || "", undefined, {
          numeric: true,
          sensitivity: "base",
        })
      );

      const centerY = node.y + node.height / 2;
      const startY = centerY - ((groupAnchors.length - 1) * PIPELINE_NAV_ROW_GAP) / 2;
      const attachX =
        side === "upstream"
          ? node.x - PIPELINE_NAV_ATTACH_GAP
          : node.x + node.width + PIPELINE_NAV_ATTACH_GAP;

      groupAnchors.forEach((anchor, index) => {
        const nav = document.createElement("div");
        nav.className = `pipeline-dag-nav pipeline-dag-nav-${side}`;
        nav.style.left = `${attachX}px`;
        nav.style.top = `${startY + index * PIPELINE_NAV_ROW_GAP}px`;

        const connector = document.createElement("span");
        connector.className = "pipeline-dag-nav-line";
        const widthPx =
          side === "upstream" ? navLayout.upstreamWidth : navLayout.downstreamWidth;
        const button = createPipelineDagNavButton(anchor, widthPx);

        if (side === "upstream") {
          nav.appendChild(button);
          nav.appendChild(connector);
        } else {
          nav.appendChild(connector);
          nav.appendChild(button);
        }
        dagNodes.appendChild(nav);
      });
    });
  }

  // Keep the inter-pipeline topology state accessible for future UI panels.
  function syncInterPipelineTopologyState() {
    selectedPipelineInterconnect = selectedPipelineKey
      ? getPipelineInterconnect(interPipelineTopology, selectedPipelineKey)
      : null;
    updateDagScopeButtonState();
  }

  function updateInterPipelineTopologyState(metricSets) {
    interPipelineTopology = buildInterPipelineTopology(metricSets);
    syncInterPipelineTopologyState();
  }


  function setConnected(ok) {
    if (ok) {
      connectionDot.classList.remove("bg-red-500", "shadow-red-500/60");
      connectionDot.classList.add("bg-emerald-500", "shadow-emerald-500/60");
      connectionStatus.textContent = "Connected";
      connectionStatus.classList.remove("text-red-400");
      connectionStatus.classList.add("text-emerald-400");
    } else {
      connectionDot.classList.remove("bg-emerald-500", "shadow-emerald-500/60");
      connectionDot.classList.add("bg-red-500", "shadow-red-500/60");
      connectionStatus.textContent = "Disconnected";
      connectionStatus.classList.remove("text-emerald-400");
      connectionStatus.classList.add("text-red-400");
    }
  }

  function setHealthBadge(el, label, probe) {
    if (!el) return;
    const state = probe?.state || "unknown";
    const statusText =
      state === "up" ? "UP" : state === "down" ? "DOWN" : "UNKNOWN";
    el.classList.remove("health-pill-up", "health-pill-down", "health-pill-unknown");
    if (state === "up") {
      el.classList.add("health-pill-up");
    } else if (state === "down") {
      el.classList.add("health-pill-down");
    } else {
      el.classList.add("health-pill-unknown");
    }
    el.textContent = `${label}: ${statusText}`;

    const parts = [`${label} ${statusText}`];
    if (Number.isFinite(probe?.status)) {
      parts.push(`HTTP ${probe.status}`);
    }
    if (Number.isFinite(probe?.latencyMs)) {
      parts.push(`${Math.round(probe.latencyMs)} ms`);
    }
    if (probe?.error) {
      parts.push(probe.error);
    }
    if (probe?.checkedAt) {
      parts.push(`at ${new Date(probe.checkedAt).toLocaleTimeString()}`);
    }
    el.title = parts.join(" | ");
  }

  function formatSnapshotTime(ts) {
    if (!ts) return null;
    const parsed = new Date(ts);
    if (!Number.isFinite(parsed.getTime())) return null;
    return parsed.toLocaleTimeString();
  }

  function getPipelineStatusState(pipelineKey) {
    const pipelineStatus = statusSnapshot?.byPipelineKey?.get(pipelineKey);
    if (!pipelineStatus) return "unknown";
    return pipelineStatus.state === "up" || pipelineStatus.state === "down"
      ? pipelineStatus.state
      : "unknown";
  }

  function getPipelineSelectorStatusTitle(pipelineKey) {
    const pipelineStatus = statusSnapshot?.byPipelineKey?.get(pipelineKey);
    if (!pipelineStatus) return "Runtime status unavailable (no /status snapshot yet).";
    return [
      pipelineStatus.rawPipelineKey,
      pipelineStatus.summary,
      ...pipelineStatus.details,
    ].join(" | ");
  }

  function updatePipelineSelectionDisplay() {
    updatePipelineSelectionDisplayUi({
      pipelineSelectValue,
      pipelineSelectIcon,
      selectedPipelineKey,
      pipelineOptionLabelByKey,
      getPipelineStatusState,
      pipelineStatusColors: PIPELINE_STATUS_COLORS,
    });
  }

  function closePipelineOverlay() {
    if (!pipelineOverlay) return;
    pipelineOverlay.classList.add("hidden");
  }

  function refreshPipelineSelectorStatusDecorations() {
    refreshPipelineSelectorDecorations({
      pipelineSelect,
      pipelineOverlay,
      pipelineSelectValue,
      pipelineSelectIcon,
      selectedPipelineKey,
      pipelineOptionLabelByKey,
      getPipelineStatusState,
      getPipelineSelectorStatusTitle,
      pipelineStatusColors: PIPELINE_STATUS_COLORS,
    });
  }

  function isRuntimeStatusOverlayOpen() {
    return !!runtimeStatusOverlay && !runtimeStatusOverlay.classList.contains("hidden");
  }

  function setRuntimeStatusMetaText(text) {
    if (!runtimeStatusMeta) return;
    runtimeStatusMeta.textContent = text;
  }

  function createRuntimeStatusCell(value, className = "") {
    const td = document.createElement("td");
    td.textContent = value;
    if (className) {
      td.className = className;
    }
    return td;
  }

  function formatRuntimeEventTime(time) {
    if (!time) return "unknown time";
    const parsed = new Date(time);
    if (!Number.isFinite(parsed.getTime())) return "unknown time";
    return parsed.toLocaleTimeString();
  }

  function renderControlPlaneEventsPanel() {
    if (!controlEventsListEl || !controlEventsMetaEl || !controlEventsTitleEl) return;

    const pipelineLabel =
      selectedPipelineKey && pipelineOptionLabelByKey.has(selectedPipelineKey)
        ? pipelineOptionLabelByKey.get(selectedPipelineKey)
        : selectedPipelineKey || "Selected pipeline";
    controlEventsTitleEl.textContent = pipelineLabel;

    if (!statusSnapshot) {
      controlEventsMetaEl.textContent = "No runtime status snapshot yet.";
      controlEventsListEl.innerHTML =
        '<div class="control-event-empty">No recent runtime events.</div>';
      return;
    }

    if (!selectedPipelineKey) {
      controlEventsMetaEl.textContent = "Select a pipeline to inspect drain and lifecycle events.";
      controlEventsListEl.innerHTML =
        '<div class="control-event-empty">No pipeline selected.</div>';
      return;
    }

    const pipelineStatus = statusSnapshot.byPipelineKey?.get(selectedPipelineKey);
    if (!pipelineStatus) {
      controlEventsMetaEl.textContent = "No runtime status found for the selected pipeline.";
      controlEventsListEl.innerHTML =
        '<div class="control-event-empty">No recent runtime events.</div>';
      return;
    }

    const events = Array.isArray(pipelineStatus.recentEvents)
      ? pipelineStatus.recentEvents
      : [];
    controlEventsMetaEl.textContent = [
      `${events.length} recent engine events`,
      `${pipelineStatus.runningCores}/${pipelineStatus.totalCores} cores running`,
      pipelineStatus.summary,
    ].join(" | ");

    if (!events.length) {
      controlEventsListEl.innerHTML =
        '<div class="control-event-empty">No recent runtime events.</div>';
      return;
    }

    controlEventsListEl.innerHTML = events
      .slice(0, 12)
      .map((event) => {
        const kind = event.typeKind || "unknown";
        const typeLabel = escapeHtml(event.typeLabel || "Unknown");
        const coreId = escapeHtml(event.coreId == null ? "n/a" : event.coreId);
        const message = event.message
          ? `<div class="control-event-message">${escapeHtml(event.message)}</div>`
          : "";
        return `
          <div class="control-event-item control-event-item-${kind}">
            <div class="control-event-header">
              <div class="control-event-name">${typeLabel}</div>
              <div class="control-event-time">${escapeHtml(formatRuntimeEventTime(event.time))}</div>
            </div>
            <div class="control-event-meta">
              <span class="control-event-chip control-event-chip-${kind}">${escapeHtml(kind)}</span>
              <span>core ${coreId}</span>
            </div>
            ${message}
          </div>
        `;
      })
      .join("");
  }

  function renderRuntimeStatusOverlay() {
    if (!runtimeStatusTbody || !runtimeStatusMeta) return;
    runtimeStatusTbody.innerHTML = "";

    if (!statusSnapshot) {
      const tr = document.createElement("tr");
      tr.appendChild(createRuntimeStatusCell("No runtime status snapshot.", "runtime-status-state"));
      tr.appendChild(createRuntimeStatusCell("-", "runtime-status-state"));
      tr.appendChild(createRuntimeStatusCell("-", "runtime-status-state"));
      tr.appendChild(createRuntimeStatusCell("-", "runtime-status-state"));
      tr.appendChild(createRuntimeStatusCell("-", "runtime-status-state"));
      tr.appendChild(createRuntimeStatusCell("-", "runtime-status-state"));
      tr.appendChild(
        createRuntimeStatusCell(pollingState.statusLastProbe?.error || "-", "runtime-status-state")
      );
      runtimeStatusTbody.appendChild(tr);
      setRuntimeStatusMetaText("No /status snapshot available yet.");
      return;
    }

    const rows = [...statusSnapshot.rows].sort((a, b) => {
      const severityDelta = getStatusSeverity(b.state) - getStatusSeverity(a.state);
      if (severityDelta !== 0) return severityDelta;
      return a.rawPipelineKey.localeCompare(b.rawPipelineKey, undefined, {
        numeric: true,
        sensitivity: "base",
      });
    });
    if (rows.length === 0) {
      const tr = document.createElement("tr");
      tr.appendChild(createRuntimeStatusCell("No pipelines reported."));
      tr.appendChild(createRuntimeStatusCell("-"));
      tr.appendChild(createRuntimeStatusCell("-"));
      tr.appendChild(createRuntimeStatusCell("-"));
      tr.appendChild(createRuntimeStatusCell("-"));
      tr.appendChild(createRuntimeStatusCell("-"));
      tr.appendChild(createRuntimeStatusCell("-"));
      runtimeStatusTbody.appendChild(tr);
    }
    rows.forEach((row) => {
      const tr = document.createElement("tr");
      tr.className = `runtime-status-row-${row.state}`;
      tr.appendChild(createRuntimeStatusCell(row.rawPipelineKey));
      tr.appendChild(createRuntimeStatusCell(row.acceptedStatus, "runtime-status-state"));
      tr.appendChild(createRuntimeStatusCell(row.readyStatus, "runtime-status-state"));
      tr.appendChild(createRuntimeStatusCell(String(row.runningCores)));
      tr.appendChild(createRuntimeStatusCell(String(row.totalCores)));
      tr.appendChild(createRuntimeStatusCell(row.latestEventSummary || "-"));
      tr.appendChild(createRuntimeStatusCell(row.topReason));
      runtimeStatusTbody.appendChild(tr);
    });

    const parts = [
      `pipelines=${statusSnapshot.total}`,
      `ready=${statusSnapshot.up}`,
      `issues=${statusSnapshot.down}`,
      `unknown=${statusSnapshot.unknown}`,
    ];
    const generatedAt = formatSnapshotTime(statusSnapshot.generatedAt);
    if (generatedAt) parts.push(`generated ${generatedAt}`);
    if (Number.isFinite(pollingState.statusLastProbe?.status)) {
      parts.push(`HTTP ${pollingState.statusLastProbe.status}`);
    }
    if (Number.isFinite(pollingState.statusLastProbe?.latencyMs)) {
      parts.push(`${Math.round(pollingState.statusLastProbe.latencyMs)} ms`);
    }
    if (pollingState.statusLastProbe?.error) {
      parts.push(pollingState.statusLastProbe.error);
    }
    setRuntimeStatusMetaText(parts.join(" | "));
  }

  function openRuntimeStatusOverlay() {
    if (!runtimeStatusOverlay) return;
    runtimeStatusOverlay.classList.remove("hidden");
    runtimeStatusOverlay.setAttribute("aria-hidden", "false");
    renderRuntimeStatusOverlay();
    ensureRecentStatusSnapshot(2500);
  }

  function closeRuntimeStatusOverlay() {
    if (!runtimeStatusOverlay) return;
    runtimeStatusOverlay.classList.add("hidden");
    runtimeStatusOverlay.setAttribute("aria-hidden", "true");
  }

  function ensureRecentStatusSnapshot(maxAgeMs = STATUS_POLL_INTERVAL_MS) {
    if (pollingState.statusFetchInFlight) return;
    const nowMs = Date.now();
    if (!Number.isFinite(pollingState.statusLastCheckedAtMs)) {
      void pollStatusEndpoint();
      return;
    }
    if (nowMs - pollingState.statusLastCheckedAtMs > maxAgeMs) {
      void pollStatusEndpoint();
    }
  }

  function showError(message) {
    errorText.textContent = message;
    errorBanner.classList.remove("hidden");
  }

  function hideError() {
    errorBanner.classList.add("hidden");
    errorText.textContent = "";
  }

  // Keep sticky scroll offsets in sync with dynamic header height.
  function updateStickyPanelOffset() {
    if (!topPanels) return;
    const height = Math.ceil(topPanels.getBoundingClientRect().height);
    const offset = Math.max(0, height + 8);
    document.documentElement.style.setProperty(
      "--sticky-panels-height",
      `${offset}px`
    );
  }

  function initStickyPanels() {
    if (!topPanels) return;
    updateStickyPanelOffset();
    window.addEventListener("resize", updateStickyPanelOffset);
    window.addEventListener("orientationchange", updateStickyPanelOffset);
    if ("ResizeObserver" in window) {
      if (stickyPanelsObserver) {
        stickyPanelsObserver.disconnect();
      }
      stickyPanelsObserver = new ResizeObserver(() => updateStickyPanelOffset());
      stickyPanelsObserver.observe(topPanels);
    }
  }

  // Reset caches/selections that become invalid when scope filters change.
  function resetVisualizationStateForFilterChange() {
    selectedEdgeId = null;
    selectedNodeId = null;
    selectedEdgeData = null;
    selectedNodeData = null;
    controlPlanePrev = null;
    nodeSeries.clear();
    channelSeries.clear();
    pipelineChartsController?.clearSeries();
    lastRenderedStructureSignature = null;
    clearDagRenderedDom();
    destroyNodeCharts();
    clearChannelChart();
    destroyPipelineCharts();
  }

  function clearDagElementCaches() {
    dagNodeElements.clear();
    dagNodePortDotsByNode.clear();
    dagControlIndicatorsByNode.clear();
    dagEdgeElements.clear();
    dagEdgeDefsElement = null;
    lastRenderedNodesById = new Map();
    lastRenderedEdgesById = new Map();
    lastRenderedControlEdgesById = new Map();
  }

  function clearDagRenderedDom() {
    dagNodes.innerHTML = "";
    dagEdges.innerHTML = "";
    dagLanes.innerHTML = "";
    clearDagElementCaches();
  }

  function ensureDagEdgeDefs() {
    if (dagEdgeDefsElement && dagEdgeDefsElement.isConnected) {
      return;
    }
    const defs = document.createElementNS("http://www.w3.org/2000/svg", "defs");
    defs.innerHTML = `
      <marker id="dag-arrow-idle" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto">
        <path d="M 0 0 L 10 5 L 0 10 z" fill="rgba(148,163,184,0.7)"></path>
      </marker>
      <marker id="dag-arrow-active" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto">
        <path d="M 0 0 L 10 5 L 0 10 z" fill="rgba(34,197,94,0.9)"></path>
      </marker>
      <marker id="dag-arrow-error" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto">
        <path d="M 0 0 L 10 5 L 0 10 z" fill="rgba(248,113,113,0.9)"></path>
      </marker>
    `;
    dagEdgeDefsElement = defs;
    dagEdges.prepend(defs);
  }

  function clearDagNavigationOverlayElements() {
    dagNodes.querySelectorAll(".pipeline-dag-nav").forEach((el) => el.remove());
    dagEdges.querySelectorAll(".dag-topic-link").forEach((el) => el.remove());
  }

  function getWindowMs() {
    return windowMinutes * 60 * 1000;
  }

  function getWindowEndMs() {
    if (freezeActive && freezeAnchorMs) {
      return freezeAnchorMs;
    }
    return lastSampleTs ? lastSampleTs.getTime() : Date.now();
  }

  function getDisplayTimeMs() {
    if (freezeActive && freezeTimeMs) return freezeTimeMs;
    if (freezeActive && freezeAnchorMs) return freezeAnchorMs;
    return getWindowEndMs();
  }

  function updateScrubLabel() {
    if (!freezeActive) {
      scrubLabel.textContent = "Live";
      return;
    }
    const ts = new Date(getDisplayTimeMs());
    scrubLabel.textContent = ts.toLocaleTimeString();
  }

  function updateScrubControls() {
    const windowMs = getWindowMs();
    scrubSlider.max = String(windowMs);
    if (freezeActive) {
      scrubSlider.disabled = false;
      const anchor = freezeAnchorMs ?? (lastSampleTs ? lastSampleTs.getTime() : Date.now());
      freezeAnchorMs = anchor;
      if (freezeTimeMs == null) {
        freezeTimeMs = anchor;
      }
      const offset = Math.max(0, Math.min(windowMs, anchor - freezeTimeMs));
      scrubSlider.value = String(windowMs - offset);
      updateScrubLabel();
      return;
    }
    scrubSlider.disabled = true;
    scrubSlider.value = String(windowMs);
    freezeAnchorMs = null;
    freezeTimeMs = null;
    updateScrubLabel();
  }

  // Re-apply activity styling for the currently rendered topology at a specific hover timestamp.
  function updateTopologyForHover(ts) {
    const perfMs = perfStart();
    try {
      if (!lastRenderedEdges.length && !lastRenderedNodes.length) return;
      const edges = lastRenderedEdges;
      const nodes = lastRenderedNodes;
      const controlEdges = lastRenderedControlEdges;
      const sampleSeconds = lastRenderedSampleSeconds ?? lastSampleSeconds;
      const displayTimeMs = Number.isFinite(ts) ? ts : getDisplayTimeMs();
      const dataEdgeRates = computeEdgeRates(edges, displayTimeMs, sampleSeconds);
      const controlEdgeRates = computeEdgeRates(controlEdges, displayTimeMs, sampleSeconds);
      lastEdgeRates = dataEdgeRates;

      const focusSets = buildFocusSets(edges);
      const nodeTraffic = new Map(nodes.map((node) => [node.id, 0]));
      const nodeErrors = new Map(nodes.map((node) => [node.id, 0]));
      edges.forEach((edge) => {
        const rates = dataEdgeRates.get(edge.id);
        if (!rates) return;
        nodeTraffic.set(edge.source, (nodeTraffic.get(edge.source) || 0) + rates.sendRate);
        nodeTraffic.set(edge.target, (nodeTraffic.get(edge.target) || 0) + rates.recvRate);
        nodeErrors.set(edge.source, (nodeErrors.get(edge.source) || 0) + rates.sendErrorRate);
        nodeErrors.set(edge.target, (nodeErrors.get(edge.target) || 0) + rates.recvErrorRate);
      });

      const portScores = new Map();
      edges.forEach((edge) => {
        const rate = dataEdgeRates.get(edge.id)?.sendRate ?? 0;
        if (!portScores.has(edge.source)) {
          portScores.set(edge.source, new Map());
        }
        const nodePorts = portScores.get(edge.source);
        nodePorts.set(edge.port, (nodePorts.get(edge.port) || 0) + rate);
      });

      const controlByTarget = new Map();
      controlEdges.forEach((edge) => {
        const rates = controlEdgeRates.get(edge.id);
        const recvRate = rates?.recvRate ?? 0;
        const entry = controlByTarget.get(edge.target) || {
          total: 0,
          edges: [],
          primary: null,
        };
        entry.total += recvRate;
        entry.edges.push(edge);
        if (
          !entry.primary ||
          (rates?.recvRate ?? 0) >
            (controlEdgeRates.get(entry.primary.id)?.recvRate ?? 0)
        ) {
          entry.primary = edge;
        }
        controlByTarget.set(edge.target, entry);
      });

      nodes.forEach((node) => {
        const nodeEl = dagNodeElements.get(node.id);
        if (!nodeEl) return;
        const hasError = (nodeErrors.get(node.id) || 0) > 0;
        const hasTraffic = (nodeTraffic.get(node.id) || 0) > 0;
        const isActive = metricMode === "errors" ? hasError : hasTraffic;
        nodeEl.classList.toggle("dag-node-active", isActive);
        if (metricMode === "errors" && hasError) {
          nodeEl.style.color = "rgba(248,113,113,0.95)";
          nodeEl.style.borderColor = "rgba(248,113,113,0.9)";
        } else if (metricMode !== "errors" && hasTraffic) {
          nodeEl.style.color = "rgba(34,197,94,0.9)";
          nodeEl.style.borderColor = "rgba(34,197,94,0.9)";
        } else {
          nodeEl.style.color = "";
          nodeEl.style.borderColor = "";
        }
      });

      dagNodePortDotsByNode.forEach((portDots, nodeId) => {
        portDots.forEach((dot, port) => {
          const isActive = (portScores.get(nodeId)?.get(port) ?? 0) > 0;
          dot.classList.toggle("dag-port-dot-active", isActive);
        });
      });

      dagControlIndicatorsByNode.forEach((control, nodeId) => {
        const info = controlByTarget.get(nodeId);
        const rateEl = control.rateEl;
        if (rateEl) {
          rateEl.textContent = formatRateWithUnit(info ? info.total : 0, "msg");
        }
      });

      edges.forEach((edge) => {
        const edgeEntry = dagEdgeElements.get(edge.id);
        const path = edgeEntry?.path;
        const label = edgeEntry?.label;
        if (!path || !label) return;
        const activity =
          dataEdgeRates.get(edge.id) || {
            sendRate: 0,
            recvRate: 0,
            sendErrorRate: 0,
            recvErrorRate: 0,
            errorRate: 0,
            active: false,
            errorActive: false,
          };
        const edgeActive = metricMode === "errors" ? activity.errorActive : activity.active;
        const edgeClass =
          metricMode === "errors"
            ? edgeActive
              ? "dag-edge-error"
              : "dag-edge-idle"
            : edgeActive
              ? "dag-edge-active"
              : "dag-edge-idle";
        path.setAttribute("class", `dag-edge ${edgeClass}`);
        if (selectedEdgeId && edge.id === selectedEdgeId) {
          path.classList.add("dag-edge-selected");
        }
        if (focusSets && !focusSets.edges.has(edge.id)) {
          path.classList.add("dag-dimmed");
        }
        const marker =
          edgeActive && metricMode === "errors"
            ? "url(#dag-arrow-error)"
            : edgeActive
              ? "url(#dag-arrow-active)"
              : "url(#dag-arrow-idle)";
        path.setAttribute("marker-end", marker);

        label.setAttribute(
          "class",
          edgeActive
            ? metricMode === "errors"
              ? "dag-edge-label dag-edge-label-error"
              : "dag-edge-label dag-edge-label-active"
            : "dag-edge-label dag-edge-label-idle"
        );
        if (metricMode === "errors") {
          label.textContent = formatRateWithUnit(activity.errorRate, "error");
        } else {
          label.textContent = formatRateWithUnit(activity.recvRate, "message");
        }
        if (focusSets && !focusSets.edges.has(edge.id)) {
          label.classList.add("dag-dimmed");
        }
      });
    } finally {
      perfEnd("updateTopologyForHover", perfMs, {
        nodes: lastRenderedNodes.length,
        edges: lastRenderedEdges.length,
      });
    }
  }

  function getSeriesWindow(points, startMs, endMs) {
    return getSeriesWindowFromRange(points, startMs, endMs);
  }

  function getPointAtTime(points, ts) {
    return getPointAtTimeFromSeries(points, ts);
  }

  function getChannelPoint(channelId, ts) {
    return getChannelPointFromSeries({
      channelSeries,
      channelId,
      ts,
      endMs: getWindowEndMs(),
      windowMs: getWindowMs(),
      displayTimeMs: getDisplayTimeMs(),
      getSeriesWindowFn: getSeriesWindowFromRange,
      getPointAtTimeFn: getPointAtTimeFromSeries,
    });
  }

  // --- Pipeline/Core selector construction and filtering ---
  function updateCoreSelectionDisplay() {
    updateCoreSelectionDisplayUi({
      coreSelectValue,
      coreSelectSwatch,
      selectedCoreId,
      lastCoreIds,
      lastCoreUsageAvg,
      lastCoreUsage,
      coreAllId: CORE_ALL,
    });
  }

  function updateFilterSelectors(metricSets) {
    const pipelineEntries = new Map();
    metricSets.forEach((set) => {
      const attrs = normalizeAttributes(set.attributes || {});
      const pipelineId = getPipelineId(attrs);
      if (!pipelineId) return;
      const groupId = getPipelineGroupId(attrs);
      const key = makePipelineSelectionKey(groupId, pipelineId);
      if (!pipelineEntries.has(key)) {
        pipelineEntries.set(key, { key, groupId, pipelineId });
      }
    });
    const sortedPipelineEntries = Array.from(pipelineEntries.values()).sort((a, b) => {
      const groupCmp = (a.groupId || "").localeCompare(b.groupId || "", undefined, {
        numeric: true,
        sensitivity: "base",
      });
      if (groupCmp !== 0) return groupCmp;
      return (a.pipelineId || "").localeCompare(b.pipelineId || "", undefined, {
        numeric: true,
        sensitivity: "base",
      });
    });

    if (!sortedPipelineEntries.length) {
      pipelineSelect.innerHTML = '<option value="">n/a</option>';
      pipelineSelect.disabled = true;
      pipelineOptionLabelByKey = new Map();
      selectedPipelineKey = null;
      selectedCoreId = null;
      lastCoreUsageAvg = null;
      lastCoreIds = [];
      updateCoreSelectionDisplay();
      updatePipelineSelectionDisplay();
      if (pipelineSelectBtn) {
        pipelineSelectBtn.disabled = true;
        pipelineSelectBtn.classList.add("opacity-50", "cursor-not-allowed");
      }
      if (pipelineOverlay) {
        pipelineOverlay.classList.add("hidden");
        pipelineOverlay.innerHTML = "";
      }
      coreSelectBtn.disabled = true;
      coreSelectBtn.classList.add("opacity-50", "cursor-not-allowed");
      coreOverlay.classList.add("hidden");
      coreOverlay.innerHTML = "";
      syncInterPipelineTopologyState();
      return;
    }

    const validPipelineKeys = new Set(sortedPipelineEntries.map((entry) => entry.key));
    if (!selectedPipelineKey || !validPipelineKeys.has(selectedPipelineKey)) {
      const orderedKeys = sortedPipelineEntries.map((entry) => entry.key);
      selectedPipelineKey =
        findMostCentralPipelineKey(interPipelineTopology, orderedKeys) ||
        sortedPipelineEntries[0].key;
    }

    const groupedPipelines = new Map();
    sortedPipelineEntries.forEach((entry) => {
      const groupKey = entry.groupId || "";
      if (!groupedPipelines.has(groupKey)) {
        groupedPipelines.set(groupKey, []);
      }
      groupedPipelines.get(groupKey).push(entry);
    });

    pipelineSelect.innerHTML = "";
    pipelineOptionLabelByKey = new Map();
    groupedPipelines.forEach((entries, groupKey) => {
      const optgroup = buildPipelineOptgroupElement(
        pipelineOptgroupTemplate,
        formatPipelineGroupLabel(groupKey)
      );
      entries.forEach((entry) => {
        pipelineOptionLabelByKey.set(entry.key, entry.pipelineId);
        const option = buildPipelineOptionElement(
          pipelineOptionTemplate,
          entry.key,
          entry.pipelineId
        );
        option.dataset.pipelineLabel = entry.pipelineId;
        option.title = getPipelineSelectorStatusTitle(entry.key);
        optgroup.appendChild(option);
      });
      pipelineSelect.appendChild(optgroup);
    });
    pipelineSelect.value = selectedPipelineKey;
    pipelineSelect.disabled = sortedPipelineEntries.length <= 1;
    if (pipelineSelectBtn) {
      pipelineSelectBtn.disabled = sortedPipelineEntries.length <= 1;
      pipelineSelectBtn.classList.toggle("opacity-50", sortedPipelineEntries.length <= 1);
      pipelineSelectBtn.classList.toggle("cursor-not-allowed", sortedPipelineEntries.length <= 1);
    }
    refreshPipelineSelectorStatusDecorations();

    const pipelineFiltered = metricSets
      .map((set) => ({ set, attrs: normalizeAttributes(set.attributes || {}) }))
      .filter(
        (entry) =>
          getPipelineSelectionKeyFromAttrs(entry.attrs) === selectedPipelineKey
      );
    const coreIds = Array.from(
      new Set(pipelineFiltered.map((entry) => entry.attrs["core.id"]).filter((id) => id))
    ).sort((a, b) => Number(a) - Number(b));
    const usageMap = buildCoreUsage(pipelineFiltered.map((entry) => entry.set));
    lastCoreUsage = usageMap;
    lastCoreUsageAvg = averageCoreUsage(usageMap);
    lastCoreIds = coreIds;

    if (!coreIds.length) {
      selectedCoreId = null;
      lastCoreUsageAvg = null;
      lastCoreIds = [];
      updateCoreSelectionDisplay();
      coreSelectBtn.disabled = true;
      coreSelectBtn.classList.add("opacity-50", "cursor-not-allowed");
      renderCoreOverlayUi({
        coreOverlay,
        coreIds: [],
        usageMap,
        overallUsage: null,
        selectedCoreId,
        coreAllId: CORE_ALL,
      });
      syncInterPipelineTopologyState();
      ensureRecentStatusSnapshot();
      return;
    }

    selectedCoreId = resolveSelectedCoreId(selectedCoreId, coreIds, CORE_ALL);

    updateCoreSelectionDisplay();
    coreSelectBtn.disabled = coreIds.length === 0;
    coreSelectBtn.classList.toggle("opacity-50", coreIds.length === 0);
    coreSelectBtn.classList.toggle("cursor-not-allowed", coreIds.length === 0);
    renderCoreOverlayUi({
      coreOverlay,
      coreIds,
      usageMap,
      overallUsage: lastCoreUsageAvg,
      selectedCoreId,
      coreAllId: CORE_ALL,
    });
    syncInterPipelineTopologyState();
    ensureRecentStatusSnapshot();
  }

  function filterMetricSets(metricSets) {
    return filterMetricSetsBySelection(metricSets, {
      selectedPipelineKey,
      selectedCoreId,
      coreAllId: CORE_ALL,
    });
  }

  function selectMetricSetsWithoutAggregation(metricSets) {
    return filterMetricSetsBySelection(metricSets, {
      selectedPipelineKey,
      selectedCoreId: selectedCoreId === CORE_ALL ? null : selectedCoreId,
      coreAllId: CORE_ALL,
    });
  }

  function getDagMetricSets(metricSets, dagScope) {
    return getDagMetricSetsBySelection(metricSets, dagScope, {
      selectedPipelineKey,
      selectedCoreId,
      coreAllId: CORE_ALL,
    });
  }

  function filterZeroMetrics(metrics) {
    if (!hideZeroActivity) return metrics;
    return (metrics || []).filter((metric) => {
      if (typeof metric.value !== "number") return true;
      return metric.value !== 0;
    });
  }

  function matchesQuery(value, query) {
    if (!query) return true;
    if (!value) return false;
    return String(value).toLowerCase().includes(query);
  }

  function filterGraphByQuery(nodes, edges, query) {
    const q = (query || "").trim().toLowerCase();
    if (!q) return { nodes, edges };
    const nodeMatches = new Set();
    nodes.forEach((node) => {
      const type = node.attrs?.["node.type"] || "";
      const ports = (node.outPorts || []).join(" ");
      const nodeLabel = node.displayId || node.id;
      const pipelineId = node.attrs?.["pipeline.id"] || "";
      const hay = `${nodeLabel} ${pipelineId} ${type} ${ports}`.toLowerCase();
      if (matchesQuery(hay, q)) {
        nodeMatches.add(node.id);
      }
    });
    const edgeMatches = new Set();
    edges.forEach((edge) => {
      const channelId = edge.channelDisplayId || edge.channelId || edge.data?.displayId || edge.data?.id || "";
      const source = edge.sourceDisplayId || edge.source;
      const target = edge.targetDisplayId || edge.target;
      const hay = `${edge.id} ${channelId} ${edge.port} ${source} ${target}`.toLowerCase();
      if (matchesQuery(hay, q)) {
        edgeMatches.add(edge.id);
      }
    });
    const keepNodes = new Set(nodeMatches);
    edges.forEach((edge) => {
      if (edgeMatches.has(edge.id)) {
        keepNodes.add(edge.source);
        keepNodes.add(edge.target);
      }
      if (nodeMatches.has(edge.source) || nodeMatches.has(edge.target)) {
        keepNodes.add(edge.source);
        keepNodes.add(edge.target);
      }
    });
    const filteredEdges = edges.filter((edge) => {
      if (!keepNodes.has(edge.source) || !keepNodes.has(edge.target)) return false;
      return (
        edgeMatches.has(edge.id) ||
        nodeMatches.has(edge.source) ||
        nodeMatches.has(edge.target)
      );
    });
    const filteredNodes = nodes.filter((node) => keepNodes.has(node.id));
    return { nodes: filteredNodes, edges: filteredEdges };
  }

  function buildFocusSets(edges) {
    if (!selectedNodeId && !selectedEdgeId) return null;
    const outgoing = new Map();
    const incoming = new Map();
    edges.forEach((edge) => {
      if (!outgoing.has(edge.source)) outgoing.set(edge.source, []);
      if (!incoming.has(edge.target)) incoming.set(edge.target, []);
      outgoing.get(edge.source).push(edge);
      incoming.get(edge.target).push(edge);
    });

    const traverse = (startNodes, edgeMap, nextNode) => {
      const nodes = new Set(startNodes);
      const edgeIds = new Set();
      const queue = [...startNodes];
      while (queue.length) {
        const node = queue.shift();
        const list = edgeMap.get(node) || [];
        list.forEach((edge) => {
          edgeIds.add(edge.id);
          const next = nextNode(edge);
          if (!nodes.has(next)) {
            nodes.add(next);
            queue.push(next);
          }
        });
      }
      return { nodes, edges: edgeIds };
    };

    if (selectedEdgeId) {
      const edge = edges.find((item) => item.id === selectedEdgeId);
      if (!edge) return null;
      const focusNodes = new Set([edge.source, edge.target]);
      const focusEdges = new Set([edge.id]);
      const upstream = traverse([edge.source], incoming, (e) => e.source);
      const downstream = traverse([edge.target], outgoing, (e) => e.target);
      upstream.nodes.forEach((node) => focusNodes.add(node));
      downstream.nodes.forEach((node) => focusNodes.add(node));
      upstream.edges.forEach((id) => focusEdges.add(id));
      downstream.edges.forEach((id) => focusEdges.add(id));
      return { nodes: focusNodes, edges: focusEdges };
    }

    if (selectedNodeId) {
      const upstream = traverse([selectedNodeId], incoming, (e) => e.source);
      const downstream = traverse([selectedNodeId], outgoing, (e) => e.target);
      const focusNodes = new Set([selectedNodeId]);
      const focusEdges = new Set();
      upstream.nodes.forEach((node) => focusNodes.add(node));
      downstream.nodes.forEach((node) => focusNodes.add(node));
      upstream.edges.forEach((id) => focusEdges.add(id));
      downstream.edges.forEach((id) => focusEdges.add(id));
      return { nodes: focusNodes, edges: focusEdges };
    }
    return null;
  }

  // Central render pipeline after data/filter updates.
  function applyFilteredView(metricSets, updateSeries) {
    const perfMs = perfStart();
    try {
      const dagScope = getDagRenderScope();
      const panelMetricSets = filterMetricSets(metricSets);
      const selectedMetricSets = selectMetricSetsWithoutAggregation(metricSets);
      const dagMetricSets = getDagMetricSets(metricSets, dagScope);
      if (updateSeries) {
        updateNodeSeries(dagMetricSets, lastSampleSeconds, lastSampleTs, dagScope);
        updateChannelSeries(dagMetricSets, lastSampleSeconds, lastSampleTs, dagScope);
      }
      const dataGraph = buildGraph(dagMetricSets, lastSampleSeconds, ["pdata"], dagScope);
      const controlGraph = buildGraph(
        dagMetricSets,
        lastSampleSeconds,
        ["control"],
        dagScope
      );
      lastDataGraph = dataGraph;
      lastControlGraph = controlGraph;
      renderGraph(dataGraph, controlGraph);
      if (selectedEdgeData) {
        renderChannelChart(
          selectedEdgeData.channelId || selectedEdgeData.data?.id || selectedEdgeData.id
        );
      }
      const engineSummary = extractEngineSummary(metricSets, {
        skipAllZeroSnapshots: SKIP_ENGINE_ALL_ZERO_SNAPSHOTS,
      });
      updateEngineCards(engineSummary, lastSampleTs);
      const pipelineSummary = extractPipelineSummary(panelMetricSets);
      updatePipelineCards(pipelineSummary, lastSampleSeconds, lastSampleTs);
      const controlPlaneSummary = extractControlPlaneSummary(selectedMetricSets);
      updateControlPlaneCards(controlPlaneSummary, lastSampleSeconds, lastSampleTs);
      renderControlPlaneEventsPanel();
      if (showPipelineCharts) {
        updatePipelineCharts();
      }
      const tokioSummary = extractTokioSummary(panelMetricSets);
      updateTokioCards(tokioSummary, lastSampleSeconds);
    } finally {
      perfEnd("applyFilteredView", perfMs, {
        series: updateSeries ? 1 : 0,
      });
    }
  }

  function clearDagSelectionClasses() {
    dagEdges
      .querySelectorAll(".dag-edge-selected")
      .forEach((el) => el.classList.remove("dag-edge-selected"));
    dagNodes
      .querySelectorAll(".dag-node-selected")
      .forEach((el) => el.classList.remove("dag-node-selected"));
    dagEdges
      .querySelectorAll(".dag-dimmed")
      .forEach((el) => el.classList.remove("dag-dimmed"));
    dagNodes
      .querySelectorAll(".dag-dimmed")
      .forEach((el) => el.classList.remove("dag-dimmed"));
    dagNodes
      .querySelectorAll(".dag-control-indicator-selected")
      .forEach((el) => el.classList.remove("dag-control-indicator-selected"));
  }

  function clearSelection() {
    selectedEdgeId = null;
    selectedNodeId = null;
    selectedEdgeData = null;
    selectedNodeData = null;
    clearDagSelectionClasses();
    renderSelectionNone();
  }

  function selectEdgeById(edgeId, options = {}) {
    const edge = lastRenderedEdgesById.get(edgeId) || lastRenderedControlEdgesById.get(edgeId);
    if (!edge) return;
    clearDagSelectionClasses();
    selectedNodeId = null;
    selectedNodeData = null;
    selectedEdgeId = edge.id;
    selectedEdgeData = edge;

    const edgeEls = dagEdgeElements.get(edge.id);
    if (edgeEls?.path) {
      edgeEls.path.classList.add("dag-edge-selected");
    }
    const controlInfo = dagControlIndicatorsByNode.get(edge.target);
    if (controlInfo?.indicator && controlInfo.edgeId === edge.id) {
      controlInfo.indicator.classList.add("dag-control-indicator-selected");
    }
    renderEdgeDetails(edge);
    if (options.scrollDetails) {
      edgeDetailBody.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }

  function selectNodeById(nodeId) {
    const node = lastRenderedNodesById.get(nodeId);
    if (!node) return;
    clearDagSelectionClasses();
    selectedEdgeId = null;
    selectedEdgeData = null;
    selectedNodeId = node.id;
    selectedNodeData = node;
    const nodeEl = dagNodeElements.get(node.id);
    if (nodeEl) {
      nodeEl.classList.add("dag-node-selected");
    }
    renderNodeDetails(node);
  }

  // --- Summary extraction and card updates ---
  function extractPipelineSummary(metricSets) {
    const pipelineSets = metricSets.filter((ms) => ms.name === "pipeline.metrics");
    const summary = {
      count: pipelineSets.length,
      cpuUtilSum: 0,
      memoryUsageBytes: 0,
      uptimeSeconds: null,
      deltas: {},
      cumulative: {},
    };

    pipelineSets.forEach((set) => {
      (set.metrics || []).forEach((metric) => {
        if (typeof metric.value !== "number" || !Number.isFinite(metric.value)) {
          return;
        }

        if (metric.name === "cpu.utilization") {
          summary.cpuUtilSum += metric.value;
          return;
        }
        if (metric.name === "memory.usage") {
          summary.memoryUsageBytes += metric.value;
          return;
        }
        if (metric.name === "uptime") {
          summary.uptimeSeconds =
            summary.uptimeSeconds == null
              ? metric.value
              : Math.max(summary.uptimeSeconds, metric.value);
          return;
        }

        const isDeltaCounter =
          metric.instrument === "delta_counter" ||
          (typeof metric.instrument === "string" && metric.instrument.includes("delta")) ||
          (typeof metric.name === "string" && metric.name.endsWith(".delta")) ||
          metric.name === "cpu.time";

        if (isDeltaCounter) {
          summary.deltas[metric.name] = (summary.deltas[metric.name] || 0) + metric.value;
          return;
        }

        if (
          metric.name === "context.switches.voluntary" ||
          metric.name === "context.switches.involuntary" ||
          metric.name === "page.faults.minor" ||
          metric.name === "page.faults.major"
        ) {
          summary.cumulative[metric.name] =
            (summary.cumulative[metric.name] || 0) + metric.value;
        }
      });
    });

    return summary;
  }

  function extractTokioSummary(metricSets) {
    const runtimeSets = metricSets.filter((ms) => ms.name === "tokio.runtime");
    const summary = {
      count: runtimeSets.length,
      workerCount: 0,
      activeTasks: 0,
      globalQueue: 0,
      cumulative: {},
    };

    runtimeSets.forEach((set) => {
      (set.metrics || []).forEach((metric) => {
        if (typeof metric.value !== "number" || !Number.isFinite(metric.value)) {
          return;
        }
        if (metric.name === "worker.count") {
          summary.workerCount += metric.value;
          return;
        }
        if (metric.name === "task.active.count") {
          summary.activeTasks += metric.value;
          return;
        }
        if (metric.name === "global.task.queue.size") {
          summary.globalQueue += metric.value;
          return;
        }
        if (
          metric.name === "worker.busy.time" ||
          metric.name === "worker.park.count" ||
          metric.name === "worker.park.unpark.count"
        ) {
          summary.cumulative[metric.name] =
            (summary.cumulative[metric.name] || 0) + metric.value;
        }
      });
    });

    return summary;
  }

  function extractControlPlaneSummary(metricSets) {
    const summary = {
      runtimeInstances: 0,
      completionInstances: 0,
      runtimeGauges: {
        "drain.active": 0,
        "drain.pending_receivers": 0,
        "pending_sends.buffered": 0,
        "timers.active": 0,
        "telemetry_timers.active": 0,
        "delayed_data.queued": 0,
      },
      runtimeCounters: {
        "shutdown.deadline_forced": 0,
        "delay_data.returned_during_drain": 0,
        "timer_tick.sent": 0,
        "collect_telemetry.sent": 0,
        "delayed_data.sent": 0,
      },
      completionGauges: {
        "pending_sends.buffered": 0,
      },
      completionCounters: {
        "deliver_ack.received": 0,
        "deliver_nack.received": 0,
        "ack.attempted": 0,
        "nack.attempted": 0,
        "ack.delivered": 0,
        "nack.delivered": 0,
        "ack.dropped_no_interest": 0,
        "nack.dropped_no_interest": 0,
      },
    };

    metricSets.forEach((set) => {
      const metrics = Array.isArray(set.metrics) ? set.metrics : [];
      if (set.name === "pipeline.runtime_control") {
        summary.runtimeInstances += 1;
        metrics.forEach((metric) => {
          if (typeof metric.value !== "number" || !Number.isFinite(metric.value)) {
            return;
          }
          if (metric.name in summary.runtimeGauges) {
            summary.runtimeGauges[metric.name] += metric.value;
            return;
          }
          if (metric.name in summary.runtimeCounters) {
            summary.runtimeCounters[metric.name] += metric.value;
          }
        });
      } else if (set.name === "pipeline.completion") {
        summary.completionInstances += 1;
        metrics.forEach((metric) => {
          if (typeof metric.value !== "number" || !Number.isFinite(metric.value)) {
            return;
          }
          if (metric.name in summary.completionGauges) {
            summary.completionGauges[metric.name] += metric.value;
            return;
          }
          if (metric.name in summary.completionCounters) {
            summary.completionCounters[metric.name] += metric.value;
          }
        });
      }
    });

    summary.instanceCount = Math.max(
      summary.runtimeInstances,
      summary.completionInstances
    );
    return summary;
  }

  function updateControlPlaneCards(summary, sampleSeconds, ts) {
    const setText = (el, value) => {
      if (el) {
        el.textContent = value;
      }
    };

    if (
      !summary ||
      (summary.runtimeInstances === 0 && summary.completionInstances === 0)
    ) {
      setText(controlDrainStateEl, "n/a");
      setText(controlDrainPendingReceiversEl, "n/a");
      setText(controlDrainInstanceCountEl, "0");
      setText(controlDrainDeadlineForcedEl, "n/a");
      setText(controlRuntimePendingSendsEl, "n/a");
      setText(controlCompletionPendingSendsEl, "n/a");
      setText(controlDelayedDataQueuedEl, "n/a");
      setText(controlTimersActiveEl, "n/a");
      setText(controlDelayedDataSentRateEl, "n/a");
      setText(controlTimerTickRateEl, "n/a");
      setText(controlCollectTelemetryRateEl, "n/a");
      setText(controlDelayReturnRateEl, "n/a");
      setText(controlDeliverAckRateEl, "n/a");
      setText(controlDeliverNackRateEl, "n/a");
      setText(controlAckAttemptRateEl, "n/a");
      setText(controlNackAttemptRateEl, "n/a");
      setText(controlAckDeliveredRateEl, "n/a");
      setText(controlNackDeliveredRateEl, "n/a");
      setText(controlAckDroppedEl, "n/a");
      setText(controlNackDroppedEl, "n/a");
      controlPlanePrev = null;
      return;
    }

    const runtimeGauges = summary.runtimeGauges;
    const runtimeCounters = summary.runtimeCounters;
    const completionGauges = summary.completionGauges;
    const completionCounters = summary.completionCounters;
    const drainActive = runtimeGauges["drain.active"] || 0;
    const pendingReceivers = runtimeGauges["drain.pending_receivers"] || 0;
    const runtimePendingSends = runtimeGauges["pending_sends.buffered"] || 0;
    const completionPendingSends = completionGauges["pending_sends.buffered"] || 0;
    const delayedDataQueued = runtimeGauges["delayed_data.queued"] || 0;
    const timersActive = runtimeGauges["timers.active"] || 0;

    setText(
      controlDrainStateEl,
      summary.instanceCount > 0
        ? `${Math.round(drainActive)}/${summary.instanceCount}`
        : String(Math.round(drainActive))
    );
    setText(controlDrainPendingReceiversEl, Math.round(pendingReceivers).toString());
    setText(controlDrainInstanceCountEl, String(summary.instanceCount || 0));
    setText(
      controlDrainDeadlineForcedEl,
      Math.round(runtimeCounters["shutdown.deadline_forced"] || 0).toString()
    );
    setText(controlRuntimePendingSendsEl, Math.round(runtimePendingSends).toString());
    setText(
      controlCompletionPendingSendsEl,
      Math.round(completionPendingSends).toString()
    );
    setText(controlDelayedDataQueuedEl, Math.round(delayedDataQueued).toString());
    setText(controlTimersActiveEl, Math.round(timersActive).toString());
    setText(
      controlAckDroppedEl,
      Math.round(completionCounters["ack.dropped_no_interest"] || 0).toString()
    );
    setText(
      controlNackDroppedEl,
      Math.round(completionCounters["nack.dropped_no_interest"] || 0).toString()
    );

    const previousRuntime = controlPlanePrev?.runtimeCounters || null;
    const previousCompletion = controlPlanePrev?.completionCounters || null;
    const delayedDataSentRate = calcCumulativeRate(
      runtimeCounters["delayed_data.sent"],
      previousRuntime ? previousRuntime["delayed_data.sent"] : null,
      sampleSeconds
    );
    const timerTickRate = calcCumulativeRate(
      runtimeCounters["timer_tick.sent"],
      previousRuntime ? previousRuntime["timer_tick.sent"] : null,
      sampleSeconds
    );
    const collectTelemetryRate = calcCumulativeRate(
      runtimeCounters["collect_telemetry.sent"],
      previousRuntime ? previousRuntime["collect_telemetry.sent"] : null,
      sampleSeconds
    );
    const delayReturnRate = calcCumulativeRate(
      runtimeCounters["delay_data.returned_during_drain"],
      previousRuntime ? previousRuntime["delay_data.returned_during_drain"] : null,
      sampleSeconds
    );
    const deliverAckRate = calcCumulativeRate(
      completionCounters["deliver_ack.received"],
      previousCompletion ? previousCompletion["deliver_ack.received"] : null,
      sampleSeconds
    );
    const deliverNackRate = calcCumulativeRate(
      completionCounters["deliver_nack.received"],
      previousCompletion ? previousCompletion["deliver_nack.received"] : null,
      sampleSeconds
    );
    const ackAttemptRate = calcCumulativeRate(
      completionCounters["ack.attempted"],
      previousCompletion ? previousCompletion["ack.attempted"] : null,
      sampleSeconds
    );
    const nackAttemptRate = calcCumulativeRate(
      completionCounters["nack.attempted"],
      previousCompletion ? previousCompletion["nack.attempted"] : null,
      sampleSeconds
    );
    const ackDeliveredRate = calcCumulativeRate(
      completionCounters["ack.delivered"],
      previousCompletion ? previousCompletion["ack.delivered"] : null,
      sampleSeconds
    );
    const nackDeliveredRate = calcCumulativeRate(
      completionCounters["nack.delivered"],
      previousCompletion ? previousCompletion["nack.delivered"] : null,
      sampleSeconds
    );

    setText(controlDelayedDataSentRateEl, formatRate(delayedDataSentRate));
    setText(controlTimerTickRateEl, formatRate(timerTickRate));
    setText(controlCollectTelemetryRateEl, formatRate(collectTelemetryRate));
    setText(controlDelayReturnRateEl, formatRate(delayReturnRate));
    setText(controlDeliverAckRateEl, formatRate(deliverAckRate));
    setText(controlDeliverNackRateEl, formatRate(deliverNackRate));
    setText(controlAckAttemptRateEl, formatRate(ackAttemptRate));
    setText(controlNackAttemptRateEl, formatRate(nackAttemptRate));
    setText(controlAckDeliveredRateEl, formatRate(ackDeliveredRate));
    setText(controlNackDeliveredRateEl, formatRate(nackDeliveredRate));

    controlPlanePrev = {
      runtimeCounters: { ...runtimeCounters },
      completionCounters: { ...completionCounters },
    };

    const timestamp = ts || lastSampleTs;
    recordPipelineMetric("control.drain.active", drainActive, timestamp);
    recordPipelineMetric(
      "control.drain.pending_receivers",
      pendingReceivers,
      timestamp
    );
    recordPipelineMetric(
      "control.runtime.pending_sends.buffered",
      runtimePendingSends,
      timestamp
    );
    recordPipelineMetric(
      "control.completion.pending_sends.buffered",
      completionPendingSends,
      timestamp
    );
    recordPipelineMetric("control.delayed_data.queued", delayedDataQueued, timestamp);
    recordPipelineMetric(
      "control.runtime.delayed_data.sent.rate",
      delayedDataSentRate,
      timestamp
    );
    recordPipelineMetric(
      "control.runtime.timer_tick.sent.rate",
      timerTickRate,
      timestamp
    );
    recordPipelineMetric(
      "control.runtime.collect_telemetry.sent.rate",
      collectTelemetryRate,
      timestamp
    );
    recordPipelineMetric(
      "control.completion.deliver_ack.received.rate",
      deliverAckRate,
      timestamp
    );
    recordPipelineMetric(
      "control.completion.deliver_nack.received.rate",
      deliverNackRate,
      timestamp
    );
    recordPipelineMetric(
      "control.completion.ack.delivered.rate",
      ackDeliveredRate,
      timestamp
    );
    recordPipelineMetric(
      "control.completion.nack.delivered.rate",
      nackDeliveredRate,
      timestamp
    );

    if (pipelineHoverTs != null) {
      applyPipelineMetricValues(pipelineHoverTs);
    }
  }

  function updateEngineCards(summary, ts) {
    const values = deriveEngineCardValues(
      summary,
      {
        lastCpuUtilPercent: lastEngineCpuUtilPercent,
        lastMemoryRssMiB: lastEngineMemoryRssMiB,
        lastUptimeSeconds: lastEngineUptimeSeconds,
      },
      { holdLastValues: HOLD_LAST_ENGINE_VALUES }
    );
    lastEngineCpuUtilPercent = values.lastCpuUtilPercent;
    lastEngineMemoryRssMiB = values.lastMemoryRssMiB;
    lastEngineUptimeSeconds = values.lastUptimeSeconds;
    engineGroupCountEl.textContent = String(values.groupCount);
    enginePipelineCountEl.textContent = String(values.pipelineCount);
    engineCoreCountEl.textContent = String(values.coreCount);
    engineCpuUtilEl.textContent =
      values.cpuUtilPercent == null ? "n/a" : `${values.cpuUtilPercent.toFixed(1)}%`;
    engineMemoryRssEl.textContent =
      values.memoryRssMiB == null ? "n/a" : `${values.memoryRssMiB.toFixed(1)} MiB`;
    engineUptimeEl.textContent = formatDurationSeconds(values.uptimeSeconds);

    const timestamp = ts || lastSampleTs;
    if (Number.isFinite(values.coreCount)) {
      recordPipelineMetric("engine.core.count", values.coreCount, timestamp);
    }
    if (Number.isFinite(values.currentCpuUtilPercent)) {
      recordPipelineMetric("engine.cpu.utilization", values.currentCpuUtilPercent, timestamp);
    }
    if (Number.isFinite(values.currentMemoryRssMiB)) {
      recordPipelineMetric("engine.memory.rss", values.currentMemoryRssMiB, timestamp);
    }
    if (Number.isFinite(values.currentUptimeSeconds)) {
      recordPipelineMetric("engine.uptime", values.currentUptimeSeconds, timestamp);
    }

    if (pipelineHoverTs != null) {
      applyPipelineMetricValues(pipelineHoverTs);
    }
  }

  function updatePipelineCards(summary, sampleSeconds, ts) {
    if (!summary || summary.count === 0) {
      pipeInstancesEl.textContent = "0";
      pipeCpuUtilEl.textContent = "n/a";
      pipeCpuCoresEl.textContent = "n/a";
      pipeCpuTimeEl.textContent = "n/a";
      pipeUptimeEl.textContent = "n/a";
      pipeMemUsageEl.textContent = "n/a";
      pipeAllocRateEl.textContent = "n/a";
      pipeFreeRateEl.textContent = "n/a";
      pipeNetAllocRateEl.textContent = "n/a";
      pipeCtxVolRateEl.textContent = "n/a";
      pipeCtxInvolRateEl.textContent = "n/a";
      pipeFaultMinorRateEl.textContent = "n/a";
      pipeFaultMajorRateEl.textContent = "n/a";
      pipelinePrev = null;
      return;
    }

    const avgCpuUtil = summary.count ? summary.cpuUtilSum / summary.count : 0;
    const cpuUtilPercent = avgCpuUtil * 100;
    pipeCpuUtilEl.textContent =
      Number.isFinite(cpuUtilPercent) ? cpuUtilPercent.toFixed(1) : "n/a";
    pipeInstancesEl.textContent = String(summary.count || 0);
    pipeCpuCoresEl.textContent =
      Number.isFinite(summary.cpuUtilSum) ? summary.cpuUtilSum.toFixed(3) : "n/a";
    pipeUptimeEl.textContent = formatDurationSeconds(summary.uptimeSeconds);

    const memMiB =
      Number.isFinite(summary.memoryUsageBytes) && summary.memoryUsageBytes >= 0
        ? summary.memoryUsageBytes / (1024 * 1024)
        : null;
    pipeMemUsageEl.textContent = memMiB == null ? "n/a" : memMiB.toFixed(1);

    const cpuTimeRate = calcRate(summary.deltas["cpu.time"], sampleSeconds);
    const allocRate = calcRate(summary.deltas["memory.allocated.delta"], sampleSeconds);
    const freeRate = calcRate(summary.deltas["memory.freed.delta"], sampleSeconds);
    const netRate =
      allocRate == null || freeRate == null ? null : Math.max(allocRate - freeRate, 0);

    pipeCpuTimeEl.textContent = cpuTimeRate == null ? "n/a" : cpuTimeRate.toFixed(3);
    pipeAllocRateEl.textContent =
      allocRate == null ? "n/a" : (allocRate / (1024 * 1024)).toFixed(2);
    pipeFreeRateEl.textContent =
      freeRate == null ? "n/a" : (freeRate / (1024 * 1024)).toFixed(2);
    pipeNetAllocRateEl.textContent =
      netRate == null ? "n/a" : (netRate / (1024 * 1024)).toFixed(2);

    const prevCounters = pipelinePrev ? pipelinePrev.counters : null;
    const currentCounters = summary.cumulative || {};
    const ctxVolRate = calcCumulativeRate(
      currentCounters["context.switches.voluntary"],
      prevCounters ? prevCounters["context.switches.voluntary"] : null,
      sampleSeconds
    );
    const ctxInvolRate = calcCumulativeRate(
      currentCounters["context.switches.involuntary"],
      prevCounters ? prevCounters["context.switches.involuntary"] : null,
      sampleSeconds
    );
    const pfMinorRate = calcCumulativeRate(
      currentCounters["page.faults.minor"],
      prevCounters ? prevCounters["page.faults.minor"] : null,
      sampleSeconds
    );
    const pfMajorRate = calcCumulativeRate(
      currentCounters["page.faults.major"],
      prevCounters ? prevCounters["page.faults.major"] : null,
      sampleSeconds
    );

    pipeCtxVolRateEl.textContent = ctxVolRate == null ? "n/a" : ctxVolRate.toFixed(1);
    pipeCtxInvolRateEl.textContent = ctxInvolRate == null ? "n/a" : ctxInvolRate.toFixed(1);
    pipeFaultMinorRateEl.textContent = pfMinorRate == null ? "n/a" : pfMinorRate.toFixed(1);
    pipeFaultMajorRateEl.textContent = pfMajorRate == null ? "n/a" : pfMajorRate.toFixed(3);

    pipelinePrev = { counters: { ...currentCounters } };

    const timestamp = ts || lastSampleTs;
    recordPipelineMetric("cpu.utilization", cpuUtilPercent, timestamp);
    recordPipelineMetric("cpu.time.rate", cpuTimeRate, timestamp);
    recordPipelineMetric("uptime", summary.uptimeSeconds, timestamp);
    recordPipelineMetric("memory.usage", memMiB, timestamp);
    recordPipelineMetric(
      "memory.alloc.rate",
      allocRate == null ? null : allocRate / (1024 * 1024),
      timestamp
    );
    recordPipelineMetric(
      "memory.free.rate",
      freeRate == null ? null : freeRate / (1024 * 1024),
      timestamp
    );
    recordPipelineMetric(
      "memory.net.rate",
      netRate == null ? null : netRate / (1024 * 1024),
      timestamp
    );
    recordPipelineMetric("context.switches.involuntary", ctxInvolRate, timestamp);
    recordPipelineMetric("context.switches.voluntary", ctxVolRate, timestamp);
    recordPipelineMetric("page.faults.minor", pfMinorRate, timestamp);
    recordPipelineMetric("page.faults.major", pfMajorRate, timestamp);

    if (pipelineHoverTs != null) {
      applyPipelineMetricValues(pipelineHoverTs);
    }
  }

  function updateTokioCards(summary, sampleSeconds) {
    if (!summary || summary.count === 0) {
      tokioWorkerCountEl.textContent = "n/a";
      tokioBusyRateEl.textContent = "n/a";
      tokioInstanceCountEl.textContent = "0";
      tokioActiveTasksEl.textContent = "n/a";
      tokioQueueSizeEl.textContent = "n/a";
      tokioParkRateEl.textContent = "n/a";
      tokioUnparkRateEl.textContent = "n/a";
      tokioPrev = null;
      return;
    }

    tokioInstanceCountEl.textContent = String(summary.count || 0);
    tokioWorkerCountEl.textContent =
      Number.isFinite(summary.workerCount) ? summary.workerCount.toFixed(0) : "n/a";
    tokioActiveTasksEl.textContent =
      Number.isFinite(summary.activeTasks) ? summary.activeTasks.toFixed(0) : "n/a";
    tokioQueueSizeEl.textContent =
      Number.isFinite(summary.globalQueue) ? summary.globalQueue.toFixed(0) : "n/a";

    const prevCounters = tokioPrev ? tokioPrev.counters : null;
    const currentCounters = summary.cumulative || {};
    const busyRate = calcCumulativeRate(
      currentCounters["worker.busy.time"],
      prevCounters ? prevCounters["worker.busy.time"] : null,
      sampleSeconds
    );
    const parkRate = calcCumulativeRate(
      currentCounters["worker.park.count"],
      prevCounters ? prevCounters["worker.park.count"] : null,
      sampleSeconds
    );
    const unparkRate = calcCumulativeRate(
      currentCounters["worker.park.unpark.count"],
      prevCounters ? prevCounters["worker.park.unpark.count"] : null,
      sampleSeconds
    );

    tokioBusyRateEl.textContent = busyRate == null ? "n/a" : busyRate.toFixed(3);
    tokioParkRateEl.textContent = parkRate == null ? "n/a" : parkRate.toFixed(1);
    tokioUnparkRateEl.textContent = unparkRate == null ? "n/a" : unparkRate.toFixed(1);

    tokioPrev = { counters: { ...currentCounters } };
  }

  // --- DAG layout and topology rendering ---
  function getNodeOutputAnchorY(node, portName) {
    return getLayoutNodeOutputAnchorY(node, portName, GRAPH_LAYOUT_CONSTANTS);
  }

  function layoutGraph(nodes, edges) {
    return computeLayoutGraph(nodes, edges, {
      interPipelineTopology,
      selectedPipelineKey,
      constants: GRAPH_LAYOUT_CONSTANTS,
    });
  }

  function formatValue(value) {
    if (value == null) return "n/a";
    if (typeof value === "number") {
      if (!Number.isFinite(value)) return "n/a";
      if (Math.abs(value) >= 1000 || Number.isInteger(value)) {
        return value.toLocaleString();
      }
      return value.toFixed(3);
    }
    return String(value);
  }

  function formatRate(value) {
    if (value == null || !Number.isFinite(value)) return "n/a";
    if (value >= 100) return `${value.toFixed(0)}/s`;
    if (value >= 10) return `${value.toFixed(1)}/s`;
    return `${value.toFixed(2)}/s`;
  }

  function formatRateWithUnit(rate, unit) {
    const formatted = formatRate(rate);
    if (formatted === "n/a") return formatted;
    const cleanUnit = normalizeUnit(unit);
    if (!cleanUnit || cleanUnit === "1") return formatted;
    if (isBytesUnit(cleanUnit)) {
      const bytes = formatBytes(rate);
      return bytes === "n/a" ? "n/a" : `${bytes}/s`;
    }
    return formatted.replace("/s", ` ${cleanUnit}/s`);
  }

  function formatRateUnitLabel(unit) {
    const cleanUnit = normalizeUnit(unit);
    if (!cleanUnit || cleanUnit === "1") return "";
    const base = isBytesUnit(cleanUnit) ? "B" : cleanUnit;
    return `${base}/s`;
  }

  function formatWindowLabel() {
    if (windowMinutes === 60) return "1 hour";
    return `${windowMinutes} minute${windowMinutes === 1 ? "" : "s"}`;
  }

  function resolveRateScale(unit, points) {
    const cleanUnit = normalizeUnit(unit);
    if (!cleanUnit || cleanUnit === "1") {
      return { divisor: 1, label: "" };
    }
    if (isBytesUnit(cleanUnit)) {
      const values = points.map((point) => point.value).filter((value) => Number.isFinite(value));
      let max = values.length ? Math.max(...values.map((value) => Math.abs(value))) : 0;
      const units = ["B", "KB", "MB", "GB", "TB"];
      let divisor = 1;
      let idx = 0;
      while (max >= 1024 && idx < units.length - 1) {
        max /= 1024;
        divisor *= 1024;
        idx += 1;
      }
      return { divisor, label: `${units[idx]}/s` };
    }
    return { divisor: 1, label: `${cleanUnit}/s` };
  }

  function formatAxisTickValue(value, scale) {
    if (!Number.isFinite(value)) return "";
    const divisor = scale && Number.isFinite(scale.divisor) ? scale.divisor : 1;
    const scaled = value / divisor;
    if (!Number.isFinite(scaled)) return "";
    if (scaled >= 100) return scaled.toFixed(0);
    if (scaled >= 10) return scaled.toFixed(1);
    return scaled.toFixed(2);
  }

  function normalizeUnit(unit) {
    return unit ? String(unit).replace(/[{}]/g, "").trim() : "";
  }

  function isBytesUnit(unit) {
    const value = String(unit || "").toLowerCase();
    return value === "by" || value === "byte" || value === "bytes" || value === "b";
  }

  function formatBytes(value) {
    if (!Number.isFinite(value)) return "n/a";
    const sign = value < 0 ? "-" : "";
    let size = Math.abs(value);
    const units = ["B", "KB", "MB", "GB", "TB"];
    let idx = 0;
    while (size >= 1024 && idx < units.length - 1) {
      size /= 1024;
      idx += 1;
    }
    const precision = size >= 100 ? 0 : size >= 10 ? 1 : 2;
    return `${sign}${size.toFixed(precision)} ${units[idx]}`;
  }

  function formatValueWithUnit(value, unit) {
    const cleanUnit = normalizeUnit(unit);
    if (isBytesUnit(cleanUnit)) {
      return formatBytes(value);
    }
    if (!cleanUnit || cleanUnit === "1") return formatValue(value);
    return `${formatValue(value)} ${cleanUnit}`;
  }

  function shouldShowNodeRate(metric) {
    if (!metric || !Number.isFinite(metric.value)) return false;
    if (!Number.isFinite(lastSampleSeconds) || lastSampleSeconds <= 0) return false;
    const unit = String(metric.unit || "").toLowerCase();
    const timeUnits = new Set(["s", "ms", "us", "ns", "min", "h", "hr"]);
    if (timeUnits.has(unit) || unit.includes("sec")) return false;
    return isDeltaCounterMetric(metric);
  }

  function renderNodeMetricTable(metrics, nodeId, setName) {
    if (!metrics || !metrics.length) {
      return '<div class="text-slate-400">No metrics.</div>';
    }
    const endMs = getWindowEndMs();
    const startMs = endMs - getWindowMs();
    const nonRate = [];
    const rateNonZero = [];
    const rateZero = [];
    metrics.forEach((metric) => {
      const showRate = shouldShowNodeRate(metric);
      if (!showRate || !nodeId || !setName) {
        if (hideZeroActivity && typeof metric.value === "number" && metric.value === 0) {
          return;
        }
        nonRate.push({ metric, showRate: false });
        return;
      }
      const metricKey = buildNodeMetricKey(setName, metric.name);
      const points = getNodeMetricPoints(nodeId, metricKey, startMs, endMs);
      const hasValues = points.length > 0;
      const allZero =
        hasValues &&
        points.every((point) => !Number.isFinite(point.value) || point.value === 0);
      const isZero =
        allZero ||
        (!hasValues && Number.isFinite(metric.value) && metric.value === 0);
      if (hideZeroActivity && isZero) {
        return;
      }
      const scale = resolveRateScale(metric.unit, points);
      const entry = { metric, showRate: true, points, scale };
      if (isZero) {
        rateZero.push(entry);
      } else {
        rateNonZero.push(entry);
      }
    });
    const ordered = [...nonRate, ...rateNonZero, ...rateZero];
    return `
      <div class="grid gap-1">
        ${ordered
          .map((entry) => {
            const metric = entry.metric;
            const baseValue = formatValueWithUnit(metric.value, metric.unit);
            if (entry.showRate && nodeId && setName) {
              const unitLabel = entry.scale.label || formatRateUnitLabel(metric.unit);
              const brief = metric.brief ? String(metric.brief) : "";
              const safeMetricName = escapeHtml(metric.name);
              const safeUnitLabel = escapeHtml(unitLabel);
              const safeBrief = escapeHtml(brief);
              const safeNodeId = escapeAttr(nodeId);
              const safeSetName = escapeAttr(setName);
              const safeMetricNameAttr = escapeAttr(metric.name);
              const safeUnit = escapeAttr(normalizeUnit(metric.unit));
              return `
                <div class="node-metric-rate-block">
                  <div class="node-metric-rate-title">
                    <span>${safeMetricName}${unitLabel ? ` <span class="node-metric-rate-unit">(${safeUnitLabel})</span>` : ""}</span>
                  </div>
                  ${brief ? `<div class="node-metric-rate-brief">${safeBrief}</div>` : ""}
                  <div class="node-metric-rate-chart" data-node-id="${safeNodeId}" data-set-name="${safeSetName}" data-metric-name="${safeMetricNameAttr}" data-unit="${safeUnit}">
                    <canvas></canvas>
                    <div class="node-metric-rate-overlay hidden"></div>
                  </div>
                </div>`;
            }
            const valueLabel = baseValue;
            const safeMetricName = escapeHtml(metric.name);
            const safeValueLabel = escapeHtml(valueLabel);
            return `
              <div class="flex items-start justify-between gap-3">
                <span class="text-slate-300">${safeMetricName}</span>
                <span class="font-mono text-slate-200">${safeValueLabel}</span>
              </div>`;
          })
          .join("")}
      </div>
    `;
  }

  function buildNodeSummary(nodeId) {
    const edges = lastGraph?.edges || [];
    if (lastEdgeRates && lastEdgeRates.size) {
      let inRate = 0;
      let outRate = 0;
      let errorRate = 0;
      edges.forEach((edge) => {
        const rates = lastEdgeRates.get(edge.id);
        if (!rates) return;
        if (edge.source === nodeId) {
          outRate += rates.sendRate || 0;
          errorRate += rates.sendErrorRate || 0;
        }
        if (edge.target === nodeId) {
          inRate += rates.recvRate || 0;
          errorRate += rates.recvErrorRate || 0;
        }
      });
      return {
        inRate,
        outRate,
        errorRate,
        inCount: null,
        outCount: null,
      };
    }

    let inCount = 0;
    let outCount = 0;
    let inErrors = 0;
    let outErrors = 0;
    edges.forEach((edge) => {
      if (edge.source === nodeId) {
        const senderMetrics = metricMap(edge.data.sender?.metrics || []);
        outCount += senderMetrics["send.count"] ?? 0;
        outErrors +=
          (senderMetrics["send.error_full"] ?? 0) +
          (senderMetrics["send.error_closed"] ?? 0);
      }
      if (edge.target === nodeId) {
        const receiverMetrics = metricMap(edge.data.receiver?.metrics || []);
        inCount += receiverMetrics["recv.count"] ?? 0;
        inErrors +=
          (receiverMetrics["recv.error_empty"] ?? 0) +
          (receiverMetrics["recv.error_closed"] ?? 0);
      }
    });
    return {
      inRate: calcRate(inCount, lastSampleSeconds),
      outRate: calcRate(outCount, lastSampleSeconds),
      errorRate: calcRate(inErrors + outErrors, lastSampleSeconds),
      inCount,
      outCount,
    };
  }

  function calcRate(deltaValue, sampleSeconds) {
    if (!Number.isFinite(sampleSeconds) || sampleSeconds <= 0) return null;
    if (!Number.isFinite(deltaValue)) return null;
    return deltaValue / sampleSeconds;
  }

  function calcCumulativeRate(currentValue, prevValue, sampleSeconds) {
    if (!Number.isFinite(sampleSeconds) || sampleSeconds <= 0) return null;
    if (!Number.isFinite(currentValue) || !Number.isFinite(prevValue)) return null;
    const delta = currentValue - prevValue;
    if (!Number.isFinite(delta) || delta < 0) return null;
    return delta / sampleSeconds;
  }

  function formatDurationSeconds(totalSeconds) {
    if (!Number.isFinite(totalSeconds) || totalSeconds < 0) {
      return "n/a";
    }
    const seconds = Math.floor(totalSeconds % 60);
    const minutes = Math.floor((totalSeconds / 60) % 60);
    const hours = Math.floor((totalSeconds / 3600) % 24);
    const days = Math.floor(totalSeconds / 86400);

    const hh = String(hours).padStart(2, "0");
    const mm = String(minutes).padStart(2, "0");
    const ss = String(seconds).padStart(2, "0");
    if (days > 0) {
      return `${days}d ${hh}:${mm}:${ss}`;
    }
    return `${hh}:${mm}:${ss}`;
  }

  function renderAttributes(attrs, filterFn) {
    const entries = Object.entries(attrs || {}).filter(([key]) =>
      filterFn ? filterFn(key) : true
    );
    if (!entries.length) {
      return '<div class="text-slate-400">No attributes.</div>';
    }
    entries.sort(([a], [b]) => a.localeCompare(b));
    return entries
      .map(
        ([key, value]) => {
          const safeKey = escapeHtml(key);
          const safeValue = escapeHtml(value);
          return `<div class="flex items-start justify-between gap-3"><span class="text-slate-300">${safeKey}</span><span class="font-mono text-slate-200">${safeValue}</span></div>`;
        }
      )
      .join("");
  }

  function renderMetricTable(metrics) {
    const filtered = filterZeroMetrics(metrics || []);
    if (!filtered.length) {
      return '<div class="text-slate-400">No metrics.</div>';
    }
    return `
      <div class="grid gap-1">
        ${filtered
          .map(
            (metric) => {
              const safeMetricName = escapeHtml(metric.name);
              const safeValue = escapeHtml(formatValueWithUnit(metric.value, metric.unit));
              return `
              <div class="flex items-start justify-between gap-3">
                <span class="text-slate-300">${safeMetricName}</span>
                <span class="font-mono text-slate-200">${safeValue}</span>
              </div>`;
            }
          )
          .join("")}
      </div>
    `;
  }

  function clearChannelChart() {
    selectionDetailsController.clearChannelChart();
  }

  function buildNodeMetricKey(setName, metricName) {
    return `${setName}::${metricName}`;
  }

  function getNodeMetricPoints(nodeId, metricKey, startMs, endMs) {
    const entry = nodeSeries.get(nodeId);
    if (!entry) return [];
    const series = entry.metrics.get(metricKey);
    if (!series) return [];
    if (startMs != null && endMs != null) {
      return getSeriesWindow(series.points, startMs, endMs);
    }
    return series.points;
  }

  function destroyNodeCharts() {
    nodeCharts.forEach((chart) => {
      if (chart._nodeHoverHandlers) {
        const { move, leave, canvas } = chart._nodeHoverHandlers;
        canvas.removeEventListener("mousemove", move);
        canvas.removeEventListener("mouseleave", leave);
        chart._nodeHoverHandlers = null;
      }
      chart.destroy();
    });
    nodeCharts.clear();
    nodeHoverTs = null;
  }

  function formatScaledRate(value, scale, unit) {
    if (!Number.isFinite(value)) return "n/a";
    if (scale && scale.label) {
      const scaled = formatAxisTickValue(value, scale);
      return scaled ? `${scaled} ${scale.label}` : "n/a";
    }
    return formatRateWithUnit(value, unit);
  }

  function getChartThemeColors() {
    const isDay = document.body.classList.contains("day-mode");
    return {
      grid: isDay ? "rgba(148,163,184,0.45)" : "rgba(55,65,81,0.4)",
      tick: isDay ? "#475569" : "#9ca3af",
    };
  }

  function applyChartTheme() {
    const theme = getChartThemeColors();
    selectionDetailsController?.applyChannelChartTheme(theme);
    nodeCharts.forEach((chart) => {
      chart.options.scales.x.ticks.color = theme.tick;
      chart.options.scales.y.ticks.color = theme.tick;
      chart.options.scales.x.grid.color = theme.grid;
      chart.options.scales.y.grid.color = theme.grid;
      chart.update("none");
    });
    pipelineChartsController?.applyTheme(theme);
  }

  const pipelineHoverPlugin = {
    id: "pipelineHover",
    afterDraw(chart) {
      const idx = chart._hoverIndex;
      if (idx == null) return;
      const xScale = chart.scales.x;
      const yScale = chart.scales.y;
      if (!xScale || !yScale) return;
      const label = chart.data.labels ? chart.data.labels[idx] : null;
      const x = xScale.getPixelForValue(label ?? idx);
      const { top, bottom } = chart.chartArea;
      const theme = getChartThemeColors();
      const ctx = chart.ctx;
      ctx.save();
      ctx.strokeStyle = theme.tick;
      ctx.globalAlpha = 0.6;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x, top);
      ctx.lineTo(x, bottom);
      ctx.stroke();
      chart.data.datasets.forEach((dataset) => {
        const value = dataset.data?.[idx];
        if (!Number.isFinite(value)) return;
        const y = yScale.getPixelForValue(value);
        ctx.fillStyle = dataset.borderColor || theme.tick;
        ctx.globalAlpha = 0.85;
        ctx.beginPath();
        ctx.arc(x, y, 3, 0, Math.PI * 2);
        ctx.fill();
      });
      ctx.restore();
    },
  };

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

  const PIPELINE_METRIC_DISPLAY = {
    "engine.core.count": {
      el: engineCoreCountEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(0) : "n/a"),
    },
    "engine.cpu.utilization": {
      el: engineCpuUtilEl,
      format: (value) => (Number.isFinite(value) ? `${value.toFixed(1)}%` : "n/a"),
    },
    "engine.memory.rss": {
      el: engineMemoryRssEl,
      format: (value) => (Number.isFinite(value) ? `${value.toFixed(1)} MiB` : "n/a"),
    },
    "engine.uptime": {
      el: engineUptimeEl,
      format: (value) => (Number.isFinite(value) ? formatDurationSeconds(value) : "n/a"),
    },
    "cpu.utilization": {
      el: pipeCpuUtilEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(1) : "n/a"),
    },
    "cpu.time.rate": {
      el: pipeCpuTimeEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(3) : "n/a"),
    },
    uptime: {
      el: pipeUptimeEl,
      format: (value) => (Number.isFinite(value) ? formatDurationSeconds(value) : "n/a"),
    },
    "memory.usage": {
      el: pipeMemUsageEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(1) : "n/a"),
    },
    "memory.alloc.rate": {
      el: pipeAllocRateEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(2) : "n/a"),
    },
    "memory.free.rate": {
      el: pipeFreeRateEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(2) : "n/a"),
    },
    "memory.net.rate": {
      el: pipeNetAllocRateEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(2) : "n/a"),
    },
    "context.switches.involuntary": {
      el: pipeCtxInvolRateEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(1) : "n/a"),
    },
    "context.switches.voluntary": {
      el: pipeCtxVolRateEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(1) : "n/a"),
    },
    "page.faults.minor": {
      el: pipeFaultMinorRateEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(1) : "n/a"),
    },
    "page.faults.major": {
      el: pipeFaultMajorRateEl,
      format: (value) => (Number.isFinite(value) ? value.toFixed(3) : "n/a"),
    },
    "control.drain.pending_receivers": {
      el: controlDrainPendingReceiversEl,
      format: (value) =>
        Number.isFinite(value) ? `${Math.round(value)}` : "n/a",
    },
    "control.runtime.pending_sends.buffered": {
      el: controlRuntimePendingSendsEl,
      format: (value) =>
        Number.isFinite(value) ? `${Math.round(value)}` : "n/a",
    },
    "control.completion.pending_sends.buffered": {
      el: controlCompletionPendingSendsEl,
      format: (value) =>
        Number.isFinite(value) ? `${Math.round(value)}` : "n/a",
    },
    "control.delayed_data.queued": {
      el: controlDelayedDataQueuedEl,
      format: (value) =>
        Number.isFinite(value) ? `${Math.round(value)}` : "n/a",
    },
    "control.runtime.delayed_data.sent.rate": {
      el: controlDelayedDataSentRateEl,
      format: (value) => formatRate(value),
    },
    "control.runtime.timer_tick.sent.rate": {
      el: controlTimerTickRateEl,
      format: (value) => formatRate(value),
    },
    "control.runtime.collect_telemetry.sent.rate": {
      el: controlCollectTelemetryRateEl,
      format: (value) => formatRate(value),
    },
    "control.completion.deliver_ack.received.rate": {
      el: controlDeliverAckRateEl,
      format: (value) => formatRate(value),
    },
    "control.completion.deliver_nack.received.rate": {
      el: controlDeliverNackRateEl,
      format: (value) => formatRate(value),
    },
    "control.completion.ack.delivered.rate": {
      el: controlAckDeliveredRateEl,
      format: (value) => formatRate(value),
    },
    "control.completion.nack.delivered.rate": {
      el: controlNackDeliveredRateEl,
      format: (value) => formatRate(value),
    },
  };

  pipelineChartsController = createPipelineChartsController({
    pipelineSeries,
    pipelineCharts,
    maxWindowMs: MAX_WINDOW_MS,
    pipelineChartConfig: PIPELINE_CHART_CONFIG,
    pipelineMetricDisplay: PIPELINE_METRIC_DISPLAY,
    getWindowEndMs,
    getWindowMs,
    getDisplayTimeMs,
    getSeriesWindow,
    getPointAtTime,
    getChartThemeColors,
    pipelineHoverPlugin,
    onGlobalHover: (ts) => {
      setGlobalHover(ts);
    },
    getGlobalHoverTs: () => globalHoverTs,
  });

  function recordPipelineMetric(metricKey, value, ts) {
    pipelineChartsController.recordMetric(metricKey, value, ts);
  }

  function setPipelineHover(ts) {
    pipelineHoverTs = pipelineChartsController.setHover(ts);
  }

  selectionDetailsController = createSelectionDetailsController({
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
    getFreezeActive: () => freezeActive,
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
    getLastSampleSeconds: () => lastSampleSeconds,
    getLastEdgeRates: () => lastEdgeRates,
    getGlobalHoverTs: () => globalHoverTs,
    getChannelChart: () => channelChart,
    setChannelChart: (chart) => {
      channelChart = chart;
    },
    getChannelChartId: () => channelChartId,
    setChannelChartId: (channelId) => {
      channelChartId = channelId;
    },
  });

  function setChannelHover(ts) {
    selectionDetailsController.setChannelHover(ts);
  }

  // Synchronize hover time across all chart types and topology highlighting.
  function setGlobalHover(ts) {
    const next = Number.isFinite(ts) ? ts : null;
    if (globalHoverTs === next) return;
    globalHoverTs = next;
    setPipelineHover(next);
    setNodeHover(next);
    setChannelHover(next);
    updateTopologyForHover(next);
  }

  function setNodeHover(ts) {
    nodeHoverTs = Number.isFinite(ts) ? ts : null;
    nodeCharts.forEach((chart) => {
      const overlay = chart._overlayEl;
      const labels = chart._tsLabels || [];
      if (!overlay || !labels.length || nodeHoverTs == null) {
        if (overlay) overlay.classList.add("hidden");
        chart._hoverIndex = null;
        chart.draw();
        return;
      }
      const idx = getClosestIndex(labels, nodeHoverTs);
      chart._hoverIndex = idx;
      const value = chart.data.datasets?.[0]?.data?.[idx];
      overlay.textContent = formatScaledRate(value, chart._rateScale, chart._rateUnit);
      overlay.classList.remove("hidden");
      chart.draw();
    });
  }

  function applyPipelineMetricValues(ts) {
    pipelineChartsController.applyMetricValues(ts);
  }

  function updatePipelineCharts() {
    if (!showPipelineCharts) return;
    pipelineChartsController.updateCharts();
  }

  // Show/hide and lifecycle-manage top-card charts.
  function togglePipelineCharts(show) {
    pipelineChartsController.toggleCharts(show);
  }

  function destroyPipelineCharts() {
    pipelineChartsController.destroyCharts();
    pipelineHoverTs = pipelineChartsController.getHoverTs();
  }

  // Initialize detail charts embedded in the node details panel.
  function initNodeRateCharts() {
    destroyNodeCharts();
    nodeHoverTs = null;
    const charts = edgeDetailBody.querySelectorAll(".node-metric-rate-chart");
    const endMs = getWindowEndMs();
    const startMs = endMs - getWindowMs();
    charts.forEach((container) => {
      const canvas = container.querySelector("canvas");
      if (!canvas || !window.Chart) return;
      const nodeId = container.dataset.nodeId;
      const setName = container.dataset.setName;
      const metricName = container.dataset.metricName;
      const unit = container.dataset.unit || "";
      if (!nodeId || !setName || !metricName) return;
      const metricKey = buildNodeMetricKey(setName, metricName);
      const points = getNodeMetricPoints(nodeId, metricKey, startMs, endMs);
      const tsLabels = points.map((point) => point.ts);
      const labels = tsLabels.map((ts) => new Date(ts).toLocaleTimeString());
      const data = points.map((point) => point.value);
      const scale = resolveRateScale(unit, points);
      const theme = getChartThemeColors();
      const chart = new Chart(canvas.getContext("2d"), {
        type: "line",
        data: {
          labels,
          datasets: [
            {
              data,
              borderWidth: 2,
              tension: 0.25,
              borderColor: "rgba(34,197,94,0.9)",
              pointRadius: 0,
            },
          ],
        },
        plugins: [pipelineHoverPlugin],
        options: {
          responsive: true,
          maintainAspectRatio: false,
          animation: false,
          interaction: {
            mode: "index",
            intersect: false,
          },
          plugins: {
            legend: { display: false },
            tooltip: { enabled: false },
          },
          scales: {
            x: {
              ticks: { color: theme.tick, maxTicksLimit: 6 },
              grid: { color: theme.grid },
            },
            y: {
              beginAtZero: true,
              ticks: {
                color: theme.tick,
                callback: (val) => formatAxisTickValue(val, scale),
              },
              grid: { color: theme.grid },
            },
          },
        },
      });
      chart._tsLabels = tsLabels;
      chart._rateScale = scale;
      chart._rateUnit = unit;
      nodeCharts.set(`${nodeId}::${metricKey}`, chart);

      const overlay = container.querySelector(".node-metric-rate-overlay");
      if (overlay) {
        chart._overlayEl = overlay;
        const move = (event) => {
          const elements = chart.getElementsAtEventForMode(
            event,
            "index",
            { intersect: false },
            false
          );
          if (!elements.length) return;
          const idx = elements[0].index;
          const ts = chart._tsLabels ? chart._tsLabels[idx] : null;
          if (!Number.isFinite(ts)) return;
          if (nodeHoverTs !== ts) {
            setGlobalHover(ts);
          }
        };
        const leave = () => {
          setGlobalHover(null);
        };
        canvas.addEventListener("mousemove", move);
        canvas.addEventListener("mouseleave", leave);
        chart._nodeHoverHandlers = { move, leave, canvas };
      }
    });
    if (globalHoverTs != null) {
      setGlobalHover(globalHoverTs);
    }
  }

  // Append rate samples for node-scoped metrics and trim old points.
  function updateNodeSeries(metricSets, sampleSeconds, ts, dagScope = null) {
    updateNodeSeriesFromMetrics({
      metricSets,
      sampleSeconds,
      ts,
      dagScope,
      nodeSeries,
      maxWindowMs: MAX_WINDOW_MS,
      shouldShowNodeRate,
      resolveScopedNodeId,
      normalizeAttributes,
      buildNodeMetricKey,
      perfStart,
      perfEnd,
    });
  }

  // Build per-channel send/recv/error series for edge detail charts and rate computation.
  function updateChannelSeries(metricSets, sampleSeconds, ts, dagScope = null) {
    updateChannelSeriesFromMetrics({
      metricSets,
      sampleSeconds,
      ts,
      dagScope,
      channelSeries,
      maxWindowMs: MAX_WINDOW_MS,
      resolveScopedChannelId,
      normalizeAttributes,
      perfStart,
      perfEnd,
    });
  }

  function renderChannelChart(channelId) {
    selectionDetailsController.renderChannelChart(channelId);
  }

  function renderSelectionNone() {
    selectionDetailsController.renderSelectionNone();
  }

  function renderEdgeDetails(edge) {
    selectionDetailsController.renderEdgeDetails(edge);
  }

  function renderNodeDetails(node) {
    selectionDetailsController.renderNodeDetails(node);
  }

  function metricMap(metrics) {
    const out = {};
    metrics.forEach((metric) => {
      if (typeof metric.value !== "number" || !Number.isFinite(metric.value)) return;
      out[metric.name] = metric.value;
    });
    return out;
  }

  function computeEdgeRates(edges, displayTimeMs, sampleSeconds) {
    return computeEdgeRatesFromSeries({
      edges,
      displayTimeMs,
      sampleSeconds,
      channelSeries,
      getWindowEndMs,
      getWindowMs,
      getDisplayTimeMs,
      calcRate,
      metricMap,
      getSeriesWindowFn: getSeriesWindowFromRange,
      getPointAtTimeFn: getPointAtTimeFromSeries,
    });
  }

  // --- Interaction wiring (zoom, drag, fullscreen, DAG scope/search) ---
  dagInteractionController = createDagInteractionController({
    dagCanvas,
    dagZoom,
    dagViewport,
    zoomOutBtn,
    zoomInBtn,
    zoomResetBtn,
    zoomValueEl,
    fullscreenBtn,
    dagScopeBtn,
    dagSearch,
    zoomMin: ZOOM_MIN,
    zoomMax: ZOOM_MAX,
    zoomFitPadding: ZOOM_FIT_PADDING,
    zoomButtonStep: ZOOM_BUTTON_STEP,
    wheelZoomSensitivity: WHEEL_ZOOM_SENSITIVITY,
    dagDragThresholdPx: DAG_DRAG_THRESHOLD_PX,
    getLayoutSize: () => layoutSize,
    onSearchQueryChange: (query) => {
      dagSearchQuery = query;
      applyFilteredView(lastMetricSets, false);
    },
    onDagScopeToggle: () => {
      const nextMode =
        dagPipelineScopeMode === DAG_SCOPE_CONNECTED
          ? DAG_SCOPE_SINGLE
          : DAG_SCOPE_CONNECTED;
      setDagPipelineScopeMode(nextMode, { rerender: true });
    },
    onFullscreenToggle: (enabled) => {
      if (!enabled && dagPipelineScopeMode === DAG_SCOPE_CONNECTED) {
        setDagPipelineScopeMode(DAG_SCOPE_SINGLE, { rerender: false });
        applyFilteredView(lastMetricSets, false);
      }
      updateDagScopeButtonState();
      updateStickyPanelOffset();
    },
  });

  if (viewSelect) {
    viewSelect.addEventListener("change", () => setActiveTab(viewSelect.value));
  }
  setActiveTab("general");

  if (modeSelect) {
    modeSelect.addEventListener("change", () => setMetricMode(modeSelect.value));
  }
  setMetricMode("throughput");

  windowSelect.addEventListener("change", () => {
    const next = Number(windowSelect.value);
    if (Number.isFinite(next) && next > 0) {
      windowMinutes = next;
    }
    updateScrubControls();
    applyFilteredView(lastMetricSets, false);
  });

  scrubToggle.addEventListener("click", () => {
    freezeActive = !freezeActive;
    scrubToggle.textContent = freezeActive ? "Unfreeze" : "Freeze";
    if (freezeActive) {
      freezeAnchorMs = lastSampleTs ? lastSampleTs.getTime() : Date.now();
      freezeTimeMs = freezeAnchorMs;
    } else {
      freezeAnchorMs = null;
      freezeTimeMs = null;
    }
    updateScrubControls();
    applyFilteredView(lastMetricSets, false);
  });

  scrubSlider.addEventListener("input", () => {
    if (!freezeActive) return;
    const windowMs = getWindowMs();
    const value = Number(scrubSlider.value);
    const anchor = freezeAnchorMs ?? getWindowEndMs();
    freezeAnchorMs = anchor;
    freezeTimeMs = anchor - (windowMs - value);
    updateScrubLabel();
    applyFilteredView(lastMetricSets, false);
  });

  updateScrubControls();

  if (healthReadyz) {
    healthReadyz.addEventListener("click", (event) => {
      event.stopPropagation();
      if (isRuntimeStatusOverlayOpen()) {
        closeRuntimeStatusOverlay();
      } else {
        openRuntimeStatusOverlay();
      }
    });
  }
  if (runtimeStatusCloseBtn) {
    runtimeStatusCloseBtn.addEventListener("click", (event) => {
      event.stopPropagation();
      closeRuntimeStatusOverlay();
    });
  }
  if (runtimeStatusRefreshBtn) {
    runtimeStatusRefreshBtn.addEventListener("click", (event) => {
      event.stopPropagation();
      void pollStatusEndpoint();
    });
  }
  if (runtimeStatusPanel) {
    runtimeStatusPanel.addEventListener("click", (event) => {
      event.stopPropagation();
    });
  }

  if (pipelineSelectBtn) {
    pipelineSelectBtn.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      if (pipelineSelectBtn.disabled || !pipelineOverlay) return;
      pipelineOverlay.classList.toggle("hidden");
    });
  }

  if (pipelineOverlay) {
    pipelineOverlay.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      const button = event.target.closest(".pipeline-overlay-option");
      if (!button) return;
      const pipelineKey = button.dataset.pipelineKey;
      if (!pipelineKey || pipelineKey === selectedPipelineKey) {
        closePipelineOverlay();
        return;
      }
      closePipelineOverlay();
      navigateToPipeline(pipelineKey);
    });
  }

  pipelineSelect.addEventListener("change", () => {
    navigateToPipeline(pipelineSelect.value || null);
  });

  coreSelectBtn.addEventListener("click", (event) => {
    event.stopPropagation();
    if (coreSelectBtn.disabled) return;
    coreOverlay.classList.toggle("hidden");
  });

  coreOverlay.addEventListener("click", (event) => {
    const btn = event.target.closest(".core-cell");
    if (!btn) return;
    selectedCoreId = btn.dataset.coreId || null;
    updateCoreSelectionDisplay();
    resetVisualizationStateForFilterChange();
    coreOverlay.classList.add("hidden");
    updateFilterSelectors(lastMetricSets);
    applyFilteredView(lastMetricSets, false);
  });

  document.addEventListener("click", (event) => {
    if (!coreSelector.contains(event.target)) {
      coreOverlay.classList.add("hidden");
    }
    if (pipelineSelector && !pipelineSelector.contains(event.target)) {
      closePipelineOverlay();
    }
    if (runtimeStatusAnchor && !runtimeStatusAnchor.contains(event.target)) {
      closeRuntimeStatusOverlay();
    }
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      if (pipelineOverlay && !pipelineOverlay.classList.contains("hidden")) {
        closePipelineOverlay();
      }
      if (isRuntimeStatusOverlayOpen()) {
        closeRuntimeStatusOverlay();
      }
    }
  });

  zeroToggle.addEventListener("change", () => {
    hideZeroActivity = zeroToggle.checked;
    setToggleVisualState({
      wrapEl: zeroToggleWrap,
      trackEl: zeroToggleTrack,
      active: hideZeroActivity,
    });
    applyFilteredView(lastMetricSets, false);
  });

  hideZeroActivity = Boolean(zeroToggle?.checked);
  setToggleVisualState({
    wrapEl: zeroToggleWrap,
    trackEl: zeroToggleTrack,
    active: hideZeroActivity,
  });

  if (controlToggle) {
    controlToggle.addEventListener("change", () => {
      showControlChannels = controlToggle.checked;
      setToggleVisualState({
        wrapEl: controlToggleWrap,
        trackEl: controlToggleTrack,
        active: showControlChannels,
      });
      setToggleVisualState({
        active: showControlChannels,
        textEl: controlToggleText,
        activeTextClass: "text-sky-300",
      });
      clearSelection();
      applyFilteredView(lastMetricSets, false);
    });
    showControlChannels = controlToggle.checked;
    setToggleVisualState({
      wrapEl: controlToggleWrap,
      trackEl: controlToggleTrack,
      active: showControlChannels,
    });
    setToggleVisualState({
      active: showControlChannels,
      textEl: controlToggleText,
      activeTextClass: "text-sky-300",
    });
  }

  if (pipelineChartToggle) {
    pipelineChartToggle.addEventListener("change", () => {
      showPipelineCharts = pipelineChartToggle.checked;
      setToggleVisualState({
        wrapEl: pipelineChartToggleWrap,
        trackEl: pipelineChartToggleTrack,
        active: showPipelineCharts,
      });
      togglePipelineCharts(showPipelineCharts);
      if (!showPipelineCharts) {
        destroyPipelineCharts();
      }
    });
    showPipelineCharts = pipelineChartToggle.checked;
    setToggleVisualState({
      wrapEl: pipelineChartToggleWrap,
      trackEl: pipelineChartToggleTrack,
      active: showPipelineCharts,
    });
    togglePipelineCharts(showPipelineCharts);
    if (!showPipelineCharts) {
      destroyPipelineCharts();
    }
  }

  if (themeToggle) {
    themeToggle.addEventListener("change", () => {
      const theme = themeToggle.checked ? "day" : "night";
      localStorage.setItem(THEME_STORAGE_KEY, theme);
      applyTheme(theme);
    });
  }
  const storedTheme = localStorage.getItem(THEME_STORAGE_KEY);
  applyTheme(storedTheme === "day" ? "day" : "night");
  initStickyPanels();

  updateDagScopeButtonState();

  dagViewport.addEventListener("click", (event) => {
    if (dagInteractionController.consumeViewportClickSuppression()) {
      return;
    }
    if (
      event.target.closest(".dag-node") ||
      event.target.closest(".dag-edge-hit")
    ) {
      return;
    }
    if (selectedEdgeId || selectedNodeId) {
      clearSelection();
    }
  });

  dagNodes.addEventListener("click", (event) => {
    const controlIndicator = event.target.closest(".dag-control-indicator");
    if (controlIndicator && dagNodes.contains(controlIndicator)) {
      event.stopPropagation();
      const controlEdgeId = controlIndicator.dataset.controlEdge || "";
      if (controlEdgeId) {
        selectEdgeById(controlEdgeId);
      }
      return;
    }

    const nodeEl = event.target.closest(".dag-node");
    if (!nodeEl || !dagNodes.contains(nodeEl)) {
      return;
    }
    event.stopPropagation();
    const nodeId = nodeEl.dataset.nodeId || "";
    if (!nodeId) return;
    selectNodeById(nodeId);
  });

  dagEdges.addEventListener("click", (event) => {
    const hit = event.target.closest(".dag-edge-hit");
    if (!hit || !dagEdges.contains(hit)) {
      return;
    }
    event.stopPropagation();
    const edgeId = hit.dataset.edgeId || "";
    if (!edgeId) return;
    selectEdgeById(edgeId, { scrollDetails: true });
  });

  function upsertDagNodeElement(node, options) {
    const { controlInfo, portScores, nodeTraffic, nodeErrors, focusSets } = options;
    let nodeEl = dagNodeElements.get(node.id);
    if (!nodeEl) {
      nodeEl = document.createElement("div");
      nodeEl.className = "dag-node";
      nodeEl.dataset.nodeId = node.id;
      dagNodeElements.set(node.id, nodeEl);
    }

    nodeEl.style.left = `${node.x}px`;
    nodeEl.style.top = `${node.y}px`;
    nodeEl.style.height = `${node.height}px`;
    nodeEl.style.width = `${node.width}px`;
    nodeEl.className = "dag-node";
    nodeEl.style.color = "";
    nodeEl.style.borderColor = "";

    if (metricMode === "errors") {
      if ((nodeErrors.get(node.id) || 0) > 0) {
        nodeEl.classList.add("dag-node-active");
        nodeEl.style.color = "rgba(248,113,113,0.95)";
        nodeEl.style.borderColor = "rgba(248,113,113,0.9)";
      }
    } else if ((nodeTraffic.get(node.id) || 0) > 0) {
      nodeEl.classList.add("dag-node-active");
      nodeEl.style.color = "rgba(34,197,94,0.9)";
      nodeEl.style.borderColor = "rgba(34,197,94,0.9)";
    }
    if (selectedNodeId && node.id === selectedNodeId) {
      nodeEl.classList.add("dag-node-selected");
    }
    if (focusSets && !focusSets.nodes.has(node.id)) {
      nodeEl.classList.add("dag-dimmed");
    }

    const visiblePorts = node.displayPorts || node.outPorts || [];
    const controlEdgeId = showControlChannels ? controlInfo?.primary?.id || "" : "";
    const renderKey = `${node.displayId || node.id}|${visiblePorts.join(",")}|${controlEdgeId}`;
    if (nodeEl.dataset.renderKey !== renderKey) {
      const safeNodeIdAttr = escapeAttr(node.id);
      const portRows = visiblePorts.length
        ? `<div>${visiblePorts
            .map((port) => {
              const isActive = (portScores.get(node.id)?.get(port) ?? 0) > 0;
              const safePortLabel = escapeHtml(port);
              const safePortAttr = escapeAttr(port);
              return `
                <div class="dag-port">
                  <span class="dag-port-label">${safePortLabel}</span>
                  <span class="dag-port-dot ${isActive ? "dag-port-dot-active" : ""}" data-node-id="${safeNodeIdAttr}" data-port="${safePortAttr}"></span>
                </div>`;
            })
            .join("")}</div>`
        : "";

      const controlHtml =
        showControlChannels && controlInfo
          ? `<button class="dag-control-indicator ${selectedEdgeId === controlInfo.primary?.id ? "dag-control-indicator-selected" : ""}" data-control-edge="${escapeAttr(controlInfo.primary?.id || "")}" data-node-id="${safeNodeIdAttr}">
              <svg class="dag-control-arrow" viewBox="0 0 10 22" aria-hidden="true">
                <line x1="5" y1="0" x2="5" y2="14" stroke="currentColor" stroke-width="2" stroke-linecap="round"></line>
                <path d="M 0 14 L 10 14 L 5 22 z" fill="currentColor"></path>
              </svg>
              <span class="dag-control-rate">${formatRateWithUnit(controlInfo.total, "msg")}</span>
            </button>`
          : "";

      const safeNodeDisplayId = escapeHtml(node.displayId || node.id);
      nodeEl.innerHTML = `
        ${controlHtml}
        <div class="dag-node-header">
          <div class="dag-node-id">${safeNodeDisplayId}</div>
        </div>
        ${portRows}
      `;
      nodeEl.dataset.renderKey = renderKey;
    }

    const portDots = new Map();
    nodeEl.querySelectorAll(".dag-port-dot").forEach((dot) => {
      const port = dot.dataset.port;
      if (!port) return;
      portDots.set(port, dot);
    });
    dagNodePortDotsByNode.set(node.id, portDots);

    const controlIndicator = nodeEl.querySelector(".dag-control-indicator");
    if (controlIndicator) {
      const rateEl = controlIndicator.querySelector(".dag-control-rate");
      controlIndicator.classList.toggle(
        "dag-control-indicator-selected",
        controlEdgeId && controlEdgeId === selectedEdgeId
      );
      dagControlIndicatorsByNode.set(node.id, {
        indicator: controlIndicator,
        rateEl,
        edgeId: controlIndicator.dataset.controlEdge || "",
      });
    } else {
      dagControlIndicatorsByNode.delete(node.id);
    }

    dagNodes.appendChild(nodeEl);
  }

  function pruneRemovedDagNodes(validNodeIds) {
    for (const [nodeId, nodeEl] of dagNodeElements) {
      if (validNodeIds.has(nodeId)) continue;
      nodeEl.remove();
      dagNodeElements.delete(nodeId);
      dagNodePortDotsByNode.delete(nodeId);
      dagControlIndicatorsByNode.delete(nodeId);
    }
  }

  function upsertDagEdgeElement(edge, source, target, options) {
    const { focusSets, dataEdgeRates } = options;
    let entry = dagEdgeElements.get(edge.id);
    if (!entry) {
      const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
      path.dataset.edgeId = edge.id;
      path.dataset.edgeRole = "path";

      const label = document.createElementNS("http://www.w3.org/2000/svg", "text");
      label.dataset.edgeId = edge.id;
      label.dataset.edgeRole = "label";

      const hit = document.createElementNS("http://www.w3.org/2000/svg", "path");
      hit.setAttribute("class", "dag-edge-hit");
      hit.dataset.edgeId = edge.id;

      entry = { path, label, hit };
      dagEdgeElements.set(edge.id, entry);
    }

    const activity =
      dataEdgeRates.get(edge.id) || {
        sendRate: 0,
        recvRate: 0,
        sendErrorRate: 0,
        recvErrorRate: 0,
        errorRate: 0,
        active: false,
        errorActive: false,
      };
    const edgeActive = metricMode === "errors" ? activity.errorActive : activity.active;
    const recvRate = activity.recvRate ?? 0;

    const startX = source.x + source.width - EDGE_INSET;
    const startY = getNodeOutputAnchorY(source, edge.port);
    const endX = target.x - EDGE_INSET;
    const endY = target.y + target.height / 2;
    const curvature = Math.min(120, Math.max(60, (endX - startX) * 0.4));
    const pathData = `M ${startX} ${startY} C ${startX + curvature} ${startY}, ${endX - curvature} ${endY}, ${endX} ${endY}`;

    const edgeClass =
      metricMode === "errors"
        ? edgeActive
          ? "dag-edge-error"
          : "dag-edge-idle"
        : edgeActive
          ? "dag-edge-active"
          : "dag-edge-idle";
    entry.path.setAttribute("d", pathData);
    entry.path.setAttribute("class", `dag-edge ${edgeClass}`);
    if (selectedEdgeId && edge.id === selectedEdgeId) {
      entry.path.classList.add("dag-edge-selected");
    }
    if (focusSets && !focusSets.edges.has(edge.id)) {
      entry.path.classList.add("dag-dimmed");
    }
    const marker =
      edgeActive && metricMode === "errors"
        ? "url(#dag-arrow-error)"
        : edgeActive
          ? "url(#dag-arrow-active)"
          : "url(#dag-arrow-idle)";
    entry.path.setAttribute("marker-end", marker);

    entry.label.setAttribute("x", endX - 10);
    entry.label.setAttribute("y", endY - 8);
    entry.label.setAttribute("text-anchor", "end");
    entry.label.setAttribute(
      "class",
      edgeActive
        ? metricMode === "errors"
          ? "dag-edge-label dag-edge-label-error"
          : "dag-edge-label dag-edge-label-active"
        : "dag-edge-label dag-edge-label-idle"
    );
    if (metricMode === "errors") {
      entry.label.textContent = formatRateWithUnit(activity.errorRate, "error");
    } else {
      entry.label.textContent = formatRateWithUnit(recvRate, "message");
    }
    if (focusSets && !focusSets.edges.has(edge.id)) {
      entry.label.classList.add("dag-dimmed");
    }

    entry.hit.setAttribute("d", pathData);
    entry.hit.dataset.edgeId = edge.id;

    dagEdges.appendChild(entry.path);
    dagEdges.appendChild(entry.label);
    dagEdges.appendChild(entry.hit);
  }

  function pruneRemovedDagEdges(validEdgeIds) {
    for (const [edgeId, entry] of dagEdgeElements) {
      if (validEdgeIds.has(edgeId)) continue;
      entry.path.remove();
      entry.label.remove();
      entry.hit.remove();
      dagEdgeElements.delete(edgeId);
    }
  }

  // Main DAG renderer for data and control edge layers.
  function renderGraph(dataGraph, controlGraph) {
    const result = renderGraphFrame({
      dataGraph,
      controlGraph,
      perfStart,
      perfEnd,
      metricMode,
      hideZeroActivity,
      dagSearchQuery,
      showControlChannels,
      selectedEdgeId,
      selectedEdgeData,
      selectedNodeId,
      selectedNodeData,
      lastRenderedStructureSignature,
      getDisplayTimeMs,
      computeEdgeRates,
      filterGraphByQuery,
      getDagRenderScope,
      updateTopologyForHover,
      layoutGraph,
      collectPipelineDagAnchors,
      computePipelineDagNavLayout,
      applyDefaultOverviewZoom: dagInteractionController.applyDefaultOverviewZoom,
      applyZoom: dagInteractionController.applyZoom,
      ensureDagEdgeDefs,
      buildFocusSets,
      pruneRemovedDagNodes,
      upsertDagNodeElement,
      pruneRemovedDagEdges,
      upsertDagEdgeElement,
      clearDagNavigationOverlayElements,
      renderConnectedTopicNavigation,
      renderPipelineDagNavigation,
      renderEdgeDetails,
      renderNodeDetails,
      renderSelectionNone,
      dagCanvas,
      dagEdges,
      dagNodes,
      dagLanes,
      dagEmpty,
      layoutSize,
      NODE_WIDTH,
      MARGIN,
      LEVEL_GAP,
      onOverlayError: (error) => {
        console.error("Connected topic overlay render failed", error);
      },
    });

    selectedEdgeId = result.selectedEdgeId;
    selectedEdgeData = result.selectedEdgeData;
    selectedNodeId = result.selectedNodeId;
    selectedNodeData = result.selectedNodeData;
    lastRenderedStructureSignature = result.lastRenderedStructureSignature;
    lastRenderedNodes = result.lastRenderedNodes;
    lastRenderedEdges = result.lastRenderedEdges;
    lastRenderedControlEdges = result.lastRenderedControlEdges;
    lastRenderedNodesById = result.lastRenderedNodesById;
    lastRenderedEdgesById = result.lastRenderedEdgesById;
    lastRenderedControlEdgesById = result.lastRenderedControlEdgesById;
    lastRenderedSampleSeconds = result.lastRenderedSampleSeconds;
    lastGraph = result.lastGraph;
    lastEdgeRates = result.lastEdgeRates;
    if (
      result.layoutSize &&
      Number.isFinite(result.layoutSize.width) &&
      Number.isFinite(result.layoutSize.height)
    ) {
      layoutSize = result.layoutSize;
    }
  }

  // --- Polling loop ---
  function scheduleNextFetch() {
    pollingState.pollTimer = scheduleNextTimer(
      pollingState.pollTimer,
      POLL_INTERVAL_MS,
      fetchAndUpdate
    );
  }

  function scheduleNextHealthPoll() {
    pollingState.healthPollTimer = scheduleNextTimer(
      pollingState.healthPollTimer,
      HEALTH_POLL_INTERVAL_MS,
      pollHealthEndpoints
    );
  }

  function scheduleNextStatusPoll() {
    pollingState.statusPollTimer = scheduleNextTimer(
      pollingState.statusPollTimer,
      STATUS_POLL_INTERVAL_MS,
      pollStatusEndpoint
    );
  }

  async function pollHealthEndpoints() {
    await runHealthPoll({
      state: pollingState,
      healthRequestTimeoutMs: HEALTH_REQUEST_TIMEOUT_MS,
      onProbeResult: (livezProbe, readyzProbe) => {
        setHealthBadge(healthLivez, "Livez", livezProbe);
        setHealthBadge(healthReadyz, "Readyz", readyzProbe);
      },
      scheduleNext: scheduleNextHealthPoll,
    });
  }

  async function pollStatusEndpoint() {
    await runStatusPoll({
      state: pollingState,
      statusRequestTimeoutMs: STATUS_REQUEST_TIMEOUT_MS,
      buildStatusSnapshot,
      onSnapshotReady: (snapshot) => {
        statusSnapshot = snapshot;
        renderControlPlaneEventsPanel();
      },
      onProbeUpdate: () => {},
      onRefreshDecorations: refreshPipelineSelectorStatusDecorations,
      isOverlayOpen: isRuntimeStatusOverlayOpen,
      renderOverlay: renderRuntimeStatusOverlay,
      scheduleNext: scheduleNextStatusPoll,
    });
  }

  async function fetchAndUpdate() {
    await runMetricsPoll({
      state: pollingState,
      metricsUrlCandidates: METRICS_URL_CANDIDATES,
      getLastSampleTs: () => lastSampleTs,
      onConnected: () => setConnected(true),
      onDisconnected: () => setConnected(false),
      onHideError: hideError,
      onShowError: showError,
      onSampleAccepted: ({ ts, sampleSeconds, metricSets }) => {
        lastSampleTs = ts;
        lastSampleSeconds = sampleSeconds;
        lastUpdateEl.textContent = ts.toLocaleTimeString();
        lastMetricSets = metricSets;
        updateInterPipelineTopologyState(metricSets);
        updateFilterSelectors(metricSets);
        applyFilteredView(metricSets, true);
      },
      scheduleNext: scheduleNextFetch,
    });
  }

  setHealthBadge(healthLivez, "Livez", {
    state: "unknown",
    error: "not checked yet",
  });
  setHealthBadge(healthReadyz, "Readyz", {
    state: "unknown",
    error: "not checked yet",
  });
  updatePipelineSelectionDisplay();

  // Initialise the live log-stream panel.  The controller wires up its own
  // DOM event listeners and manages the WebSocket lifecycle independently of
  // the metrics polling loop.
  const logsSection = document.getElementById("logs-section");
  if (logsSection) {
    createLogsController({ containerEl: logsSection });
  }

  void fetchAndUpdate();
  void pollHealthEndpoints();
  void pollStatusEndpoint();
