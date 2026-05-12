// Inter-pipeline topic topology derivation and query helpers.
// Builds deterministic upstream/downstream relationships for selectors and DAG scope.
import {
  getPipelineGroupId,
  getPipelineId,
  makePipelineSelectionKey,
  normalizeAttributes,
} from "./pipeline-utils.js";
const EMPTY_PIPELINE_INTERCONNECT = Object.freeze({
  hasTopicExporters: false,
  hasTopicReceivers: false,
  upstream: [],
  downstream: [],
});

function sortPipelineRef(a, b) {
  const groupCmp = (a.groupId || "").localeCompare(b.groupId || "", undefined, {
    numeric: true,
    sensitivity: "base",
  });
  if (groupCmp !== 0) return groupCmp;
  return (a.pipelineId || "").localeCompare(b.pipelineId || "", undefined, {
    numeric: true,
    sensitivity: "base",
  });
}

function sortEdgeRef(a, b) {
  const topicCmp = (a.topic || "").localeCompare(b.topic || "", undefined, {
    numeric: true,
    sensitivity: "base",
  });
  if (topicCmp !== 0) return topicCmp;
  const processCmp = (a.processInstanceId || "").localeCompare(
    b.processInstanceId || "",
    undefined,
    { numeric: true, sensitivity: "base" }
  );
  if (processCmp !== 0) return processCmp;
  const fromNodeCmp = (a.fromNodeId || "").localeCompare(b.fromNodeId || "", undefined, {
    numeric: true,
    sensitivity: "base",
  });
  if (fromNodeCmp !== 0) return fromNodeCmp;
  return (a.toNodeId || "").localeCompare(b.toNodeId || "", undefined, {
    numeric: true,
    sensitivity: "base",
  });
}

function resolveTopicName(attrs) {
  const value = attrs.topic;
  return value != null && value !== "" ? String(value) : "";
}

function buildTopicEndpoint(set, role) {
  const attrs = normalizeAttributes(set.attributes || {});
  const topic = resolveTopicName(attrs);
  const pipelineId = getPipelineId(attrs);
  if (!topic || !pipelineId) return null;

  const groupId = getPipelineGroupId(attrs);
  return {
    role,
    topic,
    processInstanceId: attrs["process.instance.id"] || "",
    pipelineKey: makePipelineSelectionKey(groupId, pipelineId),
    groupId,
    pipelineId,
    nodeId: attrs["node.id"] || "",
  };
}

function getOrCreatePipelineState(pipelinesByKey, endpoint) {
  let pipeline = pipelinesByKey.get(endpoint.pipelineKey);
  if (!pipeline) {
    pipeline = {
      key: endpoint.pipelineKey,
      groupId: endpoint.groupId,
      pipelineId: endpoint.pipelineId,
      hasTopicExporters: false,
      hasTopicReceivers: false,
      upstream: new Map(),
      downstream: new Map(),
    };
    pipelinesByKey.set(endpoint.pipelineKey, pipeline);
  }
  return pipeline;
}

function addNeighborEdge(relationMap, neighbor, edge) {
  let relation = relationMap.get(neighbor.pipelineKey);
  if (!relation) {
    relation = {
      key: neighbor.pipelineKey,
      groupId: neighbor.groupId,
      pipelineId: neighbor.pipelineId,
      topics: new Set(),
      edges: [],
      edgeIds: new Set(),
    };
    relationMap.set(neighbor.pipelineKey, relation);
  }

  relation.topics.add(edge.topic);
  const edgeId =
    `${edge.topic}\u0000${edge.processInstanceId}\u0000${edge.fromNodeId}\u0000${edge.toNodeId}`;
  if (!relation.edgeIds.has(edgeId)) {
    relation.edgeIds.add(edgeId);
    relation.edges.push({
      topic: edge.topic,
      processInstanceId: edge.processInstanceId,
      fromNodeId: edge.fromNodeId,
      toNodeId: edge.toNodeId,
    });
  }
}

function finalizeRelationMap(relationMap) {
  return Array.from(relationMap.values())
    .map((relation) => ({
      key: relation.key,
      groupId: relation.groupId,
      pipelineId: relation.pipelineId,
      topics: Array.from(relation.topics).sort((a, b) =>
        a.localeCompare(b, undefined, { numeric: true, sensitivity: "base" })
      ),
      edgeCount: relation.edges.length,
      edges: relation.edges.sort(sortEdgeRef),
    }))
    .sort(sortPipelineRef);
}

export function createEmptyInterPipelineTopology() {
  return {
    pipelineCount: 0,
    edgeCount: 0,
    pipelines: [],
    pipelineByKey: new Map(),
    edges: [],
  };
}

// Build directed inter-pipeline links by joining topic exporters and receivers on (process, topic).
export function buildInterPipelineTopology(metricSets) {
  const pipelinesByKey = new Map();
  const exportersByJoinKey = new Map();
  const receiversByJoinKey = new Map();
  const endpointIds = new Set();

  for (const set of metricSets || []) {
    if (set.name !== "topic.exporter" && set.name !== "topic.receiver") {
      continue;
    }

    const role = set.name === "topic.exporter" ? "exporter" : "receiver";
    const endpoint = buildTopicEndpoint(set, role);
    if (!endpoint) continue;

    const endpointId =
      `${role}\u0000${endpoint.processInstanceId}\u0000${endpoint.topic}` +
      `\u0000${endpoint.pipelineKey}\u0000${endpoint.nodeId}`;
    if (endpointIds.has(endpointId)) continue;
    endpointIds.add(endpointId);

    const pipeline = getOrCreatePipelineState(pipelinesByKey, endpoint);
    if (role === "exporter") {
      pipeline.hasTopicExporters = true;
    } else {
      pipeline.hasTopicReceivers = true;
    }

    const joinKey = `${endpoint.processInstanceId}\u0000${endpoint.topic}`;
    const map = role === "exporter" ? exportersByJoinKey : receiversByJoinKey;
    if (!map.has(joinKey)) {
      map.set(joinKey, []);
    }
    map.get(joinKey).push(endpoint);
  }

  const edges = [];
  const edgeIds = new Set();

  for (const [joinKey, exporters] of exportersByJoinKey.entries()) {
    const receivers = receiversByJoinKey.get(joinKey);
    if (!receivers || !receivers.length) continue;

    for (const exporter of exporters) {
      for (const receiver of receivers) {
        if (exporter.pipelineKey === receiver.pipelineKey) {
          continue;
        }

        const edgeId =
          `${joinKey}\u0000${exporter.pipelineKey}\u0000${exporter.nodeId}` +
          `\u0000${receiver.pipelineKey}\u0000${receiver.nodeId}`;
        if (edgeIds.has(edgeId)) continue;
        edgeIds.add(edgeId);

        const edge = {
          id: edgeId,
          topic: exporter.topic,
          processInstanceId: exporter.processInstanceId,
          from: {
            key: exporter.pipelineKey,
            groupId: exporter.groupId,
            pipelineId: exporter.pipelineId,
            nodeId: exporter.nodeId,
          },
          to: {
            key: receiver.pipelineKey,
            groupId: receiver.groupId,
            pipelineId: receiver.pipelineId,
            nodeId: receiver.nodeId,
          },
          fromNodeId: exporter.nodeId,
          toNodeId: receiver.nodeId,
        };
        edges.push(edge);

        const sourcePipeline = getOrCreatePipelineState(pipelinesByKey, exporter);
        const targetPipeline = getOrCreatePipelineState(pipelinesByKey, receiver);
        addNeighborEdge(sourcePipeline.downstream, receiver, edge);
        addNeighborEdge(targetPipeline.upstream, exporter, edge);
      }
    }
  }

  const pipelines = Array.from(pipelinesByKey.values())
    .map((pipeline) => ({
      key: pipeline.key,
      groupId: pipeline.groupId,
      pipelineId: pipeline.pipelineId,
      hasTopicExporters: pipeline.hasTopicExporters,
      hasTopicReceivers: pipeline.hasTopicReceivers,
      upstream: finalizeRelationMap(pipeline.upstream),
      downstream: finalizeRelationMap(pipeline.downstream),
    }))
    .sort(sortPipelineRef);
  const pipelineByKey = new Map(pipelines.map((pipeline) => [pipeline.key, pipeline]));

  return {
    pipelineCount: pipelines.length,
    edgeCount: edges.length,
    pipelines,
    pipelineByKey,
    edges,
  };
}

export function getPipelineInterconnect(topology, pipelineKey) {
  if (!pipelineKey || !topology || !(topology.pipelineByKey instanceof Map)) {
    return EMPTY_PIPELINE_INTERCONNECT;
  }
  const pipeline = topology.pipelineByKey.get(pipelineKey);
  if (!pipeline) return EMPTY_PIPELINE_INTERCONNECT;
  return {
    hasTopicExporters: pipeline.hasTopicExporters,
    hasTopicReceivers: pipeline.hasTopicReceivers,
    upstream: pipeline.upstream,
    downstream: pipeline.downstream,
  };
}

// Returns the selected pipeline plus all pipelines reachable through topic links
// (upstream or downstream), sorted deterministically by group/pipeline ids.
export function getTransitivelyConnectedPipelineKeys(topology, pipelineKey) {
  if (
    !pipelineKey ||
    !topology ||
    !(topology.pipelineByKey instanceof Map) ||
    !topology.pipelineByKey.has(pipelineKey)
  ) {
    return [];
  }

  const visited = new Set([pipelineKey]);
  const queue = [pipelineKey];

  while (queue.length) {
    const key = queue.shift();
    const pipeline = topology.pipelineByKey.get(key);
    if (!pipeline) continue;

    const neighbors = [...(pipeline.upstream || []), ...(pipeline.downstream || [])];
    neighbors.forEach((neighbor) => {
      const neighborKey = neighbor?.key;
      if (
        !neighborKey ||
        visited.has(neighborKey) ||
        !topology.pipelineByKey.has(neighborKey)
      ) {
        return;
      }
      visited.add(neighborKey);
      queue.push(neighborKey);
    });
  }

  return Array.from(visited.values())
    .map((key) => topology.pipelineByKey.get(key))
    .filter(Boolean)
    .sort(sortPipelineRef)
    .map((pipeline) => pipeline.key);
}

function buildUndirectedAdjacency(topology, candidateKeys) {
  const keySet = new Set(candidateKeys || []);
  const adjacency = new Map();
  keySet.forEach((key) => adjacency.set(key, new Set()));

  if (!topology || !(topology.pipelineByKey instanceof Map)) {
    return adjacency;
  }

  keySet.forEach((key) => {
    const pipeline = topology.pipelineByKey.get(key);
    if (!pipeline) return;

    const neighbors = [...(pipeline.upstream || []), ...(pipeline.downstream || [])];
    neighbors.forEach((neighbor) => {
      const neighborKey = neighbor?.key;
      if (!neighborKey || !keySet.has(neighborKey) || neighborKey === key) return;
      adjacency.get(key).add(neighborKey);
      adjacency.get(neighborKey).add(key);
    });
  });

  return adjacency;
}

function bfsDistances(adjacency, source) {
  const distances = new Map();
  const queue = [source];
  distances.set(source, 0);

  while (queue.length) {
    const node = queue.shift();
    const base = distances.get(node) ?? 0;
    const neighbors = adjacency.get(node) || [];
    for (const next of neighbors) {
      if (distances.has(next)) continue;
      distances.set(next, base + 1);
      queue.push(next);
    }
  }

  return distances;
}

// Picks a deterministic center-like pipeline from candidate keys based on topic graph structure.
// Ranking priority:
// 1) reaches the most candidate pipelines
// 2) smallest maximum distance (eccentricity)
// 3) smallest total distance
// 4) largest immediate degree
// 5) candidate order (stable fallback)
export function findMostCentralPipelineKey(topology, candidateKeys) {
  const keys = Array.isArray(candidateKeys) ? candidateKeys.filter(Boolean) : [];
  if (!keys.length) return null;
  if (keys.length === 1) return keys[0];

  const order = new Map(keys.map((key, index) => [key, index]));
  const adjacency = buildUndirectedAdjacency(topology, keys);
  let best = null;

  for (const key of keys) {
    const distances = bfsDistances(adjacency, key);
    let maxDistance = 0;
    let distanceSum = 0;
    distances.forEach((distance) => {
      if (distance > maxDistance) maxDistance = distance;
      distanceSum += distance;
    });

    const score = {
      key,
      reachedCount: distances.size,
      eccentricity: maxDistance,
      distanceSum,
      degree: adjacency.get(key)?.size || 0,
      order: order.get(key) ?? Number.MAX_SAFE_INTEGER,
    };

    if (!best) {
      best = score;
      continue;
    }

    if (score.reachedCount > best.reachedCount) {
      best = score;
      continue;
    }
    if (score.reachedCount < best.reachedCount) {
      continue;
    }

    if (score.eccentricity < best.eccentricity) {
      best = score;
      continue;
    }
    if (score.eccentricity > best.eccentricity) {
      continue;
    }

    if (score.distanceSum < best.distanceSum) {
      best = score;
      continue;
    }
    if (score.distanceSum > best.distanceSum) {
      continue;
    }

    if (score.degree > best.degree) {
      best = score;
      continue;
    }
    if (score.degree < best.degree) {
      continue;
    }

    if (score.order < best.order) {
      best = score;
    }
  }

  return best?.key || keys[0];
}
