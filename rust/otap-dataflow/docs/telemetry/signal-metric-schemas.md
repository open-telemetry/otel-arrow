# Dual signal-metric schemas (granular vs agnostic)

Status: Draft (for discussion). Tracking issue:
[open-telemetry/otel-arrow#3300](https://github.com/open-telemetry/otel-arrow/issues/3300).
Scope: design only - proposes a direction, changes no production code.

## 1. Summary

Nine pipeline nodes hand-roll **per-signal** counters (logs / metrics / spans)
with ~6 naming schemes and inconsistent or mislabeled units (see #3300 for the
full inventory). Two normalized shapes have emerged for the per-signal axis:

- **Granular** - one metric per signal, distinguished by **name** (historical
  Go-collector style): `consumed_log_records`, `consumed_metric_points`,
  `consumed_spans`.
- **Agnostic** - a **single** metric distinguished by a `signal` **attribute**
  (`logs` / `metrics` / `traces`), matching the Collector's `otelcol.signal`
  attribute, per the Collector [Pipeline Component Telemetry
  RFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md#processors).

This RFC proposes supporting **both** as first-class targets, with node
instrumentation written **once** and emitted in whichever schema is selected,
via **codegen** rather than a runtime reshape. It explores the design axes and
recommends one option each.

## 2. Goals and non-goals

**Goals:** define the two schemas precisely; write per-signal instrumentation
once, independent of the active schema; pick a generation strategy, a source of
truth, and a selection mechanism; keep the hot path and static-descriptor model
intact.

**Non-goals:** renaming every existing counter (the broader #3300 cleanup - this
RFC defines the *mechanism*).

## 3. Background: how metrics flow today

1. **Declare.** A node declares a metric-set struct via `#[metric_set(name)]` +
   `#[metric(unit)]` fields (e.g. `debug_processor/metrics.rs`).
2. **Generate.** `crates/telemetry-macros/src/lib.rs` expands it into a **static
   `MetricsDescriptor`** (name + ordered fields: name/unit/instrument/
   temporality) and a `MetricSetHandler`.
3. **Increment.** Hot paths mutate fields directly (`.add(n)` / `.inc()`).
4. **Aggregate.** Snapshots accumulate in a registry keyed by `EntityKey`.
5. **Export.** The dispatcher (`crates/telemetry/src/metrics/dispatcher.rs`)
   maps `descriptor.name` -> meter / **scope**, each `field.name` -> **OTLP metric
   name**, entity attributes -> **scope** attributes, and hard-codes **data-point
   attributes to `&[]`** (`dispatch_metrics_to`, ~line 85).

Two consequences:

- **Granular is nearly free** - the field name already *is* the metric name, so
  the descriptor the macro produces is essentially the granular schema.
- **Agnostic needs a new per-data-point attribute path** - `signal` varies per
  increment, not per entity, so it cannot ride on the scope. **This is the
  single largest piece of work** the agnostic schema implies, independent of
  generation strategy.

`semconv/` is a [Weaver](https://github.com/open-telemetry/weaver) registry used
in CI today **only** for **event** `live-check` (`semconv/README.md`); it
generates no Rust. Weaver-driven metric codegen would be new.

## 4. The two canonical schemas

Canonical nouns/units (from #3300, aligned with the Collector convention):

| Signal  | Granular noun   | Item unit      | `signal` value |
| ------- | --------------- | -------------- | -------------- |
| Logs    | `log_records`   | `{log_record}` | `logs`         |
| Metrics | `metric_points` | `{data_point}` | `metrics`      |
| Spans   | `spans`         | `{span}`       | `traces`       |

The two schemas put the signal axis in different places: **granular** uses
item-level *nouns/units* in the metric **name** (`log_records`/`metric_points`/
`spans`), while **agnostic** uses signal-*type* values in the `signal`
**attribute** (`logs`/`metrics`/`traces`), to match the Collector's
`otelcol.signal`. So a "consumed, item-level" counter is either three named
metrics (`consumed_log_records {log_record}`, ...) or one metric `consumed_items
{item}` with three `signal`-attributed data points (section 6).

## 5. Design axes

### Axis A - Codegen vs runtime conversion

- **A1 codegen** - the macro emits *both* shapes; each metric set selects one at
  registration. Call site identical (section 6); descriptor stays static; hot path
  stays a direct add.
- **A2 runtime reshape** - nodes emit one shape; the dispatcher rewrites it per
  config at export. Needs descriptor grouping metadata, the per-data-point
  attribute path, and a per-tick merge/split transform.

| Criterion        | A1 codegen             | A2 runtime reshape       |
| ---------------- | ---------------------- | ------------------------ |
| Export cost      | Static, minimal        | Extra per-tick transform |
| Descriptor       | Static (both prebuilt) | Needs grouping metadata  |
| Binary size      | Both shapes compiled   | One shape                |
| Deploy-time swap | Yes (select at reg.)   | Yes                      |
| New machinery    | Macro branch           | Mapping meta + transform |

**Recommendation: A1.** It keeps the static descriptor and the hot path, and
still gives Axis C2's deploy-time switching (compile both, select per config) -
without A2's permanent mapping metadata and per-tick transform. Its only extra
cost is binary size. (The per-data-point attribute work is needed by agnostic
either way, so it is not a differentiator.)

### Axis B - Source of truth: Weaver vs proc-macro

- **B1 Weaver-as-source + codegen** - declare both schemas in `semconv/`,
  generate Rust via templates; the same registry drives `live-check`, so
  declared and emitted cannot drift. Cost: a net-new codegen pipeline and a
  second generator beside the proc-macro.
- **B2 proc-macro generates, Weaver validates** - extend `#[metric_set]` with
  the signal-split; Weaver only declares + `live-check`-validates (its current
  role). Cost: schema lives in two places, kept consistent by CI.

**Recommendation: B2 now, B1 as end-state.** The proc-macro already produces the
granular descriptor; extending it is incremental. Adopt Weaver for declaration +
validation immediately; revisit full Weaver codegen once the API stabilizes.

### Axis C - Selection: compile-time vs runtime

- **C1 cargo feature** - mutually exclusive `schema-granular`/`schema-agnostic`
  for the whole binary. Zero runtime cost, smallest binary, but **no deploy-time
  flexibility** (rebuild to switch) and feature-unification hazards.
- **C2 runtime config** - a `metrics.signal_schema` knob chosen at startup, with
  an optional per-component override. Deploy-time flexible; one binary serves
  both audiences; costs binary size (both shapes compiled) and config plumbing.

**C2 does not require A2's reshape engine:** with A1 codegen, each metric set
picks its prebuilt shape at registration; the hot path is still a direct add.

**Recommendation: C2**, engine-wide default `granular` with optional
per-component override, paired with A1. Keep C1-style features only as an
optional build-time prune for size-constrained builds.

## 6. Proposed write-once instrumentation API

A call site identical across schemas:

```rust
pub enum Signal { Logs, Metrics, Traces }

#[metric_set(name = "processor.debug.pdata")]
pub struct DebugPdataMetrics {
    // Carries both shapes; picks one per resolved config.
    #[signal_metric(verb = "consumed")]
    pub consumed: SignalCounter<u64>,
    #[metric(unit = "{msg}")]
    pub logs_consumed: Counter<u64>, // plain metrics unchanged
}

// Hot path - same line regardless of schema:
m.consumed.add(Signal::Logs, n);
```

Both shapes are compiled in and resolved from config at registration. The same
`Signal` variant expands to the item-level noun (granular) or the signal-type
attribute value (agnostic):

- **Granular** - three descriptor fields (`consumed_log_records {log_record}`,
  `consumed_metric_points {data_point}`, `consumed_spans {span}`);
  `add(Signal::X, n)` routes to the matching field; export unchanged.
- **Agnostic** - one field `consumed_items {item}` plus per-signal slots; export
  emits one metric with a `signal` data-point attribute (`logs` / `metrics` /
  `traces`) per non-zero slot.

For the same three calls `add(Logs,120)`, `add(Metrics,80)`, `add(Traces,40)`:

```text
granular   consumed_log_records   {log_record} -> 120   (entity attrs only)
           consumed_metric_points {data_point} ->  80
           consumed_spans         {span}       ->  40

agnostic   consumed_items {item}
             -> 120  (entity + signal=logs)
             ->  80  (entity + signal=metrics)
             ->  40  (entity + signal=traces)
```

The only difference is whether the signal axis lands in the metric **name** or a
data-point **attribute**; entity attributes stay on the scope as today. Granular
emits a zero field for an idle signal; agnostic simply omits that data point
(naturally sparse).

**Required plumbing (independent of which shape is active):** the dispatcher
must gain a per-data-point attribute path so agnostic can attach `signal` -
`add_opentelemetry_metric` and the snapshot model must carry an optional
attribute set per value instead of the hard-coded `&[]`. `SignalCounter<u64>` is
the counter specialization; the abstraction must be generic over the instrument
(`SignalMetric<I>`) to also cover the `Mmsc` instrument used by `flow`
(section 6.1).

### Switching schema via config

A `schema` knob under the engine telemetry config selects the shape; switching
is a config edit + restart (no rebuild). Engine-wide default with a
per-component override during staged migration:

```yaml
engine:
  telemetry:
    metrics:
      signal_schema: granular     # engine-wide default | agnostic
groups:
  default:
    pipelines:
      main:
        nodes:
          debug:
            type: "urn:otel:processor:debug"
            telemetry:
              metrics:
                signal_schema: agnostic  # override for this component
```

Precedence: component > engine > built-in default (`granular`). The knob governs
**only** signal-split metrics (`#[signal_metric]` / `SignalCounter`); plain
`Counter`/`Gauge` fields are unaffected. Key placement is illustrative pending
the `otap_df_config` model. Flow metrics (section 6.1), being engine/policy-declared,
follow the engine-wide value.

### Worked example: debug_processor

`debug_processor` declares ten counters that mix granularities and mislabel
units (`debug_processor/metrics.rs`). They fall into three groups:

- **Item-level signal triple** - `log_signals_consumed` (log records),
  `metric_datapoints_consumed` (data points), `span_signals_consumed` (spans).
- **Message-level signal triple** - `logs_consumed`, `metrics_consumed`,
  `traces_consumed` (each `.add(1)` per batch).
- **Auxiliaries with no cross-signal peer** - `metric_signals_consumed` (metric
  *series*, an intermediate grouping above data points that logs/spans lack),
  `events_consumed`, `span_events_consumed`, `span_links_consumed`. These stay
  plain because they cannot live on a uniform `add(Signal::X, n)` axis. (Their
  unit/name hygiene is separate #3300 work.)

The two triples collapse to two signal-split declarations:

```rust
#[metric_set(name = "processor.debug.pdata")]
pub struct DebugPdataMetrics {
    #[signal_metric(verb = "consumed")]                    // item-level
    pub consumed: SignalCounter<u64>,
    #[signal_metric(verb = "consumed", per = "message")]   // {msg}
    pub consumed_messages: SignalCounter<u64>,
    #[metric(unit = "{metric}")] pub metric_signals_consumed: Counter<u64>,
    #[metric(unit = "{event}")]  pub events_consumed: Counter<u64>,
    #[metric(unit = "{event}")]  pub span_events_consumed: Counter<u64>,
    #[metric(unit = "{link}")]   pub span_links_consumed: Counter<u64>,
}
```

Increment sites change from a field per signal to `add(Signal::X, n)`, e.g.:

```rust
// before                                    after
log_signals_consumed.add(lr)             //  consumed.add(Signal::Logs, lr)
metric_datapoints_consumed.add(dp)       //  consumed.add(Signal::Metrics, dp)
span_signals_consumed.add(sp)            //  consumed.add(Signal::Traces, sp)
logs_consumed.add(1)                     //  consumed_messages.add(Logs, 1)
metrics_consumed.add(1)                  //  consumed_messages.add(Metrics, 1)
traces_consumed.add(1)                   //  consumed_messages.add(Traces, 1)
```

Net: ten bespoke counters become two signal-split declarations plus four
auxiliaries; the mislabeled `{log}` unit and naming divergence disappear because
names/units come from the canonical table (section 4); node code is schema-independent.

The `per` axis selects item-level vs message-level (batch) naming; the two sets
stay distinct, and in both the agnostic metric carries the `signal` data-point
attribute (`logs` / `metrics` / `traces`):

| `per` | Granular (logs / metrics / traces) | Agnostic |
| --- | --- | --- |
| `item` (default) | `consumed_log_records` `{log_record}` / `consumed_metric_points` `{data_point}` / `consumed_spans` `{span}` | `consumed_items` `{item}` |
| `message` | `consumed_log_messages` / `consumed_metric_messages` / `consumed_trace_messages` (all `{msg}`) | `consumed_messages` `{msg}` |

### 6.1 Flow metrics

The engine `flow` metrics (`crates/engine/src/flow_metrics.rs`:
`signals.incoming`, `.outgoing`, `.dropped`, all `{item}` `Mmsc`) are today's
purest agnostic baseline - one item measure with no signal dimension. This work
should also **align flow's verbs** to the Collector's component-telemetry
vocabulary: `incoming`->`consumed`, `outgoing`->`produced` (matching
`otelcol.<kind>.consumed.items` / `produced.items`). Under the mechanism:
**agnostic** keeps a single item measure (`consumed.items {item}`, optionally
with a `signal` split); **granular** splits per signal (`consumed.log_records
{log_record}`, etc.). Flow adds requirements beyond the node case:

- **Instrument** - flow uses `Mmsc`, so `#[signal_metric]` must be
  instrument-generic.
- **Engine/policy-declared** - flow sets come from the pipeline telemetry policy
  (`otap_df_config::policy`), not node source; codegen must cover them.
- **Per-signal counts at boundaries** - flow currently *aggregates* across
  signals; granular flow needs per-signal counts the engine does not yet carry.
  If unavailable, granular flow degrades to the agnostic `{item}` measure.
- **`dropped` has no Collector verb.** The Collector models drops as an
  `otelcol.component.outcome` (`success`/`failure`/`refused`) attribute on the
  consumed/produced metric, not a distinct metric.

## 7. Stability and compatibility

Both schemas are **breaking** vs today's names/units; follow
`stability-compatibility-guide.md`. Default `metrics.signal_schema` to
`granular`, document the per-node rename map, and let agnostic adopters flip the
config (no rebuild, switchable per component). A time-boxed "emit-both" mode is
possible but doubles cardinality and is not recommended as a steady state.

## 8. Phased rollout (non-binding)

1. Land this RFC; open an issue for the per-data-point attribute dispatcher
   change (the long pole).
2. Add `Signal` + `SignalMetric` + `#[signal_metric]` to `telemetry`/
   `telemetry-macros`, compiling both shapes and selecting via
   `metrics.signal_schema` at registration (default `granular`).
3. Declare both schemas in `semconv/`; extend `live-check` to validate them
   (B2).
4. Migrate nodes incrementally, starting with `debug_processor`.
5. Generalize over `Mmsc` and apply to `flow` (section 6.1), tracking per-signal
   boundary counts as a prerequisite for granular flow.
6. Reassess Weaver-driven codegen (B1) once stable.

## 9. Open questions

- **`signal` key** - the value vocabulary is decided (`logs` / `metrics` /
  `traces`, matching the Collector's `otelcol.signal`); the open part is the
  exact attribute *key* (`otelcol.signal` vs an `otap.*` key).
- **Flow split** - expose `signal` on flow in agnostic mode by default? How to
  source per-signal boundary counts (engine aggregates today)?
- **Verb taxonomy** - flow's verbs are decided (`consumed`/`produced`, per the
  Collector). Open: the broader set (`accepted`/`refused`,
  `sent`/`send_failed`/`enqueue_failed`, processor `dropped`) per #3300, whether
  `dropped` stays a verb or becomes an `outcome` attribute, and name ordering
  (verb-first `consumed.items` vs noun-first `signals.incoming`). Full verb
  standardization is orthogonal to the signal axis and may warrant its own doc.
- **Emit-both mode** - should `signal_schema` support a `both` value that
  emits granular and agnostic simultaneously? section 7 floats it as a
  time-boxed migration aid, but it doubles cardinality and risks
  double-counting in dashboards that scrape both - decide whether to expose it
  at all or keep switching strictly one-at-a-time.

## 10. Documentation impact: `metrics-guide.md`

[`metrics-guide.md`](metrics-guide.md) encodes assumptions this design changes;
edits should land **with** the implementation:

- **Entity-only attributes.** It states all metrics in a set share an
  entity-only attribute set. Agnostic adds a per-data-point `signal` attribute.
  The guide already allows *bounded signal-specific attributes* ("a small enum
  such as a 'state' dimension") - name `signal` as the canonical example and the
  one bounded non-entity attribute.
- **Gap today.** That section defers such attributes to `implementation-gaps.md`
  and the dispatcher hard-codes `&[]`; link the per-data-point work (section 6).
- **Units.** The "common units" list blesses `{metric}`, `{log}`, `{event}`,
  `{span}`, `{signal}` - the units #3300 flags. Align to `{log_record}`,
  `{data_point}`, `{span}`, `{item}`; map `{metric}`->`{data_point}`,
  `{log}`->`{log_record}`; revisit the `{signal}` example (ambiguous vs the new
  attribute).
- **Performance.** "Same attribute set... registers once... reports only
  scalars" - note the agnostic exception (a per-data-point `signal` alongside
  the scalar).
- **Checklist.** Add: per-signal item counters use the signal-split mechanism
  (section 6), inheriting the active schema.
- **Pre-existing drift (flag only).** Naming examples (`otelcol.node.retry`)
  diverge from actual names (`processor.retry`, per `AGENTS.md`); resolve in the
  same cleanup.

## References

- #3300 - Alignment of per-signal metric naming conventions.
- Collector [Pipeline Component Telemetry
  RFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).
- `metrics-guide.md`, `semantic-conventions-guide.md`,
  `stability-compatibility-guide.md`.
- `crates/telemetry-macros/src/lib.rs`,
  `crates/telemetry/src/metrics/dispatcher.rs`,
  `crates/engine/src/flow_metrics.rs`, `semconv/README.md`.
