# Entry Frame Stamping Refactor

## Overview

This document describes the refactoring of entry frame stamping for pipeline metrics instrumentation. The goal was to automatically stamp entry frames when nodes (processors and exporters) receive PData from their input channels, without requiring manual calls or adding trait bounds to the generic `PData` type.

## Problem Statement

### Original Issue
Processors and exporters needed to call `stamp_pdata_received()` manually to capture entry timestamps for duration metrics. This was error-prone and required each component implementation to remember to make this call.

### Design Constraint
The engine crate is generic over `PData` - it doesn't know about `OtapPdata` or its `Context.push_entry_frame()` method. Adding a trait bound like `PData: HasEntryFrame` would propagate through the entire codebase.

## Solution: Callback Embedded in MessageChannel

### Approach
Move the `on_pdata_received` callback directly into `MessageChannel`, so it's called automatically inside `recv()` for every PData message. This eliminates the need for manual stamping calls.

### Key Insight
The `MessageChannel` already has access to the node's context (node_id, interests) at construction time. By storing the callback there, we can invoke it transparently on every receive.

## Implementation Details

### 1. MessageChannel Changes

Both local (`message.rs`) and shared (`shared/exporter.rs`) `MessageChannel` structs now include:

```rust
pub struct MessageChannel<PData> {
    control_rx: Option<Receiver<NodeControlMsg<PData>>>,
    pdata_rx: Option<Receiver<PData>>,
    shutting_down_deadline: Option<Instant>,
    pending_shutdown: Option<NodeControlMsg<PData>>,
    // NEW FIELDS:
    on_pdata_received: fn(&mut PData, usize, Interests),
    node_id: usize,
    interests: Interests,
}
```

The constructor now requires these additional parameters:

```rust
pub fn new(
    control_rx: Receiver<NodeControlMsg<PData>>,
    pdata_rx: Receiver<PData>,
    on_pdata_received: fn(&mut PData, usize, Interests),
    node_id: usize,
    interests: Interests,
) -> Self
```

### 2. Automatic Callback Invocation

Inside `recv()`, when a PData message is received, the callback is automatically invoked:

```rust
Ok(mut pdata) => {
    (self.on_pdata_received)(&mut pdata, self.node_id, self.interests);
    return Ok(Message::PData(pdata));
}
```

This happens in both branches of `recv()`:
- Normal mode (control messages prioritized)
- Draining mode (after shutdown signal, draining remaining pdata)

### 3. Removed `recv_with()`

The `recv_with()` method was removed since the callback is now always invoked inside `recv()`. This simplifies the API - callers just use `recv()` and stamping happens automatically.

### 4. ProcessorWrapper Changes

`ProcessorWrapper::prepare_runtime()` now accepts the callback and interests:

```rust
pub async fn prepare_runtime(
    self,
    metrics_reporter: MetricsReporter,
    on_pdata_received: fn(&mut PData, usize, Interests),
    node_interests: Interests,
) -> Result<ProcessorWrapperRuntime<PData>, Error>
```

The callback and interests are passed to `MessageChannel::new()` during runtime preparation.

`ProcessorWrapper::start()` was simplified - it no longer needs to manually invoke the callback in the message loop since `recv()` handles it.

### 5. ExporterWrapper Changes

`ExporterWrapper::start()` now accepts the callback parameter:

```rust
pub async fn start(
    self,
    pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<PData>,
    metrics_reporter: MetricsReporter,
    node_interests: Interests,
    input_channel_receiver_metrics: Option<InputChannelReceiverMetrics>,
    on_pdata_received: fn(&mut PData, usize, Interests),
) -> Result<TerminalState, Error>
```

Both local and shared exporter paths now construct `MessageChannel` with the callback.

### 6. RuntimePipeline Changes

`RuntimePipeline` passes the `on_pdata_received` callback to both processor and exporter start calls. The callback flows from:

```
Controller::with_on_pdata_received(hook)
    → RuntimePipeline.on_pdata_received
    → ProcessorWrapper::start(on_pdata_received)
    → ExporterWrapper::start(on_pdata_received)
    → MessageChannel::new(on_pdata_received, node_id, interests)
```

### 7. Controller API

The `Controller` struct stores the callback and provides a builder method:

```rust
pub struct Controller<PData> {
    pipeline_factory: &'static PipelineFactory<PData>,
    on_pdata_received: fn(&mut PData, usize, Interests),
}

impl Controller<PData> {
    pub const fn new(pipeline_factory: &'static PipelineFactory<PData>) -> Self {
        Self {
            pipeline_factory,
            on_pdata_received: |_, _, _| {},  // no-op default
        }
    }

    pub const fn with_on_pdata_received(
        mut self,
        hook: fn(&mut PData, usize, Interests),
    ) -> Self {
        self.on_pdata_received = hook;
        self
    }
}
```

### 8. OtapPdata Integration

In `crates/otap/src/pdata.rs`, the stamping function is defined:

```rust
pub fn stamp_pdata_entry_frame(pdata: &mut OtapPdata, node_id: usize, interests: Interests) {
    pdata.context.push_entry_frame(node_id, interests);
}
```

And wired up in `main.rs`:

```rust
let controller = Controller::new(&OTAP_PIPELINE_FACTORY)
    .with_on_pdata_received(stamp_pdata_entry_frame);
```

## Files Modified

| File | Changes |
|------|---------|
| `crates/engine/src/message.rs` | Added callback fields to `MessageChannel`, updated `new()`, modified `recv()` to call callback, removed `recv_with()` |
| `crates/engine/src/shared/exporter.rs` | Same changes for shared `MessageChannel` |
| `crates/engine/src/processor.rs` | Updated `prepare_runtime()` and `start()` signatures, simplified message loop |
| `crates/engine/src/exporter.rs` | Updated `start()` to accept callback, pass to `MessageChannel::new()` |
| `crates/engine/src/runtime_pipeline.rs` | Pass callback to both processor and exporter start calls |
| `crates/controller/src/lib.rs` | Store callback in `Controller`, pass through spawn/run methods |
| `crates/otap/src/pdata.rs` | Added `stamp_pdata_entry_frame()` function |
| `src/main.rs` | Wire up `stamp_pdata_entry_frame` via `with_on_pdata_received()` |
| `crates/engine/src/testing/processor.rs` | Update test to pass no-op callback |
| `crates/engine/src/testing/exporter.rs` | Update test to pass no-op callback |
| `crates/otap/src/parquet_exporter.rs` | Update 4 test call sites |
| `crates/otap/src/otap_exporter.rs` | Update test call site |
| `crates/otap/src/otlp_exporter.rs` | Update test call site |

## Why Function Pointer Instead of Trait?

The function pointer approach was chosen because:

1. **No trait bounds on PData** - Adding a trait like `trait OnReceive { fn on_receive(&mut self); }` would require `PData: OnReceive` bounds throughout the engine.

2. **Zero allocation** - Function pointers have no heap allocation (unlike `Box<dyn Fn>`).

3. **Inlines well** - The compiler can optimize function pointer calls when the target is known at compile time.

4. **Simple default** - A no-op `|_, _, _| {}` provides a cheap default that does nothing.

## Comparison with subscribe_to Pattern

The `subscribe_to` pattern uses extension traits:

```rust
// Engine defines trait
pub trait ProducerEffectHandlerExtension<PData> {
    fn subscribe_to(&self, int: Interests, ctx: UserCallData, data: &mut PData);
}

// Otap implements it
impl ProducerEffectHandlerExtension<OtapPdata> 
    for EffectHandler<OtapPdata> { ... }
```

This works because the **call site is in component code** (otap crate), which knows the concrete types.

Entry frame stamping is different - the **call site is in engine infrastructure** (`recv()` inside `message.rs`), which only knows generic `PData`. Hence the function pointer approach.

## Thread-Local Elimination Status

As part of this work, we confirmed that thread-local (task-local) access for metrics has been eliminated:

- `current_metric_level()` - **Removed** (no longer exists)
- `current_input_channel_receiver_metrics()` - **Removed** (no longer exists)
- `Interests` and `InputChannelReceiverMetrics` are now passed directly to `start()` methods

The remaining task-local (`NODE_TASK_CONTEXT`) is used only for:
- Entity keys (for metrics registration, not hot-path)
- Telemetry handle (for log context enrichment)

## Test Results

All tests pass:
- `otap-df-engine`: 72 tests passed
- `otap-df-otap`: 507 tests passed (1 ignored)

## Future Considerations

1. **Move callback into PipelineFactory** - The callback could be part of the factory struct/trait rather than a separate `with_on_pdata_received()` builder, since the factory already knows the PData type.

2. **Receiver timestamp handling** - Receivers don't use this callback since they create new PData. They handle timestamps via `subscribe_to()` which calls `stamp_top_time()` when `ENTRY_TIMESTAMP` interest is set.