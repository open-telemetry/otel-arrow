# System Metrics Guide

This guide defines how to add and evolve system metrics for the OTAP engine. It
complements the [semantic conventions guide](semantic-conventions-guide.md) and
the [entity model](entity-model.md).

System metrics are intended to describe the behavior of stable entities over
time. This document summarizes the patterns we follow when instrumenting system
metrics in the engine.

## Entity-Centric Modeling

Start by naming the entity the metric describes. A metric set should map to a
single entity type (pipeline, node, channel sender, channel receiver, runtime
thread, and so on). Metric identity should remain stable while values evolve.

Examples of stable entities:
- CPU core, NUMA node, runtime thread
- Pipeline, node, channel endpoint (sender or receiver)
- Queue, buffer, connection pool

### Entity vs Event vs Request

Metrics are for entity behavior, not request identity.
- Events such as reloads, errors, or state changes are better captured as
  events and can be counted with metrics only when attributes stay stable.
- Requests and transactions are high-cardinality and short-lived. Use traces,
  events, or exemplars instead of encoding request identifiers in metrics.
- Prefer metrics when the signal is high volume or when trends matter more than
  individual occurrences. Use events or traces for discrete, low-volume
  occurrences.

## Metric and Metric Set

*Metrics* in this project use the instrument types supported by our internal
telemetry SDK (see [crates/telemetry](/crates/telemetry/README.md) for details):
- Counter: monotonic counts of events or outcomes, recorded as deltas.
- UpDownCounter: signed deltas that can increase or decrease over time.
- ObserveCounter: monotonic counts recorded as observed cumulative values.
- ObserveUpDownCounter: observed values that may go up or down.
- Gauge: instantaneous measurements (capacity, utilization, queue depth).

Note: Histograms (distributions such as latency or batch size) are not yet
supported, but will be added soon.

ObserveUpDownCounter and Gauge both report values that can rise or fall, but
they aggregate differently. A Gauge uses last-value aggregation, while an
ObserveUpDownCounter is a sampled cumulative value that aggregates by summing
deltas over time. In this project, ObserveUpDownCounter is used for observed
totals like `otelcol.pipeline.metrics.memory_usage` and
`otelcol.tokio.runtime.task_active_count`, while Gauge is used for instantaneous
values like `otelcol.pipeline.metrics.cpu_utilization` and
`channel.receiver.capacity`.

A *metric set* is a collection of metrics related to a single entity being
observed. That entity often belongs to a larger system of entities, so metric
set attributes are usually a composition of multiple entity attributes (for
example, resource + engine + pipeline + node + channel). All metrics in a set
share the same attribute set, which contains only entity-related attributes. In
this project, non-entity attributes are prohibited in core metrics.

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

## Attributes and Entity Context

Stable attributes identify an entity and do not change during its lifetime.
An attribute set is the fixed collection of those stable attributes attached to
all metrics in a metric set. Attribute sets are derived from the entity model
and may compose multiple entity identities when the observed entity is part of
a larger system (for example, resource + engine + pipeline + node + channel).

Before adding an attribute, ask:
- If I aggregate across this attribute, does the result still make sense?
- Is the value space bounded and known at entity creation time?

If aggregation destroys meaning or the value space is unbounded, the attribute
is mis-modeled for core/system metrics.

While technically possible, our internal SDK intentionally makes the
specification of non-entity and unbounded attributes difficult in order to
discourage their use.

`user_id`, `request_id`, and `url_path` are examples of prohibited metric set
attributes. These types of attributes are named dynamic attributes in this
project.

### Normalization Patterns

When context is useful but high-cardinality, normalize it:
- URL path -> route template
- SQL query -> normalized fingerprint
- IP address -> prefix or bucket
- Error message -> error class or code

### Checklist for New Metrics

- The metric name follows the semantic conventions guide.
- The instrument type matches the intended meaning.
- Units are specified and valid.
- Attributes are stable and cardinality is bounded.
- The metric can be interpreted using the entity model attributes.

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

## Performance Considerations

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

## ToDo/Open Questions

Should we support dynamic bounded attribute based on closed sets (enums)? For 
example, `pipeline.state` with values `starting|running|stopped` or`signal.type`
with values `metric|log|trace`.
