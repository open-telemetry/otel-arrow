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

// Scenario: The admin selection is cleared while a channel chart and legend are active.
// Guarantees: The panel, chart state, legend, and node charts all return to their empty state.
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

// Scenario: An OTLP receiver is selected with measured or unavailable external request counters.
// Guarantees: Details distinguish admission totals and request rates from internal channel traffic and omit them for exporters.
test('renderNodeDetails displays external requests only for OTLP receivers', () => {
  const previousDocument = globalThis.document;
  globalThis.document = { getElementById: () => null };
  try {
    const body = { innerHTML: '' };
    let requestPoint = { total: 12, accepted: 10, rejected: 2, value: 3 };
    const controller = createSelectionDetailsController({
      edgeDetailMeta: { textContent: '' },
      edgeDetailBody: body,
      buildNodeSummary: () => ({ inRate: 0, outRate: 1, errorRate: 0 }),
      getOtlpReceiverRequestPoint: () => requestPoint,
      formatRateWithUnit: (value, unit) => Number.isFinite(value) ? `${value} ${unit}/s` : 'n/a',
      formatValueWithUnit: (value, unit) => `${value} ${unit}`,
      escapeHtml: value => String(value),
      renderAttributes: () => '',
      destroyNodeCharts: () => {},
      initNodeRateCharts: () => {},
    });

    const node = {
      id: 'receiver',
      attrs: { 'node.urn': 'urn:otel:receiver:otlp' },
      metricSets: [],
    };
    controller.renderNodeDetails(node);
    assert.match(body.innerHTML, /External requests received/);
    assert.match(body.innerHTML, /12 request/);
    assert.match(body.innerHTML, /3 req\/s/);
    assert.match(body.innerHTML, /Accepted: 10 request; rejected: 2 request/);

    requestPoint = null;
    controller.renderNodeDetails(node);
    assert.match(body.innerHTML, /External requests received/);
    assert.doesNotMatch(body.innerHTML, /0 request/);

    controller.renderNodeDetails({
      ...node,
      attrs: { 'node.urn': 'urn:microsoft:exporter:geneva' },
    });
    assert.doesNotMatch(body.innerHTML, /External requests received/);
  } finally {
    globalThis.document = previousDocument;
  }
});

// Scenario: A Geneva exporter is selected with measured, idle, or unavailable completed-upload counters.
// Guarantees: The details show successful upload rates and interval percentages separately from internal batch traffic.
test('renderNodeDetails displays upload success only for Geneva exporters', () => {
  const previousDocument = globalThis.document;
  globalThis.document = { getElementById: () => null };
  try {
    const body = { innerHTML: '' };
    let uploadPoint = {
      success: 12, failure: 2, refused: 1, value: 3, successPercent: 75,
    };
    const controller = createSelectionDetailsController({
      edgeDetailMeta: { textContent: '' },
      edgeDetailBody: body,
      buildNodeSummary: () => ({ inRate: 10, outRate: 0, errorRate: 0 }),
      getGenevaUploadPoint: () => uploadPoint,
      formatRateWithUnit: (value, unit) => Number.isFinite(value) ? `${value} ${unit}/s` : 'n/a',
      formatValueWithUnit: (value, unit) => `${value} ${unit}`,
      escapeHtml: value => String(value),
      renderAttributes: () => '',
      destroyNodeCharts: () => {},
      initNodeRateCharts: () => {},
    });
    const node = {
      id: 'exporter', attrs: { 'node.urn': 'urn:microsoft:exporter:geneva' }, metricSets: [],
    };
    controller.renderNodeDetails(node);
    assert.match(body.innerHTML, /Successful uploads/);
    assert.match(body.innerHTML, /12 batch/);
    assert.match(body.innerHTML, /3 upload\/s/);
    assert.match(body.innerHTML, /Success: 75\.0% \(interval\)/);
    assert.match(body.innerHTML, /Failed: 2 batch; refused: 1 batch/);
    assert.match(body.innerHTML, /10 msg\/s/);

    uploadPoint = { ...uploadPoint, value: 0, successPercent: null };
    controller.renderNodeDetails(node);
    assert.match(body.innerHTML, /0 upload\/s/);
    assert.match(body.innerHTML, /Success: n\/a \(interval\)/);

    uploadPoint = null;
    controller.renderNodeDetails(node);
    assert.match(body.innerHTML, /Successful uploads/);
    assert.doesNotMatch(body.innerHTML, /0 batch|0 upload\/s/);

    controller.renderNodeDetails({
      ...node, attrs: { 'node.urn': 'urn:otel:receiver:otlp' },
    });
    assert.doesNotMatch(body.innerHTML, /Successful uploads/);
  } finally {
    globalThis.document = previousDocument;
  }
});
