import test from 'node:test';
import assert from 'node:assert/strict';

import {
  computeEdgeRates,
  updateChannelSeries,
} from '../charts-controller.js';

// Verifies per-channel sender/receiver counters are converted into rates and
// merged into a single channel sample point.
test('updateChannelSeries aggregates sender/receiver rates per channel', () => {
  const channelSeries = new Map();
  const metricSets = [
    {
      name: 'channel.sender',
      attributes: { 'channel.id': 'ch1' },
      metrics: [
        { name: 'send.count', value: 20 },
        { name: 'send.error_full', value: 2 },
        { name: 'send.error_closed', value: 1 },
      ],
    },
    {
      name: 'channel.receiver',
      attributes: { 'channel.id': 'ch1' },
      metrics: [
        { name: 'recv.count', value: 10 },
        { name: 'recv.error_empty', value: 4 },
        { name: 'recv.error_closed', value: 2 },
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
    recvErrorEmptyRate: 2,
    recvErrorClosedRate: 1,
  });
});

// Verifies edge-rate computation prefers the sampled channel-series point when
// available instead of recalculating from raw cumulative counters.
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
            recvErrorEmptyRate: 0.5,
            recvErrorClosedRate: 1.5,
          },
        ],
      },
    ],
  ]);

  const edge = {
    id: 'edge-1',
    channelId: 'ch1',
    data: {
      sender: { metrics: [{ name: 'send.count', value: 100 }] },
      receiver: { metrics: [{ name: 'recv.count', value: 100 }] },
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
    metricMap: (metrics) => {
      const out = {};
      for (const metric of metrics || []) {
        out[metric.name] = metric.value;
      }
      return out;
    },
  });

  assert.deepEqual(rates.get('edge-1'), {
    sendRate: 8,
    recvRate: 7,
    sendErrorRate: 1,
    recvErrorRate: 2,
    errorRate: 3,
    active: true,
    errorActive: true,
  });
});
