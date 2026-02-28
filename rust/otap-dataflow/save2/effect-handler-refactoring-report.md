# Effect Handler Refactoring Report

This document summarizes the series of refactoring changes made to the
`otap-dataflow` pipeline engine to eliminate task-local dispatch in the
metrics hot path, precompute runtime values at init time, and fix
`Send`-safety constraints.

---

## 1. Remove Bytes Metrics from V3 Pipeline

Removed producer/consumer bytes accounting (`produced.bytes`,
`consumed.bytes`) that was no longer needed in the V3 pipeline design.
This simplified the metrics surface and removed dead code paths from
effect handlers.

## 2. Refactor `ReceivedAtNode` Trait

Introduced the `ReceivedAtNode` trait in `crates/engine/src/lib.rs` to
decouple the engine from the specific `OtapPdata` context-stack
implementation:

```rust
pub trait ReceivedAtNode {
    fn received_at_node(&mut self, node_id: usize, node_interests: Interests);
}
```

The trait receives precomputed `Interests` rather than a raw
`MetricLevel`, so the implementation (`OtapPdata::received_at_node`)
simply pushes an entry frame without needing to consult task-local
state:

```rust
impl ReceivedAtNode for OtapPdata {
    fn received_at_node(&mut self, node_id: usize, node_interests: Interests) {
        self.context.push_entry_frame(node_id, node_interests);
    }
}
```

Two new `Interests` flags were added to support this:

| Flag               | Bit | Purpose                                           |
|--------------------|-----|---------------------------------------------------|
| `ENTRY_TIMESTAMP`  | 3   | Entry frame captures wall-clock timestamp          |
| `PIPELINE_METRICS` | 4   | Entry frame auto-subscribes to ACKS/NACKS          |

## 3. Migrate `consumed.duration_ns` from Component Metrics to Channel Metrics

Moved the `consumed.duration_ns` metric from per-component
(`ComponentMetrics`) to per-channel (`ChannelReceiverMetrics`) so that
duration is attributed to the specific input channel rather than the
component as a whole.

### New types in `channel_metrics.rs`

- `ChannelReceiverMetricsState` — holds the `ChannelReceiverMetrics`
  metric set and exposes `record_consumed_duration(duration_ns)`.
- `LocalChannelReceiverMetricsHandle` = `Rc<RefCell<ChannelReceiverMetricsState>>`
- `SharedChannelReceiverMetricsHandle` = `Arc<Mutex<ChannelReceiverMetricsState>>`
- `InputChannelReceiverMetrics` — an enum that wraps either a
  `Local(LocalChannelReceiverMetricsHandle)` or
  `Shared(SharedChannelReceiverMetricsHandle)`, providing a unified
  `record_consumed_duration()` method.

## 4. Replace Task-Local Dispatch with `EffectHandlerCore` Fields

The central goal: runtime metric decisions that previously required
`current_metric_level()` (a task-local read on every message) are now
precomputed once during `start()` and stored in `EffectHandlerCore`.

### 4a. `node_interests` field on `EffectHandlerCore`

`EffectHandlerCore` (in `effect_handler.rs`) gained a new field:

```rust
pub(crate) struct EffectHandlerCore<PData> {
    // ... existing fields ...
    /// Precomputed node interests derived from the engine metric level.
    node_interests: Interests,
}
```

With accessor and setter:

```rust
pub fn set_node_interests(&mut self, interests: Interests);
pub fn node_interests(&self) -> Interests;
```

### 4b. `Interests::from_metric_level` helper

A conversion helper on `Interests` (in `lib.rs`) maps the config-level
enum to the runtime bitflags once at init time:

```rust
impl Interests {
    pub fn from_metric_level(level: MetricLevel) -> Self {
        match level {
            MetricLevel::None => Self::empty(),
            MetricLevel::Basic | MetricLevel::Normal => Self::PIPELINE_METRICS,
            MetricLevel::Detailed => Self::PIPELINE_METRICS | Self::ENTRY_TIMESTAMP,
        }
    }
}
```

### 4c. Channel receiver metrics moved to outer handlers

Because `LocalChannelReceiverMetricsHandle` contains
`Rc<RefCell<...>>` (which is `!Send`), it cannot live inside
`EffectHandlerCore` (which must be `Send` for shared handlers). The
solution: each outer handler stores its own channel metrics handle:

| Handler variant        | Field type                            | Access pattern  |
|------------------------|---------------------------------------|-----------------|
| `local::processor`     | `Option<LocalChannelReceiverMetricsHandle>`  | `try_borrow_mut` |
| `local::exporter`      | `Option<LocalChannelReceiverMetricsHandle>`  | `try_borrow_mut` |
| `shared::processor`    | `Option<SharedChannelReceiverMetricsHandle>` | `try_lock`       |
| `shared::exporter`     | `Option<SharedChannelReceiverMetricsHandle>` | `try_lock`       |

Each provides:
- `pub(crate) fn set_input_channel_receiver_metrics(&mut self, handle: ...)` — called once during `start()`
- `pub fn record_consumed_duration(&self, duration_ns: u64)` — called on the ack/nack hot path

### 4d. Receiver handler `core` made `pub(crate)`

The `core: EffectHandlerCore<PData>` field on `local::receiver::EffectHandler`
and `shared::receiver::EffectHandler` was changed from private to
`pub(crate)`, matching the processor and exporter handlers. This allows
`receiver.rs` to call `effect_handler.core.set_node_interests(...)` directly
without requiring delegate methods.

### 4e. Init-time wiring in `start()` methods

Each wrapper's `start()` method (`processor.rs`, `exporter.rs`,
`receiver.rs`) now performs a one-time setup:

```rust
let node_interests = Interests::from_metric_level(current_metric_level());
effect_handler.core.set_node_interests(node_interests);
// For processors/exporters only:
if let InputChannelReceiverMetrics::Local(handle) = input_channel_receiver_metrics {
    effect_handler.set_input_channel_receiver_metrics(handle);
}
```

`current_metric_level()` is read exactly once per node during startup,
not on every message.

## 5. Update `pdata.rs` to Use `Interests` Instead of `MetricLevel`

All runtime metric-gating in `crates/otap/src/pdata.rs` was migrated
from `MetricLevel` comparisons to `Interests` flag checks:

### Producer impls (4 total: local/shared × processor/receiver)

Before:
```rust
let level = current_metric_level();
if level >= MetricLevel::Basic { ... }
if level >= MetricLevel::Detailed { ... }
```

After:
```rust
let interests = self.node_interests();
if interests.contains(Interests::PIPELINE_METRICS) { ... }
if interests.contains(Interests::ENTRY_TIMESTAMP) { ... }
```

No task-local read — `node_interests()` is a field access on `self`.

### Consumer free functions

`record_consumer_ack_metrics` and `record_consumer_nack_metrics` changed
signature from `level: MetricLevel` to `interests: Interests` and use
the same flag checks internally.

### Consumer impls (4 total: local/shared × processor/exporter)

All 8 call sites (notify_ack + notify_nack per impl) changed from
`self.metric_level()` to `self.node_interests()`.

### Import cleanup

- Removed `MetricLevel` from `pdata.rs` imports (no longer referenced).
- Removed `current_metric_level` usage (was never imported but was
  called — would have been a compile error had it not been replaced).

---

## Files Modified

### `crates/engine/src/`

| File | Changes |
|------|---------|
| `lib.rs` | Added `ENTRY_TIMESTAMP` and `PIPELINE_METRICS` to `Interests` bitflags; added `Interests::from_metric_level()`; defined `ReceivedAtNode` trait |
| `effect_handler.rs` | Replaced `metric_level: MetricLevel` with `node_interests: Interests`; removed `input_channel_receiver_metrics` from core |
| `channel_metrics.rs` | Added `ChannelReceiverMetricsState`, handle type aliases, `InputChannelReceiverMetrics` enum with `record_consumed_duration` |
| `local/processor.rs` | Added `input_channel_receiver_metrics` field, `node_interests()` delegate, `record_consumed_duration()`, setter |
| `local/exporter.rs` | Same pattern as local/processor |
| `shared/processor.rs` | Same pattern with `Arc<Mutex<...>>` and `try_lock` |
| `shared/exporter.rs` | Same pattern as shared/processor |
| `local/receiver.rs` | Made `core` field `pub(crate)`, added `node_interests()` getter |
| `shared/receiver.rs` | Made `core` field `pub(crate)`, added `node_interests()` getter |
| `processor.rs` | Init-time wiring: `from_metric_level`, `set_node_interests`, `set_input_channel_receiver_metrics` |
| `exporter.rs` | Same init-time wiring pattern |
| `receiver.rs` | Init-time wiring: `from_metric_level`, `core.set_node_interests` |

### `crates/otap/src/`

| File | Changes |
|------|---------|
| `pdata.rs` | Replaced all `MetricLevel` runtime checks with `Interests` flag checks across 4 producer impls, 4 consumer impls, and 2 free functions; removed `MetricLevel` import; implemented `ReceivedAtNode` for `OtapPdata` |

---

## Design Principles

1. **Read config once, use precomputed values** — `MetricLevel` is
   converted to `Interests` bitflags exactly once per node at startup.
   All hot-path checks use the precomputed bitflags.

2. **No task-local reads on the hot path** — `current_metric_level()`
   is only called during `start()`. Runtime decisions use field access
   (`self.node_interests()`).

3. **`Send`-safety by construction** — `!Send` handles
   (`Rc<RefCell<...>>`) live only in local handlers; `Send` handles
   (`Arc<Mutex<...>>`) live only in shared handlers. The shared
   `EffectHandlerCore` contains neither.

4. **Consistent visibility** — All handler `core` fields are
   `pub(crate)`, enabling direct access from wrapper `start()` methods
   without redundant delegate methods.
