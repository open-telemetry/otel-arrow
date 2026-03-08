import test from 'node:test';
import assert from 'node:assert/strict';

import { renderGraphFrame } from '../graph-renderer.js';

function hashString32(hash, value) {
  const text = String(value == null ? '' : value);
  let next = hash >>> 0;
  for (let i = 0; i < text.length; i += 1) {
    next ^= text.charCodeAt(i);
    next = Math.imul(next, 16777619);
  }
  return next >>> 0;
}

function emptySignature() {
  let hash = 2166136261;
  hash = hashString32(hash, 'single');
  hash = hashString32(hash, '0');
  return `0:0:0:${hash.toString(16)}`;
}

test('renderGraphFrame reuse path keeps a non-null layout size', () => {
  const result = renderGraphFrame({
    dataGraph: { nodes: [], edges: [], meta: {} },
    controlGraph: { nodes: [], edges: [], meta: {} },
    perfStart: () => 0,
    perfEnd: () => {},
    metricMode: 'throughput',
    hideZeroActivity: false,
    dagSearchQuery: '',
    showControlChannels: false,
    selectedEdgeId: null,
    selectedEdgeData: null,
    selectedNodeId: null,
    selectedNodeData: null,
    lastRenderedStructureSignature: emptySignature(),
    getDisplayTimeMs: () => 0,
    computeEdgeRates: () => new Map(),
    filterGraphByQuery: (nodes, edges) => ({ nodes, edges }),
    getDagRenderScope: () => ({ mode: 'single' }),
    updateTopologyForHover: () => {},
    layoutGraph: () => ({ width: 100, height: 100, lanes: [], columnWidth: 210 }),
    collectPipelineDagAnchors: () => [],
    computePipelineDagNavLayout: () => ({ leftGutter: 0, rightGutter: 0 }),
    applyDefaultOverviewZoom: () => {},
    applyZoom: () => {},
    ensureDagEdgeDefs: () => {},
    buildFocusSets: () => null,
    pruneRemovedDagNodes: () => {},
    upsertDagNodeElement: () => {},
    pruneRemovedDagEdges: () => {},
    upsertDagEdgeElement: () => {},
    clearDagNavigationOverlayElements: () => {},
    renderConnectedTopicNavigation: () => {},
    renderPipelineDagNavigation: () => {},
    renderEdgeDetails: () => {},
    renderNodeDetails: () => {},
    renderSelectionNone: () => {},
    dagCanvas: { style: {} },
    dagEdges: {
      childElementCount: 0,
      setAttribute: () => {},
    },
    dagNodes: {
      childElementCount: 1,
      style: {},
    },
    dagLanes: {
      childElementCount: 0,
      style: {},
    },
    dagEmpty: {
      classList: {
        toggle: () => {},
      },
    },
    layoutSize: null,
    NODE_WIDTH: 210,
    MARGIN: 48,
    LEVEL_GAP: 140,
  });

  assert.deepEqual(result.layoutSize, { width: 0, height: 0 });
});
