# Datapoint enum attributes for metrics

Metric sets support bounded signal-specific attributes that are emitted on each
metric datapoint. Use them for categorical dimensions that are needed to
interpret a measurement, such as `signal`, `outcome`, or an HTTP method. Do not
use them for identifiers, raw error messages, paths, or any other unbounded
value; follow the [Attributes Guide](attributes-guide.md).

An enum attribute belongs to an existing `#[metric_set]`. It does not create a
new instrument or metric set. The framework stores one bucket for each
permitted dynamic attribute combination and exports only buckets that received a
recording.

## Why this exists

Multiple nodes currently hand-roll per-signal counters for logs, metrics, and
traces, using several metric naming schemes. Enum attributes provide one
uniform metric-set pattern for those dimensions and for other bounded outcomes,
without creating a separate metric set or instrument for every value.

## Choose static or dynamic attributes

| Attribute kind | Use when | Supply value | Recording cost |
| --- | --- | --- | --- |
| Static | The value is fixed for the metric-set registration. | Once at registration. | The same as a plain metric set. |
| Dynamic | The value changes for individual recordings. | With each `with(...)` view. | A bounded bucket lookup. |

For example, a logs-only receiver can use static `signal = logs`. A processor
which records losses for logs, metrics, and traces should use dynamic `signal`;
if it also distinguishes expired and dropped records, include dynamic `outcome`.

## Declare closed values

Derive `AttributeEnum` for every value type. Variant order defines the internal
bucket order, so do not reorder existing variants. Values default to snake case.
Use `#[attribute_value]` where an OpenTelemetry semantic convention specifies a
different wire value.

```rust
#[derive(Debug, Clone, Copy, AttributeEnum)]
pub enum Signal {
    Logs,
    Metrics,
    Traces,
}

#[derive(Debug, Clone, Copy, AttributeEnum)]
pub enum LossOutcome {
    Dropped,
    Expired,
}

#[derive(Debug, Clone, Copy, AttributeEnum)]
pub enum HttpMethod {
    #[attribute_value = "GET"]
    Get,
    #[attribute_value = "POST"]
    Post,
}
```

The framework derives the exported strings and knows the complete cardinality at
compile time. A dynamic metric set whose total combinations exceed the
2000-series budget fails to compile. Keep the product of all enum variants
deliberately small.

## Static attributes

Declare a regular `#[attribute_set]`, attach it to the metric set with
`static_attributes`, and pass the value by reference when registering. The value
applies to every datapoint from that registration.

```rust
#[attribute_set(name = "receiver.signal")]
#[derive(Debug, Clone, Copy)]
pub struct SignalAttributes {
    #[attribute(key = "signal")]
    pub signal: Signal,
}

#[metric_set(name = "receiver.journald", static_attributes = SignalAttributes)]
#[derive(Debug, Default, Clone)]
pub struct JournaldMetrics {
    #[metric(unit = "{log_record}")]
    pub records: Counter<u64>,
}

let attrs = SignalAttributes {
    signal: Signal::Logs,
};
let mut metrics =
    pipeline_ctx.register_metrics_with_static_attributes::<JournaldMetrics>(&attrs);
metrics.records.add(count);
```

Use static attributes only for context that remains stable for the lifetime of
the registered metric set. If the value can change from one recording to the
next, use a dynamic attribute instead.

## Dynamic attributes

Mark the attribute set as `dynamic`, attach it to the metric set with
`dynamic_attributes`, and use the generated `with(...)` method before
recording. `with(...)` returns a view of the whole metric set for that attribute
combination.

```rust
#[attribute_set(name = "processor.loss", dynamic)]
#[derive(Debug, Clone, Copy)]
pub struct LossAttributes {
    #[attribute(key = "signal")]
    pub signal: Signal,
    #[attribute(key = "outcome")]
    pub outcome: LossOutcome,
}

#[metric_set(name = "processor.loss", dynamic_attributes = LossAttributes)]
#[derive(Debug, Default, Clone)]
pub struct LossMetrics {
    #[metric(unit = "{item}")]
    pub lost_items: Counter<u64>,
}

let mut metrics =
    pipeline_ctx.register_metrics_with_dynamic_attributes::<LossMetrics>();
metrics
    .with(LossAttributes {
        signal: Signal::Metrics,
        outcome: LossOutcome::Expired,
    })
    .lost_items
    .add(count);
```

When the attributes are loop-invariant, retain the view and record through it repeatedly:

```rust
let loss = metrics.with(LossAttributes {
    signal: Signal::Logs,
    outcome: LossOutcome::Dropped,
});
for batch in batches {
    loss.lost_items.add(batch.len() as u64);
}
```

Dynamic buckets are event-driven. A bucket is reported only in intervals where
the component records through its `with(...)` view. Use a plain or static metric
set for continuously sampled gauges and observed values.

## Export behavior

Static and dynamic enum attributes are datapoint attributes:

- OTLP metrics carry them on the metric datapoint.
- The admin Prometheus endpoint emits them as unprefixed series labels.
- Entity attributes remain scope attributes (`otel_scope_*` labels in the
  Prometheus endpoint), and resource attributes remain resource metadata
  (`target_info` labels).

This separation means two dynamic buckets that share the same component scope
remain distinct Prometheus series. See the
[Attributes Guide](attributes-guide.md#how-the-layers-are-rendered) for the
complete layer mapping.

When separate OpenTelemetry keys map to the same Prometheus label after name
conversion, their values are joined with `;` in original-key order. Avoid such
collisions when defining new attributes: the combined value cannot be queried as
either original dimension independently.

## Appendix: design constraints

- `metric_set` remains the unit of declaration, registration, aggregation, and
  admin visibility. Enum attributes add datapoint dimensions; they do not create
  a new metric family or a separate metric set per value.
- Dynamic combinations use a dense bucket indexed by enum variant order. The
  first declared attribute is the low-order dimension. For `signal` with three
  values followed by `outcome` with two values, the bucket is
  `signal_index + outcome_index * 3`. The six combinations occupy one
  contiguous bucket array.
- `with(...)` resolves that bucket with integer arithmetic and an array lookup;
  it does not allocate or use a hash table. Reuse a bound view when attributes
  stay constant across a loop.
- The framework reports only dynamic buckets that received a recording since the
  preceding report. This keeps unused combinations out of the exported metrics.

## Component-author checklist

1. Reuse an OpenTelemetry semantic-convention key and value where one exists.
2. Define the closed enum values and document their semantics.
3. Choose static only when the value is fixed for registration; otherwise choose
   dynamic.
4. Keep dynamic cardinality small and meaningful under aggregation.
5. Register through `PipelineContext` and record through the returned metric set.
6. Treat enum values and attribute keys as stable telemetry API; follow the
   [Stability and Compatibility Guide](stability-compatibility-guide.md) before
   changing them.
