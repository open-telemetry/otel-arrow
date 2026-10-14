# Ack/Nack completion protocol: intended invariants

This document states, in plain language, what the engine must guarantee about
the Ack/Nack outcome of each request that enters a pipeline. Each property is
traced to the code that is responsible for it. The TLA+ model in this
directory (`Completion.tla`) checks these properties with the TLC model
checker, one configuration per topology.

Line numbers point into commit `298846b57`. Paths are relative to
`rust/otap-dataflow/crates/`. Short names used below:

| Short name | File |
| --- | --- |
| `pdata.rs` | `otap/src/pdata.rs` |
| `effect_handler.rs` | `engine/src/effect_handler.rs` |
| `pipeline_ctrl.rs` | `engine/src/pipeline_ctrl.rs` |
| `processor.rs` | `engine/src/processor.rs` |
| `server_new.rs` | `otap/src/otap_grpc/otlp/server_new.rs` |
| `common.rs` | `otap/src/otap_grpc/common.rs` |
| `fanout` | `core-nodes/src/processors/fanout_processor/mod.rs` |
| `retry` | `core-nodes/src/processors/retry_processor/mod.rs` |
| `debug` | `core-nodes/src/processors/debug_processor/mod.rs` |

## Why this needs care

A request that a receiver admits with `wait_for_result` must get exactly one
answer: Ack only after its data was delivered, Nack otherwise. The engine
routes that answer back through a stack of frames carried inside the message
context. Nothing in the type system ties a frame to a single message:
`OtapPdata` derives `Clone`, and a copy carries every frame of the original.
A routing error does not crash. It silently produces one of these failures:

| Failure | Effect |
| --- | --- |
| Duplicate outcome | A subscriber acts twice on one request (for example, a retry processor resends data that was already Acked). |
| Wrong outcome first | The client sees a failure for data that was delivered, and its retry duplicates the data. |
| Premature Ack | The client drops data that was never delivered (data loss). |
| Missing outcome | The client waits until its own timeout (hung request). |
| Leaked slot | A slot map or inflight map shrinks capacity until new requests are refused. |

## Vocabulary

- **Request**: one call admitted by a receiver. It is *waiting* when the
  receiver allocated a slot for it and subscribed (`server_new.rs` 486-509).
- **Message**: one `OtapPdata` in a channel, in a node, or parked. Several
  messages can derive from one request (copies, splits).
- **Lineage**: the set of requests a message's data derives from. This is
  a ghost notion: the code does not record it.
- **Frame**: one entry of the context stack (`Frame`, pushed by
  `subscribe_to`, `pdata.rs` 431; `update_send_context`, 602;
  `push_entry_frame`, 705). A frame has a node id, interests (`ACKS`, `NACKS`,
  `RETURN_DATA`, and metrics-only bits), and calldata. In the model each
  frame gets a ghost identity when it is pushed. A copy of a message shares
  the identities of the frames it copied.
- **Subscription**: a frame with `ACKS` or `NACKS` interest.
- **Report**: a node finishing a message calls `notify_ack` or `notify_nack`
  (`pdata.rs` 1311, 1318), which call `route_ack` / `route_nack`
  (`effect_handler.rs` 364, 389). These send `DeliverAck` / `DeliverNack` on
  the bounded completion channel only if the context has frames. The send can
  block when the channel is full.
- **Unwind**: the dispatcher (`PipelineCompletionMsgDispatcher`,
  `pipeline_ctrl.rs` 974) pops frames until one has the matching interest
  (`unwind_frames`, 1237), drops the payload unless that frame has
  `RETURN_DATA` (1278-1280), and sends `NodeControlMsg::Ack/Nack` to that
  frame's node (`send`, 1011). Frames popped on the way are consumed.
- **Delivery**: one Ack or Nack handed to a frame's node.
- **Answer**: what the client sees. The receiver routes a delivery to the
  slot named by the calldata (`route_ack_response` / `route_nack_response`,
  `common.rs` 89-119; `AckSlot::route_response`, `server_new.rs` 129). Slot
  keys are generational, so a delivery for a slot that is already answered or
  cancelled returns `Expired` and changes nothing.
- **Hand-off boundary**: a node that, by its configuration, Acks upstream once
  it has passed data on, without waiting for downstream. In scope: fanout
  with `await_ack: none` (`fanout` 932-952).

## Behavior patterns

The model does not reproduce processors. It builds nodes from the patterns
below. The table names the Rust code that each pattern stands for. The
appendix "Copy and new-context inventory" lists every site in `core-nodes`
and `contrib-nodes` that copies a message or builds a new context, and maps
each one to a pattern here or marks it out of scope.

| Pattern | What happens to the message | Rust |
| --- | --- | --- |
| Admit | Allocate slot, subscribe `ACKS\|NACKS` with the slot key, send, wait. | `server_new.rs` 486-547 |
| Forward | Move the message to the next node. No new frame. | attributes, filter, log_sampling, routers |
| Forward with subscribe | Subscribe, forward. On Ack/Nack, report the same outcome upstream. | `debug` 329-347 (Detailed, 1 port), 273-313 |
| Copy without subscribe | Send `clone()` to each output, no new frame. | `fanout` 932-952 (`none`); `debug` 337-347 (2+ ports) |
| Copy with subscribe on top | Keep the original, send each copy with a fanout frame on top. Report upstream from the original. | `fanout` 968-1046 (`primary`, slim); 486-559, 604-930 (`all`) |
| Detached split | Keep the inbound context in a slot map. Send each part with a fresh context (`clone_detached`) and a subscription. Report upstream once all parts are answered. | partition 328-425; transform 323-440 |
| Park and resend | Subscribe `ACKS\|NACKS\|RETURN_DATA` with the attempt count in calldata. On a transient Nack with payload, resubscribe and resend after a delay. On Ack or final Nack, report upstream. | `retry` 454-606, 610-672 |
| Answer locally | Ack with no output, or refuse with a Nack. | partition 294-298; log_sampling 152-160, 176-178; transform 279-286; `retry` 660-663 |
| Consume without answer | Take the message and return normally without sending or reporting. | `debug` 337-350 with zero main ports, before the fix (see "Findings"); kept in the model, not in `T1.cfg` |
| Fail | `process()` returns an error. The engine stops the node (`processor.rs` 674-676). | `?` paths in transform, partition, log_sampling; `retry` 640-645 |
| Export | Nondeterministically Ack, transient Nack, or permanent Nack, with the payload. | all exporters |

Batch processor internals were checked in Part 1 (`formal/batch_processor`).
Here it is represented only by the detached split pattern.

## Topologies

| Config | Pipeline | Patterns exercised |
| --- | --- | --- |
| T1 | receiver -> P -> exporter | P is one of: forward, forward with subscribe, answer locally, fail, detached split |
| T2 | receiver -> fanout -> exporters A, B | fanout `await_ack` in {`none`, `primary`, `all`} (parallel mode) |
| T3 | receiver -> retry -> exporter | at most 2 retries |
| T4 | receiver -> retry -> fanout(`none`) -> exporters A, B | at most 2 retries |
| T5 | receiver -> debug -> exporters | debug main ports in {0, 1, 2}, Detailed in {off, on} |

Bounds: 2 requests, and capacity 1 on every pdata channel, every node
control channel, and the completion channel. The `*_large.cfg` configs raise
this to 3 requests, capacity 2, and 3 destinations or debug main ports (T2:
2 requests); see the README for the exact bounds and results.

## Ground truth

The model keeps ghost variables that the code does not have:

- the lineage of every message;
- for every frame identity, the list of deliveries it received;
- for every request, the list of deliveries its receiver frame received,
  and which one became the answer;
- for every request and exporter, the outcomes that exporter reported for
  messages of that lineage;
- for every request, whether it was abandoned (see Q5).

## Properties

### Q1. At most one delivery per frame

Every frame identity receives at most one delivery (Ack or Nack).

- Why: every subscriber treats a delivery as the final outcome of the work
  it subscribed for. A retry processor that receives Ack and then Nack for one
  frame resends Acked data. A receiver ignores the second delivery, but only
  because of generational slot keys.
- Code: `unwind_frames` pops each frame of one message at most once
  (`pipeline_ctrl.rs` 1237-1285). The guarantee holds only if each frame
  identity lives in exactly one message that is reported. Copies made with
  `clone()` and no subscription on top break this.

### Q2. Deliveries reach only the lineage's subscribers

A delivery goes only to the node that pushed the frame, with that frame's
own calldata, and only for a message whose lineage includes the request that
the frame was pushed for.

- Code: `unwind_ack` / `unwind_nack` copy the frame's `route` into the
  delivery (`pipeline_ctrl.rs` 1297, 1317) and send it to `frame.node_id`.
  Detached contexts start with no frames (`pdata.rs` 837-846), so a split
  part cannot reach the inbound frames directly.

### Q3. Most recent interested subscriber first

When a message is reported, the delivery goes to the topmost frame that has
the matching interest. Every frame popped before it lacks that interest.

- Code: `unwind_frames` loop (`pipeline_ctrl.rs` 1246-1284). Same-node
  subscriptions merge into the top frame (`pdata.rs` 437-444, 603-615), so a
  node never has two adjacent frames.

### Q4. Ack only after delivery

When a waiting request is answered with Ack, every exporter that the
topology requires for it has Acked at least one message of its lineage.
The required exporters are those reachable from the receiver without passing
a hand-off boundary. For fanout `primary`, only the primary destination is
required. For the debug processor, only its first main port is required
(D2).

- Why: an Ack lets the client drop its data.
- Code: fanout `process_ack` / `mark_complete` (`fanout` 604-613, 751-857);
  slim path `process_ack_slim_primary` (996-1017); retry `handle_ack`
  (`retry` 454-470); split processors report upstream only when their last
  part is answered (transform `handle_ack_nack_inbound`, 443-470).
- Note: duplicates caused by a retry after a reported Nack are intended
  (delivery is at least once). Duplicates caused by a Q1 violation are not.

### Q5. Drained means answered

When no message is in a channel, in a node, parked for retry, or waiting in
the completion channel or the dispatcher's retry buffer:

- every admitted waiting request has exactly one answer, or was abandoned;
- every slot map and inflight map in the topology is empty (receiver
  `AckSlot`, fanout `inflight` and `slim_inflight`, split processor slots).

A request is *abandoned* when, and only when:

- a node failed (pattern Fail) while it held a message or a frame of that
  request; or
- the client cancelled (the `SlotGuard` drop, `server_new.rs` 368-378).

This is a safety property (a state predicate). It detects hung requests and
leaked slots without fairness assumptions. What happens to the work of a
failed node is Part 3 (shutdown and drain). Here the property only requires
that every unanswered request is accounted for by an explicit abandonment.

### Q6. No silent drop in the dispatcher

Every `DeliverAck` / `DeliverNack` the dispatcher receives ends in exactly one
of:

- a delivery to the target node's control channel;
- the "dropped, no interest" counter (`record_ack_dropped_no_interest`,
  `pipeline_ctrl.rs` 1302; Nack at 1321);
- a drop that increments a dedicated counter or emits a log.

Known paths that drop silently today, to be confirmed by the model:

| Path | Line |
| --- | --- |
| Target control channel closed (`send`) | `pipeline_ctrl.rs` 1039 |
| Target control channel closed (`drain_pending_sends`) | 1064 |
| No control sender for the node id (`send`, `drain_pending_sends`) | 1014, 1047 |
| Dispatcher exits on completion channel close with `pending_sends` non-empty | 1187 |

For the closed-channel paths, the "attempted" counter has already been
incremented while "delivered" is not. That gap is not a dedicated signal.

Within Part 2, a control channel closes only when its node fails. The
completion channel closes only when every node has exited, which is
shutdown (Part 3). The model includes that exit only as a final step after
all nodes have stopped.

### Q7. Returned payload only where asked

A delivery carries the payload if and only if the reporter included it and
the receiving frame has `RETURN_DATA`.

- Code: frames inherit `RETURN_DATA` from the frame below
  (`subscribe_to`, `pdata.rs` 446; `update_send_context`, 620-622;
  `push_entry_frame`, 715-718). The payload is dropped at the first
  interested frame that lacks it (`pipeline_ctrl.rs` 1278-1280). Retry
  depends on it: a transient Nack without payload ends the request
  (`retry` 529-541).
- Priority: low.

### Q8. Fanout `none` answers once, with Ack

For a request that passes through fanout with `await_ack: none`, the
upstream subscriber of that request receives exactly one delivery, and it is
an Ack.

- Code: `process_fire_and_forget` (`fanout` 932-952) sends `pdata.clone()`
  to each destination with no subscription, then reports Ack upstream.
- Why it is suspect: each copy keeps the upstream frames. An exporter's
  outcome for a copy unwinds past fanout to the upstream subscriber. The
  sends `await`, so a full destination channel lets another exporter answer
  a copy before fanout reports its own Ack.
- The same check applies to the debug processor with two or more main ports
  (T5), which copies the same way and has no hand-off Ack of its own.

## Findings

TLC found four violations at `298846b57`. Each was reproduced by a failing
Rust test, fixed, and the fix mirrored in the model.

| Property | Config | What happened | Fix | Rust tests |
| --- | --- | --- | --- | --- |
| Q1, Q8 | T2 `none` | A fanout `none` copy kept the upstream frames, so its outcome answered the upstream subscriber again. A copy's Nack could reach the client before fanout's Ack (14-state trace). | Copies are `clone_without_context()` (`fanout` 943). | `fire_and_forget_copy_ack_does_not_reach_upstream` |
| Q1, Q8 | T4 | Same cause, with retry upstream: a copy's Nack reached retry's frame after the client was Acked, and retry resent the data. | Same fix. | `fire_and_forget_copy_nack_does_not_reach_acked_subscriber` |
| Q1, Q4 | T5, 2 ports | Every main port got a copy with the upstream frames: the client was Acked by one port before the other delivered, and in Detailed mode the shared debug frame forwarded two answers. | Only the first main port keeps the context (D2). | `outports_two_main_ports_answer_upstream_once`, `outports_two_main_ports_detailed_answer_upstream_once` |
| Q5 | T5, 0 ports | With every connected port a debug port, the request was consumed with no answer. | Ack the request; no Detailed subscription. | `outports_without_main_port_answers_upstream` |

Q6 is violated at shutdown (`T1_exit.cfg`) and left for Part 3 (D3).

A consequence of the fanout fix: a retry processor in front of fanout `none`
never sees a Nack, so it never retries. This is the documented meaning of
`none` (downstream outcomes are ignored). Before the fix, retries there only
happened through the bug, and they resent data to every destination.

## Behaviors that are intended, not violations

- **Fanout `none` does not wait for downstream.** Data past a hand-off
  boundary may be lost without the client knowing. Q4 does not apply past
  the boundary. Q1 and Q8 still do. The debug processor is not a hand-off
  boundary: it never Acks on its own (except as a sink with no main port),
  so Q4 applies to its first main port, which keeps the request context.
  Its other main ports receive detached copies (D2).
- **Fanout `primary` ignores the secondary destination's outcome.** The
  secondary copy's frame is consumed by fanout (`fanout` 1006-1008,
  1029-1031).
- **A late or duplicate delivery to a receiver changes nothing.** The
  generational slot key returns `Expired`, and the receiver counts it as a
  failed acknowledgement (`otlp_receiver/mod.rs` 398-421). This hides Q1
  violations from the client but not from intermediate subscribers.
- **At-least-once duplicates.** A retry after a Nack can send data again to
  an exporter that already Acked it (fanout `all`, or a split processor).
- **Untracked messages.** A message with no frames is reported nowhere
  (`effect_handler.rs` 368, 393) and its outcome is dropped by design.

## Assumptions about the environment

- Single-threaded runtime per pipeline. Each model action is one
  uninterrupted run of a node between two `await` points that can block.
  A send to a channel with free capacity does not yield.
- Exporters answer every message they take, exactly once, in any order.
- Each node control channel and data channel is FIFO. Which of its inputs a
  node handles next (control or data) is nondeterministic. Control-plane
  priority and fairness are Part 4.
- Timers, deadlines, and the retry delay are abstracted: a parked message can
  resume at any time.

## Out of scope

Shutdown and drain beyond the dispatcher exit in Q6, control-plane fairness
and ordering, batch internals, router parking, the durable buffer, fanout
sequential mode, timeouts and fallbacks, fanout `max_inflight` backpressure,
metric values (metrics-only frames are modeled as frames without Ack/Nack
interest), exporters' internal retries, proofs, and CI integration.

## Decisions to revisit

We keep fixes minimal. Each entry records a simplification we accepted,
why, and what would make us reopen it.

### D1. Fanout `none` copies drop the flow-metric accumulator

- Decision: fanout `await_ack: none` sends `clone_without_context()` copies
  (fix for Q1 and Q8). The copies carry no frames and no flow-metric
  accumulator.
- Why: before the fix every copy carried a duplicate accumulator, so flow
  compute time past the fanout was counted once per destination. Dropping it
  is simpler and no less accurate than giving it to one copy.
- Revisit when: someone needs flow metrics measured across a fanout.

### D2. Debug processor with several main ports answers from the first port

- Decision: the first main port in sorted order keeps the request context;
  the other main ports receive `clone_without_context()` copies (fix for Q1
  and Q4 in T5).
- Why: smallest change, a single port behaves exactly as before, and the
  processor never Acks before delivery. Aggregating all ports would duplicate
  fanout `all`.
- Trade-off: only the first main port decides the client's answer; the other
  ports are fire-and-forget.
- Revisit when: debug output ports are used as a real multi-destination
  router. The intended answer is a fanout after the debug processor.

### D3. Silent drops in the dispatcher stay silent in Part 2

- Decision: no signal is added for the Q6 paths (`pipeline_ctrl.rs` 1014,
  1039, 1047, 1064, and `pending_sends` at the exit at 1187).
- Why: a proper fix needs a completion metric design, and in the model the
  only reachable path is the exit at shutdown (TLC trace: T1 with
  `AllowExit`, a Nack parked in `pending_sends` when the loop breaks).
- Revisit when: Part 3 (shutdown and drain) models the exit.

### D4. T4 checked at reduced bounds

- Decision: Q2 to Q7 for T4 were checked with 1 request and 1 retry. The
  full bounds (2 requests, 2 retries) did not finish before the fix, because
  every duplicate Nack started a new retry attempt.
- Revisit when: after the fanout `none` fix, re-run T4 at full bounds.
- Resolved: with the fix in the model, T4 passes every property at full
  bounds (5,490 distinct states).

### D5. No Rust test for the Nack-before-Ack race

- Decision: the race where a client receives a copy's Nack before fanout's
  own Ack (TLC trace, T2 `none`, 14 states) has no Rust test. It needs the
  deterministic simulation suite to force a blocked send.
- Why: the fanout `none` fix removes its cause; the routing half is covered
  by `fire_and_forget_copy_nack_does_not_reach_acked_subscriber`.
- Revisit when: a similar copy pattern appears that the fix does not cover.

## Appendix: Copy and new-context inventory

Every site in `crates/core-nodes` and `crates/contrib-nodes` that copies an
`OtapPdata`, or builds a new context, and sends the result on. Line numbers
are at `298846b57`. Paths are relative to `rust/otap-dataflow/crates/`.

How it was built: a search of non-test code for `.clone()` of a message,
`clone_detached`, `clone_without_context`, `OtapPdata::new`,
`Context::default`, `Context::with_capacity`, `into_parts`, and
`send_message*` with a clone argument. Every hit was then read by hand.
Exporters only answer the message they receive, so they appear only where
they keep a copy.

"Own subscription" means the node pushes its own Ack/Nack frame on each
copy it sends. A copy made with `clone()` and no own subscription keeps the
sender's frames, which is the hazard behind Q1.

### Copies of a message

| Site | Pattern | Own subscription | Copy kind | Status |
| --- | --- | --- | --- | --- |
| `core-nodes/src/processors/fanout_processor/mod.rs` 943 (`await_ack: none`) | Copy without subscribe | no | `clone()` | Fixed: `clone_without_context()` (Q1, Q8) |
| `fanout_processor/mod.rs` 986, 981 (`primary`, slim) | Copy with subscribe on top | yes | `clone()`; original kept in `slim_inflight` | OK |
| `fanout_processor/mod.rs` 510 (`all` and full path) | Copy with subscribe on top | yes | `clone()`; original kept in `inflight` | OK |
| `fanout_processor/mod.rs` 661 | Nack upstream from the kept original | n/a | `clone()` of the original, entry then removed | OK: one Nack per request |
| `core-nodes/src/processors/debug_processor/mod.rs` 337-346 (2 or more main ports) | Copy without subscribe (Detailed: shared subscription) | no (Detailed: one frame shared by all copies) | `clone()` | Fixed: first port keeps the context, others `clone_without_context()` (Q1, Q4, D2) |
| `debug_processor/mod.rs` 337-350 (no main port) | Consume without answer | n/a | none | Fixed: Ack after the output (Q5) |
| `debug_processor/mod.rs` 346 (no output ports) | Forward | no | `clone()`, original consumed locally | OK: one copy |
| `core-nodes/src/processors/debug_processor/output.rs` 389 | New lineage per debug port | no | `clone()` of a message with no frames | OK: outcomes have no subscriber |
| `core-nodes/src/exporters/otap_exporter/mod.rs` 992 | Kept for a Nack if the stream fails | n/a | `clone()` | OK: settled once (`FirstPdataOutcome`); not modeled |

### New or detached contexts

| Site | Pattern | Own subscription | Copy kind | Status |
| --- | --- | --- | --- | --- |
| `core-nodes/src/processors/transform_processor/mod.rs` 323, 348, 392 | Detached split | yes, unless no tracking is needed | `clone_detached` | OK |
| `transform_processor/mod.rs` 269, 696 | Forward | no | move | OK |
| `transform_processor/mod.rs` 456 | Report upstream from the kept inbound context | n/a | move | OK |
| `core-nodes/src/processors/partition_processor/mod.rs` 374 | Detached split | yes, unless no tracking is needed | `clone_detached` | OK |
| `partition_processor/mod.rs` 329 | Inbound context kept in the slot map | n/a | `clone()`, never sent | OK |
| `partition_processor/mod.rs` 296, 320, 170 | Answer locally / forward / report upstream | no | move | OK |
| `core-nodes/src/processors/temporal_reaggregation_processor/mod.rs` 709, 864 | Detached merge | yes | `Context::with_capacity` | OK; a passthrough Nack can answer early (semantic, not protocol) |
| `temporal_reaggregation_processor/mod.rs` 831 | New context, nobody waiting | no | `Context::default` | OK |
| `temporal_reaggregation_processor/mod.rs` 672, 467, 537 | Forward / report upstream | no | move | OK |
| `core-nodes/src/processors/batch_processor/mod.rs` 983, 1071, 1238, 1262 | Detached split and merge | yes | new contexts | OK; checked in Part 1 |
| `core-nodes/src/processors/retry_processor/mod.rs` 588-595, 668 | Park and resend | yes | move | OK |
| `core-nodes/src/exporters/topic_exporter/mod.rs` 208, 324, 384 | Transport boundary | n/a | `clone_without_context()` | OK |
| `core-nodes/src/receivers/topic_receiver/mod.rs` 580-593 | Start a lineage | yes, when tracked | `clone_without_context()` | OK |
| `core-nodes/src/processors/durable_buffer_processor/bundle_adapter.rs` 670, 685 | New lineage after persistence | no | `Context::default` | Out of scope |
| Receivers: `otap/src/otap_grpc/otlp/server_new.rs` 502; `host_metrics_receiver` 763; `internal_telemetry_receiver/metrics.rs` 243; `journald_receiver` 895 | Admit / start a lineage | yes when waiting, else none | new context | OK |

### Nodes that only move the message

These take the message apart and send it on, or answer it, without copying
it: `attributes_processor` (423, 486), `filter_processor` (159, 231),
`log_sampling_processor` (118, 154, 176), `log_parser_processor` (94, 112),
`content_router` (786, 824), `signal_type_router` (427, 468), and in
`contrib-nodes`: `condense_attributes_processor` (650, 681),
`recordset_kql_processor` (154, 190), `resource_validator_processor` (527).
