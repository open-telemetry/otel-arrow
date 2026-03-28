import test from 'node:test';
import assert from 'node:assert/strict';

import { createSelectionDetailsController } from '../selection-details-controller.js';

// Minimal classList shim for headless tests.
function createClassList() {
  const set = new Set();
  return {
    add: (...items) => items.forEach((item) => set.add(item)),
    remove: (...items) => items.forEach((item) => set.delete(item)),
    contains: (item) => set.has(item),
  };
}

// Verifies the "no selection" branch resets UI text, tears down chart state,
// and clears legend visibility/content.
test('renderSelectionNone resets detail panel and clears chart', () => {
  const previousDocument = globalThis.document;

  const legend = {
    classList: createClassList(),
    innerHTML: '',
  };

  globalThis.document = {
    getElementById: (id) => (id === 'channelChartLegend' ? legend : null),
  };

  try {
    const selectionTitle = { textContent: '' };
    const edgeDetailMeta = { textContent: '' };
    const edgeDetailBody = { innerHTML: '' };

    let chartDestroyed = false;
    let chartRef = {
      _legendHandlers: {
        move: () => {},
        leave: () => {},
      },
      canvas: {
        removeEventListener: () => {},
      },
      destroy: () => {
        chartDestroyed = true;
      },
    };
    let chartIdRef = 'ch1';
    let nodeChartsDestroyed = false;

    const controller = createSelectionDetailsController({
      selectionTitle,
      edgeDetailMeta,
      edgeDetailBody,
      channelSeries: new Map(),
      pipelineHoverPlugin: {},
      getChartThemeColors: () => ({ tick: '#000', grid: '#111' }),
      getWindowEndMs: () => 0,
      getWindowMs: () => 1000,
      getSeriesWindow: () => [],
      getPointAtTime: () => null,
      getDisplayTimeMs: () => 0,
      getChannelPoint: () => null,
      getFreezeActive: () => false,
      formatRate: () => 'n/a',
      formatRateWithUnit: () => 'n/a',
      formatWindowLabel: () => '5m',
      formatValueWithUnit: () => 'n/a',
      renderAttributes: () => '',
      renderMetricTable: () => '',
      renderNodeMetricTable: () => '',
      metricMap: () => ({}),
      calcRate: () => 0,
      buildNodeSummary: () => ({ inRate: 0, outRate: 0, errorRate: 0 }),
      escapeHtml: (value) => String(value),
      setGlobalHover: () => {},
      destroyNodeCharts: () => {
        nodeChartsDestroyed = true;
      },
      initNodeRateCharts: () => {},
      getLastSampleSeconds: () => 1,
      getLastEdgeRates: () => new Map(),
      getGlobalHoverTs: () => null,
      getChannelChart: () => chartRef,
      setChannelChart: (next) => {
        chartRef = next;
      },
      getChannelChartId: () => chartIdRef,
      setChannelChartId: (next) => {
        chartIdRef = next;
      },
    });

    controller.renderSelectionNone();

    assert.equal(selectionTitle.textContent, 'Selection Details');
    assert.equal(edgeDetailMeta.textContent, 'None selected');
    assert.equal(edgeDetailBody.innerHTML, 'Click a node or edge to show details.');
    assert.equal(chartDestroyed, true);
    assert.equal(chartRef, null);
    assert.equal(chartIdRef, null);
    assert.equal(nodeChartsDestroyed, true);
    assert.equal(legend.innerHTML, '');
    assert.equal(legend.classList.contains('hidden'), true);
  } finally {
    globalThis.document = previousDocument;
  }
});

test("renderEdgeDetails renders control metrics for control channels", () => {
  const previousDocument = globalThis.document;

  globalThis.document = {
    getElementById: () => null,
  };

  try {
    const selectionTitle = { textContent: "" };
    const edgeDetailMeta = { textContent: "" };
    const edgeDetailBody = { innerHTML: "" };

    const controller = createSelectionDetailsController({
      selectionTitle,
      edgeDetailMeta,
      edgeDetailBody,
      channelSeries: new Map(),
      pipelineHoverPlugin: {},
      getChartThemeColors: () => ({ tick: "#000", grid: "#111" }),
      getWindowEndMs: () => 0,
      getWindowMs: () => 60_000,
      getSeriesWindow: () => [],
      getPointAtTime: () => null,
      getDisplayTimeMs: () => 0,
      getChannelPoint: () => null,
      getFreezeActive: () => false,
      formatRate: () => "n/a",
      formatRateWithUnit: () => "n/a",
      formatWindowLabel: () => "1m",
      formatValueWithUnit: (value, unit) => `${value} ${unit}`,
      renderAttributes: () => "",
      renderMetricTable: (metrics) => `METRICS:${metrics.map((metric) => metric.name).join(",")}`,
      renderNodeMetricTable: () => "",
      metricMap: (metrics) => Object.fromEntries((metrics || []).map((metric) => [metric.name, metric.value])),
      calcRate: () => 0,
      buildNodeSummary: () => ({ inRate: 0, outRate: 0, errorRate: 0 }),
      escapeHtml: (value) => String(value),
      setGlobalHover: () => {},
      destroyNodeCharts: () => {},
      initNodeRateCharts: () => {},
      getLastSampleSeconds: () => 1,
      getLastEdgeRates: () => new Map(),
      getGlobalHoverTs: () => null,
      getChannelChart: () => null,
      setChannelChart: () => {},
      getChannelChartId: () => null,
      setChannelChartId: () => {},
    });

    controller.renderEdgeDetails({
      id: "edge-1",
      channelId: "ctrl-1",
      source: "pipeline-ctrl",
      target: "receiver-1",
      data: {
        id: "ctrl-1",
        kind: "control",
        sender: {
          attrs: {
            "channel.kind": "control",
            "channel.mode": "local",
            "channel.type": "mpsc",
            "channel.impl": "internal",
            "node.id": "pipeline-ctrl",
            "node.type": "controller",
          },
          metrics: [{ name: "send.count", value: 5 }],
        },
        receiver: {
          attrs: {
            "channel.kind": "control",
            "channel.mode": "local",
            "channel.type": "mpsc",
            "channel.impl": "internal",
            "node.id": "receiver-1",
            "node.type": "receiver",
          },
          metrics: [{ name: "capacity", value: 8, unit: "{message}" }],
        },
        control: {
          metrics: [
            { name: "completion.queued", value: 3, unit: "{message}" },
            { name: "shutdown.recorded", value: 1, unit: "{1}" },
          ],
        },
      },
    });

    assert.equal(selectionTitle.textContent, "Selection Details - Channel");
    assert.match(edgeDetailBody.innerHTML, /Control Metrics/);
    assert.match(edgeDetailBody.innerHTML, /METRICS:completion\.queued,shutdown\.recorded/);
    assert.match(edgeDetailBody.innerHTML, /3 \{message\} \/ 8 \{message\}/);
  } finally {
    globalThis.document = previousDocument;
  }
});
