import test from 'node:test';
import assert from 'node:assert/strict';

import {
  computeEdgeRates,
  updateChannelSeries,
} from '../charts-controller.js';

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
