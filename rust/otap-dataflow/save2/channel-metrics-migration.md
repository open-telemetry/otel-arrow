# Channel Metrics Migration — Project Status

## Goal

Migrate all per-node request-outcome metrics (produced/consumed
success, failure, refused, duration) from the old `component_metrics`
task-local infrastructure to **channel-level metrics** owned by the
effect handlers and channel objects themselves. This eliminates
task-local indirection, makes metric ownership explicit, and aligns
produced metrics with the output channel they belong to via the
`output_port_index` field on `CallData`.

---

## Completed Phases

### Phase 1 — Remove bytes metrics
Removed unused `produced.bytes` / `consumed.bytes` counters from
request-outcome metric sets.

### Phase 2 — Refactor `ReceivedAtNode` trait
Reworked the trait so nodes stamp arrival info through the effect
handler rather than a standalone function, establishing the pattern for
later phases.

### Phase 3 — Migrate `consumed.duration_ns`
Moved consumed-duration recording from task-local component metrics
into `ChannelReceiverMetrics` state held directly on processor/exporter
effect handlers (`input_channel_receiver_metrics` field).

### Phase 4 — Replace task-local dispatch with `EffectHandlerCore` fields
Moved per-node interests (`Interests`) and the pipeline control-message
sender out of task-local context and into `EffectHandlerCore`, so they
are available on the effect handler without a `try_with` lookup.

### Phase 5 — Preprocess `MetricLevel` → `Interests`
Converted the raw `MetricLevel` enum into a precomputed `Interests`
bitflags set, stored on `EffectHandlerCore`, so hot-path code checks a
single bitflag instead of matching enum variants.

### Phase 6a — Add `output_port_index` to `CallData`
Added a `u16 output_port_index` field to `CallData`, a `stamp_output_port_index`
method on `Context`, and stamped it in all 16 `MessageSource` extension
methods across the 4 producer effect handler types. This lets ack/nack
messages carry the identity of the output channel they were produced on.

### Phase 6b — Unify request-outcome metric sets
Consolidated `ConsumedRequestMetrics` and `ProducedRequestMetrics` into
a single `request_outcome_metrics!` macro, eliminating duplicate struct
definitions.

### Phase 6c — Consumed outcomes → channel metrics
Moved consumed success/failure/refused recording from the deleted
`component_metrics` module into `ChannelReceiverMetricsState` on the
input channel. Removed `component_metrics.rs`, its registrations in
`entity_context.rs`, `lib.rs`, `runtime_pipeline.rs`, and
`pipeline_ctrl.rs`. The old `ComponentMetrics` type is gone.

### Phase 6d — Produced outcomes → channel metrics
Wired up produced success/failure/refused recording on the **sender
side** of each output channel:

| Path | Mechanism |
|---|---|
| **Processors** (local & shared) | Automatic — `ConsumerEffectHandlerExtension::notify_ack/notify_nack` in `pdata.rs` now calls `record_produced_success/failure/refused(port_index)` using `calldata.output_port_index` from the context frame. |
| **Receivers** (local & shared) | Automatic — `ControlChannel::recv()` inspects each `Ack`/`Nack` and records produced outcome on the matching output channel sender metrics. |

Implementation details:
- `LocalSender`, `SharedSender`, and `Sender<T>` expose `sender_metrics_handle()`.
- All 4 producer effect handlers hold `Vec<Option<...>>` of sender
  metrics indexed by port index, populated at construction.
- Shared paths use `SharedChannelSenderMetricsHandle` directly (not the
  `OutputChannelSenderMetrics` enum) to preserve `Send` bounds.
- `ControlChannel` receives the metrics vec from the effect handler
  during receiver startup in `receiver.rs`.

All 595 tests pass, no warnings.

---

## Remaining Work

### Item 1 — Rename `TelemetryPolicy.component_metrics`

**File:** `crates/config/src/policy.rs`

The field `TelemetryPolicy.component_metrics: MetricLevel` still uses
the old name. It now controls the **channel-level** detail (whether
outcome counters and entry timestamps are enabled), not a deleted
"component metrics" subsystem. Rename to something like
`channel_detail` or `metric_level` and update:

- The struct field + serde attribute in `policy.rs`
- `Default` impl and tests in `policy.rs`
- All readers: `runtime_pipeline.rs` (`let metric_level = telemetry_policy.component_metrics`)
- Config YAML files / examples (`scratch.yaml`, etc.)

Low risk, pure rename.

### Item 2 — Migrate `current_metric_level()` task-local

**Defined in:** `crates/engine/src/entity_context.rs` (line 145)  
**Callers:**
- `receiver.rs` — 2 call sites (local & shared startup)
- `processor.rs` — 1 call site
- `exporter.rs` — 2 call sites (local & shared startup)

Each of these reads `current_metric_level()` once at startup to
compute `Interests::from_metric_level(...)` and store it on
`EffectHandlerCore`. The metric level is already available on the
`NodeTaskContext` which is set by the pipeline runtime before node
startup. The fix is to **pass `MetricLevel` (or the computed
`Interests`) as a parameter** to the node startup functions instead of
reading the task-local. This removes the `metric_level` field from
`NodeTaskContext` and the `current_metric_level()` accessor.

Medium effort — requires threading a parameter through the startup
call chain in `runtime_pipeline.rs` → `receiver.rs` / `processor.rs` /
`exporter.rs`.

### Item 3 — Migrate `current_input_channel_receiver_metrics()` task-local

**Defined in:** `crates/engine/src/entity_context.rs` (line 154)  
**Callers:**
- `processor.rs` — 2 call sites (local & shared)
- `exporter.rs` — 2 call sites (local & shared)

Each caller reads the task-local at startup to extract the input
channel's receiver metrics handle and store it on the effect handler.
As with item 2, the handle originates from `NodeTelemetryHandle` in
`NodeTaskContext`. The fix is to **pass the
`InputChannelReceiverMetrics` handle as a parameter** to the
processor/exporter startup code instead of reading it from a
task-local. This removes the `input_channel_receiver_metrics` field
from `NodeTaskContext` and the accessor function.

Medium effort — similar shape to item 2.

### Item 4 — Clean up remaining `NodeTaskContext` fields (optional)

After items 2 and 3, `NodeTaskContext` will still hold:
- `telemetry_handle` — used for periodic telemetry collection
- `entity_key` / `input_channel_key` / `output_channel_keys` — used
  for entity-key lookups during telemetry reporting

These are **read-only after startup** and are plausibly fine as
task-locals since they are only accessed by the telemetry subsystem
(not the hot data path). Migrating them is lower priority but would
let us remove the `NODE_TASK_CONTEXT` task-local entirely.

---

## Architecture After All Phases

```
┌────────────┐    PData channel    ┌────────────┐    PData channel    ┌────────────┐
│  Receiver   │──────────────────▶│  Processor  │──────────────────▶│  Exporter   │
│             │                    │             │                    │             │
│ EH fields:  │                    │ EH fields:  │                    │ EH fields:  │
│  output_    │◀── ack/nack ──────│  output_    │◀── ack/nack ──────│  input_     │
│  channel_   │  (ControlChannel   │  channel_   │  (notify_ack/     │  channel_   │
│  sender_    │   .recv() records  │  sender_    │   notify_nack     │  receiver_  │
│  metrics[]  │   produced         │  sender_    │   records both    │  metrics    │
│             │   outcomes)        │  metrics[]  │   consumed AND    │             │
│             │                    │  input_     │   produced        │             │
│             │                    │  channel_   │   outcomes)       │             │
│             │                    │  receiver_  │                    │             │
│             │                    │  metrics    │                    │             │
└────────────┘                    └────────────┘                    └────────────┘
```

- **Consumed outcomes** (success/failure/refused + duration) are
  recorded on `ChannelReceiverMetricsState` held by the consumer's
  effect handler.
- **Produced outcomes** (success/failure/refused) are recorded on
  `ChannelSenderMetricsState` held by the producer's effect handler,
  indexed by `output_port_index`.
- No task-local access on the data path for metrics recording.
