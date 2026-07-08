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

### Why a single format

This does **not** lock out a name-split view: internal telemetry is emitted
through a pipeline, so operators can reshape downstream via the `transform`
processor's OPL.

```text
metrics | if (name == "consumed_items") { set name = "consumed_log_records" }
```

The per-`signal` split is **not yet supported**: `signal` is a *data-point*
attribute, but OPL's metric `attributes` are metric-level metadata and data
points are "Future signal fields"; and splitting one metric into per-signal
names needs a partition operator OPL lacks. So the reshape is directionally
sound but a future capability. The reverse (rebuilding `signal` from names) is
strictly messier, so the attribute shape is the better source of truth.

#### Target OPL expression and the work it implies

Eventual reshape (row = data point; `attributes` = DP attrs; `name`/`unit` =
parent-metric identity):

```text
metrics
| where name == "consumed_items"
| if (attributes["signal"] == "logs") {
      set name = "consumed_log_records" | set unit = "{log_record}"
  } else if (attributes["signal"] == "metrics") {
      set name = "consumed_metric_points" | set unit = "{data_point}"
  } else if (attributes["signal"] == "traces") {
      set name = "consumed_spans" | set unit = "{span}"
  }
| remove attributes["signal"]
```

This needs OPL to gain data-point field access and a regroup/partition step
(neither exists today), so it is a future capability, not on this RFC's critical
path. It is shown only to confirm the attribute shape is reconstructible.

To avoid the `if`/`else` boilerplate, a dedicated OPL function could partition a
signal-attributed metric into canonical granular metrics in one call, taking the
target metric name as a parameter so only that metric is split:

```text
metrics | split_signal("consumed_items")
// consumed_items {item} -> consumed_log_records {log_record}
//                        / consumed_metric_points {data_point}
//                        / consumed_spans {span}
```

### Why generalize to enum attributes

The same need recurs beyond `signal`: `outcome` (`success`/`error`),
`http.response.status_code`, error classes. Modeling all as **enum attributes**
(closed set) with a static/dynamic qualifier solves per-signal as a side effect,
matching `metrics-guide.md`'s "small enum such as a 'state' dimension".

## 2. Goals and non-goals

**Goals:** define enum attributes (static/dynamic); write instrumentation once;
pick source of truth + generation; keep the hot path and static descriptor.

**Non-goals:** renaming every counter (#3300 cleanup); a second name-split shape.

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

A closed-set (`enum`) attribute, qualified:

- **Static** - value fixed at registration; no per-call argument (e.g. a set
  dedicated to `signal = logs`).
- **Dynamic** - value passed at the call (`inc`/`add`); generated methods take
  **one enum arg per dynamic attribute**, in order. A set may declare several
  (e.g. `signal` + `outcome`, or `http.response.status_code` on an HTTP
  receiver).

Because every attribute is a closed `enum`, the **worst-case cardinality of a
metric is known at compile time** - it is the product of the variant counts of
its dynamic attributes (static attributes contribute a factor of 1). The Rust
OpenTelemetry SDK enforces a runtime cardinality limit per instrument (overflow
series are collapsed into an `otel.metric.overflow` bucket), so exceeding it
silently loses fidelity. Since our bound is compile-time, codegen can compute
the product and emit a build warning when a set would exceed that limit (or a
configured budget) - catching blow-ups (e.g. an accidental
`http.response.status_code` x `signal` combination) before they ship. Free-form
string attributes cannot offer this guarantee.

## 5. Worked examples

Enum attributes are declared on a field via `attrs(dynamic(...))` or
`attrs(static(...))`, where each `key = EnumType` names the attribute and its
closed value set (e.g. `pub enum Signal { Logs, Metrics, Traces }`).

### Dynamic: durable_buffer_processor

`durable_buffer_processor` (`otap.processor.durable_buffer`, `mod.rs`) declares
eight loss counters - two item-level aggregates plus two per-signal triples:

```text
dropped_items {item}  dropped_log_records {log_record}
                      dropped_metric_datapoints {data_point}
                      dropped_spans {span}
expired_items {item}  expired_log_records {log_record}
                      expired_metric_datapoints {data_point}
                      expired_spans {span}
```

They collapse to two `signal`-attributed metrics (the `_items` aggregate is just
the sum across `signal`):

```rust
#[metric(unit = "{item}", attrs(dynamic(signal = Signal)))]
pub dropped: ObserveCounter<u64>,   // DropOldest retention
#[metric(unit = "{item}", attrs(dynamic(signal = Signal)))]
pub expired: ObserveCounter<u64>,   // max_age retention

m.dropped.add(Signal::Metrics, n);
m.expired.add(Signal::Logs, n);
```

Going further, `dropped` vs `expired` is itself a bounded enum - a loss
`outcome`. A **second** dynamic enum attribute folds the two into one metric,
taking eight counters down to one:

```rust
pub enum LossOutcome { Dropped, Expired }

#[metric(unit = "{item}", attrs(dynamic(signal = Signal, outcome = LossOutcome)))]
pub lost: ObserveCounter<u64>,

m.lost.add(Signal::Logs, LossOutcome::Expired, 120);
m.lost.add(Signal::Metrics, LossOutcome::Dropped, 80);
```

| Metric | Unit     | Dynamic attributes                                          |
| ------ | -------- | ----------------------------------------------------------- |
| `lost` | `{item}` | `signal` = logs/metrics/traces; `outcome` = dropped/expired |

### Static: journald receiver

The opposite case: a component instance that only ever produces one value binds
it at registration, with no per-call argument. `journald` is a logs-only
receiver, so `signal` is fixed:

```rust
// signal is constant for this set - bound once, not passed per call.
#[metric(unit = "{log_record}", attrs(static(signal = Signal::Logs)))]
pub consumed: Counter<u64>,

m.consumed.add(n);   // exported with signal=logs
```

## 6. Open questions

- **Keys.** Exact attribute keys (`otelcol.signal` vs `otap.*`; likewise
  `outcome`).
- **Cardinality.** How many dynamic attrs per set; keep the combined space
  bounded - `http.response.status_code` is the high-cardinality case (dozens of
  values) vs `signal`/`outcome` (a handful).
- **Static default.** Whether nodes that handle one signal should prefer a
  static attribute over the dynamic form.

## References

- #3300; Collector [component-telemetry
  RFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).
- `metrics-guide.md`, `semantic-conventions-guide.md`,
  `stability-compatibility-guide.md`.
- `telemetry-macros/src/lib.rs`, `telemetry/src/metrics/dispatcher.rs`,
  `durable_buffer_processor/mod.rs`, `semconv/README.md`.
