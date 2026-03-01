# Component Metrics on the Ack/Nack Return Path

## Overview

This document describes the design of component-level metrics that
are measured using the ack/nack return path in the otap-dataflow
pipeline engine. These metrics let operators observe how each
pipeline node (receiver, processor, exporter) performs — counting
request outcomes and, optionally, measuring per-node processing
duration.

The system is level-gated: the `MetricLevel` configuration
(`None` / `Basic` / `Normal` / `Detailed`) controls how much
instrumentation overhead is incurred.

### Design principle: controller-owned unwinding

All context unwinding — popping frames, recording metrics, and
routing acks/nacks to subscriber nodes — is performed exclusively by
the **pipeline controller** (`PipelineCtrlMsgManager`). Individual
components (processors, exporters) never pop from the context stack
or record metrics themselves. When a component calls
`notify_ack()`/`notify_nack()`, it simply forwards the raw ack/nack
to the controller via the pipeline control channel
(`PipelineControlMsg::DeliverAck` / `DeliverNack`). The controller
then iterates the context stack using the `Unwindable` trait.

### Design principle: metrics without delivery

A key design principle is the separation of **metrics interest** from
**ack/nack delivery interest**. Nodes configured for pipeline metrics
(via `MetricLevel::Basic` or above) automatically get context frames
that capture consumed-outcome and timing data — but those frames do
**not** carry `ACKS` or `NACKS` interest unless the component
explicitly subscribes. This means:

- Nodes that never call `subscribe_to()` still get measured.
- Those nodes do **not** receive ack/nack messages on their control
  channel.
- Their metrics are recorded by the pipeline controller during
  context unwinding, when it encounters frames with
  `CONSUMER_METRICS` or `PRODUCER_METRICS` interest.

This avoids the earlier design flaw where `push_entry_frame()`
auto-subscribed to `ACKS_OR_NACKS`, causing acks to route to nodes
that ignored them (e.g., back to the exporter that originated the
ack), breaking the upstream notification chain.

## 1. Configuration: MetricLevel → Interests

Every node in a pipeline shares a single `Interests` bitflag value
derived once at pipeline startup from the `TelemetryPolicy`:

```
telemetry_policy.component_metrics  →  Interests::from_metric_level()
```

| MetricLevel | Interests flags                           |
|-------------|-------------------------------------------|
| `None`      | *(empty)*                                 |
| `Basic`     | `PIPELINE_METRICS`                        |
| `Normal`    | `PIPELINE_METRICS`                        |
| `Detailed`  | `PIPELINE_METRICS \| ENTRY_TIMESTAMP`     |

Where `PIPELINE_METRICS` = `CONSUMER_METRICS | PRODUCER_METRICS`.

These flags are pre-computed once in `RuntimePipeline::run_forever()`
and passed into every node's `start()` call, avoiding per-message
branching on the metric level enum.

### Interests flags

| Flag               | Bit | Meaning |
|--------------------|-----|---------|
| `ACKS`             | 0   | Node wants to receive ack notifications |
| `NACKS`            | 1   | Node wants to receive nack notifications |
| `RETURN_DATA`      | 2   | Payload should be returned with ack/nack (for retry) |
| `ENTRY_TIMESTAMP`  | 3   | Capture wall-clock timestamp on entry (for duration) |
| `CONSUMER_METRICS` | 4   | Record consumed-outcome metrics for this node |
| `PRODUCER_METRICS` | 5   | Record produced-outcome metrics for this node |

Compound aliases:

| Alias              | Bits  | Definition |
|--------------------|-------|------------|
| `ACKS_OR_NACKS`    | 0 + 1 | `ACKS \| NACKS` |
| `PIPELINE_METRICS` | 4 + 5 | `CONSUMER_METRICS \| PRODUCER_METRICS` |

`CONSUMER_METRICS` is set by `push_entry_frame()` for every node
that receives data (processors, exporters). `PRODUCER_METRICS` is
set by `subscribe_to()` for every node that explicitly subscribes
(receivers, processors that produce). Neither bit implies `ACKS` or
`NACKS`.

## 2. The Context Stack

Every `OtapPdata` carries a `Context` — a stack of `Frame` values,
one per participating node. Each frame records:

```rust
// Defined in engine::control — accessible to the engine
// without PData-specific knowledge.
struct Frame {
    interests: Interests,         // what this node is interested in
    calldata:  CallData,          // engine + component opaque data
    node_id:   usize,             // node index for ack/nack routing
}

struct CallData {
    user:              UserCallData,   // component-specific opaque data
    time_ns:           u64,            // receive timestamp (nanos since epoch), 0 = none
    output_port_index: u16,            // producer's output port, stamped at send time
}
```

The context stack grows as data flows from receivers through
processors to exporters. When ack/nack messages return, the
**pipeline controller** unwinds the stack frame-by-frame using the
`Unwindable` trait:

- Frames with `ACKS` / `NACKS` are **subscriber frames**: the ack is
  routed to that node and delivered on its control channel.
- Frames with only `CONSUMER_METRICS` and/or `PRODUCER_METRICS` are
  **metrics-only frames**: the controller records their metrics
  directly (TODO: not yet implemented) and continues popping.
- Frames with no relevant interests are silently discarded.

## 3. Forward Path: Stamping the Context

Three operations stamp the context as data flows forward through the
pipeline:

### 3a. Entry-frame stamping via `ReceivedAtNode`

When a processor or exporter receives PData from its input channel,
`MessageChannel::recv()` automatically calls:

```rust
pdata.received_at_node(self.node_id, self.interests);
```

For `OtapPdata`, this delegates to `Context::push_entry_frame()`, which:

1. **Short-circuits** if the node has no consumer metrics interest
   (`!interests.intersects(CONSUMER_METRICS | ENTRY_TIMESTAMP)`).
2. Pushes a new `Frame` with:
   - `interests`: `CONSUMER_METRICS` (if node's `CONSUMER_METRICS` is
     set), inheriting `RETURN_DATA` from the predecessor frame
   - `time_ns`: `nanos_since_epoch()` (if `ENTRY_TIMESTAMP` is set),
     else 0
   - `user`: empty `UserCallData`
   - `output_port_index`: 0 (will be stamped at send time)

**Crucially, this frame does NOT set `ACKS` or `NACKS`.** The frame
exists purely for metrics. If the component later calls
`subscribe_to()`, the same-node merge will add `ACKS | NACKS` to
this frame.

The `ReceivedAtNode` trait is defined in the engine crate, allowing
the engine to call PData-specific logic without knowing the concrete
type:

```rust
pub trait ReceivedAtNode {
    fn received_at_node(&mut self, node_id: usize, node_interests: Interests);
}
```

The engine's `MessageChannel` impl block uses `PData: ReceivedAtNode`
as a trait bound only on the `recv()` method, keeping the rest of the
engine generic and unconstrained.

### 3b. Component subscription via `subscribe_to()`

When a component explicitly calls
`effect_handler.subscribe_to(interests, ctx, &mut pdata)`, the
`ProducerEffectHandlerExtension` implementation for `OtapPdata`:

1. Auto-adds `ACKS | NACKS | PRODUCER_METRICS` if the node's
   `PRODUCER_METRICS` interest is set.
2. Calls `Context::subscribe_to()`, which either:
   - **Merges** into the top frame (same `node_id`): ORs interests,
     replaces `user` calldata, preserves `time_ns`.
   - **Pushes** a new frame (different `node_id`): inherits
     `RETURN_DATA` from the predecessor.
3. At `Detailed` level, stamps `time_ns` via `stamp_top_time()`.

After the same-node merge, a frame pushed by `push_entry_frame()`
and then merged by `subscribe_to()` contains:

```
CONSUMER_METRICS | PRODUCER_METRICS | ACKS | NACKS | [user interests]
```

This is a **subscriber frame**: it will receive ack/nack delivery AND
have both consumer and producer metrics recorded.

A frame that was only pushed by `push_entry_frame()` (no
`subscribe_to()`) contains:

```
CONSUMER_METRICS | [RETURN_DATA from predecessor]
```

This is a **metrics-only frame**: it will NOT receive ack/nack
delivery, but the pipeline controller will record consumer metrics
for it during unwinding.

### 3c. Output port stamping at send time

When a component sends data downstream via
`send_message_with_source_node()` or its port-specific variants, the
extension trait implementation stamps the top frame's
`output_port_index` with the port's stable u16 index:

```rust
data.context.stamp_output_port_index(self.output_port_index(&port_name));
```

Port indices are computed at construction by sorting port names
alphabetically and assigning sequential u16 values. This index
travels with the data on the forward path and returns on the ack/nack
path so produced-outcome metrics can be attributed to the correct
output channel.

## 4. The Ack/Nack Return Path

### 4a. Origin: the exporter

An exporter generates acks/nacks in response to downstream outcomes
(e.g., successful HTTP delivery → ack; timeout → nack). It
constructs an `AckMsg` or `NackMsg` and passes it to
`effect_handler.notify_ack()` / `effect_handler.notify_nack()`.

### 4b. Forwarding to the pipeline controller

The `ConsumerEffectHandlerExtension` implementations for `OtapPdata`
perform a single guard check and then forward the raw ack/nack to the
pipeline controller. **No metric recording or context unwinding
happens at the calling node.** All 8 implementations
(local/shared × processor/exporter) share the same pattern:

```rust
async fn notify_ack(&self, ack: AckMsg<OtapPdata>) -> Result<(), Error> {
    if ack.accepted.has_context_frames() {
        self.route_ack(ack).await
    } else {
        Ok(())
    }
}
```

The `has_context_frames()` guard checks whether the context stack has
any frames at all (`!stack.is_empty()`). If the stack is empty (e.g.,
data was produced without any upstream subscriptions or metrics
interest), there is nothing for the controller to unwind, and the
ack/nack is silently dropped to avoid unnecessary pipeline control
channel pressure.

Note: `has_context_frames()` is distinct from the existing
`has_subscribers()` method (which checks for frames with non-empty
interests). The stack may contain frames with only metrics-only
interests that the controller needs to process, so the correct check
is whether any frames exist at all.

`route_ack()` / `route_nack()` in `EffectHandlerCore` simply send a
`PipelineControlMsg::DeliverAck` / `DeliverNack` to the controller:

```rust
pub async fn route_ack(&self, ack: AckMsg<PData>) -> Result<(), Error> {
    self.send_pipeline_ctrl_msg(PipelineControlMsg::DeliverAck { ack }).await
}
```

The `DeliverAck` and `DeliverNack` variants carry only the ack/nack
message — no `node_id` or `metrics_stops`. All routing and metrics
decisions are made by the controller.

### 4c. The `Unwindable` trait

The pipeline controller needs to pop frames from the context stack
carried inside the ack/nack payload, but the engine crate has no
knowledge of the concrete PData type. The `Unwindable` trait bridges
this gap:

```rust
// Defined in engine::lib
pub trait Unwindable {
    /// Pop the top frame from the context stack.
    fn pop_frame(&mut self) -> Option<control::Frame>;

    /// Drop the retained payload (called when RETURN_DATA is not set
    /// on the destination frame).
    fn drop_payload(&mut self);
}
```

`OtapPdata` implements this trait by delegating to
`Context::pop_frame()` and `OtapPdata::take_payload()`. No-op
implementations exist for `()` and `String` (used in engine tests
where PData has no context stack).

The `Unwindable` bound is applied to
`PipelineCtrlMsgManager::run()` and propagated to
`RuntimePipeline::run_forever()` and the `Controller` impl block.

### 4d. Context unwinding in the pipeline controller

When the controller receives `DeliverAck` or `DeliverNack`, it calls
`unwind_ack()` / `unwind_nack()`:

```rust
async fn unwind_ack(&mut self, mut ack: AckMsg<PData>) {
    loop {
        match ack.accepted.pop_frame() {
            None => return,  // no more frames — ack silently consumed
            Some(frame) => {
                // TODO: record CONSUMER_METRICS / PRODUCER_METRICS here
                if frame.interests.contains(Interests::ACKS) {
                    if !frame.interests.contains(Interests::RETURN_DATA) {
                        ack.accepted.drop_payload();
                    }
                    ack.calldata = frame.calldata;
                    self.send(frame.node_id, NodeControlMsg::Ack(ack)).await;
                    return;
                }
            }
        }
    }
}
```

The unwinding loop:

1. **Pops the top frame** via `Unwindable::pop_frame()`.
2. **(TODO) Records metrics** for the popped frame if it has
   `CONSUMER_METRICS` or `PRODUCER_METRICS` interest. This will use
   the controller's per-node metrics table (see §4f).
3. **Checks for subscriber interest** (`ACKS` or `NACKS`). If
   found:
   - Drops the payload if `RETURN_DATA` is not set in the frame's
     interests (via `Unwindable::drop_payload()`).
   - Sets `ack.calldata` from the frame (carrying user calldata,
     timestamps, and output port index back to the subscriber).
   - Delivers `NodeControlMsg::Ack(ack)` to the frame's `node_id`.
   - Returns immediately — only the first matching subscriber
     receives delivery.
4. **Continues popping** if the frame has no matching interest
   (metrics-only or irrelevant frames).
5. **Exits silently** if the stack is exhausted with no subscriber
   found.

`unwind_nack()` is symmetric, checking for `Interests::NACKS`
instead.

### 4e. Per-node metrics recording (TODO)

The TODO comments in `unwind_ack`/`unwind_nack` mark where per-frame
metric recording will be added. For each popped frame:

- If `CONSUMER_METRICS` is set: record `consumed.success` (ack) or
  `consumed.failure`/`consumed.refused` (nack), and optionally
  `consumed.duration_ns` if `ENTRY_TIMESTAMP` is set (computed as
  `nanos_since_epoch() - frame.calldata.time_ns`).
- If `PRODUCER_METRICS` is set: record `produced.success` (ack) or
  `produced.failure`/`produced.refused` (nack), attributed to
  `frame.calldata.output_port_index`.

For nacks, the outcome is `RequestOutcome::Failure` (transient) or
`RequestOutcome::Refused` (permanent), determined by `nack.permanent`.

### 4f. Per-node metrics table in the pipeline controller

The `PipelineCtrlMsgManager` will hold a per-node metrics table built
at pipeline startup:

```rust
struct NodeMetrics {
    /// Handle to this node's input channel receiver metrics (for consumed outcomes).
    input: Option<InputChannelReceiverMetrics>,
    /// Handles to this node's output channel sender metrics, indexed by port.
    outputs: Vec<Option<OutputChannelSenderMetrics>>,
}
```

This table is not yet populated. The `MetricsStop` struct exists in
`engine::control` to support this future work:

```rust
pub struct MetricsStop {
    pub node_id:   usize,
    pub calldata:  CallData,
    pub interests: Interests,
}
```

### 4g. Receiver terminal handling

Receivers are the origin of the forward path — they only produce,
never consume. Their `ControlChannel::recv()` in the engine
automatically intercepts Ack/Nack control messages and records
produced-outcome metrics before returning the message to the
component:

```rust
// In ControlChannel::recv():
NodeControlMsg::Ack(ack) => {
    if let Some(Some(m)) = self.output_channel_sender_metrics
        .get(ack.calldata.output_port_index as usize)
    {
        m.record_produced(RequestOutcome::Success);
    }
}
```

This is done in the engine itself (not in an extension trait) because
it only reads `output_port_index` from the calldata — it doesn't
need PData-specific knowledge.

## 5. Metrics Instruments

### Channel sender metrics (producer side)

Registered under `channel.sender.*`, one set per output channel:

| Metric                       | Type    | Description |
|------------------------------|---------|-------------|
| `send.count`                 | Counter | Messages successfully enqueued |
| `send.error_full`            | Counter | Sends rejected (channel full) |
| `send.error_closed`          | Counter | Sends rejected (channel closed) |
| `produced.success`           | Counter | Acks received for produced items |
| `produced.failure`           | Counter | Transient nacks (retryable) |
| `produced.refused`           | Counter | Permanent nacks |

### Channel receiver metrics (consumer side)

Registered under `channel.receiver.*`, one set per input channel:

| Metric                       | Type    | Description |
|------------------------------|---------|-------------|
| `recv.count`                 | Counter | Messages successfully dequeued |
| `recv.error_empty`           | Counter | Receive attempts on empty channel |
| `recv.error_closed`          | Counter | Receive attempts on closed channel |
| `capacity`                   | Gauge   | Channel buffer capacity |
| `consumed.success`           | Counter | Acks received for consumed items |
| `consumed.failure`           | Counter | Transient nacks for consumed items |
| `consumed.refused`           | Counter | Permanent nacks for consumed items |
| `consumed.duration_ns`       | Mmsc    | Processing duration (min/max/sum/count) |

Duration is only populated at `MetricLevel::Detailed`.

## 6. End-to-End Flow Examples

### 6a. All nodes subscribe: Receiver → Processor → Exporter

`MetricLevel::Detailed`. Processor calls `subscribe_to()`.

**Forward path** (data flowing downstream):

1. Receiver produces data. `subscribe_to()` pushes Frame₀ with
   `{ACKS|NACKS|PRODUCER_METRICS, time_ns=T₀, port_index=0, node_id=recv}`.
2. Receiver calls `send_message_with_source_node()`, which stamps
   `output_port_index=0` on Frame₀.
3. Processor's `MessageChannel::recv()` fires
   `received_at_node()` → pushes Frame₁ with
   `{CONSUMER_METRICS, time_ns=T₁, node_id=proc}`.
4. Processor calls `subscribe_to()` → same-node merge into Frame₁:
   ORs `ACKS|NACKS|PRODUCER_METRICS` into interests, replaces user
   calldata, preserves `time_ns=T₁`.
   Frame₁ now: `{CONSUMER_METRICS|PRODUCER_METRICS|ACKS|NACKS, ...}`.
5. Processor sends downstream, stamping `output_port_index=0` on
   Frame₁.
6. Exporter's `MessageChannel::recv()` fires
   `received_at_node()` → pushes Frame₂ with
   `{CONSUMER_METRICS, time_ns=T₂, node_id=exp}`.

Context stack at exporter:
```
[Frame₀(recv, ACKS|NACKS|PRODUCER_METRICS),
 Frame₁(proc, CONSUMER_METRICS|PRODUCER_METRICS|ACKS|NACKS),
 Frame₂(exp,  CONSUMER_METRICS)]
```

**Return path** (ack flowing upstream):

7. Exporter succeeds, calls `effect_handler.notify_ack(ack)`.
8. Exporter's `notify_ack` checks `has_context_frames()` → true
   (3 frames). Sends `PipelineControlMsg::DeliverAck { ack }` to
   the pipeline controller.
9. Controller receives `DeliverAck`, calls `unwind_ack(ack)`:
   - `pop_frame()` → Frame₂ (exp). Has `CONSUMER_METRICS` but no
     `ACKS`. (TODO: records `consumed.success` and
     `consumed.duration_ns = now - T₂` for exporter.) Continues.
   - `pop_frame()` → Frame₁ (proc). Has `ACKS` → subscriber found.
     Payload not dropped (RETURN_DATA inherited). Sets
     `ack.calldata = Frame₁.calldata`. Sends
     `NodeControlMsg::Ack(ack)` to `node_id=proc`.
10. Processor receives ack on its control channel. Component calls
    `effect_handler.notify_ack(ack)`.
11. Processor's `notify_ack` checks `has_context_frames()` → true
    (1 frame: Frame₀ remains). Sends `DeliverAck { ack }` to
    controller.
12. Controller's `unwind_ack(ack)`:
    - `pop_frame()` → Frame₀ (recv). Has `ACKS` → subscriber found.
      (TODO: records `produced.success` for processor using
      `ack.calldata.output_port_index`.) Sets
      `ack.calldata = Frame₀.calldata`. Sends
      `NodeControlMsg::Ack(ack)` to `node_id=recv`.
13. Receiver's `ControlChannel::recv()` intercepts the ack:
    - Records `produced.success` on output port
      `ack.calldata.output_port_index` of the receiver's output
      channel.
    - Returns the ack to the receiver component.

### 6b. Non-subscribing processor: Receiver → Processor → Exporter

`MetricLevel::Detailed`. Processor does NOT call `subscribe_to()`.

**Forward path**:

Same as §6a steps 1–2, then:

3. Processor's `MessageChannel::recv()` fires
   `received_at_node()` → pushes Frame₁ with
   `{CONSUMER_METRICS, time_ns=T₁, node_id=proc}`.
4. Processor does NOT call `subscribe_to()`. Frame₁ remains
   metrics-only: `{CONSUMER_METRICS}`.
5. Processor sends downstream (no `output_port_index` stamped on
   Frame₁ since it didn't subscribe).
6. Exporter's `received_at_node()` → pushes Frame₂ with
   `{CONSUMER_METRICS, time_ns=T₂, node_id=exp}`.

Context stack at exporter:
```
[Frame₀(recv, ACKS|NACKS|PRODUCER_METRICS),
 Frame₁(proc, CONSUMER_METRICS),
 Frame₂(exp,  CONSUMER_METRICS)]
```

**Return path**:

7. Exporter succeeds, calls `effect_handler.notify_ack(ack)`.
8. Exporter's `notify_ack` checks `has_context_frames()` → true
   (3 frames). Sends `PipelineControlMsg::DeliverAck { ack }` to
   the pipeline controller.
9. Controller receives `DeliverAck`, calls `unwind_ack(ack)`:
   - `pop_frame()` → Frame₂ (exp). Has `CONSUMER_METRICS` but no
     `ACKS`. (TODO: records `consumed.success` and
     `consumed.duration_ns = now - T₂` for exporter.) Continues.
   - `pop_frame()` → Frame₁ (proc). Has `CONSUMER_METRICS` but no
     `ACKS`. (TODO: records `consumed.success` and
     `consumed.duration_ns = now - T₁` for processor.) Continues.
   - `pop_frame()` → Frame₀ (recv). Has `ACKS` → subscriber found.
     Sets `ack.calldata = Frame₀.calldata`. Sends
     `NodeControlMsg::Ack(ack)` to `node_id=recv`.
10. Receiver's `ControlChannel::recv()` intercepts the ack:
    - Records `produced.success` on output port
      `ack.calldata.output_port_index`.
    - Returns the ack to the receiver component.

**Result**: All three nodes have correct metrics. The processor and
exporter got measured by the controller during unwinding without
receiving acks on their control channels.

## 7. Design Properties

**Zero overhead at `MetricLevel::None`**: No frames are pushed
(the `push_entry_frame` short-circuit), no subscriptions happen, so
ack/nack messages are never generated. The `has_context_frames()`
guard returns false, and acks/nacks are silently dropped.

**Minimal overhead at `Basic`/`Normal`**: Frames are pushed and
outcome counters increment, but no timestamps are captured and no
duration histograms are computed.

**Full instrumentation at `Detailed`**: Timestamps are captured via
`nanos_since_epoch()` (a monotonic clock read) on every receive, and
durations are computed on every ack/nack.

**Controller-owned unwinding**: Components never pop from the context
stack or record metrics themselves. `notify_ack()`/`notify_nack()`
simply forward the raw ack/nack to the pipeline controller via
`DeliverAck`/`DeliverNack`. The controller uses the `Unwindable`
trait to pop frames one by one, record metrics, and deliver to the
first matching subscriber. This centralizes all unwinding and metrics
logic in one place, making components simpler and eliminating the
need for a transfer-function generic parameter on `route_ack`/`route_nack`.

**Metrics without delivery**: `push_entry_frame()` sets only
`CONSUMER_METRICS`, never `ACKS` or `NACKS`. A node must explicitly
call `subscribe_to()` to receive ack/nack messages. This prevents
the earlier bug where acks routed back to the exporter that created
them.

**Same-node merge**: When a component calls `subscribe_to()` after
the automatic `push_entry_frame()`, the frames merge instead of
stacking. This avoids double-counting and ensures the entry timestamp
is preserved alongside the component's user calldata. After merge,
the frame has both `CONSUMER_METRICS` (from entry) and
`PRODUCER_METRICS | ACKS | NACKS` (from subscribe).

**Unwindable trait**: The engine's `Unwindable` trait
(`pop_frame() -> Option<Frame>`, `drop_payload()`) allows the
controller to manipulate PData's context stack without knowing the
concrete type. The trait bound is applied narrowly:
`PipelineCtrlMsgManager::run()`, `RuntimePipeline::run_forever()`,
and the `Controller` impl block. No-op implementations for `()` and
`String` allow engine-level tests to use simple PData types.

**ReceivedAtNode trait**: The `ReceivedAtNode` trait is the only
other engine-level PData bound, applied narrowly to the
`MessageChannel::recv()` impl block and propagated to
`ProcessorWrapper::start()` and `RuntimePipeline::run_forever()`.
All other metric recording logic lives in the `otap` crate as
extension trait implementations, keeping the engine PData-agnostic.

**Port-indexed producer attribution**: Multi-output processors produce
to named ports, which are mapped to stable u16 indices. The index is
stamped at send time and read back on the return path, allowing
per-port produced-outcome tracking without additional data structures.

**Pipeline controller as metrics hub**: The `PipelineCtrlMsgManager`
will hold a per-node `NodeMetrics` table built at startup, giving it
access to every node's input and output channel metrics handles.
During context unwinding, the controller records consumed and
produced outcomes on behalf of all nodes — both subscribing and
non-subscribing — ensuring complete metrics coverage.
