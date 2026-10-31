import test from 'node:test';
import assert from 'node:assert/strict';

import {
  computeEdgeRates,
  updateChannelSeries,
  updateNodeSeries,
} from '../charts-controller.js';
import { deriveClientDeltas } from '../polling-controller.js';
import { aggregateMetricSets } from '../metric-filters.js';
import { resolveScopedNodeId } from '../graph-model.js';
import { normalizeAttributes } from '../pipeline-utils.js';

// Scenario: Channel snapshots contain bounded sender outcome, failure, and receiver signal buckets.
// Guarantees: Only successful sends become throughput and actionable send failures remain classified.
test('updateChannelSeries aggregates sender/receiver rates per channel', () => {
  const channelSeries = new Map();
  const metricSets = [
    {
      name: 'channel.sender',
      attributes: { 'channel.id': 'ch1' },
      metrics: [
        { name: 'messages', value: 20, attributes: { signal: 'logs', outcome: 'success' } },
        { name: 'messages', value: 2, attributes: { signal: 'logs', outcome: 'refused' } },
        { name: 'messages', value: 1, attributes: { signal: 'logs', outcome: 'failure' } },
        { name: 'failures', value: 2, attributes: { signal: 'logs', 'error.type': 'full' } },
        { name: 'failures', value: 1, attributes: { signal: 'logs', 'error.type': 'closed' } },
      ],
    },
    {
      name: 'channel.receiver',
      attributes: { 'channel.id': 'ch1' },
      metrics: [
        { name: 'messages', value: 10, attributes: { signal: 'logs' } },
      ],
    },
  ];

  updateChannelSeries({
    metricSets,
    sampleSeconds: 2,
    ts: new Date(1_000),
    channelSeries,
    maxWindowMs: 60_000,
    resolveScopedChannelId: (attrs) => attrs['channel.id'] || '',
    normalizeAttributes: (attrs) => attrs,
  });

  const points = channelSeries.get('ch1')?.points || [];
  assert.equal(points.length, 1);
  assert.deepEqual(points[0], {
    ts: 1_000,
    sendRate: 10,
    recvRate: 5,
    sendErrorFullRate: 1,
    sendErrorClosedRate: 0.5,
  });
});

// Scenario: An edge has both a sampled channel-series point and raw bounded metric buckets.
// Guarantees: Edge rendering prefers the sample and reports no synthetic receiver errors.
test('computeEdgeRates uses channel series point when available', () => {
  const channelSeries = new Map([
    [
      'ch1',
      {
        points: [
          {
            ts: 2_000,
            sendRate: 8,
            recvRate: 7,
            sendErrorFullRate: 0.25,
            sendErrorClosedRate: 0.75,
          },
        ],
      },
    ],
  ]);

  const edge = {
    id: 'edge-1',
    channelId: 'ch1',
    data: {
      sender: {
        metrics: [
          { name: 'messages', value: 100, attributes: { outcome: 'success' } },
        ],
      },
      receiver: { metrics: [{ name: 'messages', value: 100 }] },
    },
  };

  const rates = computeEdgeRates({
    edges: [edge],
    displayTimeMs: 2_000,
    sampleSeconds: 10,
    channelSeries,
    getWindowEndMs: () => 2_000,
    getWindowMs: () => 60_000,
    getDisplayTimeMs: () => 2_000,
    calcRate: (value, sampleSeconds) => value / sampleSeconds,
  });

  assert.deepEqual(rates.get('edge-1'), {
    sendRate: 8,
    recvRate: 7,
    sendErrorRate: 1,
    recvErrorRate: 0,
    errorRate: 1,
    active: true,
    errorActive: true,
  });
});

function requestSet(name, value, coreId = '0', pipelineId = 'p1', signal = 'logs') {
  return {
    name: 'receiver.otlp.requests',
    attributes: {
      'pipeline.group.id': 'default',
      'pipeline.id': pipelineId,
      'node.id': 'receiver',
      'core.id': coreId,
    },
    metrics: [{
      name,
      value,
      instrument: 'counter',
      temporality: 'cumulative',
      attributes: { signal, protocol: 'grpc' },
    }],
  };
}

function recordRequestSnapshot(nodeSeries, metricSets, ts, sampleSeconds, scopeByPipeline = false) {
  updateNodeSeries({
    nodeSeries,
    metricSets,
    ts: new Date(ts),
    sampleSeconds,
    dagScope: { scopeByPipeline },
    maxWindowMs: 3000,
    shouldShowNodeRate: () => false,
    resolveScopedNodeId,
    normalizeAttributes,
    buildNodeMetricKey: (set, metric) => `${set}::${metric}`,
  });
}

// Scenario: Cumulative OTLP admission counters arrive across signals and cores, then one bucket resets.
// Guarantees: Received totals include accepted and rejected requests, while rates sum reset-safe bucket deltas.
test('receiver ingress retains cumulative totals and derives aggregated request rates', () => {
  const state = {};
  const nodeSeries = new Map();
  const first = [
    requestSet('accepted', 10),
    requestSet('accepted', 5, '1'),
    requestSet('accepted', 4, '0', 'p1', 'traces'),
    requestSet('rejected', 2),
  ];
  recordRequestSnapshot(
    nodeSeries,
    aggregateMetricSets(deriveClientDeltas(first, state, 1000)),
    1000,
    null,
  );
  const points = () => nodeSeries.get('receiver').metrics
    .get('receiver.otlp.requests::received').points;
  assert.deepEqual(points()[0], {
    ts: 1000, total: 21, accepted: 19, rejected: 2, value: null,
  });

  const second = [
    requestSet('accepted', 16),
    requestSet('accepted', 8, '1'),
    requestSet('accepted', 5, '0', 'p1', 'traces'),
    requestSet('rejected', 3),
  ];
  recordRequestSnapshot(
    nodeSeries,
    aggregateMetricSets(deriveClientDeltas(second, state, 3000)),
    3000,
    2,
  );
  assert.deepEqual(points()[1], {
    ts: 3000, total: 32, accepted: 29, rejected: 3, value: 5.5,
  });

  const reset = [
    requestSet('accepted', 2),
    requestSet('accepted', 10, '1'),
    requestSet('accepted', 9, '0', 'p1', 'traces'),
    requestSet('rejected', 4),
  ];
  recordRequestSnapshot(
    nodeSeries,
    aggregateMetricSets(deriveClientDeltas(reset, state, 5000)),
    5000,
    2,
  );
  assert.deepEqual(points().at(-1), {
    ts: 5000, total: 25, accepted: 21, rejected: 4, value: 3.5,
  });
  assert.deepEqual(points().map(point => point.ts), [3000, 5000]);
});

// Scenario: OTLP requests, individual event counters, and downstream batches share a node identifier.
// Guarantees: Ingress uses only admission counters, preserves missing rates, and scopes same-named receivers by pipeline.
test('receiver ingress excludes events and channel batches and isolates pipelines', () => {
  const nodeSeries = new Map();
  const metricSets = [
    requestSet('accepted', 100),
    requestSet('rejected', 2),
    requestSet('accepted', 40, '0', 'p2'),
    {
      name: 'receiver.received',
      attributes: { 'node.id': 'receiver' },
      metrics: [{ name: 'messages', value: 500 }],
    },
    {
      name: 'channel.sender',
      attributes: { 'node.id': 'receiver' },
      metrics: [{ name: 'messages', value: 1000 }],
    },
  ];
  recordRequestSnapshot(nodeSeries, metricSets, 1000, 1, true);

  const point = id => nodeSeries.get(id).metrics
    .get('receiver.otlp.requests::received').points[0];
  assert.equal(point('default|p1@@receiver').total, 102);
  assert.equal(point('default|p2@@receiver').total, 40);
  assert.equal(point('default|p1@@receiver').value, null);
});

// Scenario: Request counters are absent, zero, or invalid after a baseline has been established.
// Guarantees: Missing telemetry stays unavailable and a measured idle receiver reports zero requests per second.
test('receiver ingress distinguishes missing telemetry from measured zero activity', () => {
  const nodeSeries = new Map();
  recordRequestSnapshot(nodeSeries, [], 1000, 1);
  assert.equal(nodeSeries.size, 0);

  const state = {};
  deriveClientDeltas([requestSet('accepted', 0)], state, 1000);
  const next = deriveClientDeltas([requestSet('accepted', 0)], state, 2000);
  next.push(requestSet('rejected', NaN));
  recordRequestSnapshot(nodeSeries, next, 2000, 1);
  const point = nodeSeries.get('receiver').metrics
    .get('receiver.otlp.requests::received').points[0];
  assert.equal(point.total, 0);
  assert.equal(point.value, 0);
});

function attemptSet(outcome, value, coreId = '0', signal = 'logs', pipelineId = 'p1') {
  return {
    name: 'exporter.attempted',
    attributes: {
      'pipeline.group.id': 'default',
      'pipeline.id': pipelineId,
      'node.id': 'exporter',
      'core.id': coreId,
    },
    metrics: [{
      name: 'messages', value, instrument: 'counter', temporality: 'cumulative',
      attributes: { signal, outcome },
    }],
  };
}

// Scenario: Shared exporter attempt counters cover multiple cores, signals, and outcomes before one bucket resets.
// Guarantees: Cumulative success totals and interval rates preserve outcome dimensions and never produce negative deltas.
test('shared exporter attempts preserve outcomes through polling and core aggregation', () => {
  const state = {};
  const nodeSeries = new Map();
  const record = (sets, ts, seconds) => recordRequestSnapshot(
    nodeSeries, aggregateMetricSets(deriveClientDeltas(sets, state, ts)), ts, seconds,
  );
  const points = () => nodeSeries.get('exporter').metrics
    .get('exporter.attempted::success').points;
  record([
    attemptSet('success', 10), attemptSet('success', 4, '1'),
    attemptSet('success', 6, '0', 'traces'), attemptSet('failure', 2),
    attemptSet('refused', 3),
  ], 1000, null);
  assert.deepEqual(points()[0], {
    ts: 1000, success: 20, failure: 2, refused: 3, value: null, successPercent: null,
  });
  record([
    attemptSet('success', 14), attemptSet('success', 7, '1'),
    attemptSet('success', 9, '0', 'traces'), attemptSet('failure', 3),
    attemptSet('refused', 4),
  ], 3000, 2);
  assert.deepEqual(points()[1], {
    ts: 3000, success: 30, failure: 3, refused: 4, value: 5,
    successPercent: 100 * 10 / 12,
  });
  record([
    attemptSet('success', 1), attemptSet('success', 9, '1'),
    attemptSet('success', 11, '0', 'traces'), attemptSet('failure', 4),
    attemptSet('refused', 5),
  ], 5000, 2);
  assert.deepEqual(points().at(-1), {
    ts: 5000, success: 21, failure: 4, refused: 5, value: null,
    successPercent: null,
  });
  assert.deepEqual(points().map(point => point.ts), [3000, 5000]);
  record([
    attemptSet('success', 4), attemptSet('success', 10, '1'),
    attemptSet('success', 13, '0', 'traces'), attemptSet('failure', 5),
    attemptSet('refused', 6),
  ], 7000, 2);
  assert.deepEqual(points().at(-1), {
    ts: 7000, success: 27, failure: 5, refused: 6, value: 3, successPercent: 75,
  });
});

// Scenario: Shared attempt counters are missing, idle, failed-only, or mixed with other exporter and channel metrics.
// Guarantees: Only shared message outcomes count, idle percentages stay unavailable, and exporters remain pipeline-scoped.
test('shared exporter attempts distinguish unavailable, idle, and failed-only exporters', () => {
  const state = {};
  const nodeSeries = new Map();
  const record = (sets, ts, seconds) => recordRequestSnapshot(
    nodeSeries, deriveClientDeltas(sets, state, ts), ts, seconds, true,
  );
  record([], 1000, null);
  assert.equal(nodeSeries.size, 0);
  const sets = [
    attemptSet('success', 0), attemptSet('failure', 2, '0', 'logs', 'p2'),
    attemptSet('unknown', 99), attemptSet('refused', NaN),
    { ...attemptSet('success', 50), name: 'exporter.geneva.uploads' },
    { ...attemptSet('success', 50), name: 'channel.receiver' },
    {
      ...attemptSet('success', 50),
      metrics: [
        { name: 'items', value: 500, attributes: { outcome: 'success' } },
        { name: 'payload.size', value: 1000, attributes: { outcome: 'success' } },
        { name: 'duration', value: { count: 10, sum: 20 }, attributes: { outcome: 'success' } },
      ],
    },
  ];
  record(sets, 2000, null);
  record([
    ...sets.filter(set => set.attributes['pipeline.id'] !== 'p2'),
    attemptSet('failure', 4, '0', 'logs', 'p2'),
  ], 4000, 2);
  const point = id => nodeSeries.get(`default|${id}@@exporter`).metrics
    .get('exporter.attempted::success').points.at(-1);
  assert.deepEqual(point('p1'), {
    ts: 4000, success: 0, failure: 0, refused: 0, value: 0, successPercent: null,
  });
  assert.deepEqual(point('p2'), {
    ts: 4000, success: 0, failure: 4, refused: 0, value: 0, successPercent: 0,
  });

  const withoutDeltas = new Map();
  recordRequestSnapshot(withoutDeltas, [attemptSet('success', 1)], 5000, 1);
  assert.equal(withoutDeltas.get('exporter').metrics
    .get('exporter.attempted::success').points[0].value, null);
});

// Scenario: A previously unseen failure bucket appears while successful export attempts continue.
// Guarantees: Unknown failure baselines do not produce a misleading 100 percent success interval.
test('shared exporter percentages wait for new outcome bucket baselines', () => {
  const state = {};
  const nodeSeries = new Map();
  deriveClientDeltas([attemptSet('success', 10)], state, 1000);
  recordRequestSnapshot(nodeSeries, aggregateMetricSets(deriveClientDeltas([
    attemptSet('success', 12), attemptSet('failure', 1),
  ], state, 3000)), 3000, 2);
  const points = nodeSeries.get('exporter').metrics
    .get('exporter.attempted::success').points;
  assert.equal(points[0].successPercent, null);
  recordRequestSnapshot(nodeSeries, aggregateMetricSets(deriveClientDeltas([
    attemptSet('success', 14), attemptSet('failure', 2),
  ], state, 5000)), 5000, 2);
  assert.equal(points.at(-1).successPercent, 100 * 2 / 3);
});
