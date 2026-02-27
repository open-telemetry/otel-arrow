# V3 Pipeline Metrics — Revision Summary

## What changed

The V3 pipeline component metrics design was revised to introduce
**MetricLevel gating** — an engine-wide setting that controls
instrumentation overhead per node. The implementation was partially
completed (W1–W10 of 13 work items).

## Design revision

The original V3 design unconditionally stored timestamps and byte
counts in every entry frame. The revision addresses five concerns:

1. **MetricLevel enum** (None / Basic / Normal / Detailed) — ordered
   so `>=` comparisons gate incremental cost. `None` means zero
   overhead; each higher level adds one capability.

2. **Metrics-gated entry frame interests** — at `None`, frames have
   empty interests (no ack/nack delivery). At `Basic+`, `ACKS | NACKS`
   are auto-subscribed. At `Detailed`, `time_ns` is stamped.

3. **Forward-path byte counting** — bytes are counted when data enters
   a node (Normal+), not carried through the ack/nack return path. No
   outcome breakdown — one counter per node.

4. **Auto-subscribe with default handler** — nodes that receive
   ack/nack only because of auto-subscribe get a default handler that
   records the outcome metric and propagates.

5. **Timestamps only at Detailed** — `nanos_since_epoch()` is only
   called and stored when `MetricLevel::Detailed` is configured.

Every section of `v3-pipeline-metrics-design.md` was updated to
reflect these changes, including: V3 Key Ideas, Design §1–7, Data
Flow diagram, CallData Struct, Timestamp Representation, Node Type
Behaviors, Files Changed, What Does NOT Change, Comparison with V2,
Implementation Plan (phases 1–4), Resolved Design Questions, and
Status (with 13 concrete work items and suggested ordering).

## Code changes completed (W1–W4)

All changes compile clean and pass 811 workspace tests.

### W1. `MetricLevel` enum — `crates/engine/src/control.rs`

Added the enum with `#[derive(PartialOrd, Ord)]` and doc comments
on each variant.

### W2. Removed `req_bytes` from `CallData` — multiple files

- Removed `pub req_bytes: u64` from `CallData` in `control.rs`.
- Removed all reads of `calldata.req_bytes` and
  `frame.calldata.req_bytes` across:
  - `component_metrics.rs` (`record_produced_for_control_msg`)
  - `pdata.rs` (4 × `notify_ack` + 4 × `notify_nack` impls,
    3 helper functions, `push_entry_frame`, `received_at_node`)

### W3. `push_entry_frame` new signature — `crates/otap/src/pdata.rs`

Changed from `(node_id, time_ns, req_bytes)` to
`(node_id, metric_level)`. The method now:
- Adds `ACKS | NACKS` to interests if `metric_level >= Basic`.
- Stamps `time_ns = nanos_since_epoch()` if `metric_level >= Detailed`.
- Otherwise leaves both at zero/empty.

### W4. `ReceivedAtNode` trait signature — `lib.rs`, `pdata.rs`, `processor.rs`, `testing/mod.rs`

Changed from `(node_id, time_ns: u64)` to
`(node_id, metric_level: MetricLevel)`. Updated:
- `OtapPdata` impl (calls `push_entry_frame` with metric level).
- `TestMsg` no-op impl.
- Both processor run loops (Local + Shared) — currently pass
  `MetricLevel::None` with a TODO for W12 to wire engine config.

### Files touched

| File | Summary |
|------|---------|
| `crates/config/src/policy.rs` | +`MetricLevel` enum (serde, JsonSchema); +`component_metrics` field in `TelemetryPolicy`; +config serde/ordering tests |
| `crates/engine/src/control.rs` | `MetricLevel` re-export from config; −`req_bytes` field |
| `crates/engine/src/lib.rs` | `build_node_wrapper` accepts `metric_level` param; wired from `telemetry_policy.component_metrics` |
| `crates/engine/src/runtime_pipeline.rs` | `NodeTaskContext::new()` uses `metric_level` from telemetry policy |
| `crates/engine/src/processor.rs` | Uses `current_metric_level()` for `received_at_node` |
| `crates/engine/src/component_metrics.rs` | Level-gated metric set groups (request/bytes/duration); +level-gating unit tests |
| `crates/engine/src/entity_context.rs` | `NodeTaskContext` + `current_metric_level()` + `register_component_metrics(level)` |
| `crates/engine/src/testing/mod.rs` | `TestMsg::received_at_node` updated signature |
| `crates/otap/src/pdata.rs` | Level-gated `push_entry_frame`, `subscribe_to`, `received_at_node`, ack/nack helpers; +level-gating Context tests |
| `v3-pipeline-metrics-design.md` | Full design revision + work items list |

## Remaining work items (W5–W13)

| # | Item | Status |
|---|------|--------|
| W5 | Forward-path byte counting in `received_at_node` + receiver handlers | **Done** |
| W6 | Level-gate receiver `subscribe_to` (auto-subscribe, conditional timestamp) | **Done** |
| W7 | Restructure metric sets into level-gated groups (request/bytes/duration) | **Done** |
| W8 | Level-gate consumer recording in `notify_ack`/`notify_nack` | **Done** |
| W9 | Level-gate producer recording in `record_produced_for_control_msg` | **Done** |
| W10 | Plumb `MetricLevel` to `NodeTaskContext` + `current_metric_level()` | **Done** |
| W11 | Default ack/nack handler for auto-subscribed nodes (may be no-op) | **Done** (no-op) |
| W12 | Engine-wide `MetricLevel` configuration | **Done** |
| W13 | Tests for all four metric levels | **Done** |

All work items W1–W13 are complete.

## Code changes completed (W5–W13)

All changes compile clean and pass all workspace tests.

### W7. Restructured metric sets — `crates/engine/src/component_metrics.rs`

Replaced 5 outcome-keyed metric sets (ConsumedSuccess/Failure/Refused,
ProducedSuccess/Refused) with level-gated groups:
- **Basic+**: `ConsumedRequestMetrics` (success/failure/refused counters),
  `ProducedRequestMetrics` (success/refused counters).
- **Normal+**: `ConsumedBytesMetrics` (forward-path byte counter),
  `ProducedBytesMetrics` (forward-path byte counter).
- **Detailed**: `ConsumedDurationMetrics` (per-outcome duration counters;
  placeholder for histograms once telemetry crate supports them).

`ComponentMetricsState` now holds `Option<MetricSet<T>>` for each
group, populated based on metric level. Recording methods internally
skip absent sets. API simplified: `record_consumed_success(duration_ns)`
(no `bytes` param), `record_produced_success()` (no `bytes` param),
`record_consumed_bytes(bytes)` and `record_produced_bytes(bytes)` added
for forward-path counting.

### W10. `MetricLevel` in `NodeTaskContext` + `current_metric_level()`

- Added `metric_level: MetricLevel` field to `NodeTaskContext`. 
- `NodeTaskContext::new()` takes a `metric_level` parameter.
- Added `pub fn current_metric_level() -> MetricLevel` reading from
  `NODE_TASK_CONTEXT` task-local (returns `None` if no context).
- `register_component_metrics(&self, level: MetricLevel)` now takes
  a level parameter and only registers metric sets appropriate for
  that level.

### W11. Default ack/nack handler (no-op)

Confirmed that existing `notify_ack`/`notify_nack` implementations
already record metrics and propagate via `route_ack`/`route_nack`.
No distinction between auto-subscribed and explicitly subscribed
frames from the handler's perspective — no code changes needed.

### W12. Engine-wide `MetricLevel` configuration

- Moved `MetricLevel` enum to config crate (`policy.rs`) with
  `Serialize`/`Deserialize`/`JsonSchema` derives; engine crate
  re-exports via `pub use otap_df_config::policy::MetricLevel`.
- Added `component_metrics: MetricLevel` to `TelemetryPolicy`
  (defaults to `None`).
- `PipelineFactory::build()` extracts `component_metric_level` from
  `telemetry_policy` and passes it to `build_node_wrapper`.
- `build_node_wrapper` accepts `metric_level` param and uses it for
  `register_component_metrics(metric_level)`.
- `RuntimePipeline::run_forever()` extracts `metric_level` from
  `telemetry_policy.component_metrics` and passes it to all
  `NodeTaskContext::new()` calls.
- `processor.rs` run loops use `current_metric_level()` for
  `received_at_node`.
- All `TODO(W12)` comments removed.

### W13. Tests for all four metric levels

Added tests in three locations:

**Config crate** (`policy.rs`):
- `metric_level_ordering` — verifies `None < Basic < Normal < Detailed`.
- `metric_level_serde_roundtrip` — JSON round-trip for all variants.
- `telemetry_policy_with_component_metrics` — YAML deserialization.
- `telemetry_policy_defaults_component_metrics_to_none` — default check.

**Engine crate** (`component_metrics.rs`):
- `none_level_no_metric_sets` — recording is no-op, no panic.
- `basic_level_outcome_counters` — request counters present, no bytes/duration.
- `normal_level_bytes_counters` — + byte counters present.
- `detailed_level_duration_counters` — + duration counters present.

**Otap crate** (`pdata.rs`):
- `push_entry_frame_none_no_interests` — no ACKS/NACKS at None.
- `push_entry_frame_basic_auto_subscribes` — ACKS|NACKS at Basic.
- `push_entry_frame_normal_same_as_basic` — same interests as Basic.
- `push_entry_frame_detailed_stamps_time` — ACKS|NACKS + non-zero time_ns.
- `push_entry_frame_inherits_return_data` — RETURN_DATA inheritance.
- `subscribe_to_merges_into_entry_frame` — same-node merge preserves time_ns.
- `none_level_ack_nack_not_routable` — next_ack skips None-level frame.
- `basic_level_ack_routable` — next_ack finds Basic-level frame.

### W5. Forward-path byte counting in `received_at_node`

`OtapPdata::received_at_node()` now records `consumed.bytes` at
`Normal+` via `current_component_metrics().record_consumed_bytes()`,
reading `payload.num_bytes()` from the payload.

### W6. Level-gated receiver `subscribe_to`

Both receiver `ProducerEffectHandlerExtension` impls (local + shared)
now read `current_metric_level()` and:
- At `Basic+`: merge `ACKS | NACKS` into the provided interests.
- At `Detailed`: stamp `time_ns` via `stamp_top_time(nanos_since_epoch())`.
- At `None`: pass interests through unchanged, no timestamp.

### W8. Level-gated consumer recording in `notify_ack`/`notify_nack`

Extracted `record_consumer_ack_metrics(context)` and
`record_consumer_nack_metrics(context, permanent)` helper functions
that gate on `current_metric_level()`:
- At `None`: no metric recording (zero overhead).
- At `Basic+`: peek top frame, record outcome count.
- At `Detailed`: additionally compute duration from `time_ns`.

All 4 consumer impls (local/shared × processor/exporter) simplified
to single-line calls to these helpers.

### W9. Level-gated producer recording

`record_produced_for_control_msg()` now gates on
`current_metric_level() >= Basic` before accessing the component
metrics handle.

### Files touched (W5–W10)

| File | Summary |
|------|---------|
| `crates/engine/src/component_metrics.rs` | 5 metric sets → level-gated groups; new `record_consumed_bytes`/`record_produced_bytes`; producer recording level-gated |
| `crates/engine/src/entity_context.rs` | +`MetricLevel` in `NodeTaskContext`; +`current_metric_level()`; `register_component_metrics(level)` |
| `crates/engine/src/lib.rs` | `register_component_metrics(MetricLevel::None)` |
| `crates/engine/src/runtime_pipeline.rs` | `NodeTaskContext::new(..., MetricLevel::None)` at all 6 call sites |
| `crates/otap/src/pdata.rs` | `received_at_node` + byte counting; receiver `subscribe_to` level-gated; `notify_ack`/`notify_nack` level-gated via helpers |
