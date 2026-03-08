import {
  getPipelineSelectionKeyFromAttrs,
  normalizeAttributes,
} from "./pipeline-utils.js";

const DEFAULT_SCOPED_ID_SEPARATOR = "@@";

export function buildScopedMetricId(
  baseId,
  pipelineKey,
  scopeByPipeline,
  scopedIdSeparator = DEFAULT_SCOPED_ID_SEPARATOR
) {
  if (!baseId) return "";
  if (!scopeByPipeline || !pipelineKey) {
    return String(baseId);
  }
  return `${pipelineKey}${scopedIdSeparator}${baseId}`;
}

export function resolveScopedNodeId(
  attrs,
  scopeByPipeline,
  scopedIdSeparator = DEFAULT_SCOPED_ID_SEPARATOR
) {
  const nodeId = attrs["node.id"];
  if (!nodeId) return "";
  const pipelineKey = getPipelineSelectionKeyFromAttrs(attrs);
  return buildScopedMetricId(nodeId, pipelineKey, scopeByPipeline, scopedIdSeparator);
}

export function resolveScopedChannelId(
  attrs,
  scopeByPipeline,
  scopedIdSeparator = DEFAULT_SCOPED_ID_SEPARATOR
) {
  const channelId = attrs["channel.id"];
  if (!channelId) return "";
  const pipelineKey = getPipelineSelectionKeyFromAttrs(attrs);
  return buildScopedMetricId(channelId, pipelineKey, scopeByPipeline, scopedIdSeparator);
}

function mergeAttributes(target, source) {
  for (const [key, value] of Object.entries(source)) {
    if (value == null || value === "") continue;
    if (!(key in target) || target[key] === "") {
      target[key] = value;
    }
  }
}

function resolveChannelPort(attrs) {
  const value = attrs?.["node.port"];
  if (value != null && value !== "") return String(value);
  return "default";
}

// Convert metric sets into DAG nodes/edges with optional per-pipeline ID scoping.
export function buildGraph(
  metricSets,
  sampleSeconds,
  allowedKinds,
  dagScope = null,
  scopedIdSeparator = DEFAULT_SCOPED_ID_SEPARATOR
) {
  const channels = new Map();
  const nodes = new Map();
  const pipelineIds = new Set();
  const channelNodes = new Set();
  const kindFilter = allowedKinds ? new Set(allowedKinds) : null;
  const scopeByPipeline = dagScope?.scopeByPipeline === true;

  for (const set of metricSets) {
    const attrs = normalizeAttributes(set.attributes || {});
    const nodeId = attrs["node.id"];
    const pipelineKey = getPipelineSelectionKeyFromAttrs(attrs);
    const scopedNodeId = resolveScopedNodeId(attrs, scopeByPipeline, scopedIdSeparator);
    const scopedChannelId = resolveScopedChannelId(attrs, scopeByPipeline, scopedIdSeparator);
    const pipelineId = attrs["pipeline.id"];
    if (pipelineId) pipelineIds.add(pipelineId);

    if (set.name === "channel.sender" || set.name === "channel.receiver") {
      if (kindFilter && !kindFilter.has(attrs["channel.kind"])) continue;
      const channelId = attrs["channel.id"];
      if (!channelId || !scopedChannelId) continue;
      let channel = channels.get(scopedChannelId);
      const resolvedPort = resolveChannelPort(attrs);
      if (!channel) {
        channel = {
          id: scopedChannelId,
          displayId: String(channelId),
          kind: attrs["channel.kind"],
          senders: [],
          receivers: [],
        };
        channels.set(scopedChannelId, channel);
      }
      const role = set.name === "channel.sender" ? "sender" : "receiver";
      const endpoint = {
        nodeId: scopedNodeId || "unknown",
        displayNodeId: nodeId || scopedNodeId || "unknown",
        pipelineKey: pipelineKey || "",
        attrs,
        metrics: set.metrics || [],
        port: resolvedPort,
      };
      if (role === "sender") {
        channel.senders.push(endpoint);
      } else {
        channel.receivers.push(endpoint);
      }
      if (endpoint.nodeId) channelNodes.add(endpoint.nodeId);
      continue;
    }

    if (!nodeId || !scopedNodeId) continue;
    let node = nodes.get(scopedNodeId);
    if (!node) {
      node = {
        id: scopedNodeId,
        displayId: String(nodeId),
        attrs: {},
        displayAttrs: {},
        metricSets: [],
        outPorts: [],
      };
      nodes.set(scopedNodeId, node);
    }
    node.metricSets.push({
      name: set.name,
      attrs,
      metrics: set.metrics || [],
    });
    mergeAttributes(node.attrs, attrs);
    mergeAttributes(node.displayAttrs, attrs);
    if (pipelineKey) {
      node.attrs["ui.pipeline.key"] = pipelineKey;
    }
  }

  const ensureNode = (nodeId, displayId) => {
    if (!nodes.has(nodeId)) {
      nodes.set(nodeId, {
        id: nodeId,
        displayId: displayId || nodeId,
        attrs: {},
        displayAttrs: {},
        metricSets: [],
        outPorts: [],
      });
    }
    return nodes.get(nodeId);
  };

  for (const channel of channels.values()) {
    (channel.senders || []).forEach((sender) => {
      if (!sender?.nodeId) return;
      channelNodes.add(sender.nodeId);
      const node = ensureNode(sender.nodeId, sender.displayNodeId);
      mergeAttributes(node.attrs, sender.attrs);
      if (sender.pipelineKey) {
        node.attrs["ui.pipeline.key"] = sender.pipelineKey;
      }
    });
    (channel.receivers || []).forEach((receiver) => {
      if (!receiver?.nodeId) return;
      channelNodes.add(receiver.nodeId);
      const node = ensureNode(receiver.nodeId, receiver.displayNodeId);
      mergeAttributes(node.attrs, receiver.attrs);
      if (receiver.pipelineKey) {
        node.attrs["ui.pipeline.key"] = receiver.pipelineKey;
      }
    });
  }

  nodes.forEach((node) => {
    if (!node.displayAttrs) node.displayAttrs = {};
    if (node.attrs["node.id"]) node.displayId = node.attrs["node.id"];
    if (node.attrs["node.id"]) {
      node.displayAttrs["node.id"] = node.displayAttrs["node.id"] || node.attrs["node.id"];
    }
    if (node.attrs["node.type"]) {
      node.displayAttrs["node.type"] =
        node.displayAttrs["node.type"] || node.attrs["node.type"];
    }
    if (node.attrs["node.urn"]) {
      node.displayAttrs["node.urn"] = node.displayAttrs["node.urn"] || node.attrs["node.urn"];
    }
  });

  for (const nodeId of nodes.keys()) {
    if (!channelNodes.has(nodeId)) {
      nodes.delete(nodeId);
    }
  }

  const edges = [];
  for (const channel of channels.values()) {
    const senders = channel.senders || [];
    const receivers = channel.receivers || [];
    if (!senders.length || !receivers.length) continue;
    senders.forEach((sender) => {
      receivers.forEach((receiver) => {
        if (!sender?.nodeId || !receiver?.nodeId) return;
        if (
          scopeByPipeline &&
          sender.pipelineKey &&
          receiver.pipelineKey &&
          sender.pipelineKey !== receiver.pipelineKey
        ) {
          return;
        }
        const port = sender.port || "default";
        const edgeId = `${channel.id}::${sender.nodeId}::${receiver.nodeId}::${port}`;
        edges.push({
          id: edgeId,
          channelId: channel.id,
          channelDisplayId: channel.displayId || channel.id,
          source: sender.nodeId,
          target: receiver.nodeId,
          sourceDisplayId: sender.displayNodeId || sender.nodeId,
          targetDisplayId: receiver.displayNodeId || receiver.nodeId,
          port,
          data: {
            id: channel.id,
            displayId: channel.displayId || channel.id,
            kind: channel.kind,
            sender,
            receiver,
            multiSender: senders.length > 1,
            multiReceiver: receivers.length > 1,
          },
        });
      });
    });
  }

  for (const edge of edges) {
    const node = nodes.get(edge.source);
    if (!node) continue;
    if (!node.outPorts.includes(edge.port)) {
      node.outPorts.push(edge.port);
    }
  }

  const nodeList = Array.from(nodes.values());
  nodeList.sort((a, b) =>
    (a.displayId || a.id).localeCompare(b.displayId || b.id, undefined, {
      numeric: true,
      sensitivity: "base",
    })
  );
  nodeList.forEach((node) => node.outPorts.sort());

  return {
    nodes: nodeList,
    edges,
    meta: {
      pipelines: Array.from(pipelineIds.values()).sort(),
      channelCount: channels.size,
      sampleSeconds,
    },
  };
}
