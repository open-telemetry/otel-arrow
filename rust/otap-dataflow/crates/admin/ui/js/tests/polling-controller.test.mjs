import test from "node:test";
import assert from "node:assert/strict";

import { deriveClientDeltas } from "../polling-controller.js";

// Verifies client-side delta derivation for cumulative snapshots across polls.
test("deriveClientDeltas computes per-series deltas and initializes baseline", () => {
  const state = { clientDeltaPrevBySeries: new Map() };
  const set = {
    name: "channel.sender",
    attributes: { "channel.id": "ch1" },
    metrics: [{ name: "send.count", value: 10 }],
  };

  const first = deriveClientDeltas([set], state, 1_000);
  assert.equal(first[0].metrics[0].value, 0);

  const second = deriveClientDeltas(
    [
      {
        ...set,
        metrics: [{ name: "send.count", value: 16 }],
      },
    ],
    state,
    2_000
  );
  assert.equal(second[0].metrics[0].value, 6);
});

// Verifies counter reset handling: negative deltas are clamped to zero and
// the baseline is updated for subsequent polls.
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

// Verifies only delta-eligible metrics are transformed and series identity
// remains isolated by metric attributes.
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
        metrics: [{ name: "recv.count", value: 8 }],
      },
      {
        name: "channel.receiver",
        attributes: { "channel.id": "ch1", "core.id": "1" },
        metrics: [{ name: "recv.count", value: 3 }],
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
        metrics: [{ name: "recv.count", value: 11 }],
      },
      {
        name: "channel.receiver",
        attributes: { "channel.id": "ch1", "core.id": "1" },
        metrics: [{ name: "recv.count", value: 9 }],
      },
    ],
    state,
    2_000
  );

  assert.equal(next[0].metrics[0].value, 5);
  assert.equal(next[1].metrics[0].value, 3);
  assert.equal(next[2].metrics[0].value, 6);
});
