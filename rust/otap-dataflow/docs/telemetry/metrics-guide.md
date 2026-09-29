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
main entities or nodes of the observed system. It is not for product
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
telemetry SDK (see [crates/telemetry](../../crates/telemetry/README.md) for details):

- Counter: monotonic counts of events or outcomes, recorded as deltas.
- UpDownCounter: signed deltas that can increase or decrease over time.
- ObserveCounter: monotonic counts recorded as observed cumulative values.
- ObserveUpDownCounter: observed values that may go up or down.
- Gauge: instantaneous measurements (last-value), used for capacity,
  utilization, queue depth.
- MMSC: a compact distribution summary retaining exact minimum, maximum, sum,
  and count.
- HistogramNormal: an exponential histogram retaining a normal-resolution
  bucket distribution.
- HistogramDetailed: an exponential histogram retaining a higher-resolution
  bucket distribution.

### Choosing counters and distribution tiers

To choose between OpenTelemetry metric instrument types, it is useful to
consider what you want to observe:

| Instrument           | Useful for calculating                    | Size          | Update cost |
|----------------------|-------------------------------------------|---------------|-------------|
| Counter              | Rate                                      | 8 bytes       | 0.2 ns      |
| UpDownCounter        | Total                                     | 8 bytes       | 0.2 ns      |
| Gauge                | Average                                   | 8 bytes       | 0.2 ns      |
| MinMaxSumCount       | Total, Rate, Average, Extremes            | 32 bytes      | 0.9 ns      |
| ExponentialHistogram | Total, Rate, Average, Extremes, Quantiles | 128-256 bytes | 5-7 ns      |

To choose between MinMaxSumCount and ExponentialHistogram, ask whether
detailed information about quantiles will be used or is useful for
observability. Histogram resolution is not currently configurable, see
the implementation gaps.

All distribution instruments accept the same observations, so a field can
move between them without changing what a shipped binary records. Values must
be non-negative and finite. A negative, NaN, or infinite value trips a debug
assertion; in a release build a non-finite one is dropped, so a stray NaN
cannot poison a sum for the rest of the reporting interval. That check is
most of what MinMaxSumCount costs above a plain counter; the histogram tiers
get it for free, since they already decompose the value to find its bucket.

The tiers differ on one point, and only in debug builds. Negative zero
compares equal to zero, and `Mmsc` counts it as one, while the histogram tiers
assert on it because the type behind them declares a non-negative domain and
tests the sign bit. Every tier reports a plain positive zero for it, so the
difference never reaches an export.

### Recording semantics and export temporality

Instrument types describe how node code records measurements, independently
of how a downstream consumer prefers to receive them. ITS uses the OpenTelemetry
`lowmemory` temporality mapping as its canonical representation:

- Counter and histogram/MMSC aggregates are delta.
- UpDownCounter, ObserveCounter, and ObserveUpDownCounter aggregates are
  cumulative.
- Gauge values have no aggregation temporality.

Node authors should therefore choose an instrument from the meaning and
recording form of the measurement, not from an exporter requirement. The native
pipeline currently emits the canonical representation unchanged; it cannot yet
produce a different cumulative or delta preference for each consumer. A
downstream conversion mechanism is tracked in
[#3543](https://github.com/open-telemetry/otel-arrow/issues/3543).

ObserveUpDownCounter and Gauge both report values that can rise or fall, but
they aggregate differently.

- A Gauge uses last-value aggregation,
- An ObserveUpDownCounter is a sampled cumulative value that aggregates by
  summing deltas over time.

In this project, ObserveUpDownCounter is used for observed totals like
`otelcol.pipeline.memory_usage` and
`otelcol.tokio.runtime.task_active_count`, while Gauge is used for instantaneous
values like `otelcol.pipeline.cpu_utilization` and
`channel.receiver.capacity` or `channel.receiver.queue.depth`.

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

Bounded enum attributes are supported as registration-time attributes or
per-measurement attributes. See
[Item Attributes for Metrics](item-attributes.md) for the
declaration and recording APIs.

### Minimal metric declarations

The north-star design is a minimal set of metric declarations that describes an
entity's behavior without creating a separate counter for each signal, outcome,
or other bounded dimension. Model those dimensions as attributes on one metric
set when they are needed to interpret the measurement.

For example, prefer one `lost_items` counter with bounded `signal` and `outcome`
attributes over distinct `dropped_logs`, `dropped_metrics`, `expired_logs`, and
similar counters. Do not introduce signal-specific or outcome-specific counters
when a bounded attribute set expresses the same meaning.

This follows the OpenTelemetry Collector's
[component universal telemetry RFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).

For more information, see [issue #3300](https://github.com/open-telemetry/otel-arrow/issues/3300).

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
  - `otelcol.node.otlp_grpc_exporter`
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

## Shared receiver and exporter boundary metrics

Receiver and exporter implementations should use the shared node-boundary
metric contract in addition to the engine-owned `node.input` and `node.output`
metrics.

| Metric | Node expectation |
| --- | --- |
| `receiver.received.messages` | Record one classified external message when receiver-local handling reaches its terminal local outcome. |
| `receiver.received.payload.size` | Record the encoded application payload bytes observed at the receiver boundary. |
| `receiver.processing.duration` | Measure the receiver's documented local processing boundary, ending before downstream handoff or channel wait. |
| `exporter.attempted.messages` | Record each node-local delivery attempt, including preparation failures and retries represented as attempts. |
| `exporter.attempted.duration` | Measure from attempt start through the terminal local or backend result, excluding Ack/Nack notification delivery. |
| `exporter.attempted.payload.size` | Record the encoded application payload bytes produced or submitted by the attempt when available. |
| `exporter.attempted.items` | Record the signal items handled by the attempt. |

### Choose the boundary from external work

Do not assume that one PData message corresponds to one external request. First
identify how external work maps to PData:

```text
receiver ratio: external messages : emitted PData messages
exporter ratio: input PData messages : node-local export attempts

1:1   one input maps to one output
1:N   one input fans out to several outputs
N:1   several inputs aggregate into one output
N:M   inputs and outputs are regrouped across independently owned batches
```

Receiver metrics count classified external messages. Exporter metrics count
node-local attempts, including work that ends before submission. Node metrics
count PData. Do not force their cardinalities to match.

`receiver.received` applies to ingress receivers with independently
classifiable external messages. Scrapers, generators, and similar receivers
without one use `node.output` plus node-specific collection metrics. A future
`receiver.pulled` contract could cover receiver-initiated collection.

| Work shape | Receiver boundary | Exporter boundary |
| --- | --- | --- |
| `1:1` | Record one `receiver.received` observation for the external message. | Record one `exporter.attempted` observation for the node-local delivery attempt, including a preparation-only terminal outcome. |
| `1:N` fan-out | Record one terminal local outcome for the external message; `node.output` records each emitted PData message. | Record one attempt per external submission when fan-out succeeds. A failure before a submission starts (i.e. during encoding or compression) records one preparation-only attempt. |
| `N:1` aggregation | Record each external message independently; the later aggregate PData emission belongs to `node.output`. | Record one attempt for the external batch, using batch-level items and payload size rather than repeating an input PData count. |
| `N:M` regrouping | Track external messages and PData emissions independently. Maintain explicit ownership when one external message contributes to several outputs or one output combines several messages. | Track logical batch ownership independently from attempts. ACK/NACK follows the PData-to-batch mapping; record each submission and retry when physical submission is the attempt boundary. |

Document stable external, PData, and batch identities where ownership is not
1:1. Receiver duration excludes batching and handoff wait. Exporter duration
covers attempt-owned preparation and backend work, but excludes batch buildup,
retry backoff, and Ack/Nack notification.

Shared helpers own optional-measurement policy checks. `runtime_metrics:
detailed` enables all optional measurements; per-node `policies.telemetry`
flags enable them individually. Compose the helper under a `boundary` field
when the node also has diagnostic metrics.

### Receiver implementation

Every receiver implementation should follow this shape:

```rust
let completed = self.metrics.boundary.processing().run(|processing| {
    // Node-specific: classify, decode, validate, or otherwise process the request.
    processing.set_payload_size_with(|| request.encoded_len());
    let decoded = self.decode(request)?;
    let signal = decoded.signal_type();
    match self.process(decoded) {
        Ok(value) => Ok((signal, value)), // success
        Err(error) if error.is_refusal() => {
            Err(processing.refused(signal, error)) // refused
        }
        Err(error) => Err(processing.failed(signal, error)), // failure
    }
});

// Shared instrumentation: records the terminal local outcome before handoff
// and returns the node result.
let result = self.metrics.boundary.record(completed);

// Node-specific: propagate the result and hand accepted data downstream.
let decoded = result?;
effect_handler.send_message(decoded).await?;
```

Receiver outcomes describe receiver-local work:

- `success` means classification, processing, and local admission completed and
  the message became eligible for downstream handoff.
- `refused` means validation, policy, admission, or capacity explicitly
  rejected the message.
- `failure` means decoding or other receiver-local processing was attempted but
  did not complete because of an internal error.

For request protocols, classify by protocol semantics rather than status-code
class alone.

> [!IMPORTANT]
> An outcome reports whether the work owned by that metric completed.
> `receiver.received` success means the receiver accepted the external message,
> not that the resulting PData completed downstream handoff or delivery. This
> avoids duplicating `node.output` and retaining per-message metric state
> through batching. Channel handoff can therefore fail after receiver success.
> `node.output` covers emitted PData; use node-specific diagnostics for failed
> handoff.

Record the completed observation before downstream handoff. Return classified
errors through `processing.failed` or `processing.refused`; failures before
signal classification require node-specific diagnostics. For asynchronous
batching, record success when the external message enters the node-owned batch,
not when the batch flushes.

### Exporter implementation

Every exporter implementation should follow this shape:

```rust
let signal = data.signal_type();

let completed = self
    .metrics
    .boundary
    .attempt(signal)
    .run(async |attempt| {
        // Node-specific: encode and submit one attempt.
        attempt.set_item_count_with(|| data.num_items() as u64);
        let encoded = self
            .encode(data.payload_ref())
            .map_err(|error| attempt.failed(error))?;
        attempt.set_payload_size_with(|| encoded.len());
        match self.submit(encoded).await {
            Ok(response) => Ok(response), // success
            Err(error) if error.is_refusal() => {
                Err(attempt.refused(error)) // refused
            }
            Err(error) => Err(attempt.failed(error)), // failure
        }
    })
    .await;

// Shared instrumentation: records one terminal attempt and returns its result.
let result = self.metrics.boundary.record(completed);

// Node-specific: record diagnostics and apply Ack/Nack policy.
match result {
    Ok(_) => effect_handler.notify_ack(AckMsg::new(data)).await?,
    Err(error_type) => {
        self.metrics.record_error(signal, error_type);
        effect_handler
            .notify_nack(NackMsg::new("export attempt failed", data))
            .await?;
    }
}
```

Record payload size when encoded bytes are available. Use `attempt.failed` for
ordinary failures and `attempt.refused` for explicit validation, policy,
admission, or capacity rejection.

`exporter.attempted.messages` counts node-local delivery attempts,
including attempts that fail before a backend call. Each physical retry starts
a new attempt when physical submission is the node's documented attempt
boundary. Use `node.input.messages` to count PData messages entering the
exporter; do not use the attempt metric as a duplicate input-message count.

> [!IMPORTANT]
> `exporter.attempted` outcomes describe one node-local attempt. Success means
> that attempt reached its documented completion boundary; it does not imply
> that sibling attempts succeeded or that the contributing PData was ACKed.
> Use `node.input` for the PData lifecycle.

Additional exporter rules:

- Logical batches own PData ACK/NACK mapping; attempts own submissions and
  retries. Fan-out siblings have independent measurements and outcomes.
- For asynchronous buffering, document whether completion means acceptance
  into a node-owned writer or completion of a physical sink operation. If the
  writer acceptance is the attempt boundary, later background flush, retry,
  synchronization, file closure, or object-store operations are node-specific
  and do not create shared attempts. If physical sink completion is the attempt
  boundary, record each submission and retry while retaining the ownership
  needed to complete the contributing PData. Omit payload size when encoded
  bytes cannot be attributed naturally to the attempt.
- Record one failed or refused attempt when preparation fails before a
  submission exists. Record a successful no-op when the node accepts and
  completes the work without a submission.
- Give every retry recorded as a shared attempt a fresh timing origin. Reusing
  the original timing origin incorrectly includes earlier attempts and backoff.

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
