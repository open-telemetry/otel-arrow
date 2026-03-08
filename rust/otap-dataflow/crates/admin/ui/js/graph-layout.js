const DEFAULT_CONSTANTS = {
  NODE_WIDTH: 210,
  NODE_HEADER_HEIGHT: 38,
  NODE_PADDING_Y: 6,
  PORT_ROW_HEIGHT: 20,
  NODE_FOOTER_HEIGHT: 12,
  LEVEL_GAP: 140,
  ROW_GAP: 40,
  MARGIN: 48,
  TOP_PADDING: 18,
  EDGE_INSET: 6,
  MULTI_PIPELINE_COLUMN_GAP: 120,
  MULTI_PIPELINE_ROW_GAP: 5,
};

function getPipelineLayoutLabel(pipelineKey, interPipelineTopology) {
  if (!pipelineKey) return "";
  const pipeline = interPipelineTopology?.pipelineByKey?.get(pipelineKey);
  if (!pipeline) return pipelineKey;
  const groupLabel = pipeline.groupId ? `${pipeline.groupId}/` : "";
  return `${groupLabel}${pipeline.pipelineId || pipeline.key || pipelineKey}`;
}

function comparePipelineKeys(a, b, interPipelineTopology) {
  return getPipelineLayoutLabel(a, interPipelineTopology).localeCompare(
    getPipelineLayoutLabel(b, interPipelineTopology),
    undefined,
    { numeric: true, sensitivity: "base" }
  );
}

function collectPipelineNeighborsForLayout(pipelineKey, allowedKeysSet, interPipelineTopology) {
  const pipeline = interPipelineTopology?.pipelineByKey?.get(pipelineKey);
  if (!pipeline) return [];
  const neighbors = [];
  (pipeline.upstream || []).forEach((entry) => {
    if (!entry?.key || !allowedKeysSet.has(entry.key)) return;
    neighbors.push({ key: entry.key, side: "upstream" });
  });
  (pipeline.downstream || []).forEach((entry) => {
    if (!entry?.key || !allowedKeysSet.has(entry.key)) return;
    neighbors.push({ key: entry.key, side: "downstream" });
  });
  return neighbors;
}

function computePipelineColumnMap(pipelineKeys, selectedPipelineKey, interPipelineTopology) {
  const keys = Array.from(new Set(pipelineKeys || []));
  const keySet = new Set(keys);
  const columns = new Map();
  if (!keys.length) return columns;

  const rootKey =
    selectedPipelineKey && keySet.has(selectedPipelineKey)
      ? selectedPipelineKey
      : keys.sort((a, b) => comparePipelineKeys(a, b, interPipelineTopology))[0];
  const info = new Map();
  info.set(rootKey, { distance: 0, firstHop: "center" });
  const queue = [rootKey];

  while (queue.length) {
    const current = queue.shift();
    const currentInfo = info.get(current);
    if (!currentInfo) continue;
    const neighbors = collectPipelineNeighborsForLayout(
      current,
      keySet,
      interPipelineTopology
    );
    neighbors.forEach((neighbor) => {
      const candidateDistance = currentInfo.distance + 1;
      const candidateFirstHop =
        currentInfo.firstHop === "center" ? neighbor.side : currentInfo.firstHop;
      const existing = info.get(neighbor.key);
      if (!existing || candidateDistance < existing.distance) {
        info.set(neighbor.key, {
          distance: candidateDistance,
          firstHop: candidateFirstHop,
        });
        queue.push(neighbor.key);
      }
    });
  }

  keys.forEach((key) => {
    const state = info.get(key);
    if (!state || state.firstHop === "center") {
      columns.set(key, 0);
      return;
    }
    if (state.firstHop === "upstream") {
      columns.set(key, -Math.max(1, state.distance));
      return;
    }
    columns.set(key, Math.max(1, state.distance));
  });

  return columns;
}

export function shouldCollapseDefaultOutputPort(node) {
  const ports = node?.outPorts || [];
  return ports.length === 1 && ports[0] === "default";
}

export function getNodeOutputAnchorY(node, portName, constants = DEFAULT_CONSTANTS) {
  const { NODE_PADDING_Y, NODE_HEADER_HEIGHT, PORT_ROW_HEIGHT } = constants;
  if (!node) return 0;
  if (shouldCollapseDefaultOutputPort(node)) {
    return node.y + node.height / 2;
  }
  const portIndex = node.portIndex?.[portName] ?? 0;
  return node.y + NODE_PADDING_Y + NODE_HEADER_HEIGHT + (portIndex + 0.5) * PORT_ROW_HEIGHT;
}

function layoutSinglePipelineGraph(nodes, edges, constants) {
  const {
    NODE_WIDTH,
    NODE_HEADER_HEIGHT,
    NODE_FOOTER_HEIGHT,
    NODE_PADDING_Y,
    PORT_ROW_HEIGHT,
    LEVEL_GAP,
    ROW_GAP,
    MARGIN,
    TOP_PADDING,
    EDGE_INSET,
  } = constants;

  const nodeById = new Map(nodes.map((node) => [node.id, node]));
  const trafficTotals = new Map(nodes.map((node) => [node.id, { sent: 0, received: 0 }]));
  const metricValue = (metrics, name) => {
    if (!metrics) return 0;
    for (const metric of metrics) {
      if (metric.name === name && typeof metric.value === "number") {
        return metric.value;
      }
    }
    return 0;
  };

  const lineIntersectsRect = (x1, y1, x2, y2, rect) => {
    const left = rect.x;
    const right = rect.x + rect.width;
    const top = rect.y;
    const bottom = rect.y + rect.height;
    const dx = x2 - x1;
    const dy = y2 - y1;
    let t0 = 0;
    let t1 = 1;
    const p = [-dx, dx, -dy, dy];
    const q = [x1 - left, right - x1, y1 - top, bottom - y1];

    for (let i = 0; i < 4; i += 1) {
      if (p[i] === 0) {
        if (q[i] < 0) return false;
      } else {
        const r = q[i] / p[i];
        if (p[i] < 0) {
          if (r > t1) return false;
          if (r > t0) t0 = r;
        } else {
          if (r < t0) return false;
          if (r < t1) t1 = r;
        }
      }
    }
    return true;
  };

  const trafficScore = (nodeId) => {
    const totals = trafficTotals.get(nodeId);
    if (!totals) return 0;
    return totals.sent + totals.received;
  };

  const nodeLabel = (node) => node?.displayId || node?.id || "";

  const maxLabelChars = nodes.reduce((max, node) => Math.max(max, nodeLabel(node).length), 6);
  const columnWidth = Math.min(210, Math.max(140, 44 + maxLabelChars * 6.8));

  edges.forEach((edge) => {
    const sendCount = metricValue(edge.data.sender?.metrics, "send.count");
    const recvCount = metricValue(edge.data.receiver?.metrics, "recv.count");
    if (trafficTotals.has(edge.source)) {
      trafficTotals.get(edge.source).sent += sendCount;
    }
    if (trafficTotals.has(edge.target)) {
      trafficTotals.get(edge.target).received += recvCount;
    }
  });

  const receivers = [];
  const processors = [];
  const exporters = [];

  nodes.forEach((node) => {
    const type = (node.attrs["node.type"] || "processor").toLowerCase();
    if (type === "receiver") {
      receivers.push(node);
    } else if (type === "exporter") {
      exporters.push(node);
    } else {
      processors.push(node);
    }
  });

  receivers.sort((a, b) => {
    const aSent = trafficTotals.get(a.id)?.sent ?? 0;
    const bSent = trafficTotals.get(b.id)?.sent ?? 0;
    if (bSent !== aSent) return bSent - aSent;
    return nodeLabel(a).localeCompare(nodeLabel(b), undefined, {
      numeric: true,
      sensitivity: "base",
    });
  });
  exporters.sort((a, b) => {
    const aRecv = trafficTotals.get(a.id)?.received ?? 0;
    const bRecv = trafficTotals.get(b.id)?.received ?? 0;
    if (bRecv !== aRecv) return bRecv - aRecv;
    return nodeLabel(a).localeCompare(nodeLabel(b), undefined, {
      numeric: true,
      sensitivity: "base",
    });
  });
  const processorTraffic = (node) => trafficScore(node.id);
  processors.sort((a, b) => {
    const aTraffic = processorTraffic(a);
    const bTraffic = processorTraffic(b);
    if (bTraffic !== aTraffic) return bTraffic - aTraffic;
    return nodeLabel(a).localeCompare(nodeLabel(b), undefined, {
      numeric: true,
      sensitivity: "base",
    });
  });

  const processorIds = new Set(processors.map((node) => node.id));
  const incoming = new Map(processors.map((node) => [node.id, 0]));
  const outgoing = new Map(processors.map((node) => [node.id, []]));

  edges.forEach((edge) => {
    if (!processorIds.has(edge.source) || !processorIds.has(edge.target)) {
      return;
    }
    incoming.set(edge.target, incoming.get(edge.target) + 1);
    outgoing.get(edge.source).push(edge.target);
  });

  const level = new Map();
  const maxParent = new Map();
  const queue = [];
  for (const [nodeId, count] of incoming.entries()) {
    if (count === 0) {
      level.set(nodeId, 0);
      queue.push(nodeId);
    }
  }

  while (queue.length) {
    const nodeId = queue.shift();
    const base = level.get(nodeId) ?? 0;
    for (const target of outgoing.get(nodeId) || []) {
      const candidate = base + 1;
      maxParent.set(target, Math.max(maxParent.get(target) ?? 0, candidate));
      incoming.set(target, incoming.get(target) - 1);
      if (incoming.get(target) === 0) {
        level.set(target, maxParent.get(target) ?? 0);
        queue.push(target);
      }
    }
  }

  let maxLevel = 0;
  for (const node of processors) {
    if (!level.has(node.id)) {
      level.set(node.id, 0);
    }
    maxLevel = Math.max(maxLevel, level.get(node.id));
  }

  const processorColumns = [];
  for (let i = 0; i <= maxLevel; i += 1) {
    processorColumns.push([]);
  }
  processors.forEach((node) => {
    const l = level.get(node.id) ?? 0;
    processorColumns[l].push(node);
  });
  processorColumns.forEach((bucket) =>
    bucket.sort((a, b) => {
      const aTraffic = processorTraffic(a);
      const bTraffic = processorTraffic(b);
      if (bTraffic !== aTraffic) return bTraffic - aTraffic;
      return a.id.localeCompare(b.id);
    })
  );

  const columns = [];
  const laneMeta = [];
  let columnIndex = 0;
  if (receivers.length) {
    columns.push(receivers);
    laneMeta.push({
      label: "Receivers",
      start: columnIndex,
      end: columnIndex,
    });
    columnIndex += 1;
  }
  if (processors.length) {
    columns.push(...processorColumns);
    laneMeta.push({
      label: "Processors",
      start: columnIndex,
      end: columnIndex + processorColumns.length - 1,
    });
    columnIndex += processorColumns.length;
  }
  if (exporters.length) {
    columns.push(exporters);
    laneMeta.push({
      label: "Exporters",
      start: columnIndex,
      end: columnIndex,
    });
    columnIndex += 1;
  }
  if (!columns.length) {
    columns.push([]);
  }

  let canvasHeight = 0;
  for (const node of nodes) {
    node.displayPorts = shouldCollapseDefaultOutputPort(node)
      ? []
      : node.outPorts.length
        ? node.outPorts
        : [];
    const portCount = node.displayPorts.length;
    node.portIndex = {};
    node.outPorts.forEach((port, idx) => {
      node.portIndex[port] = idx;
    });
    node.height = NODE_HEADER_HEIGHT + portCount * PORT_ROW_HEIGHT + NODE_FOOTER_HEIGHT;
    node.width = columnWidth;
  }

  columns.forEach((bucket, columnIndexValue) => {
    let y = MARGIN + TOP_PADDING;
    bucket.forEach((node) => {
      node.x = MARGIN + columnIndexValue * (columnWidth + LEVEL_GAP);
      node.y = y;
      y += node.height + ROW_GAP;
    });
    canvasHeight = Math.max(canvasHeight, y);
  });

  if (edges.length) {
    const columnFor = new Map();
    columns.forEach((bucket) => {
      bucket.forEach((node) => columnFor.set(node.id, bucket));
    });

    const padding = 12;
    const maxIterations = 3;
    for (let iter = 0; iter < maxIterations; iter += 1) {
      let moved = false;
      edges.forEach((edge) => {
        const source = nodeById.get(edge.source);
        const target = nodeById.get(edge.target);
        if (!source || !target) return;
        const startX = source.x + source.width - EDGE_INSET;
        const startY = getNodeOutputAnchorY(source, edge.port, constants);
        const endX = target.x - EDGE_INSET;
        const endY = target.y + target.height / 2;

        nodes.forEach((node) => {
          if (node.id === edge.source || node.id === edge.target) {
            return;
          }
          const rect = {
            x: node.x - padding,
            y: node.y - padding,
            width: node.width + padding * 2,
            height: node.height + padding * 2,
          };
          if (!lineIntersectsRect(startX, startY, endX, endY, rect)) {
            return;
          }

          const sourceScore = trafficScore(edge.source);
          const targetScore = trafficScore(edge.target);
          const nodeScore = trafficScore(node.id);
          let movedNode = node;
          let movedScore = nodeScore;

          if (sourceScore < movedScore) {
            movedNode = source;
            movedScore = sourceScore;
          }
          if (targetScore < movedScore) {
            movedNode = target;
            movedScore = targetScore;
          }
          const bucket = columnFor.get(movedNode.id);
          if (!bucket) return;

          const minY = node.y + node.height + ROW_GAP;
          if (movedNode.y < minY) {
            movedNode.y = minY;
            moved = true;
          }
        });
      });

      columns.forEach((bucket) => {
        let yCursor = MARGIN + TOP_PADDING;
        bucket.forEach((node) => {
          if (node.y < yCursor) {
            node.y = yCursor;
            moved = true;
          }
          yCursor = node.y + node.height + ROW_GAP;
        });
      });

      if (!moved) break;
    }

    canvasHeight = 0;
    columns.forEach((bucket) => {
      if (!bucket.length) return;
      const last = bucket[bucket.length - 1];
      canvasHeight = Math.max(canvasHeight, last.y + last.height + ROW_GAP);
    });
  }

  const columnCount = columns.length;
  const canvasWidth = MARGIN * 2 + columnCount * columnWidth + (columnCount - 1) * LEVEL_GAP;

  return {
    width: Math.max(canvasWidth, 640),
    height: Math.max(canvasHeight + MARGIN, 520),
    lanes: laneMeta,
    columnWidth,
  };
}

function layoutMultiPipelineGraph(nodes, edges, pipelineKeys, options, constants) {
  const { interPipelineTopology, selectedPipelineKey } = options;
  const { MARGIN, LEVEL_GAP, MULTI_PIPELINE_COLUMN_GAP, MULTI_PIPELINE_ROW_GAP, NODE_WIDTH } =
    constants;

  const keys = Array.from(new Set(pipelineKeys || [])).sort((a, b) =>
    comparePipelineKeys(a, b, interPipelineTopology)
  );
  if (keys.length <= 1) {
    return layoutSinglePipelineGraph(nodes, edges, constants);
  }

  const nodesByPipeline = new Map();
  keys.forEach((key) => nodesByPipeline.set(key, []));
  nodes.forEach((node) => {
    const key = node.attrs?.["ui.pipeline.key"];
    if (!key || !nodesByPipeline.has(key)) return;
    nodesByPipeline.get(key).push(node);
  });

  const nodePipelineById = new Map();
  nodes.forEach((node) => {
    const key = node.attrs?.["ui.pipeline.key"];
    if (key) {
      nodePipelineById.set(node.id, key);
    }
  });

  const edgesByPipeline = new Map();
  keys.forEach((key) => edgesByPipeline.set(key, []));
  edges.forEach((edge) => {
    const sourceKey = nodePipelineById.get(edge.source);
    const targetKey = nodePipelineById.get(edge.target);
    if (!sourceKey || sourceKey !== targetKey || !edgesByPipeline.has(sourceKey)) return;
    edgesByPipeline.get(sourceKey).push(edge);
  });

  const subLayouts = new Map();
  keys.forEach((key) => {
    const pipelineNodes = nodesByPipeline.get(key) || [];
    const pipelineEdges = edgesByPipeline.get(key) || [];
    subLayouts.set(key, layoutSinglePipelineGraph(pipelineNodes, pipelineEdges, constants));
  });

  const columnMap = computePipelineColumnMap(
    keys,
    selectedPipelineKey,
    interPipelineTopology
  );
  const keysByColumn = new Map();
  keys.forEach((key) => {
    const column = columnMap.get(key) ?? 0;
    if (!keysByColumn.has(column)) {
      keysByColumn.set(column, []);
    }
    keysByColumn.get(column).push(key);
  });
  keysByColumn.forEach((columnKeys) =>
    columnKeys.sort((a, b) => comparePipelineKeys(a, b, interPipelineTopology))
  );

  const orderedColumns = Array.from(keysByColumn.keys()).sort((a, b) => a - b);
  const columnWidths = new Map();
  orderedColumns.forEach((column) => {
    const width = (keysByColumn.get(column) || []).reduce((maxWidth, key) => {
      return Math.max(maxWidth, subLayouts.get(key)?.width || 0);
    }, 0);
    columnWidths.set(column, width);
  });

  const columnX = new Map();
  let xCursor = 0;
  orderedColumns.forEach((column, index) => {
    if (index > 0) {
      xCursor += MULTI_PIPELINE_COLUMN_GAP;
    }
    columnX.set(column, xCursor);
    xCursor += columnWidths.get(column) || 0;
  });

  const columnHeights = new Map();
  orderedColumns.forEach((column) => {
    const columnKeys = keysByColumn.get(column) || [];
    const height = columnKeys.reduce((sum, key, index) => {
      const pipelineHeight = subLayouts.get(key)?.height || 0;
      return sum + pipelineHeight + (index > 0 ? MULTI_PIPELINE_ROW_GAP : 0);
    }, 0);
    columnHeights.set(column, height);
  });
  const totalHeight = orderedColumns.reduce((maxHeight, column) => {
    return Math.max(maxHeight, columnHeights.get(column) || 0);
  }, 0);

  const offsets = new Map();
  orderedColumns.forEach((column) => {
    const columnKeys = keysByColumn.get(column) || [];
    const columnHeight = columnHeights.get(column) || 0;
    let yCursor = Math.max(0, (totalHeight - columnHeight) / 2);
    columnKeys.forEach((key, index) => {
      if (index > 0) {
        yCursor += MULTI_PIPELINE_ROW_GAP;
      }
      offsets.set(key, { x: columnX.get(column) || 0, y: yCursor });
      yCursor += subLayouts.get(key)?.height || 0;
    });
  });

  const lanes = [];
  keys.forEach((key) => {
    const offset = offsets.get(key) || { x: 0, y: 0 };
    const subLayout = subLayouts.get(key);
    if (!subLayout) return;

    (nodesByPipeline.get(key) || []).forEach((node) => {
      node.x += offset.x;
      node.y += offset.y;
    });

    (subLayout.lanes || []).forEach((lane) => {
      const laneStartX = MARGIN + lane.start * (subLayout.columnWidth + LEVEL_GAP);
      const laneWidth =
        (lane.end - lane.start + 1) * subLayout.columnWidth +
        (lane.end - lane.start) * LEVEL_GAP;
      lanes.push({
        label: lane.label,
        x: offset.x + laneStartX,
        width: laneWidth,
      });
    });
  });

  return {
    width: Math.max(640, xCursor),
    height: Math.max(520, totalHeight),
    lanes,
    columnWidth: NODE_WIDTH,
  };
}

export function layoutGraph(nodes, edges, options = {}) {
  const constants = { ...DEFAULT_CONSTANTS, ...(options.constants || {}) };
  const pipelineKeys = Array.from(
    new Set((nodes || []).map((node) => node.attrs?.["ui.pipeline.key"]).filter((value) => value))
  );
  if (pipelineKeys.length > 1) {
    return layoutMultiPipelineGraph(nodes, edges, pipelineKeys, options, constants);
  }
  return layoutSinglePipelineGraph(nodes, edges, constants);
}
