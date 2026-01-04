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

Prefer metrics when the signal is high volume or when trends matter more than
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
totals like `pipeline.metrics.memory_usage` and `tokio.runtime.task_active_count`,
while Gauge is used for instantaneous values like `pipeline.metrics.cpu_utilization`
and `channel.receiver.capacity`.

A *metric set* is a collection of metrics related to a single entity being
observed. That entity often belongs to a larger system of entities, so metric
set attributes are usually a composition of multiple entity attributes (for
example, resource + engine + pipeline + node + channel). All metrics in a set
share the same attribute set, which contains only entity-related attributes. In
this project, non-entity attributes are prohibited in core metrics.

Metric naming must follow the
[semantic conventions guide](semantic-conventions-guide.md). Descriptions and
units are mandatory. Units must follow UCUM-like conventions and use braces
notation for semantic units (e.g. `{signal}`, `{message}`).

Metric set naming should follow the pattern `<entity>` or
`<entity>.<subentity>` when applicable (e.g. `pipeline`, `channel.sender`,
`channel.receiver`).

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
