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

Use **node metrics** to see the messages, signal items, and logical payload size
a specific receiver, processor, or exporter consumes and produces. Use **flow
metrics** to measure a selected, contiguous processor range as one operation:
how many items entered and left, how long processing took, and which decision
nodes removed items.

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
emits message outcome counters on its `node.input` and `node.output`
metric sets:

### Messages, Items, and Size

A message is the PData batch that moves between nodes. An item is an individual
log record, metric data point, or span in that batch. One message can contain
multiple items, so message counts measure batch traffic while item counts
measure the number of telemetry records inside those batches. Size measures the
logical byte size of the current payload representation.

| Metric | Meaning | Emitted by | Availability |
| --- | --- | --- | --- |
| `node.input.messages` | Messages received by a node | `node.input` | `normal` or `detailed` |
| `node.output.messages` | Messages emitted by a node | `node.output` | `normal` or `detailed` |
| `node.input.items` | Items a node receives | `node.input` | `detailed`, or `normal` plus item-count opt-in |
| `node.output.items` | Items a node emits | `node.output` | `detailed`, or `normal` plus item-count opt-in |
| `node.input.size` | Logical payload bytes a node receives | `node.input` | `detailed`, or `normal` plus size opt-in |
| `node.output.size` | Logical payload bytes a node emits | `node.output` | `detailed`, or `normal` plus size opt-in |

Message, item, and size counters have bounded `signal` and `outcome` data-point
attributes. `signal` is one of `logs`, `metrics`, or `traces`; `outcome` is
`success`, `failure`, or `refused`, recorded during terminal ACK/NACK
unwinding. The metric-set entity attributes identify the pipeline and node, so
group by those attributes when comparing nodes.

At the detailed level, node metrics also report terminal latency in seconds.
`node.input.duration` measures from node input until the terminal ACK or NACK
for processors and exporters. Receivers have no input, so
`node.output.duration` measures from receiver output until that terminal
outcome. This is downstream completion latency, not processor compute time;
use `flow.compute.duration` for compute time across a processor range.

### Enable Item Counts and Size

Item counting and logical payload sizing are disabled at the normal level
because they can require payload inspection. They require
`policies.telemetry.runtime_metrics: detailed` or `normal` with a per-node opt
in.

> [!WARNING]
> Item counting and sizing add work to the data path. Their cost depends on the
> payload representation and structure. Measure the impact on a representative
> workload before enabling them broadly. Prefer per-node opt-in when only a
> specific stage needs these measurements.

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
        size: true
    config: {}
```

This narrower configuration is appropriate when only a small part of a
pipeline needs payload measurements. `detailed` enables item counts and size
for every node without node-level settings.

### Interpret Node Counts

For a linear topology, a node's `output.items` normally matches the next
node's `input.items` for the same signal. A filtering or sampling processor
can produce fewer items than it consumes; a fan-out processor can produce an
item on more than one output. Compare counts only along the particular edge or
topology behavior being investigated.

Node metrics are the right choice when operators need to locate where a signal
count changes, including receiver admission, processors, and exporter output.
Use the runnable
[`trafficgen-input-output-metrics.yaml`](../configs/trafficgen-input-output-metrics.yaml)
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
          - input_messages
          - input_items
          - input_size
          - output_messages
          - output_items
          - output_size
          - compute_duration
          - dropped_items
```

The `start_node` and `end_node` fields name processors; the range includes both
boundary processors. The engine validates that the end processor is reachable
from the start processor and rejects interleaved flow ranges. Omit `metrics` to
enable every supported flow metric. When present, it must not be empty and must
not repeat a metric.

### Flow Metrics and Attributes

Flow metrics use the `flow.input`, `flow.output`, `flow.compute`, and
`flow.dropped` instrumentation scopes and include these scope attributes:

| Attribute | Meaning |
| --- | --- |
| `flow.id` | The configured `id`. |
| `flow.node.start` | The configured start processor. |
| `flow.node.end` | The configured end processor. |
| `flow.purpose` | The configured `purpose`, or an empty value when omitted. |
| `flow.node.decision` | The decision processor that emitted `dropped.items`, or an empty value for other flow metrics. |

`flow.purpose` lets OpenTelemetry Views select a specific kind of flow when
multiple flows use the shared directional scopes. For example, a view can
select `scope_name: flow.compute` with
`scope_attributes: { flow.purpose: transform }` to
rename or route only transformation-flow metrics.

The metrics have a bounded `signal` data-point attribute with values `logs`,
`metrics`, and `traces`. They intentionally do not have an `outcome` attribute.
Flow metrics are recorded while a PData batch moves forward through the
processor range, before its terminal ACK/NACK outcome is known. They describe
range traversal and decision-node drops, independently of the eventual node
outcome.

| Configuration value | Emitted metric | Meaning |
| --- | --- | --- |
| `input_messages` | `flow.input.messages` | PData messages entering the start processor. |
| `input_items` | `flow.input.items` | Signal items entering the start processor. |
| `input_size` | `flow.input.size` | Logical payload bytes entering the start processor. |
| `output_messages` | `flow.output.messages` | PData sends leaving the end processor. |
| `output_items` | `flow.output.items` | Signal items leaving the end processor. |
| `output_size` | `flow.output.size` | Logical payload bytes leaving the end processor. |
| `compute_duration` | `flow.compute.duration` | Histogram of aggregate processor compute duration in the range, in seconds. |
| `dropped_items` | `flow.dropped.items` | Signal items a decision processor in the range chose to drop. |

For a linear flow, the sum of `flow.dropped.items` across
`flow.node.decision` equals `flow.input.items - flow.output.items`. There is no
per-decision-node kept metric: counts that survive one decision can reach a
later decision, so per-node kept counts are not additive. Use the flow's
`flow.output.items` as the flow-wide surviving count.

See
[`trafficgen-input-output-metrics.yaml`](../configs/trafficgen-input-output-metrics.yaml)
for a runnable comparison of node and flow metrics around a sampling processor.
