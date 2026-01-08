# System metrics guide

This guide defines how to add and evolve system metrics for the OTAP dataflow
engine. It complements
the [semantic conventions guide](semantic-conventions-guide.md)
and the [entity model](entity-model.md).

System metrics are intended to describe the behavior of stable entities over
time. This document summarizes the patterns we follow when instrumenting system
metrics in the engine.

In this documentation, core system metrics/telemetry refers to telemetry used
to operate a system in a reliable way and to understand the behavior of the
main entities/components of the observed system. It is not for product
analytics or business telemetry.

## Related guides

- Attribute policy: [attributes-guide.md](attributes-guide.md)
- Stability
  rules: [stability-compatibility-guide.md](stability-compatibility-guide.md)
- Implementation status: [implementation-gaps.md](implementation-gaps.md)

## Entity-centric modeling

Start by naming the entity the metric describes. A metric set should map to a
single entity type (pipeline, node, channel sender, channel receiver, runtime
thread, and so on). Metric identity should remain stable while values evolve.

Examples of stable entities:

- CPU core, NUMA node, runtime thread
- Pipeline, node, channel endpoint (sender or receiver)
- Queue, buffer, connection pool

### Entity vs event vs request

Metrics are for entity behavior, not request identity.

- Events such as reloads, errors, or state changes are better captured as
  events and can be counted with metrics only when attributes stay stable.
- Requests and transactions are high-cardinality and short-lived. Use traces,
  events, or exemplars instead of encoding request identifiers in metrics.
- Prefer metrics when the signal is high volume or when trends matter more than
  individual occurrences. Use events or traces for discrete, low-volume
  occurrences.

## Metric and metric set

*Metrics* in this project use the instrument types supported by our internal
telemetry SDK (see [crates/telemetry](/crates/telemetry/README.md) for details):

- Counter: monotonic counts of events or outcomes, recorded as deltas.
- UpDownCounter: signed deltas that can increase or decrease over time.
- ObserveCounter: monotonic counts recorded as observed cumulative values.
- ObserveUpDownCounter: observed values that may go up or down.
- Gauge: instantaneous measurements (last-value), used for capacity,
  utilization, queue depth.

Histogram support status is tracked in
[Implementation Gaps](implementation-gaps.md).

ObserveUpDownCounter and Gauge both report values that can rise or fall, but
they aggregate differently.

- A Gauge uses last-value aggregation,
- An ObserveUpDownCounter is a sampled cumulative value that aggregates by
  summing deltas over time.

In this project, ObserveUpDownCounter is used for observed totals like
`otelcol.pipeline.metrics.memory_usage` and
`otelcol.tokio.runtime.task_active_count`, while Gauge is used for instantaneous
values like `otelcol.pipeline.metrics.cpu_utilization` and
`channel.receiver.capacity`.

Guideline:

- Use Gauge for point-in-time levels (queue depth, active tasks, memory in use).
- Use (Observe)Counter for counts (items processed, drops).
- Use ObserveUpDownCounter only when you have a strong reason to preserve the
  "observed cumulative" interpretation across collection intervals.

A *metric set* is a collection of metrics related to a single entity being
observed. That entity often belongs to a larger system of entities, so metric
set attributes are usually a composition of multiple entity attributes (for
example, resource + engine + pipeline + node + channel). All metrics in a set
share the same attribute set, which contains only entity-related attributes. In
this project, core metrics prioritize entity identity. However, bounded
signal-specific attributes MAY be used when they are necessary to interpret the
measurement (for example, a small enum such as a "state" dimension). When used,
signal-specific attributes MUST be:

- bounded and documented as a closed set
- meaningful under aggregation
- preferably namespaced under the metric namespace as recommended by OTel naming
  guidance

Support status for bounded signal-specific attributes is tracked in
[implementation-gaps.md](implementation-gaps.md).

Metric naming must follow the
[semantic conventions guide](semantic-conventions-guide.md). Descriptions and
units are mandatory. Units must follow UCUM conventions and use braces notation
only for annotation units (e.g. `{batch}`, `{signal}`). See the [Units](#units)
section below for details.

Metric set naming should follow the pattern `otelcol.<entity>` or
`otelcol.<entity>.<subentity>` when applicable. Examples of metric sets in this
project:

- For generic entities:
  - `otelcol.pipeline`, `otelcol.node`
  - `otelcol.channel.sender`, `otelcol.channel.receiver`
  - ...
- For specific node types:
  - `otelcol.node.retry`
  - `otelcol.node.batch`
  - `otelcol.node.otlp_receiver`
  - `otelcol.node.otlp_exporter`
  - ...

## Attributes and entity context

Metric attributes MUST follow the project-wide attribute policy in
[Attributes Guide](attributes-guide.md).

Metric-specific rule: attributes attached to core system metrics MUST remain
meaningful under aggregation.

Normalization patterns are documented in
[Attributes Guide](attributes-guide.md).

## Units

Units must be specified for every metric as part of its metadata. They must
follow UCUM conventions and use braces notation only for annotation units.

The most common units in this project are:

- Named units:
  - `By`: bytes
  - `s`: seconds (preferred over `ms` for time durations)
- Annotation units:
  - `{batch}`: batches of telemetry signals
  - `{signal}`: individual telemetry signals (metrics, logs, traces)
  - `{metric}`: individual metric data points
  - `{log}`: individual log records
  - `{event}`: individual event records (log with an event name)
  - `{span}`: individual trace spans

## Performance considerations

Metric sets are optimized for low overhead:

- The same attribute set is shared across all metrics in a metric set.
- A metric set instance registers its attributes once during setup, and the
  collection phase reports only scalar values.
- On the hot path, we increment or set values in pre-allocated non-atomic slots,
  avoiding dynamic lookups and allocations.
- Metric sets are per-core to avoid cross-core contention, and the cold path
  (flush, aggregate, encode) is NUMA-aware and batch-oriented.
- Reset-on-flush and sparse enumeration minimize work by touching only non-zero
  fields and dirty counters.

More details about the telemetry SDK implementation are in
[crates/telemetry](../../crates/telemetry/README.md).

## Metric stability and compatibility

Metrics and metric sets MUST follow the stability model in
[stability-compatibility-guide.md](stability-compatibility-guide.md).

### Checklist for new metrics

- The metric name follows the semantic conventions guide.
- The instrument type matches the intended meaning.
- Units are specified and valid.
- Attributes are stable and cardinality is bounded.
- The metric can be interpreted using the entity model attributes.
- Failure-oriented metrics SHOULD include a low-cardinality error classifier
  when applicable (`error.type`).
