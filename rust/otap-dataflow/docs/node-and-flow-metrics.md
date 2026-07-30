# Node and Flow Metrics

This guide explains the two metric layers available for tracing signal volume
through an OTel Arrow Dataflow Engine pipeline:

```text
receiver --> processor A --> processor B --> exporter
   |            |                |           |
   +------------+----------------+-----------+-- node metrics (all nodes)
                +----------------+
                `-- flow metrics (processors only)
```

Use **node metrics** to see the signal items a specific receiver, processor, or
exporter consumes and produces. Use **flow metrics** to measure a selected,
contiguous processor range as one operation: how many items entered and left,
how long processing took, and which decision nodes removed items.

Both metric layers are emitted through the engine's internal observability
pipeline. Configure an `engine.observability.pipeline` to export them, or use
the admin metrics endpoint while developing. See
[Configuration](configuration.md#engine-section) for observability-pipeline
configuration.

## Choose the Right Metric Layer

| Question | Use |
| --- | --- |
| Which node changed the count of logs, metrics, or traces? | Node item metrics |
| What is the aggregate compute time for selected processor stages? | Flow metrics |
| Which decision processor dropped items within a processor range? | Flow metrics |
| How many items did a receiver admit or exporter emit? | Node item metrics |

## Node Metrics

With `policies.telemetry.runtime_metrics: normal` or `detailed`, every node
emits message outcome counters on its existing `node.consumer` and
`node.producer` metric sets:

### Messages and Items

A message is the PData batch that moves between nodes. An item is an individual
log record, metric data point, or span in that batch. One message can contain
multiple items, so message counts measure batch traffic while item counts
measure the volume of telemetry data inside those batches.

| Metric | Meaning | Emitted by | Availability |
| --- | --- | --- | --- |
| `consumed.messages` | Messages received by a node | `node.consumer` | `normal` or `detailed` |
| `produced.messages` | Messages emitted by a node | `node.producer` | `normal` or `detailed` |
| `consumed.items` | Items a node receives | `node.consumer` | `detailed`, or `normal` plus item-count opt-in |
| `produced.items` | Items a node emits | `node.producer` | `detailed`, or `normal` plus item-count opt-in |

Both message and item counters have bounded `signal` and `outcome` data-point
attributes. `signal` is one of `logs`, `metrics`, or `traces`; `outcome` is
`success`, `failure`, or `refused`, recorded during terminal ACK/NACK
unwinding. The metric-set entity attributes identify the pipeline and node, so
group by those attributes when comparing nodes.

### Enable Item Counts

Item counting is disabled by default because examining OTLP payloads can be
expensive. It requires `policies.telemetry.runtime_metrics: detailed` or
`normal` with a per-node opt in; `normal` alone does not enable item counts.

> [!WARNING]
> Item counting adds work to the data path. Its cost depends on the signal
> representation and batch size; in particular, OTLP-encoded payloads must be
> inspected to count their items. Measure the impact on a representative
> workload before enabling it broadly. Prefer per-node opt-in when only a
> specific stage needs signal-level accounting.

To enable it for every node in a pipeline, use `detailed`:

```yaml
policies:
  telemetry:
    runtime_metrics: detailed
```

To enable it only for selected nodes, use `normal` at the pipeline and opt in
the relevant nodes:

```yaml
policies:
  telemetry:
    runtime_metrics: normal
nodes:
  sampler:
    type: processor:log_sampling
    policies:
      telemetry:
        item_counts: true
    config: {}
```

This narrower configuration is appropriate when only a small part of a
pipeline needs signal-level accounting. `detailed` enables item counts for
every node without a node-level `item_counts` setting.

### Interpret Node Counts

For a linear topology, a node's `produced.items` normally matches the next
node's `consumed.items` for the same signal. A filtering or sampling processor
can produce fewer items than it consumes; a fan-out processor can produce an
item on more than one output. Compare counts only along the particular edge or
topology behavior being investigated.

Node metrics are the right choice when operators need to locate where a signal
count changes, including receiver admission, processors, and exporter output.
Use the runnable
[`trafficgen-per-signal-metrics-demo.yaml`](../configs/trafficgen-per-signal-metrics-demo.yaml)
example to inspect the metrics on every node or on an individually opted-in
processor.

## Flow Metrics

Flow metrics are an explicit telemetry policy for a contiguous range of
**processor** nodes. They are not enabled by `runtime_metrics`; declare each
flow under the pipeline's `policies.telemetry.flow_metrics` list.

```yaml
policies:
  telemetry:
    flow_metrics:
      - id: ingest_processing
        bounds:
          start_node: enrich
          end_node: filter
        purpose: transform
        metrics:
          - compute_duration
          - consumed_items
          - produced_items
          - dropped_items
```

The `start_node` and `end_node` fields name processors; the range includes both
boundary processors. The engine validates that the end processor is reachable
from the start processor and rejects interleaved flow ranges. Omit `metrics` to
enable every supported flow metric. When present, it must not be empty and must
not repeat a metric.

### Flow Metrics and Attributes

All flow metrics use the `flow` instrumentation scope and include these scope
attributes:

| Attribute | Meaning |
| --- | --- |
| `flow.id` | The configured `id`. |
| `flow.node.start` | The configured start processor. |
| `flow.node.end` | The configured end processor. |
| `flow.purpose` | The configured `purpose`, or an empty value when omitted. |
| `flow.node.decision` | The decision processor that emitted `dropped.items`, or an empty value for other flow metrics. |

`flow.purpose` lets OpenTelemetry Views select a specific kind of flow when
multiple flows use the shared `flow` scope. For example, a view can select
`scope_name: flow` with `scope_attributes: { flow.purpose: transform }` to
rename or route only transformation-flow metrics.

The metrics have a bounded `signal` data-point attribute with values `logs`,
`metrics`, and `traces`. They intentionally do not have an `outcome` attribute.
Flow metrics are recorded while a PData batch moves forward through the
processor range, before its terminal ACK/NACK outcome is known. They describe
range traversal and decision-node drops, independently of the eventual node
outcome.

| Configuration value | Emitted metric | Meaning |
| --- | --- | --- |
| `consumed_items` | `consumed.items` | Signal items entering the start processor. |
| `compute_duration` | `compute.duration` | Aggregate processor compute duration in the range. |
| `produced_items` | `produced.items` | Signal items leaving the end processor. |
| `dropped_items` | `dropped.items` | Signal items a decision processor in the range chose to drop. |

For a linear flow, the sum of `dropped.items` across
`flow.node.decision` equals `consumed.items - produced.items`. There is no
per-decision-node kept metric: counts that survive one decision can reach a
later decision, so per-node kept counts are not additive. Use the flow's
`produced.items` as the flow-wide surviving count.

See
[`trafficgen-flow-metrics-demo.yaml`](../configs/trafficgen-flow-metrics-demo.yaml)
for a runnable flow with sampling, filtering, transform, and recordset
decision nodes.
