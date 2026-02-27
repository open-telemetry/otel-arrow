# V2 Pipeline Component Metrics Design

## Problems with V1

1. **Bytes/items are stale on the return path**: When an ack/nack arrives at `notify_ack`/`notify_nack`, the payload may have been consumed via `take_payload()` (when `RETURN_DATA` is not set). Reading `num_items()` / `num_bytes()` from the ack/nack message yields 0, not the actual counts.

2. **Outcome unknown on forward path**: V1 records `produced` metrics immediately when `send_message_subscribed` succeeds. But the *outcome* (success/failure/refused) is only known when the ack/nack returns from downstream. Similarly, the pipeline controller tries to read counts from the ack/nack data to record producer-side outcome metrics, but the data may be empty.

3. **Redundant computation**: Every pipeline stage computes `num_items()` and `num_bytes()` even when the payload hasn't been modified since the previous node.

4. **Thread-local coupling**: V1 uses `current_component_metrics()` which reads from a task-local, coupling metric recording to the executing task context.

## V2 Key Insight

**Store `num_items` and `num_bytes` in the context Frame at subscribe time** (forward path). These counts travel through the pipeline in the context stack alongside the data. On the return path (ack/nack), the Frame's counts are extracted — no need to read from the now-possibly-empty payload.

Both producer and consumer metrics are recorded **only when the outcome is known** (ack or nack), using the counts captured in the Frame.

## Design

### 1. Frame carries telemetry counts

```rust
pub struct Frame {
    pub interests: Interests,
    pub calldata: CallData,
    pub node_id: usize,
    pub num_items: u64,           // NEW
    pub num_bytes: u64,   // NEW
}
```

### 2. AckMsg/NackMsg carry counts from the popped Frame

```rust
pub struct AckMsg<PData> {
    pub accepted: Box<PData>,
    pub calldata: CallData,
    pub num_items: u64,           // NEW: populated from Frame
    pub num_bytes: u64,   // NEW: populated from Frame
}

pub struct NackMsg<PData> {
    pub reason: String,
    pub calldata: CallData,
    pub refused: Box<PData>,
    pub permanent: bool,
    pub num_items: u64,           // NEW: populated from Frame
    pub num_bytes: u64,   // NEW: populated from Frame
}
```

### 3. Forward path: subscribe captures counts

`subscribe_to` and `subscribe_or_update` accept `num_items` and `num_bytes`:

```rust
impl Context {
    pub(crate) fn subscribe_to(
        &mut self,
        interests: Interests,
        calldata: CallData,
        node_id: usize,
        num_items: u64,
        num_bytes: u64,
    );

    pub(crate) fn subscribe_or_update(
        &mut self,
        interests: Interests,
        calldata: CallData,
        node_id: usize,
        num_items: u64,
        num_bytes: u64,
    );
}
```

In `send_message_subscribed`, counts are computed once before the subscribe+send. No forward-path metrics are recorded — that happens on the return path when the outcome is known.

### 4. Return path: counts extracted from Frame

`Context::next_ack` and `Context::next_nack` populate the ack/nack message's `num_items`/`num_bytes` from the popped Frame:

```rust
pub fn next_ack(mut ack: AckMsg<OtapPdata>) -> Option<(usize, AckMsg<OtapPdata>)> {
    ack.accepted.context.next_with_interest(Interests::ACKS).map(|frame| {
        ack.num_items = frame.num_items;
        ack.num_bytes = frame.num_bytes;
        ack.calldata = frame.calldata;
        if (frame.interests & Interests::RETURN_DATA).is_empty() {
            let _drop = ack.accepted.take_payload();
        }
        (frame.node_id, ack)
    })
}
```

### 5. Where metrics are recorded

**Consumer side** (in `notify_ack`/`notify_nack` extension traits, `crates/otap`):
- `notify_ack` → `consumed.success` using `ack.num_items/num_bytes`
- `notify_nack(permanent)` → `consumed.refused`
- `notify_nack(!permanent)` → `consumed.failure`

These record metrics for the node that is *consuming* data (the one calling notify_ack/nack).

**Producer side** (in `pipeline_ctrl.rs`, engine crate):
- `DeliverAck` → `produced.success` using `ack.num_items/num_bytes`
- `DeliverNack(permanent)` → `produced.refused`
- `DeliverNack(!permanent)` → `produced.failure`

These record metrics for the *producer* node (the one that originally subscribed and sent the data).

### 6. Simplified ComponentMetricsState

Remove the forward-path-only `ProducedMetrics` (no outcome). Keep only outcome-based counters:

| Metric | When recorded | By whom |
|--------|--------------|---------|
| `produced.success.{items,bytes}` | Ack delivered to producer | pipeline_ctrl |
| `produced.failure.{items,bytes}` | Non-permanent Nack delivered | pipeline_ctrl |
| `produced.refused.{items,bytes}` | Permanent Nack delivered | pipeline_ctrl |
| `consumed.success.{items,bytes}` | Consumer calls notify_ack | extension trait |
| `consumed.failure.{items,bytes}` | Consumer calls notify_nack(!perm) | extension trait |
| `consumed.refused.{items,bytes}` | Consumer calls notify_nack(perm) | extension trait |

### 7. Remove Instrumented trait bound from engine

The `Instrumented` trait on PData was only needed so the pipeline controller could read counts from the ack/nack payload. With counts now in the ack/nack message directly (from the Frame), the engine no longer needs to call `num_items()`/`num_bytes()` on PData. The `Instrumented` bound can be removed from `PipelineCtrlMsgManager`, `PipelineFactory`, `Controller`, etc.

The `Instrumented` trait itself stays in the engine for use by the otap extension traits when capturing counts at subscribe time.

## Future: Count Propagation Optimization

For V2, counts are always computed at subscribe time (each call to `send_message_subscribed` reads from the data). A future optimization:

- Track whether `take_payload()` or `into_parts()` was called since the last Frame
- If not, inherit counts from the previous Frame
- This avoids redundant computation for pass-through processors

## Data Flow

```
Forward path (subscribe + send):
    Receiver                     Processor                    Exporter
    ─────────                    ─────────                    ────────
    compute num_items/bytes      compute num_items/bytes      (receives data)
    subscribe_to(frame)          subscribe_or_update(frame)   (processes data)
    send_message(data)──────────→send_message(data)──────────→notify_ack()/notify_nack()

Return path (ack/nack):
    Exporter calls notify_ack(ack)
      → record consumed.success for Exporter (using ack.num_items from popped Processor frame)
      → Context::next_ack pops Processor's frame, sets ack.{num_items,num_bytes}
      → route_ack → DeliverAck{node_id=Processor}
      → pipeline_ctrl records produced.success for Processor
      → Processor receives Ack, calls notify_ack(ack)
      → record consumed.success for Processor (using ack.num_items from popped Receiver frame)
      → Context::next_ack pops Receiver's frame, sets ack.{num_items,num_bytes}
      → route_ack → DeliverAck{node_id=Receiver}
      → pipeline_ctrl records produced.success for Receiver
```

## Files Changed

| File | Changes |
|------|---------|
| `crates/otap/src/pdata.rs` | Frame gets `num_items`/`num_bytes`; Context methods accept counts; `next_ack`/`next_nack` populate ack/nack from Frame; `notify_ack`/`notify_nack` use message counts; `send_message_subscribed` removes forward-path recording; deprecated `subscribe_to` impls pass counts |
| `crates/engine/src/control.rs` | `AckMsg`/`NackMsg` get `num_items`/`num_bytes` fields |
| `crates/engine/src/component_metrics.rs` | Remove `ProducedMetrics` (no-outcome); remove `Instrumented` trait; remove blanket impls |
| `crates/engine/src/pipeline_ctrl.rs` | Use `ack.num_items`/`nack.num_items` instead of `Instrumented` trait; remove `Instrumented` bound |
| `crates/engine/src/entity_context.rs` | Simplify `register_component_metrics` (fewer metric sets) |
| `crates/engine/src/lib.rs` | Remove `Instrumented` bound from `PipelineFactory` |
| `crates/controller/src/lib.rs` | Remove `Instrumented` bound from `Controller` |

## Status

**All items implemented.** (Updated 2026-02-06)

| # | Change | Status |
|---|--------|--------|
| 1 | Frame carries `num_items`/`num_bytes` | Done |
| 2 | `AckMsg`/`NackMsg` carry counts from popped Frame | Done |
| 3 | `subscribe_to`/`subscribe_or_update` accept counts | Done |
| 4 | `next_ack`/`next_nack` populate counts from Frame | Done |
| 5 | Metrics recorded on return path only (consumer in `notify_ack`/`notify_nack`, producer in `pipeline_ctrl`) | Done |
| 6 | Simplified `ComponentMetricsState` — `ProducedMetrics` (no-outcome) removed | Done |
| 7 | `Instrumented` trait bound removed from engine (`PipelineCtrlMsgManager`, `PipelineFactory`, `Controller`) | Done |
