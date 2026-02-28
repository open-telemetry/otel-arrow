# V3 Pipeline Component Metrics Design

## Background

This document supersedes the V2 design. It follows from
[send-with-subscription-design.md](send-with-subscription-design.md) and is
informed by the team's feedback: **instrument where Ack/Nack messages are
routed**, so that sending and receiving acks/nacks is automatically
instrumented with low overhead.

The V2 design added `num_items` / `num_bytes` fields to `Frame`, `AckMsg`,
and `NackMsg`, and recorded produced-side metrics in the `pipeline_ctrl.rs`
event loop. That has two problems:

1. **`pipeline_ctrl.rs` is generic over `PData`** — it cannot access the
   OtapPdata `Context`, and adding `Instrumented` trait bounds couples the
   engine to data-type concerns.
2. **Producer metrics fire at delivery, not receipt** — the true time a
   producer learns an outcome is when the Ack/Nack is dequeued from its
   control channel, not when the pipeline controller enqueues it.

## V3 Key Ideas

### Metric levels

An engine-wide `MetricLevel` setting controls per-node instrumentation
overhead:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetricLevel {
    #[default]
    None,
    Basic,
    Normal,
    Detailed,
}
```

| Level | Forward path | Ack/Nack path |
|-------|-------------|---------------|
| None | No instrumentation | No instrumentation |
| Basic | — | Count requests by outcome (success/failure/refused) |
| Normal | Count bytes | Count requests by outcome |
| Detailed | Count bytes | Count requests by outcome + duration histogram |

At `Basic` and above, the engine auto-subscribes the node to
`ACKS | NACKS` so that outcome information is delivered. At `Normal`
and above, byte counts are recorded on the forward path when data
enters the node (no outcome breakdown — a single counter per node).
At `Detailed`, a receive timestamp is stored in the entry frame to
compute request duration at ack/nack time.

Metric level is an engine-wide setting — all nodes share the same
level. Per-node overrides may be added later but are not part of
this design.

### One frame per component

There is exactly **one frame per component** on the context stack. When
data enters a node (pulled from the input channel), the engine pushes an
**entry frame** for that node. The frame's initial interests and engine
fields depend on the node's metric level:

- **None**: empty interests, no engine fields populated.
- **Basic+**: interests include `ACKS | NACKS` (auto-subscribe for
  outcome counting).
- **Detailed**: additionally, `time_ns` is stamped with the receive time.

If the component then calls `subscribe_to`, it updates the **existing**
frame in place — merging interests and filling in its component-specific
data — rather than pushing a second frame.

Receivers are different: they **originate** the context (no input channel).
Their single frame is created by `subscribe_to`, which pushes a new frame
with auto-subscribe interests (if `Basic+`), `time_ns` (if `Detailed`),
and component data.

### CallData struct replaces slot convention

The current `CallData` type alias (`SmallVec<[Context8u8; 3]>`) is
renamed to `UserCallData` — it remains the opaque, component-owned
payload. A new `CallData` struct wraps it with one engine-managed field:

```rust
/// Renamed: what was CallData is now UserCallData.
pub type UserCallData = SmallVec<[Context8u8; 3]>;

/// New CallData: engine-managed envelope around component data.
#[derive(Clone, Debug, Default)]
pub struct CallData {
    /// Component-specific opaque data (formerly the entire CallData).
    pub user: UserCallData,
    /// Receive timestamp (monotonic nanos since process epoch).
    /// Only populated when MetricLevel == Detailed; 0 otherwise.
    pub time_ns: u64,
}
```

`CallData` is carried in `Frame`, `AckMsg`, and `NackMsg` exactly as
before — no new struct fields on those types. The engine reads
`time_ns` at `Detailed` level only; components read/write `user`.
Byte counts are recorded on the forward path and not stored in
`CallData`.

### Measurement boundaries

- **Bytes** (Normal+) = counted on the forward path when data enters a
  node. A single counter per node, no outcome breakdown. This avoids
  carrying byte counts through the ack/nack return path.
- **Outcome counts** (Basic+) = recorded at ack/nack time. Consumer
  outcomes in `notify_ack`/`notify_nack`; producer outcomes when the
  Ack/Nack is received on the control channel.
- **Consumer duration** (Detailed only) = time from data receipt (entry
  frame `time_ns`) to ack/nack **send**. Measured in
  `notify_ack`/`notify_nack`. Each frame's single timestamp yields
  exactly one duration measurement on the consumer.
- No producer-side duration measurement (avoids double-counting).

### Metrics-driven auto-subscribe

If a node's metric level is `Basic` or above, the engine automatically
includes `ACKS | NACKS` in the entry frame's interests. This ensures
ack/nack messages are delivered to the node for outcome counting, even
if the component never calls `subscribe_to`.

Nodes that receive ack/nack only because of auto-subscribe (and have no
component-level subscription) get a **default ack/nack handler** that:

1. Records the outcome metric (success/failure/refused count; duration
   if Detailed).
2. Propagates the ack/nack to the next subscriber via
   `route_ack`/`route_nack`.

If metric level is `None` and the component never subscribes, the frame
retains empty interests and `next_ack`/`next_nack` skip it — no metrics
fire and no ack/nack is delivered.

## Design

### 1. Entry frame: one per component

When data arrives at a processor or exporter (pulled from its input
channel), the engine pushes an **entry frame**. The frame's contents
depend on the engine's metric level:

```rust
// Pushed automatically when data is received from the input channel.
// interests and time_ns depend on the engine's MetricLevel.
let interests = if metric_level >= MetricLevel::Basic {
    Interests::ACKS | Interests::NACKS   // auto-subscribe for outcome counting
} else {
    Interests::empty()                    // no metrics, no auto-subscribe
};
let time_ns = if metric_level >= MetricLevel::Detailed {
    nanos_since_epoch()                   // receive time for duration histogram
} else {
    0                                     // no timestamp overhead
};

Frame {
    interests,
    node_id:   self_node_id,
    calldata:  CallData {
        user:    UserCallData::new(),     // empty — component hasn't subscribed yet
        time_ns,
    },
}
```

At `Normal` and above, the engine also counts bytes from the payload
on the forward path (a simple counter increment, not stored in the
frame).

This does **not** replace `set_source_node`. Source tagging is a sender
concern — it pushes a frame with the **sender's** node_id so the receiver
can identify the source. The entry frame is a separate, receiver-side
concern: it uses the **receiver's** node_id. The two frames coexist on
the stack.

For **processors**: The engine's run loop calls a new
`DataReceivedExtension` trait method between `message_channel.recv()`
and `processor.process()`. For **exporters**: the engine cannot
intercept (the exporter controls its own run loop), so the exporter
component calls `effect_handler.on_pdata_received(&mut pdata)` when it
pulls PData from the channel.

For **receivers** (which originate the context), there is no input
channel and no entry frame push. The receiver's frame is created when
it calls `subscribe_to`, which pushes a new frame with auto-subscribe
interests (if `Basic+`) and `time_ns` (if `Detailed`).

### 2. subscribe_to fills in the entry frame

When a processor calls `subscribe_to`, it detects that the top frame
belongs to the same node (the entry frame) and **updates it in place**:

```rust
pub(crate) fn subscribe_to(
    &mut self,
    mut interests: Interests,
    user_calldata: UserCallData,   // component's own data only
    node_id: usize,
) {
    if let Some(top) = self.stack.last_mut() {
        if top.node_id == node_id {
            // Same node → merge interests, replace user data.
            // Engine field (time_ns) is preserved from the entry frame.
            top.interests |= interests;
            top.calldata.user = user_calldata;
            return;
        }
        // Inherit RETURN_DATA from preceding frame
        // (new-frame path: should not happen in normal pipeline flow)
    }
    // Fall through: push new frame (receiver path — no prior entry frame)
    // time_ns and auto-subscribe interests are set by the caller
    // (effect handler) based on MetricLevel, not here.
    self.stack.push(Frame {
        interests,
        node_id,
        calldata: CallData {
            user: user_calldata,
            ..Default::default()
        },
    });
}
```

Key behaviors:
- **Interests merge**: E.g. `ACKS | NACKS` accumulated into one frame.
  Auto-subscribe interests from the entry frame are preserved.
- **Engine field preserved**: `time_ns` set by the entry frame remains
  untouched; only `user` is replaced.
- **No second frame**: The `debug_assert_ne!` in the current `subscribe_to`
  is replaced by the update-in-place path.
- **Receiver path**: When `subscribe_to` pushes a new frame (receiver
  case, no prior entry frame), the caller (effect handler) is responsible
  for adding auto-subscribe interests and stamping `time_ns` based on
  the metric level.

### 3. CallData struct fields

| Field | Type | Set by | Read by |
|-------|------|--------|--------|
| `time_ns` | `u64` | Engine (entry frame, Detailed only) | Consumer metric (duration histogram) |
| `user` | `UserCallData` | Component (`subscribe_to`) | Component (via `ack.calldata.user`) |

The engine reads `time_ns` only at `Detailed` level. The `user` field
is opaque to the engine and owned by the component. Byte counts are
not stored in `CallData` — they are recorded on the forward path.

### 4. Consumer metrics: measured at `notify_ack` / `notify_nack`

Before routing the ack/nack, `notify_ack` / `notify_nack` **peeks at the
top frame** — this is the current node's own entry frame. Behaviour
depends on the metric level:

- **Basic+**: increment the outcome counter (success / failure / refused).
- **Detailed**: additionally compute duration from `time_ns` and record
  in the duration histogram.

```rust
// In the ConsumerEffectHandlerExtension impl (crates/otap/src/pdata.rs)
async fn notify_ack(&self, ack: AckMsg<OtapPdata>) -> Result<(), Error> {
    if metric_level >= MetricLevel::Basic {
        // Basic+: count outcome
        self.consumer_metrics().record_success_count();

        if metric_level >= MetricLevel::Detailed {
            // Detailed: duration histogram
            if let Some(frame) = ack.accepted.context.peek_top() {
                let duration_ns = nanos_since_epoch()
                    .saturating_sub(frame.calldata.time_ns);
                self.consumer_metrics().record_success_duration(duration_ns);
            }
        }
    }

    // Proceed with normal routing: next_ack pops frames to find subscriber
    self.route_ack(ack, Context::next_ack).await
}
```

The same pattern applies to `notify_nack`, recording failure
(non-permanent) or refused (permanent) outcomes.

Byte counting is **not** done here — bytes are counted on the forward
path at receive time (see §1).

For nodes with metrics-driven auto-subscribe that do not have their own
ack/nack handling, the **default ack/nack handler** performs the same
metric recording and then calls `route_ack`/`route_nack` to propagate.

### 5. `Context::next_ack` / `next_nack` — unchanged structure

The transfer function works exactly as today: pop frames until one with
matching interest is found, copy its `calldata` to the ack/nack message,
optionally drop payload if `RETURN_DATA` is not set.

Because the entry frame and subscribe data are **merged into one frame**,
a single pop both delivers the ack to the correct `node_id` AND carries
the full `CallData` struct (engine fields + component `user` data):

```rust
// No changes to the signature or AckMsg/NackMsg types.
pub fn next_ack(mut ack: AckMsg<OtapPdata>) -> Option<(usize, AckMsg<OtapPdata>)> {
    ack.accepted.context.next_with_interest(Interests::ACKS).map(|frame| {
        ack.calldata = frame.calldata;  // CallData { time_ns, user }
        if (frame.interests & Interests::RETURN_DATA).is_empty() {
            let _drop = ack.accepted.take_payload();
        }
        (frame.node_id, ack)
    })
}
```

Frames with empty interests are skipped. At `MetricLevel::None`, a node
that never subscribes has empty interests and is skipped. At `Basic+`,
the auto-subscribe interests ensure the frame is found.

With metrics enabled, **all nodes** (including pass-through processors
and exporters) participate in ack/nack routing. This is by design —
it enables outcome counting for every pipeline component.

### 6. Producer metrics: instrumented control channel receiver

The producer-side metric fires when the node **actually receives** the
Ack/Nack from its control channel. This is a wrapper around the control
receiver, wired in during pipeline build:

```rust
pub(crate) struct InstrumentedControlReceiver<PData> {
    inner: Receiver<NodeControlMsg<PData>>,
    metrics: ProducedMetrics,
}

impl<PData> InstrumentedControlReceiver<PData> {
    pub async fn recv(&mut self) -> Result<NodeControlMsg<PData>, ...> {
        let msg = self.inner.recv().await?;
        if self.metrics.level >= MetricLevel::Basic {
            match &msg {
                NodeControlMsg::Ack(_) => {
                    self.metrics.record_success();
                }
                NodeControlMsg::Nack(_) => {
                    // All nacks are refusals from the producer's perspective.
                    // Failure is a consumer-side concept (the component itself
                    // had an error); the producer only sees refusal.
                    self.metrics.record_refused();
                }
                _ => {}
            }
        }
        Ok(msg)
    }
}
```

No byte counts are read from the ack/nack — producer bytes are counted
on the forward path at send time. This wrapper only records outcome
counts (Basic+).

This wrapper is created in `runtime_pipeline.rs` during pipeline build,
where the `NodeTelemetryHandle` and metrics registry are available.

### 7. Metric definitions

Metrics are gated by the engine-wide `MetricLevel`:

**Consumer metrics** (per-node):

| Level | Metric | Unit | When |
|-------|--------|------|------|
| Basic+ | `consumed.requests{outcome=success}` | `{1}` | `notify_ack` |
| Basic+ | `consumed.requests{outcome=failure}` | `{1}` | `notify_nack(!permanent)` |
| Basic+ | `consumed.requests{outcome=refused}` | `{1}` | `notify_nack(permanent)` |
| Normal+ | `consumed.bytes` | `{By}` | Forward path (on receive) |
| Detailed | `consumed.duration{outcome=success}` | `{s}` | `notify_ack` (histogram) |
| Detailed | `consumed.duration{outcome=failure}` | `{s}` | `notify_nack(!permanent)` (histogram) |
| Detailed | `consumed.duration{outcome=refused}` | `{s}` | `notify_nack(permanent)` (histogram) |

**Producer metrics** (per-node):

| Level | Metric | Unit | When |
|-------|--------|------|------|
| Basic+ | `produced.requests{outcome=success}` | `{1}` | Control channel receives `Ack` |
| Basic+ | `produced.requests{outcome=refused}` | `{1}` | Control channel receives `Nack` (any) |
| Normal+ | `produced.bytes` | `{By}` | Forward path (on send) |

There is no `produced.failure` metric. From the producer's perspective,
all nacks are refusals — failure is a consumer-side concept indicating
the component itself introduced an error.

Byte counters are forward-path only, with no outcome breakdown. Duration
histograms are Detailed-only and consumer-side only (no double-counting).

## Data Flow

```
Forward path (entry frame + optional subscribe, at Basic+):

    Receiver                        Processor                       Exporter
    ─────────                       ─────────                       ────────
    subscribe_to(ACKS,cd,recv_id)   data arrives from input channel
      → pushes frame with            → received_at_node pushes
        interests=ACKS|NACKS,          entry frame:
        time_ns(Detailed),             interests=ACKS|NACKS (Basic+)
        user=[cd]                      time_ns (Detailed only)
      count bytes (Normal+)            count bytes (Normal+)
    send_message(data)              subscribe_to(ACKS,cd,proc_id)
                                     → merges interests, sets user
                                     (time_ns preserved)
                                    send_message(data)
                                     ──────────────────────────────→  data arrives
                                                                     → received_at_node
                                                                       pushes entry frame:
                                                                       interests=ACKS|NACKS
                                                                       time_ns (Detailed)
                                                                       count bytes (Normal+)
                                                                     (exporter does
                                                                      not subscribe)

Context stack when Exporter receives data (Basic+ level):

  ┌────────────────────────────────────────────────────────────────────────┐
  │ Frame: Recv  | interests=ACKS|NACKS | time_ns, user=[comp_cd]        │  bottom
  │ Frame: Proc  | interests=ACKS|NACKS | time_ns, user=[comp_cd]        │  middle
  │ Frame: Exp   | interests=ACKS|NACKS | time_ns, user=[]               │  top
  └────────────────────────────────────────────────────────────────────────┘

Return path (ack, Basic+):
    Exporter calls notify_ack(ack)
      ① Peek top frame (Exp) → time_ns (if Detailed)
         Record consumed.success count for Exporter
         Record consumed.success duration (Detailed only)
      ② next_ack pops: find Exp (ACKS), then Proc (ACKS)
         ack.calldata = Proc's CallData {time_ns, user}
      ③ route_ack → DeliverAck{node_id=Processor}

    Pipeline controller passes through to Processor's control channel

    Processor's control channel recv()
      ④ Record produced.success count for Processor (no bytes)

    Processor calls notify_ack(ack)
      ⑤ Peek top frame (Proc) → time_ns (if Detailed)
         Record consumed.success count for Processor
         Record consumed.success duration (Detailed only)
      ⑥ next_ack pops: find Recv (ACKS)
         ack.calldata = Recv's CallData {time_ns, user}
      ⑦ route_ack → DeliverAck{node_id=Receiver}

    Receiver's control channel recv()
      ⑧ Record produced.success count for Receiver (no bytes)
```

## CallData Struct

```rust
pub type UserCallData = SmallVec<[Context8u8; 3]>;

#[derive(Clone, Debug, Default)]
pub struct CallData {
    pub user: UserCallData,   // component-owned, opaque to engine
    pub time_ns: u64,         // receive timestamp (Detailed level only; 0 otherwise)
}
```

When `subscribe_to` updates an existing entry frame, it replaces
only `calldata.user`; `time_ns` is preserved from the entry frame.

When `next_ack`/`next_nack` copies the frame's calldata into the ack/nack
message, the entire `CallData` struct is moved — so the producer receives
`time_ns` and `user`. The component reads its own data from
`ack.calldata.user`. The engine does not read any fields from the
producer-received calldata (bytes are counted on the forward path).

## Timestamp Representation

Only used at `MetricLevel::Detailed`. `Instant` cannot be serialized
directly. We store a monotonic nanos value using a process-local epoch:

```rust
static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);

fn nanos_since_epoch() -> u64 {
    Instant::now().duration_since(*EPOCH).as_nanos() as u64
}

fn instant_from_nanos(nanos: u64) -> Instant {
    *EPOCH + Duration::from_nanos(nanos)
}
```

This fits in a single `Context8u8` (u64) and avoids platform-specific
`Instant` encoding issues.

## Node Type Behaviors

| Node type | Entry frame pushed by | Auto-subscribe? | Consumer metric? | Producer metric? |
|-----------|----------------------|----------------|-----------------|------------------|
| Receiver | `subscribe_to` (effect handler stamps time_ns if Detailed) | Yes (Basic+) | Yes (Basic+) | Yes (Basic+) |
| Processor | `on_pdata_received` (from input channel) | Yes (Basic+) | Yes (Basic+) | Yes (Basic+, if subscribed or auto-subscribed) |
| Exporter | `on_pdata_received` (from input channel) | Yes (Basic+) | Yes (Basic+) | No (terminal) |

- Receivers originate the context. Their frame is the bottom of the stack.
  At `Basic+`, the effect handler adds `ACKS | NACKS` to interests.
  At `Detailed`, `time_ns` is stamped when `subscribe_to` is called.
- At `Basic+`, all nodes participate in ack/nack routing. Exporters
  receive ack/nack via auto-subscribe and their default handler records
  the consumer outcome then propagates.
- At `None`, behaviour is unchanged from the pre-metrics engine: only
  nodes that explicitly subscribe receive ack/nack.
- A processor that does NOT subscribe at `None` level has its frame
  skipped by `next_ack`/`next_nack` — no metrics and no ack delivery.

## Files Changed

| File | Changes |
|------|---------|
| `crates/engine/src/control.rs` | Rename `CallData` → `UserCallData`; add new `CallData` struct with `user`, `time_ns`; add `MetricLevel` enum; `AckMsg`/`NackMsg` keep `calldata: CallData` (type changes structurally) |
| `crates/otap/src/pdata.rs` | Add `push_entry_frame(node_id, time_ns)` to Context; modify `subscribe_to` for same-node update-in-place (merges interests, replaces `user`, preserves engine fields); `subscribe_to` new-frame path captures `time_ns = nanos_since_epoch()` for receivers; `notify_ack`/`notify_nack` peek top frame for consumer metrics; impl `DataReceivedExtension` for 4 effect handler variants (local/shared × processor/exporter) |
| `crates/engine/src/lib.rs` | Add `DataReceivedExtension<PData>` trait (one method: `on_pdata_received`) |
| `crates/engine/src/processor.rs` | Call `effect_handler.on_pdata_received(&mut pdata)` in run loop between `recv()` and `process()` |
| `crates/engine/src/effect_handler.rs` | No changes — routing logic unchanged |
| `crates/engine/src/pipeline_ctrl.rs` | No changes — controller passes through |
| `crates/engine/src/runtime_pipeline.rs` | Wrap control channel receiver with `InstrumentedControlReceiver` during pipeline build |
| `crates/engine/src/channel_metrics.rs` (or new file) | `InstrumentedControlReceiver`, `ProducedMetrics`, `ConsumedMetrics` metric set definitions |
| `crates/engine/src/entity_context.rs` | Per-node metric handles for consumer/producer metrics |

## What Does NOT Change

- **`AckMsg` / `NackMsg` field names** — `calldata` field stays; its type
  changes from `SmallVec` alias to `CallData` struct.
- **`Frame` struct field names** — `calldata` field stays; same type change.
- **`set_source_node`** — unchanged. Source tagging is a sender concern,
  orthogonal to entry frames. Entry frames are pushed by the receiver.
- **`pipeline_ctrl.rs`** — pass-through for Ack/Nack, no metric logic.
- **`Context::next_ack` / `next_nack`** signature and behavior — unchanged.
- **Component code** — components change `CallData` → `UserCallData` in
  their subscribe calls. Reads change from `ack.calldata[n]` to
  `ack.calldata.user[n]`. Engine fields are invisible to components.
- **Exporter trait** — `start(self, msg_chan, effect_handler)` signature
  unchanged; exporters add one call to `effect_handler.on_pdata_received()`
  in their PData match arm.
- **MetricLevel::None behaviour** — at None, the engine behaves exactly as
  before: no auto-subscribe, no entry frame interests, no metric recording.
  Components that never subscribe are completely unaffected.

## Comparison with V2

| Aspect | V2 | V3 |
|--------|----|----|
| Frames per node | 2 (source + subscriber) | **1 (entry frame, optionally updated)** |
| Metric levels | None | **None/Basic/Normal/Detailed** (engine-wide) |
| Byte counts | `Frame` fields + `AckMsg`/`NackMsg` fields | **Forward-path counter** (no outcome breakdown, not in CallData) |
| Timestamp | Not captured | `CallData.time_ns` (Detailed level only) |
| Consumer metrics | In `notify_ack`/`notify_nack` | In `notify_ack`/`notify_nack` (outcome counts at Basic+, duration at Detailed) |
| Consumer latency | Not measured | Measured at Detailed: `now() - instant_from_nanos(calldata.time_ns)` |
| Producer metrics | In `pipeline_ctrl.rs` `DeliverAck`/`DeliverNack` | In `InstrumentedControlReceiver.recv()` — outcome counts only |
| Producer latency | Not measured | Not measured (no double-counting) |
| New fields on AckMsg/NackMsg | `num_items`, `num_bytes` | None |
| New fields on Frame | `num_items`, `num_bytes` | None |
| Engine PData trait bounds | Removes `Instrumented` | None needed |
| pipeline_ctrl.rs changes | Records producer metrics | None |
| Non-subscribing nodes | N/A | **Basic+**: auto-subscribe, default handler; **None**: skipped |

## Implementation Plan

### Guiding Principles

1. **Modify existing, don't duplicate.** Change `subscribe_to` behavior
   rather than adding `subscribe_or_update`. Keep `set_source_node`
   unchanged (sender concern). Add one small new trait
   (`DataReceivedExtension`) for the receiver-side entry frame — this
   cannot be achieved by modifying an existing method because no
   existing method runs at receive time. Do not add new traits like
   `SendWithSubscriptionLocalExtension` — the existing `subscribe_to` +
   `send_message` pattern continues to work.
2. **Follow the channel_metrics pattern** for MetricSet registration,
   handle types (`Rc<RefCell<…>>` / `Arc<Mutex<…>>`), task-local access,
   and periodic reporting.
3. **No `Instrumented` trait bound.** Producer metrics are outcome counts
   only (no byte data from CallData). No new trait requirement on PData
   or PipelineFactory.
4. **MetricLevel gates everything.** At `None`, no entry frame interests,
   no timestamp, no byte counting, no metric recording. Each higher
   level adds incremental cost: `Basic` adds auto-subscribe + outcome
   counts, `Normal` adds forward-path byte counting, `Detailed` adds
   timestamp storage + duration histograms.
5. **Bytes on the forward path.** Byte counts are recorded when data
   enters a node, not carried through ack/nack. No outcome breakdown
   for bytes — a single counter per node.

### Phase 1: CallData struct and Context API changes

**Step 1.1 — CallData type change + MetricLevel** (`crates/engine/src/control.rs`)

Rename the existing type alias and add `MetricLevel`:
```rust
// Before:
pub type CallData = SmallVec<[Context8u8; 3]>;

// After:
pub type UserCallData = SmallVec<[Context8u8; 3]>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetricLevel {
    #[default]
    None,
    Basic,
    Normal,
    Detailed,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CallData {
    pub user: UserCallData,
    pub time_ns: u64,   // only populated at Detailed level
}
```

Update `AckMsg::new()`, `NackMsg::new*()` to initialise with
`CallData::default()`. All downstream code that constructs a `CallData`
via `smallvec![…]` changes to use `UserCallData` (or
`CallData { user: smallvec![…], ..Default::default() }`). The
`ProducerEffectHandlerExtension::subscribe_to` trait signature changes
its `ctx` parameter from `CallData` to `UserCallData`.

**Step 1.2 — Modify `subscribe_to` for same-node update-in-place**
(`crates/otap/src/pdata.rs`)

Remove the `debug_assert_ne!`. Add same-node detection:

```rust
pub(crate) fn subscribe_to(
    &mut self,
    mut interests: Interests,
    user_calldata: UserCallData,   // ← was CallData
    node_id: usize,
) {
    if let Some(top) = self.stack.last_mut() {
        if top.node_id == node_id {
            // Same node → merge interests, replace user data.
            // Engine field (time_ns) is preserved.
            top.interests |= interests;
            top.calldata.user = user_calldata;
            return;
        }
        // Different node → inherit RETURN_DATA from predecessor.
        interests |= top.interests & Interests::RETURN_DATA;
    }
    self.stack.push(Frame {
        interests,
        node_id,
        calldata: CallData {
            user: user_calldata,
            ..Default::default()
        },
    });
}
```

This gives the entry-frame merging behaviour (V3 §2) by modifying the
existing method. The frame is created without a timestamp — receivers
stamp it explicitly (see below) and processors/exporters stamp it
when they pull from the queue, both conditional on `MetricLevel::Detailed`.

Also add a helper to stamp the top frame's receive time:

```rust
pub(crate) fn stamp_top_time(&mut self, time_ns: u64) {
    if let Some(top) = self.stack.last_mut() {
        top.calldata.time_ns = time_ns;
    }
}
```

**Step 1.3 — MetricLevel-gated entry frame and auto-subscribe**

Receivers are the pipeline entry point — they set the timestamp
and auto-subscribe interests in their `subscribe_to` effect handler.
Processors and exporters pull data from a channel, so they set
entry frame interests and timestamp via `push_entry_frame` /
`on_pdata_received`. All gated by `MetricLevel`.

**Receiver-side** — add auto-subscribe interests and stamp time in the
receiver `ProducerEffectHandlerExtension` impls (`crates/otap/src/pdata.rs`):

```rust
impl ProducerEffectHandlerExtension<OtapPdata>
    for otap_df_engine::local::receiver::EffectHandler<OtapPdata>
{
    fn subscribe_to(&self, int: Interests, ctx: UserCallData, data: &mut OtapPdata) {
        let metric_level = self.metric_level();
        let mut interests = int;
        if metric_level >= MetricLevel::Basic {
            interests |= Interests::ACKS | Interests::NACKS;
        }
        data.context.subscribe_to(interests, ctx, self.receiver_id().index);
        if metric_level >= MetricLevel::Detailed {
            data.context.stamp_top_time(nanos_since_epoch());
        }
        if metric_level >= MetricLevel::Normal {
            record_consumed_bytes(data.payload_num_bytes());
        }
    }
}
// ... same for shared::receiver
```

**Processor/exporter-side** — push entry frame when PData arrives.

Add `push_entry_frame` to `Context` (`crates/otap/src/pdata.rs`):

```rust
pub(crate) fn push_entry_frame(&mut self, node_id: usize, metric_level: MetricLevel) {
    let mut interests = Interests::empty();
    if let Some(last) = self.stack.last() {
        interests = last.interests & Interests::RETURN_DATA;
    }
    if metric_level >= MetricLevel::Basic {
        interests |= Interests::ACKS | Interests::NACKS;
    }
    let time_ns = if metric_level >= MetricLevel::Detailed {
        nanos_since_epoch()
    } else {
        0
    };
    self.stack.push(Frame {
        interests,
        node_id,
        calldata: CallData {
            user: UserCallData::new(),
            time_ns,
        },
    });
}
```

The `DataReceivedExtension` trait was replaced with a `ReceivedAtNode`
trait on PData itself (`crates/engine/src/lib.rs`):

```rust
pub trait ReceivedAtNode {
    fn received_at_node(&mut self, node_id: usize, metric_level: MetricLevel);
}
```

Implement in `pdata.rs` for `OtapPdata`:

```rust
impl ReceivedAtNode for OtapPdata {
    fn received_at_node(&mut self, node_id: usize, metric_level: MetricLevel) {
        self.context.push_entry_frame(node_id, metric_level);
        if metric_level >= MetricLevel::Normal {
            record_consumed_bytes(self.payload_num_bytes());
        }
    }
}
```

Wire into the engine's processor run loop
(`crates/engine/src/processor.rs`) with a `PData: ReceivedAtNode`
bound on `ProcessorWrapper::start()`:

```rust
while let Ok(mut msg) = message_channel.recv().await {
    if let Message::PData(ref mut pdata) = msg {
        pdata.received_at_node(
            effect_handler.processor_id().index,
            effect_handler.metric_level(),
        );
    }
    processor.process(msg, &mut effect_handler).await?;
}
```

The `ReceivedAtNode` bound propagates to `RuntimePipeline::run_forever()`
(separate impl block) and the `Controller` run methods (separate impl
block), keeping non-run methods (e.g. core allocation) bound-free.
`TestMsg` in `engine::testing` gets a no-op `ReceivedAtNode` impl.

For exporters: the engine cannot intercept the component's run loop,
so exporter components call `pdata.received_at_node(...)` when they
match `Message::PData`. This is a single line added per exporter
implementation.

`set_source_node` stays unchanged — it serves the orthogonal purpose
of source tagging (sender concern).

**Step 1.4 — Add `peek_top` accessor** (`crates/otap/src/pdata.rs`)

```rust
pub fn peek_top(&self) -> Option<&Frame> {
    self.stack.last()
}
```

Consumer metrics in `notify_ack`/`notify_nack` use this to read the
current node's entry frame.

### Phase 2: Metric definitions and handle construction

**Step 2.1 — Metric set definitions** (new file
`crates/engine/src/component_metrics.rs`)

Metric sets are level-gated. Only metrics at or below the configured
level are registered:

```rust
// Basic+: outcome counters
#[metric_set(name = "component.consumed.requests")]
pub struct ConsumedRequestMetrics {
    #[metric(name = "count", unit = "{1}")]
    pub success: Counter<u64>,
    #[metric(name = "count", unit = "{1}")]
    pub failure: Counter<u64>,
    #[metric(name = "count", unit = "{1}")]
    pub refused: Counter<u64>,
}

// Normal+: byte counter (forward-path, no outcome breakdown)
#[metric_set(name = "component.consumed")]
pub struct ConsumedBytesMetrics {
    #[metric(name = "bytes", unit = "{By}")]
    pub bytes: Counter<u64>,
}

// Detailed: duration histogram (per outcome)
#[metric_set(name = "component.consumed.duration")]
pub struct ConsumedDurationMetrics {
    #[metric(name = "success", unit = "{s}")]
    pub success: Histogram<f64>,
    #[metric(name = "failure", unit = "{s}")]
    pub failure: Histogram<f64>,
    #[metric(name = "refused", unit = "{s}")]
    pub refused: Histogram<f64>,
}

// Similar: ProducedRequestMetrics (Basic+), ProducedBytesMetrics (Normal+)
```

No `ProducedFailureMetrics` or producer duration — from the producer's
perspective, all nacks are refusals.

`ComponentMetricsState` wraps the level-appropriate metric sets.
`record_*` methods check the level internally to skip no-op paths.

**Step 2.2 — Handle types and task-local storage**

```rust
pub type LocalComponentMetricsHandle  = Rc<RefCell<ComponentMetricsState>>;
pub type SharedComponentMetricsHandle = Arc<Mutex<ComponentMetricsState>>;

pub enum ComponentMetricsHandle {
    Local(LocalComponentMetricsHandle),
    Shared(SharedComponentMetricsHandle),
}
```

In `entity_context.rs`:
- Add `component_metrics: Option<ComponentMetricsHandle>` to
  `NodeTaskContext` and `NodeTelemetryState`.
- Add `metric_level: MetricLevel` to `NodeTaskContext`.
- Add `NodeTelemetryHandle::register_component_metrics(level)` — registers
  only the MetricSets appropriate for the given level, stores the handle,
  returns `ComponentMetricsHandle`.
- Add `pub fn current_component_metrics() -> Option<ComponentMetricsHandle>`
  — reads from the `NODE_TASK_CONTEXT` task-local.
- Add `pub fn current_metric_level() -> MetricLevel` — reads from
  the `NODE_TASK_CONTEXT` task-local.
- `NodeTaskContext::new()` calls `telemetry_handle.component_metrics()`
  to populate the field (same pattern as V1).

**Step 2.3 — Registration at build time** (`crates/engine/src/lib.rs`)

In `build_node_wrapper`, after creating `NodeTelemetryHandle`:
```rust
let _ = node_telemetry_handle.register_component_metrics(metric_level);
```

**Step 2.4 — Reporting** (all effect handler types)

Add `report_component_metrics(&mut self, reporter)` to each of the
effect handler structs (local/shared × processor/receiver/exporter).
Implementation reads from `current_component_metrics()`. Components
call this during `CollectTelemetry`.

### Phase 3: Consumer metric recording

**Step 3.1 — Instrument `notify_ack` / `notify_nack`**
(`crates/otap/src/pdata.rs`)

In each of the four `ConsumerEffectHandlerExtension` impls, add
level-gated metric recording before routing:

```rust
async fn notify_ack(&self, ack: AckMsg<OtapPdata>) -> Result<(), Error> {
    let level = current_metric_level();
    if level >= MetricLevel::Basic {
        // Basic+: count outcome
        record_consumed_success_count();

        if level >= MetricLevel::Detailed {
            // Detailed: duration histogram
            if let Some(frame) = ack.accepted.context.peek_top() {
                let duration_ns = nanos_since_epoch()
                    .saturating_sub(frame.calldata.time_ns);
                record_consumed_success_duration(duration_ns);
            }
        }
    }
    self.route_ack(ack, Context::next_ack).await
}

async fn notify_nack(&self, nack: NackMsg<OtapPdata>) -> Result<(), Error> {
    let level = current_metric_level();
    if level >= MetricLevel::Basic {
        if nack.permanent {
            record_consumed_refused_count();
        } else {
            record_consumed_failure_count();
        }

        if level >= MetricLevel::Detailed {
            if let Some(frame) = nack.refused.context.peek_top() {
                let duration_ns = nanos_since_epoch()
                    .saturating_sub(frame.calldata.time_ns);
                if nack.permanent {
                    record_consumed_refused_duration(duration_ns);
                } else {
                    record_consumed_failure_duration(duration_ns);
                }
            }
        }
    }
    self.route_nack(nack, Context::next_nack).await
}
```

Helper functions access the metric handle via
`current_component_metrics()` (task-local), same as the V1 pattern.
Byte counting is NOT done here — it happens on the forward path.

### Phase 4: Producer metric recording

**Step 4.1 — InstrumentedControlReceiver**
(`crates/engine/src/component_metrics.rs` or new file)

Wraps `Receiver<NodeControlMsg<PData>>`:

```rust
pub(crate) struct InstrumentedControlReceiver<PData> {
    inner: Receiver<NodeControlMsg<PData>>,
    producer_metrics: ComponentMetricsHandle,
    metric_level: MetricLevel,
}
```

On `recv()`: at `Basic+`, inspects the message and records outcome
counts — `produced.requests{outcome=success}` for Ack,
`produced.requests{outcome=refused}` for any Nack. No byte data is
read from the ack/nack (bytes are counted on the forward path at
send time).

**Step 4.2 — Wire in during pipeline build**
(`crates/engine/src/runtime_pipeline.rs`)

When spawning each node, wrap its control channel receiver:
```rust
let instrumented_ctrl_rx = InstrumentedControlReceiver::new(
    ctrl_rx,
    node_telemetry_handle.component_metrics().unwrap_or_default(),
    metric_level,
);
```

The wrapped receiver is passed into the node's run function instead
of the raw `Receiver`.

### Deferred

| Item | Notes |
|------|-------|
| `num_items` counter | Not included in initial metric sets. Add alongside `consumed.bytes`. |
| Consumer duration histogram | Detailed level records duration. Histogram support requires `otap-df-telemetry` bucket configuration. |
| Fan-out / split semantics | Single frame per component holds regardless; downstream branches each carry the full stack. |
| `DelayedData` interaction | Self-delivery: node already has entry frame; `subscribe_to` merges if called again. |
| Per-node metric level overrides | Current design uses engine-wide level. Per-node overrides may be added later. |

### Step execution order

```
1.1  CallData struct + MetricLevel  (control.rs — type change + enum, ripple to all users)
1.2  subscribe_to in-place          (pdata.rs — Context method)
1.3  push_entry_frame + extension   (pdata.rs — Context method + ReceivedAtNode trait
                                     + impls + engine processor.rs run loop;
                                     auto-subscribe + timestamp + bytes gated by level)
1.4  peek_top accessor              (pdata.rs — Context method)
2.1  Metric set definitions         (new component_metrics.rs — level-gated sets)
2.2  Handle + task-local            (entity_context.rs + MetricLevel in NodeTaskContext)
2.3  Build registration             (lib.rs — pass MetricLevel)
2.4  Effect handler report          (local/ + shared/ effect handlers)
3.1  Consumer instrumentation       (pdata.rs — notify_ack/notify_nack, level-gated)
4.1  InstrumentedControlReceiver    (component_metrics.rs — outcome counts only)
4.2  Build wiring                   (runtime_pipeline.rs — pass MetricLevel)
```

Steps 1.1–1.4 can land in one commit. Steps 2.1–2.4 in a second.
Steps 3.1 and 4.1–4.2 each in their own commit. This order minimises
intermediate breakage.

## Resolved Design Questions

1. **Entry frame at receive time, not send time.** Entry frames are
   pushed when the node receives data from its input channel — not when
   the sender calls `set_source_node`. For processors, the engine's run
   loop calls `on_pdata_received` between `recv()` and `process()`. For
   exporters, the component calls `on_pdata_received` in its PData
   match arm. For receivers (which originate data), `subscribe_to`
   captures the timestamp when pushing the initial frame. Source tagging
   (`set_source_node`) remains unchanged — it is a sender concern,
   orthogonal to entry frames.

2. **Metrics-gated entry frame interests.** Entry frames are always
   pushed for processors and exporters (to support `subscribe_to`
   merge-in-place). However, the frame's interests and engine fields
   depend on `MetricLevel`: at `None`, interests are empty and no
   timestamp is stored; at `Basic+`, `ACKS | NACKS` are included
   (auto-subscribe); at `Detailed`, `time_ns` is also stamped. This
   ensures zero overhead at `None` and incremental cost at higher
   levels.

3. **Bytes counted on forward path, not in CallData.** Byte counts
   do not need outcome breakdown. They are recorded as a simple
   counter when data enters each node (Normal+), avoiding the
   complexity of carrying byte counts through the ack/nack return
   path.

4. **Auto-subscribe installs default ack/nack handler.** Nodes that
   receive ack/nack only because of metrics-driven auto-subscribe
   (no component `subscribe_to` call) get a default handler that
   records the outcome metric and propagates. This ensures pass-
   through processors and exporters participate in metrics without
   requiring component-level ack/nack handling code.

5. **Timestamps only at Detailed level.** Computing and storing
   `nanos_since_epoch()` has non-trivial cost at high throughput.
   The timestamp is only stored when `MetricLevel::Detailed` is
   configured, ensuring `None`/`Basic`/`Normal` incur no timestamp
   overhead.

## Status

**Design revised.** V3 updated with MetricLevel gating (None/Basic/Normal/Detailed),
forward-path byte counting, and metrics-driven auto-subscribe.

### Work items (delta from current implementation)

Current code has Phases 1–4 implemented without metric levels. The
following items bring the implementation in line with the revised design.

#### W1. Add `MetricLevel` enum (`control.rs`)
- Add `MetricLevel` enum (None/Basic/Normal/Detailed) with `#[derive(PartialOrd, Ord)]`
  so `>=` comparisons work.
- Re-export from `crates/engine/src/lib.rs`.
- **No existing code changes** — purely additive.

#### W2. Remove `req_bytes` from `CallData` (`control.rs`)
- Remove `pub req_bytes: u64` field from `CallData`.
- Update `CallData::default()` derivation (field simply gone).
- Ripple: remove all reads of `calldata.req_bytes` and
  `frame.calldata.req_bytes` in:
  - `record_produced_for_control_msg()` (`component_metrics.rs`)
  - All 4 `notify_ack`/`notify_nack` impls in `pdata.rs`
  - `push_entry_frame()` in `pdata.rs` (remove `req_bytes` param)
  - `OtapPdata::received_at_node()` in `pdata.rs` (remove bytes calc)

#### W3. Remove `req_bytes` from `push_entry_frame` signature (`pdata.rs`)
- Change `push_entry_frame(node_id, time_ns, req_bytes)` →
  `push_entry_frame(node_id, metric_level)`.
- Compute `time_ns` inside the method: `nanos_since_epoch()` if
  `metric_level >= Detailed`, else `0`.
- Add `ACKS | NACKS` to interests if `metric_level >= Basic`.
- Update caller `OtapPdata::received_at_node()`.

#### W4. Change `ReceivedAtNode` trait signature (`lib.rs`, `pdata.rs`, `processor.rs`)
- Change `received_at_node(&mut self, node_id: usize, time_ns: u64)` →
  `received_at_node(&mut self, node_id: usize, metric_level: MetricLevel)`.
- Update `OtapPdata` impl: call `push_entry_frame(node_id, metric_level)`.
- Update `TestMsg` no-op impl.
- Update caller in `processor.rs` run loop: pass `metric_level` instead
  of `nanos_since_epoch()`.

#### W5. Forward-path byte counting (`pdata.rs`)
- In `OtapPdata::received_at_node()`, after `push_entry_frame`, if
  `metric_level >= Normal`: call `record_consumed_bytes(self.payload.num_bytes())`.
- Add `record_consumed_bytes()` helper that writes to the task-local
  component metrics handle.
- Same pattern in receiver `subscribe_to` effect handlers.

#### W6. Level-gate receiver `subscribe_to` effect handlers (`pdata.rs`)
- In the 2 receiver `ProducerEffectHandlerExtension` impls
  (local + shared), read `metric_level` from effect handler.
- If `Basic+`: OR in `ACKS | NACKS` to interests before calling
  `context.subscribe_to()`.
- If `Detailed`: call `context.stamp_top_time(nanos_since_epoch())`
  (currently always called — gate it).
- If `Normal+`: call `record_consumed_bytes(data.payload_num_bytes())`.

#### W7. Restructure metric sets (`component_metrics.rs`)
- Replace 5 current metric sets (ConsumedSuccess/Failure/Refused,
  ProducedSuccess/Refused — each with `bytes` and optional `duration_ns`)
  with 3 level-gated groups:
  - **Basic+**: `ConsumedRequestMetrics` (success/failure/refused counters),
    `ProducedRequestMetrics` (success/refused counters).
  - **Normal+**: `ConsumedBytesMetrics` (single bytes counter),
    `ProducedBytesMetrics` (single bytes counter).
  - **Detailed**: `ConsumedDurationMetrics` (success/failure/refused
    histograms).
- Update `ComponentMetricsState` to hold `Option<MetricSet<T>>` for
  each group, populated based on level.
- Update `record_*` methods: outcome counts take no bytes param;
  bytes recorded separately; duration only at Detailed.

#### W8. Level-gate consumer metric recording (`pdata.rs`)
- In all 4 `notify_ack`/`notify_nack` impls: read `current_metric_level()`.
- `Basic+`: increment outcome counter (no bytes, no duration).
- `Detailed`: additionally peek `time_ns`, compute duration, record histogram.
- Remove current `record_consumed_success(req_bytes, duration_ns)` pattern;
  replace with `record_consumed_success_count()` + conditional
  `record_consumed_success_duration(duration_ns)`.

#### W9. Level-gate producer metric recording (`component_metrics.rs`)
- In `record_produced_for_control_msg()`: read `current_metric_level()`.
- `Basic+`: increment outcome counter (success or refused). No bytes
  read from ack/nack.
- Remove `ack.calldata.req_bytes` / `nack.calldata.req_bytes` reads.

#### W10. Plumb `MetricLevel` to effect handlers and entity context
- Add `metric_level: MetricLevel` to `NodeTaskContext` in
  `entity_context.rs`.
- Add `pub fn current_metric_level() -> MetricLevel` reading from
  task-local.
- Add `metric_level()` accessor to effect handler types (or read from
  task-local).
- `NodeTelemetryHandle::register_component_metrics(level)` — only
  register metric sets appropriate for the level.

#### W11. Auto-subscribe default ack/nack handler
- For nodes that have `ACKS | NACKS` from auto-subscribe but no
  component `subscribe_to` call: install a default handler that
  records the consumer outcome metric and calls
  `route_ack`/`route_nack`.
- This may be implementable by making the existing
  `ConsumerEffectHandlerExtension` impls the default (they already
  record + route), so no separate handler is needed — the auto-
  subscribe interests ensure the ack/nack is delivered, and the
  existing `notify_ack`/`notify_nack` code records the metric.
- **Investigate**: confirm that the existing routing works for auto-
  subscribed nodes that don't explicitly call `subscribe_to`. The
  entry frame already has `ACKS | NACKS` interests, so `next_ack`
  will pop it and deliver to that node_id. The node's `notify_ack`
  impl records the consumer metric and re-routes. This should work
  as-is if the effect handler impls are already wired up.

#### W12. Engine-wide `MetricLevel` configuration
- Add `metric_level: MetricLevel` to the pipeline/engine configuration
  struct.
- Pass through to `RuntimePipeline` → node build → effect handlers.
- Default to `MetricLevel::None` (zero overhead unless configured).

#### W13. Update tests
- Update existing component metric tests to exercise level gating.
- Test `None`: no metrics recorded, no auto-subscribe, frames skipped.
- Test `Basic`: outcome counts only, no bytes, no duration.
- Test `Normal`: outcome counts + forward-path bytes.
- Test `Detailed`: outcome counts + bytes + duration histogram.

### Suggested ordering

```
W1   MetricLevel enum              (additive, no breakage)
W2   Remove req_bytes from CallData (breaks compile — fix all refs)
W3   push_entry_frame new sig      (depends on W1, W2)
W4   ReceivedAtNode new sig        (depends on W3)
W5   Forward-path byte counting    (depends on W3, W7)
W7   Restructure metric sets       (can overlap with W2–W4)
W6   Receiver effect handlers      (depends on W1, W7)
W8   Consumer recording            (depends on W7, W10)
W9   Producer recording            (depends on W2, W7, W10)
W10  Plumb MetricLevel             (depends on W1, W7)
W11  Default ack/nack handler      (depends on W8 — may be no-op)
W12  Engine config                 (depends on W10)
W13  Tests                         (final)
```

W1 → W2 → W3 → W4 can be one commit (struct changes + signature
ripple). W7 + W10 as a second. W5 + W6 + W8 + W9 as a third.
W11 + W12 + W13 to close it out.

### Implementation notes (deviations from design)

- **No `InstrumentedControlReceiver` wrapper.** Instead of a separate wrapper struct,
  `record_produced_for_control_msg()` is called directly inside the existing `recv()`
  methods of `MessageChannel` (message.rs, shared/exporter.rs) and `ControlChannel`
  (local/receiver.rs, shared/receiver.rs). This approach covers all node types
  (receivers, processors, exporters) without requiring build-time wiring of a wrapper.
- **Reporting path**: Component metrics are reported alongside channel metrics in
  `PipelineCtrlMsgManager`'s timer loop (pipeline_ctrl.rs), gated by the
  `channel_metrics` telemetry flag.
- **Handle type**: `ComponentMetricsHandle::Local` (Rc<RefCell>) is used for all
  nodes since the current engine runs on a single-threaded local task set.
