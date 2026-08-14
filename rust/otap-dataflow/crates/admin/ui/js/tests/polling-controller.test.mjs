import test from "node:test";
import assert from "node:assert/strict";

import { deriveClientDeltas } from "../polling-controller.js";

// Scenario: A cumulative channel outcome bucket is observed across two admin polls.
// Guarantees: The first sample establishes a baseline and the second exposes only its delta.
test("deriveClientDeltas computes per-series deltas and initializes baseline", () => {
  const state = { clientDeltaPrevBySeries: new Map() };
  const set = {
    name: "channel.sender",
    attributes: { "channel.id": "ch1" },
    metrics: [{ name: "messages", value: 10, attributes: { outcome: "success" } }],
  };

  const first = deriveClientDeltas([set], state, 1_000);
  assert.equal(first[0].metrics[0].value, 0);

  const second = deriveClientDeltas(
    [
      {
        ...set,
        metrics: [{ name: "messages", value: 16, attributes: { outcome: "success" } }],
      },
    ],
    state,
    2_000
  );
  assert.equal(second[0].metrics[0].value, 6);
});

// Scenario: A cumulative counter restarts at a lower value between admin polls.
// Guarantees: Reset handling prevents a negative spike and advances the next baseline.
test("deriveClientDeltas handles counter resets without negative spikes", () => {
  const state = { clientDeltaPrevBySeries: new Map() };
  const attrs = { "pipeline.id": "p1", "core.id": "0" };

  deriveClientDeltas(
    [
      {
        name: "pipeline",
        attributes: attrs,
        metrics: [{ name: "memory.allocated.delta", value: 100 }],
      },
    ],
    state,
    1_000
  );

  const afterReset = deriveClientDeltas(
    [
      {
        name: "pipeline",
        attributes: attrs,
        metrics: [{ name: "memory.allocated.delta", value: 5 }],
      },
    ],
    state,
    2_000
  );
  assert.equal(afterReset[0].metrics[0].value, 0);

  const recovered = deriveClientDeltas(
    [
      {
        name: "pipeline",
        attributes: attrs,
        metrics: [{ name: "memory.allocated.delta", value: 13 }],
      },
    ],
    state,
    3_000
  );
  assert.equal(recovered[0].metrics[0].value, 8);
});

// Scenario: A gauge and two signal buckets with identical channel scope arrive together.
// Guarantees: Gauges remain unchanged and datapoint attributes isolate cumulative baselines.
test("deriveClientDeltas leaves gauges unchanged and separates attribute scopes", () => {
  const state = { clientDeltaPrevBySeries: new Map() };

  deriveClientDeltas(
    [
      {
        name: "tokio.runtime",
        attributes: { "pipeline.id": "p1", "core.id": "0" },
        metrics: [{ name: "worker.count", value: 4, instrument: "gauge" }],
      },
      {
        name: "channel.receiver",
        attributes: { "channel.id": "ch1", "core.id": "0" },
        metrics: [{ name: "messages", value: 8, attributes: { signal: "logs" } }],
      },
      {
        name: "channel.receiver",
        attributes: { "channel.id": "ch1", "core.id": "0" },
        metrics: [{ name: "messages", value: 3, attributes: { signal: "traces" } }],
      },
    ],
    state,
    1_000
  );

  const next = deriveClientDeltas(
    [
      {
        name: "tokio.runtime",
        attributes: { "pipeline.id": "p1", "core.id": "0" },
        metrics: [{ name: "worker.count", value: 5, instrument: "gauge" }],
      },
      {
        name: "channel.receiver",
        attributes: { "channel.id": "ch1", "core.id": "0" },
        metrics: [{ name: "messages", value: 11, attributes: { signal: "logs" } }],
      },
      {
        name: "channel.receiver",
        attributes: { "channel.id": "ch1", "core.id": "0" },
        metrics: [{ name: "messages", value: 9, attributes: { signal: "traces" } }],
      },
    ],
    state,
    2_000
  );

  assert.equal(next[0].metrics[0].value, 5);
  assert.equal(next[1].metrics[0].value, 3);
  assert.equal(next[2].metrics[0].value, 6);
});
