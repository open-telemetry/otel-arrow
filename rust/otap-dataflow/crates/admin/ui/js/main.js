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
  import { buildMetricsCandidates, fetchMetricsFromCandidates } from "./metrics-api.js";
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
    getNodeOutputAnchorY as getLayoutNodeOutputAnchorY,
    layoutGraph as computeLayoutGraph,
  } from "./graph-layout.js";
  import {
    filterMetricSets as filterMetricSetsBySelection,
    getDagMetricSets as getDagMetricSetsBySelection,
    isDeltaCounterMetric,
  } from "./metric-filters.js";
  import { buildStatusSnapshot, getStatusSeverity } from "./status-runtime.js";
  import {
    buildInterPipelineTopology,
    createEmptyInterPipelineTopology,
    findMostCentralPipelineKey,
    getPipelineInterconnect,
    getTransitivelyConnectedPipelineKeys,
  } from "./inter-pipeline-topology.js";

  // Query params tune metrics query behavior.
  const urlParams = new URLSearchParams(window.location.search);

  // Metrics endpoint strategy: reset snapshots each poll, and keep-all-zeroes is configurable.
  const keepAllZeroesParam = urlParams.get("keep_all_zeroes");
  const keepAllZeroes =
    keepAllZeroesParam == null ? true : keepAllZeroesParam === "true";
  const METRICS_URL_CANDIDATES = buildMetricsCandidates({
    query: `format=json&reset=true&keep_all_zeroes=${keepAllZeroes ? "true" : "false"}`,
  });
  let resolvedMetricsUrl = null;
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
  const tabPanelTokio = document.getElementById("tab-panel-tokio");

  // Runtime state: selection, filtering, history caches, chart instances, and layout/zoom.
  let zoomLevel = 1;
  let zoomUserOverridden = false;
  let dagDragState = null;
  let suppressDagViewportClickOnce = false;
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
  let pollTimer = null;
  let healthPollTimer = null;
  let statusPollTimer = null;
  let fetchInFlight = false;
  let healthFetchInFlight = false;
  let statusFetchInFlight = false;
  let statusSnapshot = null;
  let statusLastCheckedAtMs = null;
  let statusLastProbe = null;
  let pipelineOptionLabelByKey = new Map();
  let activeFetchController = null;
  let latestFetchRequestId = 0;
  let latestAppliedFetchRequestId = 0;
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
  };

  // --- View mode and connection status helpers ---
  function setActiveTab(tab) {
    const isGeneral = tab === "general";
    tabPanelGeneral.classList.toggle("hidden", !isGeneral);
    tabPanelTokio.classList.toggle("hidden", isGeneral);
    if (viewSelect) {
      viewSelect.value = isGeneral ? "general" : "tokio";
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
    zoomUserOverridden = false;
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

    zoomUserOverridden = false;
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
      tr.appendChild(createRuntimeStatusCell(statusLastProbe?.error || "-", "runtime-status-state"));
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
    if (Number.isFinite(statusLastProbe?.status)) parts.push(`HTTP ${statusLastProbe.status}`);
    if (Number.isFinite(statusLastProbe?.latencyMs)) {
      parts.push(`${Math.round(statusLastProbe.latencyMs)} ms`);
    }
    if (statusLastProbe?.error) parts.push(statusLastProbe.error);
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
    if (statusFetchInFlight) return;
    const nowMs = Date.now();
    if (!Number.isFinite(statusLastCheckedAtMs)) {
      void pollStatusEndpoint();
      return;
    }
    if (nowMs - statusLastCheckedAtMs > maxAgeMs) {
      void pollStatusEndpoint();
    }
  }

  async function probeHealthEndpoint(path) {
    const controller = new AbortController();
    const started =
      typeof performance !== "undefined" && typeof performance.now === "function"
        ? performance.now()
        : Date.now();
    const timeoutId = window.setTimeout(() => {
      controller.abort();
    }, HEALTH_REQUEST_TIMEOUT_MS);
    try {
      const response = await fetch(path, {
        method: "GET",
        cache: "no-store",
        signal: controller.signal,
      });
      const ended =
        typeof performance !== "undefined" && typeof performance.now === "function"
          ? performance.now()
          : Date.now();
      return {
        state: response.ok ? "up" : "down",
        status: response.status,
        latencyMs: ended - started,
      };
    } catch (error) {
      const ended =
        typeof performance !== "undefined" && typeof performance.now === "function"
          ? performance.now()
          : Date.now();
      return {
        state: "unknown",
        latencyMs: ended - started,
        error:
          error?.name === "AbortError"
            ? `timeout>${HEALTH_REQUEST_TIMEOUT_MS}ms`
            : error?.message || "request failed",
      };
    } finally {
      window.clearTimeout(timeoutId);
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
    nodeSeries.clear();
    channelSeries.clear();
    pipelineSeries.clear();
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
    return points.filter((point) => point.ts >= startMs && point.ts <= endMs);
  }

  function getPointAtTime(points, ts) {
    if (!points || !points.length) return null;
    let chosen = null;
    for (const point of points) {
      if (point.ts <= ts) {
        chosen = point;
      } else {
        break;
      }
    }
    return chosen || points[0] || null;
  }

  function getChannelPoint(channelId, ts) {
    const series = channelSeries.get(channelId);
    if (!series || !series.points.length) return null;
    const endMs = getWindowEndMs();
    const startMs = endMs - getWindowMs();
    const points = getSeriesWindow(series.points, startMs, endMs);
    if (!points.length) return null;
    const targetTs = Number.isFinite(ts) ? ts : getDisplayTimeMs();
    return getPointAtTime(points, targetTs) || points[points.length - 1];
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
    if (channelChart) {
      if (channelChart._legendHandlers) {
        const { move, leave } = channelChart._legendHandlers;
        channelChart.canvas.removeEventListener("mousemove", move);
        channelChart.canvas.removeEventListener("mouseleave", leave);
        channelChart._legendHandlers = null;
      }
      channelChart.destroy();
      channelChart = null;
      channelChartId = null;
    }
    renderChannelLegend(null);
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
    if (channelChart) {
      channelChart.options.scales.x.ticks.color = theme.tick;
      channelChart.options.scales.y.ticks.color = theme.tick;
      channelChart.options.scales.x.grid.color = theme.grid;
      channelChart.options.scales.y.grid.color = theme.grid;
      channelChart.update("none");
    }
    nodeCharts.forEach((chart) => {
      chart.options.scales.x.ticks.color = theme.tick;
      chart.options.scales.y.ticks.color = theme.tick;
      chart.options.scales.x.grid.color = theme.grid;
      chart.options.scales.y.grid.color = theme.grid;
      chart.update("none");
    });
    pipelineCharts.forEach((chart) => {
      chart.options.scales.x.ticks.color = theme.tick;
      chart.options.scales.y.ticks.color = theme.tick;
      chart.options.scales.x.grid.color = theme.grid;
      chart.options.scales.y.grid.color = theme.grid;
      chart.update("none");
    });
  }

  function recordPipelineMetric(metricKey, value, ts) {
    if (!ts || !Number.isFinite(value)) return;
    const nowMs = ts.getTime();
    const cutoff = nowMs - MAX_WINDOW_MS;
    const entry = pipelineSeries.get(metricKey) || { points: [] };
    entry.points.push({ ts: nowMs, value });
    entry.points = entry.points.filter((point) => point.ts >= cutoff);
    pipelineSeries.set(metricKey, entry);
  }

  function updatePipelineMetricLegends(show) {
    const metricEls = document.querySelectorAll(".metric-legend");
    metricEls.forEach((el) => {
      const key = el.dataset.metric;
      const color =
        Object.values(PIPELINE_CHART_CONFIG)
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
  };

  function getPipelinePointAtTime(metricKey, ts) {
    const series = pipelineSeries.get(metricKey);
    if (!series || !series.points.length) return null;
    const endMs = getWindowEndMs();
    const startMs = endMs - getWindowMs();
    const points = getSeriesWindow(series.points, startMs, endMs);
    if (!points.length) return null;
    const targetTs = Number.isFinite(ts) ? ts : getDisplayTimeMs();
    return getPointAtTime(points, targetTs) || points[points.length - 1];
  }

  function setPipelineHover(ts) {
    pipelineHoverTs = Number.isFinite(ts) ? ts : null;
    applyPipelineMetricValues(pipelineHoverTs);
    pipelineCharts.forEach((chart) => {
      const labels = chart._tsLabels || [];
      if (!labels.length || pipelineHoverTs == null) {
        chart._hoverIndex = null;
        chart.draw();
        return;
      }
      const idx = getClosestIndex(labels, pipelineHoverTs);
      chart._hoverIndex = idx;
      chart.draw();
    });
  }

  function setChannelHover(ts) {
    if (!channelChart || !channelChartId) {
      return;
    }
    if (!Number.isFinite(ts)) {
      channelChart._hoverIndex = null;
      channelChart.draw();
      renderChannelLegend(channelChartId);
      return;
    }
    const labels = channelChart._tsLabels || [];
    if (!labels.length) {
      renderChannelLegend(channelChartId);
      return;
    }
    const idx = getClosestIndex(labels, ts);
    channelChart._hoverIndex = idx;
    channelChart.draw();
    const point = getChannelPoint(channelChartId, labels[idx]);
    renderChannelLegend(channelChartId, point || undefined);
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
    Object.entries(PIPELINE_METRIC_DISPLAY).forEach(([key, cfg]) => {
      if (!cfg.el || !cfg.format) return;
      const point = getPipelinePointAtTime(key, ts);
      cfg.el.textContent = cfg.format(point ? point.value : null);
    });
  }

  function getPipelineSeriesWindow(metricKey) {
    const endMs = getWindowEndMs();
    const startMs = endMs - getWindowMs();
    const series = pipelineSeries.get(metricKey);
    if (!series) return [];
    return getSeriesWindow(series.points, startMs, endMs);
  }

  function updatePipelineCharts() {
    if (!showPipelineCharts) return;
    const theme = getChartThemeColors();
    Object.values(PIPELINE_CHART_CONFIG).forEach((config) => {
      const canvas = document.getElementById(config.canvasId);
      if (!canvas || !window.Chart) return;
      const labelSet = new Set();
      const seriesMaps = new Map();
      config.metrics.forEach((metric) => {
        const points = getPipelineSeriesWindow(metric.key);
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
        const chart = new Chart(canvas.getContext("2d"), {
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
          if (pipelineHoverTs !== ts) {
            setGlobalHover(ts);
          }
        };
        const leave = () => {
          setGlobalHover(null);
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
      if (globalHoverTs != null) {
        setGlobalHover(globalHoverTs);
      }
    });
  }

  // Show/hide and lifecycle-manage top-card charts.
  function togglePipelineCharts(show) {
    document
      .querySelectorAll(".metric-card-chart")
      .forEach((el) => el.classList.toggle("hidden", !show));
    updatePipelineMetricLegends(show);
    if (show) {
      updatePipelineCharts();
    }
  }

  function destroyPipelineCharts() {
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
    pipelineHoverTs = null;
    applyPipelineMetricValues(null);
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
    const perfMs = perfStart();
    try {
      if (!Number.isFinite(sampleSeconds) || sampleSeconds <= 0) return;
      if (!ts) return;
      const nowMs = ts.getTime();
      const cutoff = nowMs - MAX_WINDOW_MS;
      const scopeByPipeline = dagScope?.scopeByPipeline === true;
      metricSets.forEach((set) => {
        if (
          set.name === "channel.sender" ||
          set.name === "channel.receiver" ||
          set.name === "pipeline.metrics" ||
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
      perfEnd("updateNodeSeries", perfMs, { sets: metricSets?.length || 0 });
    }
  }

  // Build per-channel send/recv/error series for edge detail charts and rate computation.
  function updateChannelSeries(metricSets, sampleSeconds, ts, dagScope = null) {
    const perfMs = perfStart();
    try {
      if (!Number.isFinite(sampleSeconds) || sampleSeconds <= 0) return;
      if (!ts) return;
      const scopeByPipeline = dagScope?.scopeByPipeline === true;

      const perChannel = new Map();
      const ensureChannel = (id) => {
        if (!perChannel.has(id)) {
          perChannel.set(id, {
            send: 0,
            recv: 0,
            sendErrorFull: 0,
            sendErrorClosed: 0,
            recvErrorEmpty: 0,
            recvErrorClosed: 0,
          });
        }
        return perChannel.get(id);
      };

      metricSets.forEach((set) => {
        if (set.name !== "channel.sender" && set.name !== "channel.receiver") {
          return;
        }
        const attrs = normalizeAttributes(set.attributes || {});
        const channelId = resolveScopedChannelId(attrs, scopeByPipeline);
        if (!channelId) return;

        const metrics = set.metrics || [];
        if (set.name === "channel.sender") {
          const sendMetric = metrics.find((metric) => metric.name === "send.count");
          const sendValue = sendMetric && typeof sendMetric.value === "number" ? sendMetric.value : 0;
          const sendErrorFullMetric = metrics.find(
            (metric) => metric.name === "send.error_full"
          );
          const sendErrorClosedMetric = metrics.find(
            (metric) => metric.name === "send.error_closed"
          );
          const sendErrorFullValue =
            sendErrorFullMetric && typeof sendErrorFullMetric.value === "number"
              ? sendErrorFullMetric.value
              : 0;
          const sendErrorClosedValue =
            sendErrorClosedMetric && typeof sendErrorClosedMetric.value === "number"
              ? sendErrorClosedMetric.value
              : 0;
          const channelEntry = ensureChannel(channelId);
          channelEntry.send += sendValue;
          channelEntry.sendErrorFull += sendErrorFullValue;
          channelEntry.sendErrorClosed += sendErrorClosedValue;
        } else {
          const recvMetric = metrics.find((metric) => metric.name === "recv.count");
          const recvValue = recvMetric && typeof recvMetric.value === "number" ? recvMetric.value : 0;
          const recvErrorEmptyMetric = metrics.find(
            (metric) => metric.name === "recv.error_empty"
          );
          const recvErrorClosedMetric = metrics.find(
            (metric) => metric.name === "recv.error_closed"
          );
          const recvErrorEmptyValue =
            recvErrorEmptyMetric && typeof recvErrorEmptyMetric.value === "number"
              ? recvErrorEmptyMetric.value
              : 0;
          const recvErrorClosedValue =
            recvErrorClosedMetric && typeof recvErrorClosedMetric.value === "number"
              ? recvErrorClosedMetric.value
              : 0;
          const channelEntry = ensureChannel(channelId);
          channelEntry.recv += recvValue;
          channelEntry.recvErrorEmpty += recvErrorEmptyValue;
          channelEntry.recvErrorClosed += recvErrorClosedValue;
        }
      });

      const nowMs = ts.getTime();
      const cutoff = nowMs - MAX_WINDOW_MS;

      perChannel.forEach((counts, channelId) => {
        const sendRate = counts.send / sampleSeconds;
        const recvRate = counts.recv / sampleSeconds;
        const sendErrorFullRate = counts.sendErrorFull / sampleSeconds;
        const sendErrorClosedRate = counts.sendErrorClosed / sampleSeconds;
        const recvErrorEmptyRate = counts.recvErrorEmpty / sampleSeconds;
        const recvErrorClosedRate = counts.recvErrorClosed / sampleSeconds;
        const series = channelSeries.get(channelId) || { points: [] };
        series.points.push({
          ts: nowMs,
          sendRate,
          recvRate,
          sendErrorFullRate,
          sendErrorClosedRate,
          recvErrorEmptyRate,
          recvErrorClosedRate,
        });
        series.points = series.points.filter((point) => point.ts >= cutoff);
        channelSeries.set(channelId, series);
      });
    } finally {
      perfEnd("updateChannelSeries", perfMs, { sets: metricSets?.length || 0 });
    }
  }

  // Edge detail chart renderer (single chart reused per selected channel).
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

    if (channelChart && channelChart.canvas !== canvas) {
      clearChannelChart();
    }

    const theme = getChartThemeColors();
    if (!channelChart || channelChartId !== channelId) {
      clearChannelChart();
      channelChartId = channelId;
      channelChart = new Chart(canvas.getContext("2d"), {
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
      channelChart._tsLabels = points.map((point) => point.ts);
      attachLegendInteraction(channelChart, channelId);
      renderChannelLegend(channelId);
      return;
    }

    channelChart.data.labels = labels;
    channelChart.data.datasets[0].data = sendData;
    channelChart.data.datasets[1].data = recvData;
    channelChart.data.datasets[2].data = sendErrorFullData;
    channelChart.data.datasets[3].data = sendErrorClosedData;
    channelChart.data.datasets[4].data = recvErrorEmptyData;
    channelChart.data.datasets[5].data = recvErrorClosedData;
    channelChart.options.scales.x.ticks.color = theme.tick;
    channelChart.options.scales.y.ticks.color = theme.tick;
    channelChart.options.scales.x.grid.color = theme.grid;
    channelChart.options.scales.y.grid.color = theme.grid;
    channelChart.update("none");
    channelChart._tsLabels = points.map((point) => point.ts);
    if (globalHoverTs != null) {
      setGlobalHover(globalHoverTs);
    }
    renderChannelLegend(channelId);
  }

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
      : freezeActive
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
      .map(
        (row) => {
          const safeLabel = escapeHtml(row.label);
          return `
          <div class="channel-chart-row">
            <span class="channel-chart-label">
              <span class="channel-chart-dot" style="color:${row.color}; background:${row.color};"></span>
              ${safeLabel}
            </span>
            <span class="font-mono text-slate-100">${formatLegendRate(row.value)}</span>
          </div>`;
        }
      )
      .join("");
  }

  function attachLegendInteraction(chart, channelId) {
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

  function renderSelectionNone() {
    if (selectionTitle) {
      selectionTitle.textContent = "Selection Details";
    }
    edgeDetailMeta.textContent = "None selected";
    edgeDetailBody.innerHTML = "Click a node or edge to show details.";
    clearChannelChart();
    destroyNodeCharts();
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

  function buildSpecificAttributes(primary, secondary) {
    const out = {};
    for (const [key, value] of Object.entries(primary || {})) {
      if (key.startsWith("channel.")) continue;
      if (value == null || value === "") continue;
      if (secondary && secondary[key] === value) continue;
      out[key] = value;
    }
    ["node.id", "node.type"].forEach((key) => {
      if (primary && primary[key] != null && primary[key] !== "") {
        out[key] = primary[key];
      }
    });
    return out;
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
    const queuePercent =
      queueRatio == null ? "n/a" : `${Math.round(queueRatio * 100)}%`;
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
    const channelDisplayId =
      edge.channelDisplayId || channel?.displayId || channelId;
    const sourceDisplayId =
      edge.sourceDisplayId || senderAttrs["node.id"] || edge.source;
    const targetDisplayId =
      edge.targetDisplayId || receiverAttrs["node.id"] || edge.target;
    const useChannelSeries = !(channel?.multiSender || channel?.multiReceiver);
    const seriesPoint = useChannelSeries ? getChannelPoint(channelId, getDisplayTimeMs()) : null;
    const edgeRates = lastEdgeRates.get(edge.id);
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
    const senderName =
      channelKind === "control" ? "Pipeline controller" : sourceDisplayId;
    const senderType =
      channelKind === "control" ? "controller" : senderAttrs["node.type"] || "node";
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

  function metricMap(metrics) {
    const out = {};
    metrics.forEach((metric) => {
      if (typeof metric.value !== "number" || !Number.isFinite(metric.value)) return;
      out[metric.name] = metric.value;
    });
    return out;
  }

  function computeEdgeRates(edges, displayTimeMs, sampleSeconds) {
    const rates = new Map();
    edges.forEach((edge) => {
      const senderMetrics = metricMap(edge.data.sender?.metrics || []);
      const receiverMetrics = metricMap(edge.data.receiver?.metrics || []);
      const channelId = edge.channelId || edge.data?.id || edge.id;
      const useChannelSeries = !(edge.data?.multiSender || edge.data?.multiReceiver);
      const point = useChannelSeries ? getChannelPoint(channelId, displayTimeMs) : null;
      const fallbackSendRate = calcRate(senderMetrics["send.count"] ?? 0, sampleSeconds) ?? 0;
      const fallbackRecvRate = calcRate(receiverMetrics["recv.count"] ?? 0, sampleSeconds) ?? 0;
      const fallbackSendErrRate =
        calcRate(
          (senderMetrics["send.error_full"] ?? 0) +
            (senderMetrics["send.error_closed"] ?? 0),
          sampleSeconds
        ) ?? 0;
      const fallbackRecvErrRate =
        calcRate(
          (receiverMetrics["recv.error_empty"] ?? 0) +
            (receiverMetrics["recv.error_closed"] ?? 0),
          sampleSeconds
        ) ?? 0;
      const sendRate = point?.sendRate ?? fallbackSendRate;
      const recvRate = point?.recvRate ?? fallbackRecvRate;
      const sendErrorRate =
        point == null
          ? fallbackSendErrRate
          : (point.sendErrorFullRate || 0) + (point.sendErrorClosedRate || 0);
      const recvErrorRate =
        point == null
          ? fallbackRecvErrRate
          : (point.recvErrorEmptyRate || 0) + (point.recvErrorClosedRate || 0);
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

  // --- Interaction wiring (zoom, filters, toggles, theme, search) ---
  function formatZoomPercent(value) {
    const percent = value * 100;
    const rounded = Math.round(percent);
    if (Math.abs(percent - rounded) < 0.005) {
      return `${rounded}%`;
    }
    return `${percent.toFixed(2)}%`;
  }

  function applyZoom() {
    const scaledWidth = layoutSize.width * zoomLevel;
    const scaledHeight = layoutSize.height * zoomLevel;
    dagZoom.style.width = `${scaledWidth}px`;
    dagZoom.style.height = `${scaledHeight}px`;
    dagCanvas.style.transform = `scale(${zoomLevel})`;
    zoomValueEl.textContent = formatZoomPercent(zoomLevel);
  }

  function computeFitZoom() {
    const viewportWidth = dagViewport.clientWidth - ZOOM_FIT_PADDING * 2;
    const viewportHeight = dagViewport.clientHeight - ZOOM_FIT_PADDING * 2;
    if (viewportWidth <= 0 || viewportHeight <= 0) {
      return zoomLevel;
    }
    const widthScale = viewportWidth / Math.max(layoutSize.width, 1);
    const heightScale = viewportHeight / Math.max(layoutSize.height, 1);
    // Keep default view as an overview: fit content, but do not auto-zoom above 100%.
    return Math.min(1, widthScale, heightScale);
  }

  function applyDefaultOverviewZoom(force = false) {
    if (!layoutSize.width || !layoutSize.height) return;
    if (!force && zoomUserOverridden) return;
    const fitZoom = computeFitZoom();
    const clamped = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, fitZoom));
    zoomLevel = Math.round(clamped * 10000) / 10000;
  }

  function setZoom(nextZoom, options = {}) {
    const userInitiated = options.userInitiated ?? true;
    if (userInitiated) {
      zoomUserOverridden = true;
    }
    const clamped = Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, nextZoom));
    zoomLevel = Math.round(clamped * 10000) / 10000;
    applyZoom();
  }

  function zoomAtViewportPoint(nextZoom, clientX, clientY, options = {}) {
    const rect = dagViewport.getBoundingClientRect();
    const pointX = clientX - rect.left;
    const pointY = clientY - rect.top;
    const currentZoom = Math.max(zoomLevel, 0.0001);
    const logicalX = (dagViewport.scrollLeft + pointX) / currentZoom;
    const logicalY = (dagViewport.scrollTop + pointY) / currentZoom;

    setZoom(nextZoom, options);

    dagViewport.scrollLeft = logicalX * zoomLevel - pointX;
    dagViewport.scrollTop = logicalY * zoomLevel - pointY;
  }

  function shouldStartDagDrag(target) {
    if (!(target instanceof Element)) return true;
    return !target.closest(
      ".dag-node, .dag-edge-hit, .dag-control-indicator, .pipeline-dag-nav-chip, button, input, select, textarea, a"
    );
  }

  function handleDagDragMouseMove(event) {
    if (!dagDragState) return;
    const dx = event.clientX - dagDragState.startX;
    const dy = event.clientY - dagDragState.startY;
    if (
      !dagDragState.moved &&
      (Math.abs(dx) >= DAG_DRAG_THRESHOLD_PX || Math.abs(dy) >= DAG_DRAG_THRESHOLD_PX)
    ) {
      dagDragState.moved = true;
    }
    if (!dagDragState.moved) return;
    dagViewport.scrollLeft = dagDragState.startScrollLeft - dx;
    dagViewport.scrollTop = dagDragState.startScrollTop - dy;
    event.preventDefault();
  }

  function endDagDrag() {
    if (!dagDragState) return;
    if (dagDragState.moved) {
      suppressDagViewportClickOnce = true;
    }
    dagDragState = null;
    dagViewport.classList.remove("dag-dragging");
  }

  function handleDagDragMouseUp() {
    endDagDrag();
  }

  zoomOutBtn.addEventListener("click", () =>
    setZoom(zoomLevel - ZOOM_BUTTON_STEP, { userInitiated: true })
  );
  zoomInBtn.addEventListener("click", () =>
    setZoom(zoomLevel + ZOOM_BUTTON_STEP, { userInitiated: true })
  );
  zoomResetBtn.addEventListener("click", () => {
    zoomUserOverridden = false;
    applyDefaultOverviewZoom(true);
    applyZoom();
  });

  window.addEventListener("resize", () => {
    if (zoomUserOverridden) return;
    applyDefaultOverviewZoom(true);
    applyZoom();
  });

  dagViewport.addEventListener(
    "wheel",
    (event) => {
      if (!layoutSize.width || !layoutSize.height) return;
      event.preventDefault();
      const factor = Math.exp(-event.deltaY * WHEEL_ZOOM_SENSITIVITY);
      zoomAtViewportPoint(zoomLevel * factor, event.clientX, event.clientY, {
        userInitiated: true,
      });
    },
    { passive: false }
  );

  dagViewport.addEventListener("mousedown", (event) => {
    if (event.button !== 0) return;
    if (!shouldStartDagDrag(event.target)) return;
    dagDragState = {
      startX: event.clientX,
      startY: event.clientY,
      startScrollLeft: dagViewport.scrollLeft,
      startScrollTop: dagViewport.scrollTop,
      moved: false,
    };
    dagViewport.classList.add("dag-dragging");
    event.preventDefault();
  });

  window.addEventListener("mousemove", handleDagDragMouseMove);
  window.addEventListener("mouseup", handleDagDragMouseUp);
  window.addEventListener("blur", endDagDrag);

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

  dagSearch.addEventListener("input", () => {
    dagSearchQuery = dagSearch.value || "";
    applyFilteredView(lastMetricSets, false);
  });

  if (dagScopeBtn) {
    dagScopeBtn.addEventListener("click", () => {
      if (dagScopeBtn.disabled) return;
      const nextMode =
        dagPipelineScopeMode === DAG_SCOPE_CONNECTED
          ? DAG_SCOPE_SINGLE
          : DAG_SCOPE_CONNECTED;
      setDagPipelineScopeMode(nextMode, { rerender: true });
    });
  }
  updateDagScopeButtonState();

  fullscreenBtn.addEventListener("click", () => {
    const enabled = document.body.classList.toggle("dag-fullscreen");
    if (!enabled && dagPipelineScopeMode === DAG_SCOPE_CONNECTED) {
      setDagPipelineScopeMode(DAG_SCOPE_SINGLE, { rerender: false });
      applyFilteredView(lastMetricSets, false);
    }
    fullscreenBtn.textContent = enabled ? "Exit full page" : "Full page";
    updateDagScopeButtonState();
    updateStickyPanelOffset();
  });

  dagViewport.addEventListener("click", (event) => {
    if (suppressDagViewportClickOnce) {
      suppressDagViewportClickOnce = false;
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

  function hashString32(hash, value) {
    const text = String(value == null ? "" : value);
    let next = hash >>> 0;
    for (let i = 0; i < text.length; i += 1) {
      next ^= text.charCodeAt(i);
      next = Math.imul(next, 16777619);
    }
    return next >>> 0;
  }

  function buildRenderedStructureSignature(
    nodes,
    edges,
    controlEdges,
    dagScopeMode,
    includeControlChannels
  ) {
    let hash = 2166136261;
    hash = hashString32(hash, dagScopeMode);
    hash = hashString32(hash, includeControlChannels ? "1" : "0");

    const nodeParts = nodes
      .map((node) => {
        const ports = Array.from(new Set((node.displayPorts || node.outPorts || []).map(String)))
          .sort((a, b) => a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" }))
          .join(",");
        const nodeType = node.attrs?.["node.type"] || "";
        return `${node.id}|${nodeType}|${ports}`;
      })
      .sort((a, b) => a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" }));
    nodeParts.forEach((part) => {
      hash = hashString32(hash, part);
    });

    const edgeParts = edges
      .map((edge) => `${edge.id}|${edge.source}|${edge.target}|${edge.port || ""}`)
      .sort((a, b) => a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" }));
    edgeParts.forEach((part) => {
      hash = hashString32(hash, part);
    });

    const visibleControlEdges = includeControlChannels ? controlEdges : [];
    const controlParts = visibleControlEdges
      .map((edge) => `${edge.id}|${edge.source}|${edge.target}|${edge.port || ""}`)
      .sort((a, b) => a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" }));
    controlParts.forEach((part) => {
      hash = hashString32(hash, part);
    });

    return `${nodes.length}:${edges.length}:${visibleControlEdges.length}:${hash.toString(16)}`;
  }

  function syncSelectionDetails(nodes, edges, controlEdges) {
    if (selectedEdgeId) {
      const selectedEdge =
        edges.find((edge) => edge.id === selectedEdgeId) ||
        (showControlChannels
          ? controlEdges.find((edge) => edge.id === selectedEdgeId)
          : null);
      if (selectedEdge) {
        selectedEdgeData = selectedEdge;
        renderEdgeDetails(selectedEdge);
        return;
      }
      if (selectedEdgeData && selectedEdgeData.id === selectedEdgeId) {
        renderEdgeDetails(selectedEdgeData);
        return;
      }
      selectedEdgeId = null;
      selectedEdgeData = null;
      renderSelectionNone();
      return;
    }
    if (selectedNodeId) {
      const selectedNode = nodes.find((node) => node.id === selectedNodeId);
      if (selectedNode) {
        selectedNodeData = selectedNode;
        renderNodeDetails(selectedNode);
        return;
      }
      if (selectedNodeData && selectedNodeData.id === selectedNodeId) {
        renderNodeDetails(selectedNodeData);
        return;
      }
      selectedNodeId = null;
      selectedNodeData = null;
      renderSelectionNone();
      return;
    }
    renderSelectionNone();
  }

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
    const perfMs = perfStart();
    let renderMode = "full";
    try {
      const dataGraphResolved = dataGraph || { nodes: [], edges: [], meta: {} };
      const controlGraphResolved = controlGraph || { nodes: [], edges: [], meta: {} };
      let nodes = [...(dataGraphResolved.nodes || [])];
      let edges = [...(dataGraphResolved.edges || [])];
      const controlEdges = controlGraphResolved.edges || [];
      const controlEdgeIds = new Set(controlEdges.map((edge) => edge.id));
      const sampleSeconds =
        dataGraphResolved.meta?.sampleSeconds ?? controlGraphResolved.meta?.sampleSeconds;
      const displayTimeMs = getDisplayTimeMs();
      const dataEdgeRates = computeEdgeRates(edges, displayTimeMs, sampleSeconds);
      const controlEdgeRates = computeEdgeRates(controlEdges, displayTimeMs, sampleSeconds);
      lastEdgeRates = dataEdgeRates;

    if (hideZeroActivity) {
      const visibleEdges = edges.filter((edge) => {
        const activity = dataEdgeRates.get(edge.id);
        if (!activity) return false;
        return metricMode === "errors" ? activity.errorActive : activity.active;
      });
      const activeNodeIds = new Set();
      visibleEdges.forEach((edge) => {
        activeNodeIds.add(edge.source);
        activeNodeIds.add(edge.target);
      });
      const portMap = new Map();
      visibleEdges.forEach((edge) => {
        if (!portMap.has(edge.source)) portMap.set(edge.source, new Set());
        portMap.get(edge.source).add(edge.port);
      });
      nodes = nodes
        .filter((node) => activeNodeIds.has(node.id))
        .map((node) => ({
          ...node,
          outPorts: portMap.get(node.id) ? Array.from(portMap.get(node.id)) : [],
        }));
      edges = edges.filter(
        (edge) => activeNodeIds.has(edge.source) && activeNodeIds.has(edge.target)
      );
      if (selectedNodeId && !nodes.find((node) => node.id === selectedNodeId)) {
        selectedNodeId = null;
        selectedNodeData = null;
      }
    }

    if (dagSearchQuery) {
      const searchResult = filterGraphByQuery(nodes, edges, dagSearchQuery);
      nodes = searchResult.nodes;
      edges = searchResult.edges;
      if (selectedNodeId && !nodes.find((node) => node.id === selectedNodeId)) {
        selectedNodeId = null;
        selectedNodeData = null;
      }
    }

    const dataEdgeIds = new Set(edges.map((edge) => edge.id));
    if (
      selectedEdgeId &&
      !controlEdgeIds.has(selectedEdgeId) &&
      !dataEdgeIds.has(selectedEdgeId)
    ) {
      selectedEdgeId = null;
      selectedEdgeData = null;
    }

    if (!showControlChannels && selectedEdgeId && controlEdgeIds.has(selectedEdgeId)) {
      selectedEdgeId = null;
      selectedEdgeData = null;
    }

    const portScores = new Map();
    edges.forEach((edge) => {
      const rate = dataEdgeRates.get(edge.id)?.sendRate ?? 0;
      if (!portScores.has(edge.source)) {
        portScores.set(edge.source, new Map());
      }
      const nodePorts = portScores.get(edge.source);
      nodePorts.set(edge.port, (nodePorts.get(edge.port) || 0) + rate);
    });

    nodes.forEach((node) => {
      if (!node.outPorts || node.outPorts.length < 2) return;
      const scores = portScores.get(node.id);
      node.outPorts = [...node.outPorts].sort((a, b) => {
        const scoreA = scores?.get(a) ?? 0;
        const scoreB = scores?.get(b) ?? 0;
        if (scoreB !== scoreA) return scoreB - scoreA;
        return a.localeCompare(b);
      });
    });

    const activeDagScope = getDagRenderScope();
    const structureSignature = buildRenderedStructureSignature(
      nodes,
      edges,
      controlEdges,
      activeDagScope.mode,
      showControlChannels
    );

    lastRenderedNodes = nodes;
    lastRenderedEdges = edges;
    lastRenderedControlEdges = controlEdges;
    lastRenderedNodesById = new Map(nodes.map((node) => [node.id, node]));
    lastRenderedEdgesById = new Map(edges.map((edge) => [edge.id, edge]));
    lastRenderedControlEdgesById = new Map(controlEdges.map((edge) => [edge.id, edge]));
    lastRenderedSampleSeconds = sampleSeconds;
    lastGraph = dataGraphResolved;

    dagEmpty.classList.toggle("hidden", edges.length > 0);
    if (!edges.length) {
      selectedEdgeId = null;
      selectedEdgeData = null;
    }
    if (!nodes.length) {
      selectedNodeId = null;
      selectedNodeData = null;
    }

    const hasRenderedDagDom =
      dagNodes.childElementCount > 0 ||
      dagEdges.childElementCount > 0 ||
      dagLanes.childElementCount > 0;
    if (structureSignature === lastRenderedStructureSignature && hasRenderedDagDom) {
      renderMode = "reuse";
      updateTopologyForHover(displayTimeMs);
      syncSelectionDetails(nodes, edges, controlEdges);
      return;
    }

    lastRenderedStructureSignature = structureSignature;
    renderMode = "incremental";

    const layout = layoutGraph(nodes, edges);
    const baseNodeMap = new Map(nodes.map((node) => [node.id, node]));
    const pipelineNavAnchors =
      activeDagScope.mode === DAG_SCOPE_CONNECTED
        ? []
        : collectPipelineDagAnchors(baseNodeMap);
    const pipelineNavLayout = computePipelineDagNavLayout(pipelineNavAnchors);
    const leftNavGutter = pipelineNavLayout.leftGutter;
    const rightNavGutter = pipelineNavLayout.rightGutter;
    if (leftNavGutter > 0) {
      nodes.forEach((node) => {
        node.x += leftNavGutter;
      });
    }
    layout.width += leftNavGutter + rightNavGutter;

    layoutSize = { width: layout.width, height: layout.height };
    dagCanvas.style.width = `${layout.width}px`;
    dagCanvas.style.height = `${layout.height}px`;
    dagEdges.setAttribute("width", layout.width);
    dagEdges.setAttribute("height", layout.height);
    dagEdges.setAttribute("viewBox", `0 0 ${layout.width} ${layout.height}`);
    dagNodes.style.width = `${layout.width}px`;
    dagNodes.style.height = `${layout.height}px`;
    dagLanes.style.width = `${layout.width}px`;
    dagLanes.style.height = `${layout.height}px`;
    applyDefaultOverviewZoom();
    applyZoom();

    ensureDagEdgeDefs();

    const nodeMap = new Map(nodes.map((node) => [node.id, node]));
    const focusSets = buildFocusSets(edges);
    const columnWidth = layout.columnWidth ?? NODE_WIDTH;
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
        (rates?.recvRate ?? 0) > (controlEdgeRates.get(entry.primary.id)?.recvRate ?? 0)
      ) {
        entry.primary = edge;
      }
      controlByTarget.set(edge.target, entry);
    });

    dagLanes.innerHTML = "";
    layout.lanes.forEach((lane) => {
      const hasAbsoluteLane = Number.isFinite(lane.x) && Number.isFinite(lane.width);
      const startX = hasAbsoluteLane
        ? leftNavGutter + lane.x
        : leftNavGutter + MARGIN + lane.start * (columnWidth + LEVEL_GAP);
      const width = hasAbsoluteLane
        ? lane.width
        : (lane.end - lane.start + 1) * columnWidth +
          (lane.end - lane.start) * LEVEL_GAP;
      const label = document.createElement("div");
      label.className = "dag-lane-label";
      label.style.left = `${startX + width / 2}px`;
      label.textContent = lane.label;
      dagLanes.appendChild(label);

      const bar = document.createElement("div");
      bar.className = "dag-lane-bar";
      bar.style.left = `${startX}px`;
      bar.style.width = `${width}px`;
      dagLanes.appendChild(bar);
    });

    const nodeIds = new Set(nodes.map((node) => node.id));
    pruneRemovedDagNodes(nodeIds);
    nodes.forEach((node) => {
      upsertDagNodeElement(node, {
        controlInfo: controlByTarget.get(node.id),
        portScores,
        nodeTraffic,
        nodeErrors,
        focusSets,
      });
    });

    const edgeIds = new Set(edges.map((edge) => edge.id));
    pruneRemovedDagEdges(edgeIds);
    edges.forEach((edge) => {
      const source = nodeMap.get(edge.source);
      const target = nodeMap.get(edge.target);
      if (!source || !target) return;
      upsertDagEdgeElement(edge, source, target, { focusSets, dataEdgeRates });
    });

    clearDagNavigationOverlayElements();

    if (activeDagScope.mode === DAG_SCOPE_CONNECTED) {
      try {
        renderConnectedTopicNavigation(nodeMap, activeDagScope);
      } catch (error) {
        // Do not block base DAG rendering if overlay computation fails.
        console.error("Connected topic overlay render failed", error);
      }
    } else {
      renderPipelineDagNavigation(nodeMap, pipelineNavAnchors, pipelineNavLayout);
    }

      syncSelectionDetails(nodes, edges, controlEdges);
    } finally {
      perfEnd("renderGraph", perfMs, {
        mode: renderMode,
        nodes: lastRenderedNodes.length,
        edges: lastRenderedEdges.length,
        control: lastRenderedControlEdges.length,
      });
    }
  }

  // --- Polling loop ---
  function scheduleNextFetch() {
    if (pollTimer != null) {
      window.clearTimeout(pollTimer);
    }
    pollTimer = window.setTimeout(() => {
      void fetchAndUpdate();
    }, POLL_INTERVAL_MS);
  }

  function scheduleNextHealthPoll() {
    if (healthPollTimer != null) {
      window.clearTimeout(healthPollTimer);
    }
    healthPollTimer = window.setTimeout(() => {
      void pollHealthEndpoints();
    }, HEALTH_POLL_INTERVAL_MS);
  }

  function scheduleNextStatusPoll() {
    if (statusPollTimer != null) {
      window.clearTimeout(statusPollTimer);
    }
    statusPollTimer = window.setTimeout(() => {
      void pollStatusEndpoint();
    }, STATUS_POLL_INTERVAL_MS);
  }

  async function pollHealthEndpoints() {
    if (healthFetchInFlight) return;
    healthFetchInFlight = true;
    const checkedAt = Date.now();
    try {
      const [livezProbe, readyzProbe] = await Promise.all([
        probeHealthEndpoint("/livez"),
        probeHealthEndpoint("/readyz"),
      ]);
      livezProbe.checkedAt = checkedAt;
      readyzProbe.checkedAt = checkedAt;
      setHealthBadge(healthLivez, "Livez", livezProbe);
      setHealthBadge(healthReadyz, "Readyz", readyzProbe);
    } finally {
      healthFetchInFlight = false;
      scheduleNextHealthPoll();
    }
  }

  async function pollStatusEndpoint() {
    if (statusFetchInFlight) return;
    statusFetchInFlight = true;
    const checkedAt = Date.now();
    const controller = new AbortController();
    const started =
      typeof performance !== "undefined" && typeof performance.now === "function"
        ? performance.now()
        : Date.now();
    const timeoutId = window.setTimeout(() => {
      controller.abort();
    }, STATUS_REQUEST_TIMEOUT_MS);

    try {
      const response = await fetch("/status", {
        method: "GET",
        cache: "no-store",
        signal: controller.signal,
      });
      const ended =
        typeof performance !== "undefined" && typeof performance.now === "function"
          ? performance.now()
          : Date.now();
      const probe = {
        status: response.status,
        latencyMs: ended - started,
        checkedAt,
      };

      if (!response.ok) {
        statusLastProbe = {
          ...probe,
          error: `status endpoint returned HTTP ${response.status}`,
        };
        refreshPipelineSelectorStatusDecorations();
        if (isRuntimeStatusOverlayOpen()) {
          renderRuntimeStatusOverlay();
        }
        return;
      }

      const payload = await response.json();
      statusSnapshot = buildStatusSnapshot(payload);
      statusLastCheckedAtMs = checkedAt;
      statusLastProbe = probe;
      refreshPipelineSelectorStatusDecorations();
      if (isRuntimeStatusOverlayOpen()) {
        renderRuntimeStatusOverlay();
      }
    } catch (error) {
      const ended =
        typeof performance !== "undefined" && typeof performance.now === "function"
          ? performance.now()
          : Date.now();
      statusLastProbe = {
        latencyMs: ended - started,
        checkedAt,
        error:
          error?.name === "AbortError"
            ? `timeout>${STATUS_REQUEST_TIMEOUT_MS}ms`
            : error?.message || "status request failed",
      };
      refreshPipelineSelectorStatusDecorations();
      if (isRuntimeStatusOverlayOpen()) {
        renderRuntimeStatusOverlay();
      }
    } finally {
      window.clearTimeout(timeoutId);
      statusFetchInFlight = false;
      scheduleNextStatusPoll();
    }
  }

  async function fetchMetricsSnapshot(signal) {
    const { data, resolvedUrl } = await fetchMetricsFromCandidates(
      METRICS_URL_CANDIDATES,
      resolvedMetricsUrl,
      { signal }
    );
    resolvedMetricsUrl = resolvedUrl;
    return data;
  }

  // Parse and validate snapshot timestamp payload.
  // Returns null when value is missing or not a valid date.
  function parseSnapshotTimestamp(value) {
    if (value == null) return null;
    const ts = new Date(value);
    if (!Number.isFinite(ts.getTime())) {
      return null;
    }
    return ts;
  }

  async function fetchAndUpdate() {
    if (fetchInFlight) return;
    fetchInFlight = true;
    const requestId = ++latestFetchRequestId;
    const controller = new AbortController();
    activeFetchController = controller;
    try {
      const data = await fetchMetricsSnapshot(controller.signal);
      if (requestId < latestAppliedFetchRequestId) {
        return;
      }
      latestAppliedFetchRequestId = requestId;
      setConnected(true);
      hideError();

      const ts = parseSnapshotTimestamp(data.timestamp);
      if (!ts) {
        showError("Received metrics snapshot with invalid timestamp; sample ignored.");
        return;
      }

      const tsMs = ts.getTime();
      const prevTsMs = lastSampleTs ? lastSampleTs.getTime() : null;
      // Ignore stale/reordered snapshots to prevent negative or zero rate windows.
      if (prevTsMs != null && tsMs <= prevTsMs) {
        return;
      }

      const sampleSeconds = prevTsMs == null ? null : (tsMs - prevTsMs) / 1000;
      lastSampleTs = ts;
      lastSampleSeconds = sampleSeconds;
      lastUpdateEl.textContent = ts.toLocaleTimeString();
      const metricSets = data.metric_sets || [];
      lastMetricSets = metricSets;
      updateInterPipelineTopologyState(metricSets);
      updateFilterSelectors(metricSets);
      applyFilteredView(metricSets, true);
    } catch (err) {
      if (err?.name === "AbortError") {
        return;
      }
      setConnected(false);
      showError(err.message || "Failed to load metrics.");
    } finally {
      if (activeFetchController === controller) {
        activeFetchController = null;
      }
      fetchInFlight = false;
      scheduleNextFetch();
    }
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

  void fetchAndUpdate();
  void pollHealthEndpoints();
  void pollStatusEndpoint();
