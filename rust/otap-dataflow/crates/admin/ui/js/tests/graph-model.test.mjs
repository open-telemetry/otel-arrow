import test from "node:test";
import assert from "node:assert/strict";

import { buildGraph } from "../graph-model.js";

test("buildGraph attaches channel.control metric sets to the matching channel", () => {
  const graph = buildGraph(
    [
      {
        name: "channel.sender",
        attributes: {
          "channel.id": "ctrl-1",
          "channel.kind": "control",
          "channel.mode": "local",
          "channel.type": "mpsc",
          "node.id": "pipeline-ctrl",
          "node.type": "controller",
        },
        metrics: [{ name: "send.count", value: 5 }],
      },
      {
        name: "channel.receiver",
        attributes: {
          "channel.id": "ctrl-1",
          "channel.kind": "control",
          "channel.mode": "local",
          "channel.type": "mpsc",
          "node.id": "receiver-1",
          "node.type": "receiver",
        },
        metrics: [{ name: "recv.count", value: 4 }],
      },
      {
        name: "channel.control",
        attributes: {
          "channel.id": "ctrl-1",
          "channel.kind": "control",
          "channel.mode": "local",
          "channel.type": "mpsc",
        },
        metrics: [
          { name: "completion.queued", value: 3, unit: "{message}" },
          { name: "shutdown.recorded", value: 1, unit: "{1}" },
        ],
      },
    ],
    1,
    ["control"]
  );

  assert.equal(graph.edges.length, 1);
  assert.equal(graph.edges[0].data.control?.metrics?.length, 2);
  assert.deepEqual(
    graph.edges[0].data.control.metrics.map((metric) => metric.name),
    ["completion.queued", "shutdown.recorded"]
  );
});
