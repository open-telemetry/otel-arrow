# Item attributes for metrics

Metric sets support bounded attributes that are emitted on each telemetry item.
Use them for categorical dimensions that are needed to interpret a measurement,
such as `signal`, `outcome`, or an HTTP method. Do not use them for identifiers,
raw error messages, paths, or any other unbounded value; follow the [Attributes
Guide](attributes-guide.md).

An enum attribute belongs to an existing `#[metric_set]`. It does not create a
new instrument or metric set. The framework stores one bucket for each permitted
measurement attribute combination and exports only buckets that received a
recording.

## Why this exists

Multiple nodes currently hand-roll per-signal counters for logs, metrics, and
traces, using several metric naming schemes. Enum attributes provide one uniform
metric-set pattern for those dimensions and for other bounded outcomes, without
creating a separate metric set or instrument for every value.

## Choose registration or measurement attributes

| Kind | When | Supply | Cost |
| --- | --- | --- | --- |
| Registration | Fixed at registration. | At registration. | Plain-set cost. |
| Measurement | Varies by recording. | `with(...)`. | Bounded lookup. |

For example, a component that only handles logs can use registration `signal =
logs`. A component which records losses for logs, metrics, and traces should use
measurement `signal`; if it also distinguishes expired and dropped records,
include measurement `outcome`. One metric set can combine both kinds: use a
registration attribute for context shared by every recording and measurement
attributes for the dimensions that vary.

## Declare closed values

Derive `AttributeEnum` for every value type. Variant order defines the internal
bucket order, so do not reorder existing variants. Values default to snake case.
Use `#[attribute_value]` where an OpenTelemetry semantic convention specifies a
different wire value.

```rust
use otap_df_telemetry_macros::AttributeEnum;

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
compile time. A measurement metric set whose total combinations exceed the
2000-series budget fails to compile. Keep the product of all enum variants
deliberately small.

## Registration API

Component code always registers through the generated `MyMetrics::register(...)`
method. Pass the component's `PipelineContext` as the first argument:
`MyMetrics::register(&pipeline_ctx)` for metric sets without registration
attributes, or `MyMetrics::register(&pipeline_ctx, &attrs)` when they are
declared. `PipelineContext` is the registrar; it selects the metric set's entity
scope and telemetry registry.

The lower-level registrar and registry methods are intentionally hidden from
generated documentation: they exist so macro-generated code can select the
component's entity scope and so pre-existing metric declarations can continue to
register during migration. Do not call them from new component instrumentation.

## Registration-time attributes

Declare `#[attribute_set(item, registration)]`, attach it to the metric set
with `registration_attributes`, and pass the value when registering. The value
applies to every emitted item from that registration.

Every non-composed field in an attribute set becomes an attribute. Its key
defaults to the field name with underscores replaced by dots. Use
`#[attribute_key = "..."]` only when the exported key differs from that default.
Unlike scope and entity attribute sets, registration attributes do not need a
schema name because they are emitted directly on telemetry items.

```rust
// This component only works on logs.

use otap_df_telemetry::instrument::Counter;
use otap_df_config::SignalType;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

#[attribute_set(item, registration)]
#[derive(Debug, Clone, Copy)]
pub struct SignalAttributes {
    pub signal: SignalType,
}

#[metric_set(name = "component.records", registration_attributes = SignalAttributes)]
#[derive(Debug, Default, Clone)]
pub struct RecordMetrics {
    #[metric(unit = "{log_record}")]
    pub records: Counter<u64>,
}

let mut metrics = RecordMetrics::register(
    &pipeline_ctx,
    &SignalAttributes {
        signal: SignalType::Logs,
    },
);
metrics.records.add(count); // Emits `records{signal="logs"}` in the `component.records` scope.
```

Use registration attributes only for context that remains stable for the
lifetime of the registered metric set. If the value can change from one
recording to the next, use a measurement attribute instead.

## Measurement-time attributes

Use `#[attribute_set(item, measurement)]`, attach the type to the metric
set with `measurement_attributes`, and use the generated `with(...)` method when
recording. `with(...)` returns a view of the whole metric set for that attribute
combination.

Every item attribute set also implements `AttributeSetHandler`. A future log or
trace emitter can serialize the same typed values through `iter_attributes()`;
only metric sets use the additional dense-bucket implementation generated for
`measurement`.

```rust
// This component has multiple possible outcomes.

use otap_df_telemetry::instrument::Counter;
use otap_df_config::SignalType;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

#[derive(Debug, Clone, Copy, AttributeEnum)]
pub enum LossOutcome {
    Dropped,
    Expired,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct LossAttributes {
    pub signal: SignalType,
    #[attribute_key = "result"]
    pub outcome: LossOutcome,
}

#[metric_set(name = "component.loss", measurement_attributes = LossAttributes)]
#[derive(Debug, Default, Clone)]
pub struct LossMetrics {
    #[metric(unit = "{item}")]
    pub lost_items: Counter<u64>,
}

let mut metrics = LossMetrics::register(&pipeline_ctx);
metrics
    .with(LossAttributes {
        signal: SignalType::Metrics,
        outcome: LossOutcome::Expired,
    })
    .lost_items
    .add(count); // Emits `lost_items{signal="metrics", result="expired"}` in the `component.loss` scope.

// When attributes are loop-invariant, retain the view and record through it repeatedly.
let mut loss = metrics.with(LossAttributes {
    signal: SignalType::Logs,
    outcome: LossOutcome::Dropped,
});
for batch in batches {
    loss.lost_items.add(batch.len() as u64); // Emits `lost_items{signal="logs", result="dropped"}` in the `component.loss` scope.
}
```

Measurement buckets are event-driven. A bucket is reported only in intervals
where the component records through its `with(...)` view. Use a plain or
registration metric set for continuously sampled gauges and observed values.

## Combined registration and measurement attributes

A metric set can use fixed context and per-record dimensions together. Supply
both `registration_attributes` and `measurement_attributes`, then register with
the fixed attribute value and record through `with(...)`.

```rust
// This component only works on logs but also has multiple possible outcomes.

use otap_df_telemetry::instrument::Counter;
use otap_df_config::SignalType;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

#[derive(Debug, Clone, Copy, AttributeEnum)]
pub enum LossOutcome {
    Dropped,
    Expired,
}

#[attribute_set(item, registration)]
#[derive(Debug, Clone, Copy)]
pub struct SignalAttributes {
    pub signal: SignalType,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct OutcomeAttributes {
    #[attribute_key = "result"]
    pub outcome: LossOutcome,
}

#[metric_set(
    name = "component.records",
    registration_attributes = SignalAttributes,
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct RecordMetrics {
    #[metric(unit = "{log_record}")]
    pub records: Counter<u64>,
}

let signal = SignalAttributes {
    signal: SignalType::Logs,
};
let mut metrics = RecordMetrics::register(&pipeline_ctx, &signal);
metrics
    .with(OutcomeAttributes {
        outcome: LossOutcome::Dropped,
    })
    .records
    .add(count); // Emits `records{signal="logs", result="dropped"}` in the `component.records` scope.
```

The registration and measurement attribute sets MUST NOT declare the same key.
The macro rejects overlapping keys at compile time so every emitted item has one
unambiguous value for each attribute.

## Export behavior

Registration and measurement enum attributes are item attributes:

- OTLP metrics carry them on the metric datapoint.
- The admin Prometheus endpoint emits them as unprefixed series labels.
- Entity attributes remain scope attributes (`otel_scope_*` labels in the
  Prometheus endpoint), and resource attributes remain resource metadata
  (`target_info` labels).

This separation means two measurement buckets that share the same component
scope remain distinct Prometheus series. See the [Attributes
Guide](attributes-guide.md#how-the-layers-are-rendered) for the complete layer
mapping.

## Terminal handoff

Nodes that return a terminal state must include every metric snapshot created
since the last collection interval. Use `terminal_snapshots()` for every metric
set. A plain metric set returns its one bucket; a measurement set returns only
touched, non-empty buckets. Both clear their returned values because terminal
handoff transfers ownership.

```rust
let mut snapshots = operational_metrics.terminal_snapshots();
snapshots.extend(outcome_metrics.terminal_snapshots());
TerminalState::new(deadline, snapshots);
```

When separate OpenTelemetry keys map to the same Prometheus label after name
conversion, their values are joined with `;` in original-key order. Avoid such
collisions when defining new attributes: the combined value cannot be queried as
either original dimension independently.

## Appendix: design constraints

- `metric_set` remains the unit of declaration, registration, aggregation, and
  admin visibility. Enum attributes add item dimensions; they do not create
  a new metric family or a separate metric set per value.
- Measurement combinations use a dense bucket indexed by enum variant order. The
  first declared attribute is the low-order dimension. For `signal` with three
  values followed by `outcome` with two values, the bucket is `signal_index +
  outcome_index * 3`. The six combinations occupy one contiguous bucket array.
- `with(...)` resolves that bucket with integer arithmetic and an array lookup;
  it does not allocate or use a hash table. Reuse a bound view when attributes
  stay constant across a loop.
- The framework reports only measurement buckets that received a recording since
  the preceding report. This keeps unused combinations out of the exported
  metrics.

## Component-author checklist

1. Reuse an OpenTelemetry semantic-convention key and value where one exists.
2. Define the closed enum values and document their semantics.
3. Choose registration only when the value is fixed for registration; otherwise
   choose measurement.
4. Keep measurement cardinality small and meaningful under aggregation.
5. Register through `PipelineContext` and record through the returned metric
   set.
6. Treat enum values and attribute keys as stable telemetry API; follow the
   [Stability and Compatibility Guide](stability-compatibility-guide.md) before
   changing them.
