import test from 'node:test';
import assert from 'node:assert/strict';

import {
  extractEngineSummary,
  deriveEngineCardValues,
} from '../engine-metrics.js';

// Scenario: Windows reports private commit separately from resident memory.
// Guarantees: Both values are converted to MiB independently without multiplying process memory by core count.
test('engine memory cards keep private commit separate from RSS', () => {
  const summary = extractEngineSummary([
    {
      name: 'engine',
      metrics: [
        { name: 'memory.rss', value: 2 * 1024 * 1024 },
        { name: 'memory.commit', value: 8 * 1024 * 1024 },
      ],
    },
    { name: 'pipeline', metrics: [] },
    { name: 'pipeline', metrics: [] },
  ]);
  const values = deriveEngineCardValues(summary);

  assert.equal(values.memoryRssMiB, 2);
  assert.equal(values.memoryCommitMiB, 8);
  assert.equal(values.currentMemoryCommitMiB, 8);
  assert.equal(values.lastMemoryCommitMiB, 8);
});

// Scenario: A non-Windows snapshot reports RSS without private commit.
// Guarantees: Commit remains unavailable rather than becoming zero or falling back to RSS.
test('engine memory cards leave absent private commit unavailable', () => {
  const summary = extractEngineSummary([
    {
      name: 'engine',
      metrics: [{ name: 'memory.rss', value: 2 * 1024 * 1024 }],
    },
  ]);
  const values = deriveEngineCardValues(summary);

  assert.equal(values.memoryRssMiB, 2);
  assert.equal(values.memoryCommitMiB, null);
  assert.equal(values.currentMemoryCommitMiB, null);
  assert.equal(values.lastMemoryCommitMiB, null);
});

// Scenario: Commit is the only nonzero engine sample, or is explicitly zero.
// Guarantees: Valid commit samples are not discarded or mistaken for missing values.
test('engine memory cards accept commit-only activity and explicit zero', () => {
  for (const [rss, commit] of [[0, 1024 * 1024], [1024 * 1024, 0], [0, 0]]) {
    const values = deriveEngineCardValues(extractEngineSummary([
      {
        name: 'engine',
        metrics: [
          { name: 'cpu.utilization', value: 0 },
          { name: 'memory.rss', value: rss },
          { name: 'memory.commit', value: commit },
        ],
      },
    ]));

    assert.equal(values.currentMemoryCommitMiB, commit / (1024 * 1024));
  }
});

// Scenario: A Windows commit sample is followed by missing or invalid telemetry.
// Guarantees: The last display value can be held without recording a synthetic current chart sample.
test('engine memory cards retain commit history without fabricating samples', () => {
  const previous = { lastMemoryCommitMiB: 8 };
  for (const summary of [
    null,
    extractEngineSummary([]),
    extractEngineSummary([{
      name: 'engine',
      metrics: [{ name: 'memory.commit', value: NaN }],
    }]),
  ]) {
    const values = deriveEngineCardValues(summary, previous);

    assert.equal(values.memoryCommitMiB, 8);
    assert.equal(values.currentMemoryCommitMiB, null);
    assert.equal(values.lastMemoryCommitMiB, 8);
  }
  const values = deriveEngineCardValues(
    extractEngineSummary([]),
    previous,
    { holdLastValues: false },
  );
  assert.equal(values.memoryCommitMiB, null);
  assert.equal(values.currentMemoryCommitMiB, null);
});
