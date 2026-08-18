import test from "node:test";
import assert from "node:assert/strict";

import { buildGraph } from "../graph-model.js";

// Scenario: One channel endpoint is exported as several measurement and state metric sets.
// Guarantees: Attribute buckets merge into one sender, one receiver, and one graph edge.
test("buildGraph merges channel measurement buckets by endpoint scope", () => {
  const senderAttributes = {
    "channel.id": "ch1",
    "channel.kind": "pdata",
    "node.id": "source",
    "node.port": "default",
  };
  const receiverAttributes = {
    "channel.id": "ch1",
    "channel.kind": "pdata",
    "node.id": "target",
  };
  const metricSets = [
    {
      name: "channel.sender",
      attributes: senderAttributes,
      metrics: [{ name: "messages", value: 5, attributes: { outcome: "success" } }],
    },
    {
      name: "channel.sender",
      attributes: senderAttributes,
      metrics: [{ name: "messages", value: 1, attributes: { outcome: "refused" } }],
    },
    {
      name: "channel.sender",
      attributes: senderAttributes,
      metrics: [{ name: "failures", value: 1, attributes: { "error.type": "full" } }],
    },
    {
      name: "channel.receiver",
      attributes: receiverAttributes,
      metrics: [{ name: "messages", value: 4, attributes: { signal: "logs" } }],
    },
    {
      name: "channel.receiver",
      attributes: receiverAttributes,
      metrics: [
        { name: "queue.depth", value: 1 },
        { name: "capacity", value: 10 },
      ],
    },
  ];

  const graph = buildGraph(metricSets, 1, ["pdata"]);

  assert.equal(graph.edges.length, 1);
  assert.equal(graph.edges[0].data.sender.metrics.length, 3);
  assert.equal(graph.edges[0].data.receiver.metrics.length, 3);
  assert.equal(graph.edges[0].data.multiSender, false);
  assert.equal(graph.edges[0].data.multiReceiver, false);
});
