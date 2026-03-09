import test from 'node:test';
import assert from 'node:assert/strict';

import { createPipelineChartsController } from '../pipeline-charts-controller.js';

// Verifies controller behavior in a DOM-lite environment:
// metric recording, formatted value application, hover tracking, and cleanup.
test('pipeline charts controller records metrics and applies formatted values', () => {
  const previousDocument = globalThis.document;
  const previousWindow = globalThis.window;

  globalThis.document = {
    querySelectorAll: () => [],
    getElementById: () => null,
  };
  globalThis.window = {
    Chart: null,
  };

  try {
    const pipelineSeries = new Map();
    const pipelineCharts = new Map();
    const valueEl = { textContent: '' };

    const controller = createPipelineChartsController({
      pipelineSeries,
      pipelineCharts,
      maxWindowMs: 60_000,
      pipelineChartConfig: {},
      pipelineMetricDisplay: {
        'engine.cpu.utilization': {
          el: valueEl,
          format: (value) => (Number.isFinite(value) ? value.toFixed(1) : 'n/a'),
        },
      },
      getWindowEndMs: () => 1_000,
      getWindowMs: () => 60_000,
      getDisplayTimeMs: () => 1_000,
      getSeriesWindow: (points, startMs, endMs) =>
        (points || []).filter((point) => point.ts >= startMs && point.ts <= endMs),
      getPointAtTime: (points, ts) => {
        let chosen = null;
        for (const point of points || []) {
          if (point.ts <= ts) {
            chosen = point;
          } else {
            break;
          }
        }
        return chosen || points?.[0] || null;
      },
      getChartThemeColors: () => ({ tick: '#aaa', grid: '#bbb' }),
      pipelineHoverPlugin: {},
      onGlobalHover: () => {},
      getGlobalHoverTs: () => null,
    });

    controller.recordMetric('engine.cpu.utilization', 12.34, new Date(1_000));
    controller.applyMetricValues(1_000);
    assert.equal(valueEl.textContent, '12.3');

    assert.equal(controller.setHover(1_000), 1_000);
    assert.equal(controller.getHoverTs(), 1_000);

    controller.destroyCharts();
    assert.equal(controller.getHoverTs(), null);
    assert.equal(valueEl.textContent, '12.3');
  } finally {
    globalThis.document = previousDocument;
    globalThis.window = previousWindow;
  }
});
