// Incremental graph-render orchestration.
// Computes a structural signature to skip expensive DOM rebuilds when topology
// is unchanged, while still updating selection and activity state each frame.
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

function syncSelectionDetails({
  nodes,
  edges,
  controlEdges,
  showControlChannels,
  selectedEdgeId,
  selectedEdgeData,
  selectedNodeId,
  selectedNodeData,
  renderEdgeDetails,
  renderNodeDetails,
  renderSelectionNone,
}) {
  let nextSelectedEdgeId = selectedEdgeId;
  let nextSelectedEdgeData = selectedEdgeData;
  let nextSelectedNodeId = selectedNodeId;
  let nextSelectedNodeData = selectedNodeData;

  if (nextSelectedEdgeId) {
    const selectedEdge =
      edges.find((edge) => edge.id === nextSelectedEdgeId) ||
      (showControlChannels
        ? controlEdges.find((edge) => edge.id === nextSelectedEdgeId)
        : null);
    if (selectedEdge) {
      nextSelectedEdgeData = selectedEdge;
      renderEdgeDetails(selectedEdge);
      return {
        selectedEdgeId: nextSelectedEdgeId,
        selectedEdgeData: nextSelectedEdgeData,
        selectedNodeId: nextSelectedNodeId,
        selectedNodeData: nextSelectedNodeData,
      };
    }
    if (nextSelectedEdgeData && nextSelectedEdgeData.id === nextSelectedEdgeId) {
      renderEdgeDetails(nextSelectedEdgeData);
      return {
        selectedEdgeId: nextSelectedEdgeId,
        selectedEdgeData: nextSelectedEdgeData,
        selectedNodeId: nextSelectedNodeId,
        selectedNodeData: nextSelectedNodeData,
      };
    }
    nextSelectedEdgeId = null;
    nextSelectedEdgeData = null;
    renderSelectionNone();
    return {
      selectedEdgeId: nextSelectedEdgeId,
      selectedEdgeData: nextSelectedEdgeData,
      selectedNodeId: nextSelectedNodeId,
      selectedNodeData: nextSelectedNodeData,
    };
  }
  if (nextSelectedNodeId) {
    const selectedNode = nodes.find((node) => node.id === nextSelectedNodeId);
    if (selectedNode) {
      nextSelectedNodeData = selectedNode;
      renderNodeDetails(selectedNode);
      return {
        selectedEdgeId: nextSelectedEdgeId,
        selectedEdgeData: nextSelectedEdgeData,
        selectedNodeId: nextSelectedNodeId,
        selectedNodeData: nextSelectedNodeData,
      };
    }
    if (nextSelectedNodeData && nextSelectedNodeData.id === nextSelectedNodeId) {
      renderNodeDetails(nextSelectedNodeData);
      return {
        selectedEdgeId: nextSelectedEdgeId,
        selectedEdgeData: nextSelectedEdgeData,
        selectedNodeId: nextSelectedNodeId,
        selectedNodeData: nextSelectedNodeData,
      };
    }
    nextSelectedNodeId = null;
    nextSelectedNodeData = null;
    renderSelectionNone();
    return {
      selectedEdgeId: nextSelectedEdgeId,
      selectedEdgeData: nextSelectedEdgeData,
      selectedNodeId: nextSelectedNodeId,
      selectedNodeData: nextSelectedNodeData,
    };
  }
  renderSelectionNone();
  return {
    selectedEdgeId: nextSelectedEdgeId,
    selectedEdgeData: nextSelectedEdgeData,
    selectedNodeId: nextSelectedNodeId,
    selectedNodeData: nextSelectedNodeData,
  };
}

export function renderGraphFrame({
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
  applyDefaultOverviewZoom,
  applyZoom,
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
  onOverlayError = () => {},
}) {
  const perfMs = perfStart();
  let renderMode = "full";
  let nextSelectedEdgeId = selectedEdgeId;
  let nextSelectedEdgeData = selectedEdgeData;
  let nextSelectedNodeId = selectedNodeId;
  let nextSelectedNodeData = selectedNodeData;
  let nextStructureSignature = lastRenderedStructureSignature;
  let nextLastRenderedNodes = [];
  let nextLastRenderedEdges = [];
  let nextLastRenderedControlEdges = [];
  let nextLastRenderedNodesById = new Map();
  let nextLastRenderedEdgesById = new Map();
  let nextLastRenderedControlEdgesById = new Map();
  let nextLastRenderedSampleSeconds = null;
  let nextLastGraph = dataGraph || { nodes: [], edges: [], meta: {} };
  let nextLastEdgeRates = new Map();
  let nextLayoutSize =
    layoutSize && Number.isFinite(layoutSize.width) && Number.isFinite(layoutSize.height)
      ? layoutSize
      : { width: 0, height: 0 };

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
    nextLastEdgeRates = dataEdgeRates;

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
      if (nextSelectedNodeId && !nodes.find((node) => node.id === nextSelectedNodeId)) {
        nextSelectedNodeId = null;
        nextSelectedNodeData = null;
      }
    }

    if (dagSearchQuery) {
      const searchResult = filterGraphByQuery(nodes, edges, dagSearchQuery);
      nodes = searchResult.nodes;
      edges = searchResult.edges;
      if (nextSelectedNodeId && !nodes.find((node) => node.id === nextSelectedNodeId)) {
        nextSelectedNodeId = null;
        nextSelectedNodeData = null;
      }
    }

    const dataEdgeIds = new Set(edges.map((edge) => edge.id));
    if (
      nextSelectedEdgeId &&
      !controlEdgeIds.has(nextSelectedEdgeId) &&
      !dataEdgeIds.has(nextSelectedEdgeId)
    ) {
      nextSelectedEdgeId = null;
      nextSelectedEdgeData = null;
    }

    if (!showControlChannels && nextSelectedEdgeId && controlEdgeIds.has(nextSelectedEdgeId)) {
      nextSelectedEdgeId = null;
      nextSelectedEdgeData = null;
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

    nextLastRenderedNodes = nodes;
    nextLastRenderedEdges = edges;
    nextLastRenderedControlEdges = controlEdges;
    nextLastRenderedNodesById = new Map(nodes.map((node) => [node.id, node]));
    nextLastRenderedEdgesById = new Map(edges.map((edge) => [edge.id, edge]));
    nextLastRenderedControlEdgesById = new Map(controlEdges.map((edge) => [edge.id, edge]));
    nextLastRenderedSampleSeconds = sampleSeconds;
    nextLastGraph = dataGraphResolved;

    dagEmpty.classList.toggle("hidden", edges.length > 0);
    if (!edges.length) {
      nextSelectedEdgeId = null;
      nextSelectedEdgeData = null;
    }
    if (!nodes.length) {
      nextSelectedNodeId = null;
      nextSelectedNodeData = null;
    }

    const hasRenderedDagDom =
      dagNodes.childElementCount > 0 ||
      dagEdges.childElementCount > 0 ||
      dagLanes.childElementCount > 0;
    if (structureSignature === nextStructureSignature && hasRenderedDagDom) {
      renderMode = "reuse";
      updateTopologyForHover(displayTimeMs);
      const synced = syncSelectionDetails({
        nodes,
        edges,
        controlEdges,
        showControlChannels,
        selectedEdgeId: nextSelectedEdgeId,
        selectedEdgeData: nextSelectedEdgeData,
        selectedNodeId: nextSelectedNodeId,
        selectedNodeData: nextSelectedNodeData,
        renderEdgeDetails,
        renderNodeDetails,
        renderSelectionNone,
      });
      nextSelectedEdgeId = synced.selectedEdgeId;
      nextSelectedEdgeData = synced.selectedEdgeData;
      nextSelectedNodeId = synced.selectedNodeId;
      nextSelectedNodeData = synced.selectedNodeData;
      return {
        selectedEdgeId: nextSelectedEdgeId,
        selectedEdgeData: nextSelectedEdgeData,
        selectedNodeId: nextSelectedNodeId,
        selectedNodeData: nextSelectedNodeData,
        lastRenderedStructureSignature: nextStructureSignature,
        lastRenderedNodes: nextLastRenderedNodes,
        lastRenderedEdges: nextLastRenderedEdges,
        lastRenderedControlEdges: nextLastRenderedControlEdges,
        lastRenderedNodesById: nextLastRenderedNodesById,
        lastRenderedEdgesById: nextLastRenderedEdgesById,
        lastRenderedControlEdgesById: nextLastRenderedControlEdgesById,
        lastRenderedSampleSeconds: nextLastRenderedSampleSeconds,
        lastGraph: nextLastGraph,
        lastEdgeRates: nextLastEdgeRates,
        layoutSize: nextLayoutSize,
      };
    }

    nextStructureSignature = structureSignature;
    renderMode = "incremental";

    const layout = layoutGraph(nodes, edges);
    const baseNodeMap = new Map(nodes.map((node) => [node.id, node]));
    const pipelineNavAnchors =
      activeDagScope.mode === "connected"
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

    nextLayoutSize = { width: layout.width, height: layout.height };
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

    if (activeDagScope.mode === "connected") {
      try {
        renderConnectedTopicNavigation(nodeMap, activeDagScope);
      } catch (error) {
        onOverlayError(error);
      }
    } else {
      renderPipelineDagNavigation(nodeMap, pipelineNavAnchors, pipelineNavLayout);
    }

    const synced = syncSelectionDetails({
      nodes,
      edges,
      controlEdges,
      showControlChannels,
      selectedEdgeId: nextSelectedEdgeId,
      selectedEdgeData: nextSelectedEdgeData,
      selectedNodeId: nextSelectedNodeId,
      selectedNodeData: nextSelectedNodeData,
      renderEdgeDetails,
      renderNodeDetails,
      renderSelectionNone,
    });
    nextSelectedEdgeId = synced.selectedEdgeId;
    nextSelectedEdgeData = synced.selectedEdgeData;
    nextSelectedNodeId = synced.selectedNodeId;
    nextSelectedNodeData = synced.selectedNodeData;
  } finally {
    perfEnd("renderGraph", perfMs, {
      mode: renderMode,
      nodes: nextLastRenderedNodes.length,
      edges: nextLastRenderedEdges.length,
      control: nextLastRenderedControlEdges.length,
    });
  }

  return {
    selectedEdgeId: nextSelectedEdgeId,
    selectedEdgeData: nextSelectedEdgeData,
    selectedNodeId: nextSelectedNodeId,
    selectedNodeData: nextSelectedNodeData,
    lastRenderedStructureSignature: nextStructureSignature,
    lastRenderedNodes: nextLastRenderedNodes,
    lastRenderedEdges: nextLastRenderedEdges,
    lastRenderedControlEdges: nextLastRenderedControlEdges,
    lastRenderedNodesById: nextLastRenderedNodesById,
    lastRenderedEdgesById: nextLastRenderedEdgesById,
    lastRenderedControlEdgesById: nextLastRenderedControlEdgesById,
    lastRenderedSampleSeconds: nextLastRenderedSampleSeconds,
    lastGraph: nextLastGraph,
    lastEdgeRates: nextLastEdgeRates,
    layoutSize: nextLayoutSize,
  };
}
