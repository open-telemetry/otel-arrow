# Datapoint-level enum attributes for metrics

Status: Draft. Tracking issue:
[#3300](https://github.com/open-telemetry/otel-arrow/issues/3300).
Scope: design only.

## 1. Summary

Nine nodes hand-roll per-signal counters (logs/metrics/spans) with ~6 naming
schemes and mislabeled units (#3300). An earlier draft standardized *two* export
shapes (name-split "granular" vs attribute-split "agnostic") and let operators
pick. This revision is simpler and more general:

1. **One format.** Signal is a **datapoint attribute** (`signal` =
   `logs`/`metrics`/`traces`), matching the Collector's `otelcol.signal`
   ([RFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md#processors)).
2. **Generalize.** `signal` is one case of **bounded enum attributes that vary
   per datapoint** (e.g. `outcome` = `success`/`error`, or
   `http.response.status_code`). Define the mechanism once, with a
   **static/dynamic** qualifier per attribute.
3. **Keep `metric_set` as the unit.** This is *not* a new instrument family; the
   `metric_set` stays the unit of declaration, registration, registry state, and
   admin/UI visibility. Enum attributes are extra dimensions on a set's
   snapshots/datapoints, not separate sets or instruments. Sets with no enum
   attributes keep today's behavior and hot path.

### Why a single format

Agnostic is the default and canonical shape, and it stays the source of truth:
it carries `signal` explicitly, so any downstream consumer can reconstruct the
name-split ("granular") view operators may want for legacy dashboards. The
reverse - rebuilding `signal` from split names - is lossier, so agnostic is the
better format to emit and store.

The granular reshape belongs in the **ITS (Introspective Telemetry Service)
metrics system**, not at the export boundary: either as a native capability of
the ITS metrics receiver, or as an OPL function that partitions a
signal-attributed metric into canonical granular names, e.g.

```text
metrics | split_signal("consumed_items")
// consumed_items {item} -> consumed_log_records {log_record}
//                        / consumed_metric_points {data_point}
//                        / consumed_spans {span}
```

This needs OPL to gain data-point field access and a regroup/partition step
(neither exists today), so the split is a future ITS capability. Migrating the
hand-rolled counters to the agnostic shape is therefore gated on ITS being able
to produce the legacy shape when a consumer needs it - not on this RFC.

### Why generalize to enum attributes

The same need recurs beyond `signal`: `outcome` (`success`/`error`),
`http.response.status_code`, error classes. Modeling all as **enum attributes**
(closed set) with a static/dynamic qualifier solves per-signal as a side effect,
matching `metrics-guide.md`'s "small enum such as a 'state' dimension".

## 2. Goals and non-goals

**Goals:** define enum attributes (static/dynamic) **on the existing
`metric_set`**; write instrumentation once; keep the hot path and static
descriptor; keep `metric_set` as the unit of registration and admin/UI
visibility.

**Non-goals:** a new instrument family (this reuses `metric_set`); renaming
every counter (#3300 cleanup); a second name-split shape.

## 3. Background: metrics flow today

1. **Declare** `#[metric_set]` + `#[metric(unit)]` fields.
2. **Generate** a static `MetricsDescriptor` + `MetricSetHandler`
   (`telemetry-macros/src/lib.rs`).
3. **Increment** fields directly (`.add`/`.inc`).
4. **Aggregate** into a registry keyed by `EntityKey`.
5. **Export**: dispatcher maps names/scope attrs, and hard-codes data-point
   attributes to `&[]` (`dispatcher.rs` `dispatch_metrics_to`, ~line 85).

A per-datapoint attribute has nowhere to go today: `signal` varies per
increment, not per entity. **Adding a per-data-point attribute path is the
single biggest piece of work** - and with one format it is the *only* new
mechanism. `semconv/` (Weaver) is used only for event `live-check` today.

## 4. Enum attributes: static vs dynamic

Enum attributes are **declared per metric set**, not per field. An attribute is
a closed-set (`enum`) value; each variant derives its string form via an
`AttributeEnum` derive, so keys/values are known at compile time. Two
qualifiers:

- **Static** - value fixed at registration; no per-call argument (e.g. a
  logs-only set dedicated to `signal = logs`). Every datapoint in the set
  carries it.
- **Dynamic** - value chosen when instrumentation records the metric. A set may
  declare several (e.g. `signal` + `outcome`, or `http.response.status_code` on
  an HTTP receiver). Each recorded datapoint carries one variant per dynamic
  attribute.

Attributes are grouped into reusable `#[attribute_set]` structs (one enum field
per attribute), and a `metric_set` references them:

- `static_attributes = <Set>` - static; the values are supplied once at
  registration and apply to the whole set.
- `dynamic_attributes = <Set>` - dynamic; the values are supplied per record via
  a bound view (see section 5).

```rust
#[derive(Debug, Clone, Copy, AttributeEnum)]
pub enum Signal { Logs, Metrics, Traces }

#[derive(Debug, Clone, Copy, AttributeEnum)]
pub enum LossOutcome { Dropped, Expired }

// A dynamic set: values vary per record.
#[attribute_set(name = "durable_buffer.loss.attrs", dynamic)]
#[derive(Debug, Clone, Copy)]
pub struct DurableBufferLossAttributes {
    #[attribute(key = "signal")]
    pub signal: Signal,
    #[attribute(key = "outcome")]
    pub outcome: LossOutcome,
}

// A static set: value fixed once at registration.
#[attribute_set(name = "signal.attrs")]
#[derive(Debug, Clone, Copy)]
pub struct SignalAttributes {
    #[attribute(key = "signal")]
    pub signal: Signal,
}
```

Because every attribute is a closed `enum`, the **worst-case cardinality of a
set is known at compile time** - the product of its dynamic attributes' variant
counts (static attributes contribute a factor of 1). The Rust OpenTelemetry SDK
enforces a runtime cardinality limit per instrument (overflow series collapse
into `otel.metric.overflow`), silently losing fidelity. Since our bound is
compile-time, codegen can emit a build warning when a set would exceed that
limit (or a configured budget), catching blow-ups (e.g. an accidental
`http.response.status_code` x `signal` product) before they ship - a guarantee
free-form string attributes cannot offer.

## 5. Worked example: durable_buffer_processor (dynamic)

`durable_buffer_processor` (`otap.processor.durable_buffer`, `mod.rs`) hand-rolls
eight loss counters - two item-level aggregates plus two per-signal triples:

```text
dropped_items {item}  dropped_log_records {log_record}
                      dropped_metric_datapoints {data_point}
                      dropped_spans {span}
expired_items {item}  expired_log_records {log_record}
                      expired_metric_datapoints {data_point}
                      expired_spans {span}
```

### Declaration

Using the `DurableBufferLossAttributes` set from section 4 (`signal` +
`outcome`), all eight collapse into a single `lost_items` counter on one metric
set:

```rust
#[metric_set(
    name = "otap.processor.durable_buffer.loss",
    dynamic_attributes = DurableBufferLossAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct DurableBufferLossMetrics {
    #[metric(unit = "{item}")]
    pub lost_items: Counter<u64>,
}
```

| Metric       | Unit     | Dynamic attributes                                          |
| ------------ | -------- | ----------------------------------------------------------- |
| `lost_items` | `{item}` | `signal` = logs/metrics/traces; `outcome` = dropped/expired |

### Registration and recording

A dynamic set registers with no values; the recorder supplies the per-record
variants through a generated `with(...)` that takes the attribute struct and
returns a bound view of the whole set:

```rust
let mut loss = pipeline_ctx.register_dynamic_metrics::<DurableBufferLossMetrics>();

loss.with(DurableBufferLossAttributes {
        signal: Signal::Logs,
        outcome: LossOutcome::Expired,
    })
    .lost_items
    .add(120);
```

When the attributes are loop-invariant, the returned view can be hoisted so the
bucket resolves once:

```rust
let h = loss.with(DurableBufferLossAttributes { signal, outcome }); // resolved once
for b in batches { h.lost_items.add(b.len() as u64); }
```

When they vary per iteration, the bucket is resolved each time - cheap integer
arithmetic plus one `Vec` index (below), no hashing or allocation.

### Indexing

Codegen computes a **dense mixed-radix bucket index** from the enum variant
indexes (declaration order) - the same idea as flattening nested loops into one
array index:

```text
Signal:      Logs=0   Metrics=1   Traces=2     (radix 3)
LossOutcome: Dropped=0 Expired=1                (radix 2)
```

`signal` is the low-order digit (radix 3) and `outcome` the high-order digit, so
`bucket = signal + outcome * 3`. The full 3 x 2 space maps to a flat `Vec` of
length 6:

```text
bucket  signal   outcome    formula
  0     Logs     Dropped    0 + 0*3
  1     Metrics  Dropped    1 + 0*3
  2     Traces   Dropped    2 + 0*3
  3     Logs     Expired    0 + 1*3
  4     Metrics  Expired    1 + 1*3
  5     Traces   Expired    2 + 1*3
```

The set is one struct holding `buckets: [DurableBufferLossMetrics; 6]` (or a
`Vec` sized at registration) plus a `touched: BitSet` of 6 bits. So the
`loss.with(...)` call above resolves to:

```rust
// with(Attrs { signal: Metrics, outcome: Expired }).lost_items.add(80)
let bucket = 1 /*Metrics*/ + 1 /*Expired*/ * 3;   // = 4
buckets[bucket].lost_items.add(80);
touched.set(bucket);                              // mark 4 as live
```

Buckets 0/1/2 (`Dropped`) and 3/4/5 (`Expired`) are exactly the former
`dropped_*` / `expired_*` triples, now addressed by arithmetic - no hashing, no
per-record allocation.

### Export

Reporting iterates only the `touched` bits. If a run only lost logs to expiry
and metrics to drops, just buckets 3 and 1 are touched, emitting two datapoints
(`{signal=logs, outcome=expired}` and `{signal=metrics, outcome=dropped}`); the
four never-hit combinations cost nothing beyond their zeroed slots.

## 6. Worked example: journald (static)

The opposite case: a component that only ever produces one value binds it at
registration, with no per-record argument. `journald` is a logs-only receiver,
so `signal` is fixed for the whole set via `static_attributes` (the
`SignalAttributes` set from section 4):

```rust
#[metric_set(name = "receiver.journald", static_attributes = SignalAttributes)]
#[derive(Debug, Default, Clone)]
pub struct JournaldMetrics {
    #[metric(unit = "{log_record}")]
    pub consumed: Counter<u64>,
}
```

The value is supplied once at registration and recording is unchanged - the
fixed attribute is implicit, and the set has a single bucket:

```rust
let mut journald = pipeline_ctx.register_static_metrics::<JournaldMetrics>(
    SignalAttributes { signal: Signal::Logs },
);

journald.consumed.add(n);            // exported with signal=logs
```

## 7. Design notes

- **`metric_set` stays the unit** of declaration, registration, registry state,
  and admin/UI visibility. Enum attributes are extra dimensions on the set's
  snapshots/datapoints, not separate sets or separately generated instruments.
- **Single recording form.** `with(...)` is self-documenting (each attribute
  named), order-independent, and always correct regardless of the attributes'
  types. A terser alternative is possible (open questions), but the doc keeps one
  form for consistency. Avoid a per-field positional `add(Signal::Logs,
  LossOutcome::Expired, n)`: it re-attaches attributes to individual fields
  (losing the metric-set framing), is order-sensitive, and cannot share a bucket
  across fields.
- **No runtime hashmap.** The dense mixed-radix bucketing above keeps the hot
  path to index arithmetic plus one `Vec` index. Net effect: no impact for sets
  without enum attributes; minimal cost for dynamic sets; no node-side explosion
  of hand-written per-signal/per-outcome metric sets.

## 8. Open questions

- **Keys.** Exact attribute keys (`otelcol.signal` vs `otap.*`; likewise
  `outcome`).
- **Cardinality.** How many dynamic attrs per set; keep the combined space
  bounded - `http.response.status_code` is the high-cardinality case (dozens of
  values) vs `signal`/`outcome` (a handful). The dense `Vec<M>` allocates one
  slot per possible combination, so this budget also bounds per-set memory.
- **Granular via ITS.** Whether the legacy name-split shape is delivered as a
  native capability of the ITS metrics receiver or as an OPL `split_signal`
  function, and what OPL support (data-point field access, partition) that
  requires.
- **Recording shorthand.** Whether to later add an alternative to the
  struct-bound view (section 7) - e.g. builder setters
  `.signal(...).outcome(...)` - trading extra codegen for less boilerplate.

## References

- #3300; Collector [component-telemetry
  RFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).
- `metrics-guide.md`, `semantic-conventions-guide.md`,
  `stability-compatibility-guide.md`.
- `telemetry-macros/src/lib.rs`, `telemetry/src/metrics/dispatcher.rs`,
  `durable_buffer_processor/mod.rs`, `semconv/README.md`.
