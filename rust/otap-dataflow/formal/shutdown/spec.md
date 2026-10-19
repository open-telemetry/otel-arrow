# Graceful shutdown and drain: intended invariants

This document states, in plain language, what the engine must guarantee when
a pipeline shuts down: which work is delivered, which is abandoned, and how
long it may take. Each property is traced to the code that is responsible
for it. The TLA+ model in this directory (`Shutdown.tla`) checks these
properties with the TLC model checker, one configuration per topology.

The sequence, properties and leads below describe the code at `298846b57`,
where TLC and Rust tests confirmed the leads. Section "Fixes" lists what
changed; README.md has the results.

Line numbers point into commit `298846b57`. Paths are relative to
`rust/otap-dataflow/crates/`. Short names used below:

| Short name | File |
| --- | --- |
| `pipeline_ctrl.rs` | `engine/src/pipeline_ctrl.rs` |
| `message.rs` | `engine/src/message.rs` |
| `scheduler.rs` | `engine/src/node_local_scheduler.rs` |
| `processor.rs` | `engine/src/processor.rs` |
| `runtime_pipeline.rs` | `engine/src/runtime_pipeline.rs` |
| `otlp_receiver` | `core-nodes/src/receivers/otlp_receiver/mod.rs` |
| `common.rs` | `otap/src/otap_grpc/common.rs` |
| `fanout` | `core-nodes/src/processors/fanout_processor/mod.rs` |
| `retry` | `core-nodes/src/processors/retry_processor/mod.rs` |
| `router_admission` | `core-nodes/src/processors/exclusive_router_admission.rs` |

## Why this needs care

Shutdown is the one moment when every node is asked to stop while work is
still moving. A request admitted just before shutdown must still get one
answer, and the pipeline must still stop in bounded time. The failure modes
are:

| Failure | Effect |
| --- | --- |
| Silent drop | Data parked in a node or channel is discarded and nobody is told. The upstream subscriber never gets an answer. |
| Missing outcome | A waiting client gets no answer until its own timeout. |
| Drain stall | A graceful drain cannot finish and waits for the deadline, so it turns into a forced one. |
| Overrun | A node keeps running past the deadline, so the pipeline does not stop in bounded time. |

## The intended sequence

1. **Request.** The runtime-control manager receives `Shutdown`
   (`pipeline_ctrl.rs` `run` 429, handler 532-600). It records the deadline,
   cancels every runtime and telemetry timer (551-552), ignores later
   `StartTimer` (647-660), and sends `DrainIngress` to every receiver
   (558-566). A pipeline with no receivers goes straight to step 3
   (573-594). With the L3 fix, timers are frozen at step 3 instead.
2. **Receiver drain.** Each receiver stops admitting new work, waits for its
   in-flight `wait_for_result` slots to be answered or force-Nacks them at
   the deadline, and then sends `ReceiverDrained`. Reference: OTLP receiver
   (`otlp_receiver` 736-800; `AckRegistry::force_shutdown`, `common.rs` 74).
3. **Downstream shutdown.** On the last `ReceiverDrained` (601-646) the
   manager sends `Shutdown` to every processor and exporter.
4. **Drain by closure.** Each node inbox latches `Shutdown` (`message.rs`
   622-636, 660-676, 702-716) and releases it to the node only when its pdata
   input is closed and empty and its local scheduler is drained
   (`shutdown_drain_complete`, 365-379). Processors keep honoring
   `accept_pdata()` during the drain; exporters drain their buffered input
   regardless (`DrainPolicy` 268-276, `shutdown_drain_accepts_pdata`
   361-363).
5. **Retained work.** On latch, the local scheduler turns each delayed
   resume (`requeue_later`) into an immediate `ResumeData` and drops every
   payload-free wakeup (`scheduler.rs` `begin_shutdown` 441-460, `is_drained`
   463-465). After the latch, new `requeue_later` and `set_wakeup` calls are
   rejected (257, 283). With the L7 fix, pending wakeups are delivered once
   instead of dropped.
6. **Deadline.** If the deadline passes first, the manager triggers the
   forced-shutdown signal, sends `Shutdown` to every non-receiver and every
   receiver still pending, and exits (468-504). Processors exit at once on
   the signal (`processor.rs` 812). Each inbox also releases its latched
   `Shutdown` at the deadline on its own (`message.rs` 506-512, 550-556).
   With the L6 fix, receivers and exporters are cancelled 1 s after the
   signal if they have not stopped on their own.
7. **Dispatcher exit.** The completion dispatcher exits when its channel
   closes (`pipeline_ctrl.rs` 1160-1235, break at 1187). The channel closes
   when every holder of a completion sender has dropped it. Holders are node
   effect handlers and their clones; the original is dropped at
   `runtime_pipeline.rs` 918.

## Vocabulary

- **Request**: one unit of work admitted by a receiver. It is *waiting* if
  the receiver keeps a slot for it until it is answered (`wait_for_result`).
- **Outcome**: the Ack or Nack that answers a request at its receiver. As in
  Part 2, it travels back through the dispatcher; the frame stack is not
  modeled here.
- **Real outcome**: an outcome that some exporter or processor decided from
  the data path (an exporter Ack or Nack, a processor Nack).
- **Forced Nack**: a Nack the receiver produces itself at the deadline
  (`force_shutdown`).
- **Shutdown request**: the moment the manager accepts `Shutdown`.
- **Released**: the inbox hands the latched `Shutdown` to its node.
- **Stopped**: the node's task has returned. Its control and pdata receivers
  are dropped, and its pdata senders are dropped, which closes the input of
  every node downstream of it once all its upstreams have stopped.
- **Retained payload**: a payload a node parked in its local scheduler with
  `requeue_later`.
- **Deadline fired**: the manager has passed its deadline branch (468).

## Model abstractions

The model tracks lifecycle, not frames. Each actor below is mapped to the
Rust function it mirrors; actions in `Shutdown.tla` use these names.

| Actor | Model | Rust |
| --- | --- | --- |
| Manager | Phases `running`, `drainIngress`, `drainDown`, `exited`. The deadline branch triggers the forced signal and exits. Control sends are non-blocking; a full channel parks the send in `pending_sends`, which is lost when the manager exits. | `RuntimeCtrlMsgManager::run`, `send`, `drain_pending_sends` (`pipeline_ctrl.rs` 429, 915, 941) |
| Receiver | Admits up to N requests, each waiting or fire-and-forget (config). On `DrainIngress`: stop admitting, wait for every waiting slot, then send `ReceiverDrained` and stop. At the deadline: force-Nack every open slot; with the L6 fix it may also be force-stopped. A config flag lets it stop **without** sending `ReceiverDrained` (an error path). | `otlp_receiver` 736-800, 847-855; `AckRegistry::force_shutdown`; `start_until_forced_shutdown` |
| Inbox | `RecvResult` follows the order of the tests in `recv_with_policy`: the early closed check (413-426), the latch, release on closure, the fairness burst path (457-472, 597-607), control before local items, the deadline, and the closed-input shutdown in normal mode (605, 654, 725). `consecutive_control` is kept, capped at `BurstLimit`, which stands for `CONTROL_BURST_LIMIT` (32). | `InboxCore::recv_with_policy` (`message.rs` 396-748) |
| Local scheduler | Delayed resumes and wakeups; `begin_shutdown` converts resumes, and clears wakeups (before the L7 fix) or delivers each once (after). | `scheduler.rs` 256, 274, 441-465 |
| Processor | One of the variants in the topology table. | see table |
| Exporter | Each message it takes is answered with Ack or Nack, or (config) dropped with no outcome, or (config) stuck forever. At `Shutdown` it waits for its in-flight answers; a bounded exporter gives up at the deadline and Nacks the rest, an unbounded one (L6) waits for the stuck ones forever. | generic; affected exporters listed under S5b |
| Dispatcher | Moves outcomes from nodes to the receiver. Has `pending_sends`. Exits only when every sender holder has stopped. A send to a stopped node is dropped and recorded. | `PipelineCompletionMsgDispatcher::run`, `send`, `drain_pending_sends` (1160, 1011, 1044) |
| Deadline | A nondeterministic event that may fire at any step after the shutdown request, or is disabled by config. No clocks. | `pipeline_ctrl.rs` 468-504 |

## Topologies

| Config | Pipeline | Variants | Leads |
| --- | --- | --- | --- |
| T1 | receiver -> stateless processor -> exporter | receiver may skip `ReceiverDrained`; dispatcher exit | L1, S7 baseline |
| T2 | receiver -> holding processor (local wakeup flush, batch-like) -> exporter | - | wakeups at the latch, **L9** |
| T3 | receiver -> retry (`requeue_later`) -> exporter | admission gating on/off; receiver waiting or fire-and-forget; any order of "upstream stopped" and "Shutdown delivered" | L2, **L2b** |
| T4 | receiver -> fanout (`await_ack: all`) -> 2 exporters | `T4.cfg`: `max_inflight` gating on/off, timeouts off; `T4_timeout.cfg`: a destination timeout (runtime `TimerTick`) and a destination that never answers; `T4_live.cfg`: S5a, S5b | L3, L4 |
| T5 | receiver -> admission-gated processor -> exporter | `gate = "never"` (`T5_closed.cfg`, L5) or `gate = "router"` (`T5.cfg`, parks a payload keyed by a wakeup, L7) | L5 (expected), **L7** |

Each main config lets TLC choose the deadline per behavior: on, or off with
deadlock detection (S7). Extra configs isolate one expected violation or one
property:

| Config | Purpose | Expected |
| --- | --- | --- |
| `T1_skip.cfg` | receiver stops without `ReceiverDrained`, no deadline | expected a deadlock (D2); it passes, see S7 |
| `T1_overrun.cfg` | exporter whose shutdown waits forever (L6) | S5b fails before the L6 fix, holds after it |
| `T1_late.cfg`, `T4_late.cfg` | S8 only | S8 fails (D6) |
| `T5_closed.cfg` | admission closes for good (L5) | deadlock (D3) |

S8 is kept out of the main configs because the dispatcher's silent drop
would hide every other counterexample.

Bounds: 2 requests (T5: 3, so that one message can sit in the exporter
channel, one parked and one buffered), capacity 1 on every channel, at most
1 retry. The `*_large.cfg` configs use capacity 2 and one more request (T4:
2 requests); README.md "Bounds" has the details.

## Ground truth

The model keeps ghost variables that the code does not have:

- `outs`: for every request, the outcomes its receiver slot received, each
  tagged with its source: `exp` (exporter), `proc` (processor decision, for
  example a fanout timeout), `shut` (a Nack produced because of shutdown),
  or `forced` (deadline);
- `flags`: the label of every property a step violated. Retained payloads
  dropped at a node's stop, releases that broke the drain contract, ticks
  after the request, and outcomes dropped at a closed control channel each
  set a flag at the step where they happen.

## Properties

Modeling and fix priority was L2b, L7, L6, then the fanout leads L3 and L4.
Everything else is recorded, not fixed.

### S1. Ingress closes first

No processor or exporter receives `Shutdown` before every receiver has sent
`ReceiverDrained`, unless the deadline fired.

- Code: downstream `Shutdown` is sent only in the last-`ReceiverDrained`
  branch (`pipeline_ctrl.rs` 610-645), the no-receiver branch (573-594), and
  the deadline branch (481-489).
- Note: a node can still *stop* before that, because its input can close
  (see S3 and L2b). S1 is about the `Shutdown` message only.

### S2. Timers freeze

No runtime `TimerTick` is generated once timers are frozen.

- At `298846b57` timers freeze at the shutdown request:
  `tick_timers.cancel_all()` and `telemetry_timers.cancel_all()` (551-552);
  `StartTimer` is ignored while draining (649-653); while draining the next
  wake-up is the deadline only (506-508).
- With the L3 fix they freeze when downstream `Shutdown` is sent
  (`freeze_timers`), so timers keep running while receivers drain. This
  still freezes timer-driven work before any downstream `Shutdown`, which is
  the documented intent of the receiver-first sequence.
- Ticks that were already queued in a node's control channel, or in the
  manager's `pending_sends`, may still be delivered after the freeze. That
  is intended (D4).

### S3. Drain by closure

A node is released from drain only when its pdata input is closed and empty
and its local scheduler is drained, or the deadline fired.

- Code: `shutdown_drain_complete` (`message.rs` 365-379) guards the normal
  release (437-444).
- Exceptions at `298846b57`, confirmed by TLC and fixed by the L2b fix:
  - the early closed check (413-426) shuts the inbox down when admission is
    closed and the input is closed and empty, without checking the
    scheduler (L2);
  - in normal mode (no `Shutdown` latched yet), a closed pdata input calls
    `closed_pdata_shutdown` (342-348) at 605, 654 and 725, again without
    checking the scheduler (L2b);
  - in drain mode, the fairness burst path (457-472) runs before control and
    local items once `consecutive_control` reaches `CONTROL_BURST_LIMIT`.
    Due `ResumeData` count as control (`pop_local_due`, 381-386), so a node
    with more than 32 retained payloads reaches it while the scheduler still
    holds some. If the input is closed and empty, `try_recv` returns `Closed`
    and `Shutdown` is released without the scheduler check (L2c, a variant
    of L2b found while writing the model);
  - the other drain-mode `Closed` releases (514-522, 563-571) run after
    `pop_local_due` (495), so they matter only if retained work can appear
    while the node is blocked in `select!`. The model found no such case;
    the fix guards them anyway.
- The check is made at release, after `begin_shutdown` has converted the
  delayed resumes; payload-free wakeups do not count.

### S4. Retained payloads come back

Every payload parked with `requeue_later` is delivered back to its node as
`ResumeData` before the node stops, unless the deadline fired.

- Code: `begin_shutdown` converts delayed resumes into `due_now`
  (`scheduler.rs` 447-452); `is_drained` keeps the inbox open until they are
  popped (463-465).
- Gap at `298846b57` (L2b, confirmed, fixed): `InboxCore::shutdown`
  (`message.rs` 314-322) calls `begin_shutdown` and then drops both
  receivers. The next `recv_with_policy` returns `Closed` at 404-406, the
  processor loop ends (`processor.rs` 657), and `due_now` is never popped.
- The model applies the same check to the payloads a processor keeps in its
  own state: the holding processor's buffer (T2) and the router's parked
  message (T5). Each must be sent or Nacked before the node stops.

### S5a. Processors stop by the deadline

Every processor stops within one step after the deadline fires.

- Code: forced-shutdown signal (`processor.rs` 810-814), wired to processors
  only at `298846b57` (`runtime_pipeline.rs` 716, 796).
- Holds in every config, before and after the fixes.

### S5b. Every node stops by the deadline plus a bounded grace

Every node, including receivers and exporters, stops within a bounded number
of steps after the deadline fires.

- Fails at `298846b57` (L6), confirmed by TLC (`T1_overrun.cfg`). The
  forced-shutdown signal does not reach exporters or receivers, and these
  shutdown waits have no deadline:
  - `otlp_grpc_exporter` 431-443 and `otlp_http_exporter` 428-447 (await
    every in-flight export);
  - `otap_exporter` 707-738 (join workers; TODO at the module header,
    #3870);
  - `azure_monitor_exporter/exporter.rs` 400-410, 429-454 (drain in-flight);
  - `otlp_receiver` 861-880 (await both servers after `Shutdown`).
- The model represents these as one generic exporter that may wait forever
  for an answer (`T1_overrun.cfg`). The L6 fix extends the forced-shutdown
  signal to receivers and exporters; S5b then holds in every config.
- A second way to overrun, found while writing the model: at the deadline
  the manager makes one last `drain_pending_sends` pass and then exits
  (448, 503). A `DrainIngress` or `Shutdown` still parked in
  `pending_sends` because the receiver's control channel was full is never
  delivered, and the receiver keeps running. TLC reached this in T1, T2 and
  T5 (L8, 15 steps); the L6 fix covers it.
- S5a and S5b are checked as liveness properties (`Dl ~> ...`) under weak
  fairness of every node, the manager and the dispatcher. "Bounded" means
  that, in the finite model, no behavior after the deadline avoids the stop
  forever.

### S6. Every admitted request ends with exactly one outcome

Every admitted waiting request ends with exactly one outcome at its
receiver: its real outcome, or a forced Nack at the deadline. Never silence.

- Code: receiver drain waits for an empty registry (`otlp_receiver`
  754-755) and force-Nacks at the deadline (745-750, 852).
- Covers L1 and L4. For fire-and-forget requests the property is about the
  retained payload instead (S4).
- Forced Nack semantics: see D1.

### S7. Graceful when possible

If every exporter answers (or a configured timeout covers the one that does
not), no processor keeps admission closed forever, and every receiver sends
`ReceiverDrained`, then every admitted waiting request gets its real outcome
and the deadline never fires.

- Checked cheaply: in behaviors with the deadline disabled, TLC deadlock
  detection (`CHECK_DEADLOCK TRUE`) is on. Any stuck state before "all
  stopped" is a drain stall. Invariant `GracefulOutcomes` checks the "real
  outcome" half: no `shut` or `forced` outcome without a deadline.
- A real outcome is an exporter Ack or Nack, or a processor's own decision
  such as a fanout timeout Nack.
- Covers L3: at `298846b57` the drain stalls in `T4_timeout.cfg` (12
  steps); with the L3 fix it completes.
- The receiver variant that skips `ReceiverDrained` (`T1_skip.cfg`) was
  expected to stall here, as the stand-in for the receiver error paths in
  D2. It does not: the receiver's pdata sender drops when it stops, so the
  next node drains by channel closure and the pipeline stops anyway. The
  manager then exits when its channel closes. A receiver that returns an
  error still aborts the pipeline join, which the model does not cover.

### S8. No late outcome is lost silently

An outcome sent to a node that has already stopped is counted or flagged,
never just dropped.

- Code: today the dispatcher drops it silently when the target control
  channel is closed (`pipeline_ctrl.rs` 1039, 1064) and when it exits with
  `pending_sends` non-empty (1187). Part 2 D3 deferred this to Part 3.
- Still fails after the fixes, in `T1_late.cfg` and `T4_late.cfg`: an
  answer that arrives for a request that was already answered is dropped
  with no signal. Recorded as D6.

## Leads

| Lead | Hypothesis | Verdict | Checked by |
| --- | --- | --- | --- |
| L1 | Dispatcher exit drops an answer a receiver still waits for. | Not reachable: the dispatcher exits only after every sender holder has stopped, and receivers answer every open slot before they stop (see below). Answers for already-answered requests are still dropped silently (S8, D6). | S6 in all configs; S8 in `T1_late`, `T4_late` |
| L2 | Early closed check drops retained `ResumeData` of a node that gates admission. | Latent: no node both gates admission and uses `requeue_later`. The L2b fix closes it too. | S3, S4 in T3 (gating on) |
| L2b | A closed input in normal mode shuts the inbox down without draining the scheduler. | Confirmed (TLC, 12 steps; DST test). Fixed. | S3, S4 in T3 |
| L2c | The drain-mode burst path releases on a closed input without draining the scheduler. | Confirmed (DST test, 32 of 37 payloads delivered). Fixed with L2b. | S3, S4 in T3 |
| L3 | A node that resolves work only on `TimerTick` cannot do so during the receiver drain. | Confirmed for fanout timeouts (TLC, 12 steps). Fixed. Parquet age flush and the durable buffer have the same dependency and are covered by the same fix. | S7 in `T4_timeout` |
| L4 | Nodes drop in-flight state at `Shutdown` without answering. | Confirmed for fanout (Rust tests). Fixed for fanout; partition and transform deferred (D7). | S6 in T4; S8 in `T4_late` |
| L5 | Admission stays closed and strands buffered pdata until the deadline. | Known limitation, reproduced as expected (D3). | S7 in `T5_closed` |
| L6 | A node outlives the deadline. | Confirmed (TLC; Rust test). Fixed. | S5b in `T1_overrun` |
| L7 | `begin_shutdown` drops wakeups as "carrying no payload", but routers keep a payload keyed by that wakeup. | Confirmed (TLC, 15 steps; DST test). Fixed. | S7, S4 in T5 |
| L8 | At the deadline the manager exits with `DrainIngress` or `Shutdown` still in `pending_sends`, and the receiver never learns about shutdown. | Found by TLC (15 steps). Fixed by the L6 fix. | S5b and deadlock in T1, T2, T5 |
| L9 | After the latch `set_wakeup` returns `ShuttingDown`; batch treats that as a processor error, so an input that arrives into an empty buffer during drain fails the node. | Found in review. Confirmed (Rust test; TLC, 8 steps once the model's holding processor fails like batch). Fixed. The durable buffer was checked and is not affected (see below). | S4 in T2 |

### L1: why the dispatcher exit cannot drop a waited-for answer

- The completion channel closes only when every completion sender is
  dropped. Senders live in node effect handlers (`runtime_pipeline.rs` 630,
  712, 851) and their clones. Receivers hand clones to their servers
  (`otlp_receiver` 368-382, `otap_receiver` 529-531) and to per-connection
  tasks (`syslog_cef_receiver` 716, 733). So the dispatcher cannot exit
  while any holder is still running.
- An answer for a waiting request goes to the receiver node's control
  channel. That channel closes only when the receiver node stops. Before it
  stops, the OTLP and OTAP receivers answer every open slot, either
  normally or by `force_shutdown` (`otlp_receiver` 793-800, `otap_receiver`
  674-680).
- So a drop at 1039, 1064 or 1187 can only hit a node that has stopped:
  a processor whose upstream subscriber is the L4 problem, or a receiver
  that already answered the client. The drop is still silent (S8).
- Side observation, not modeled: syslog connection tasks are detached
  (`_ = spawn_local`, 733) and keep a completion sender alive after the
  receiver stops. A connection blocked in a read keeps the dispatcher
  alive, which is another S5b candidate (D2).

### L2b: retained payload lost when the input closes first

Any ordering of "upstream stopped" and "`Shutdown` delivered" can reach it.
`Shutdown` does not have to be stuck in `pending_sends`:

1. The receiver sends `ReceiverDrained` and stops. Its pdata sender is
   dropped, so retry's input is closed.
2. Before the manager handles `ReceiverDrained`, retry's inbox is in normal
   mode with an empty control channel. Its `select!` sees the pdata channel
   closed (`message.rs` 722-727) and calls `closed_pdata_shutdown`.
3. `shutdown()` converts the delayed resume to `due_now` and drops both
   receivers (314-322). Retry gets a synthetic `Shutdown` and ignores it
   (`retry` 705).
4. The next `recv_with_policy` returns `Closed` (404-406). The processor loop
   ends (`processor.rs` 657). The retained payload is dropped and nobody is
   told.

With a waiting receiver the payload's request cannot still be open at step
1 (the receiver waits for its registry to empty), unless the deadline
force-Nacked it. With a fire-and-forget receiver (for example syslog or host
metrics, which never subscribe), the receiver drains at once and the data
is lost silently. T3 covers both receiver kinds. TLC found this in 12 steps,
and `dst_retained_payloads_survive_input_closing_before_shutdown_seeded`
reproduced it: it failed at `298846b57` for every seed where the input
closed before `Shutdown` arrived, and passed where `Shutdown` came first.

### L7: router payloads keyed by a cleared wakeup

- In backpressure mode, content_router and signal_type_router park one
  message per full port and re-probe it on a local wakeup
  (`router_admission` 169-206, 244-274).
- `begin_shutdown` clears every wakeup (`scheduler.rs` 454-458) on the
  assumption that wakeups carry no payload (436-440). The router's payload
  is in its own state, keyed by that wakeup, so it is never re-probed after
  the latch. `set_wakeup` is then rejected, and the router keeps the message
  parked (`router_admission` 196-198, 268-270).
- If every pause-candidate port is parked, `accept_pdata` stays false
  (161-166, 290-296). The inbox honors admission during drain, the upstream
  data stays buffered, `shutdown_drain_complete` is never true, and the
  parked messages are Nacked only when the deadline releases `Shutdown`
  (`handle_shutdown` / `drain_for_shutdown`, 277-288).
- So the drain cannot finish gracefully even when downstream recovers. T5
  with `gate = router` found this in 15 steps, and
  `dst_parked_payload_wakeup_survives_shutdown_latch_seeded` reproduced it.

### L9: batch fails on a refused timer after the latch

Found by review of this PR, after the TLC runs.

- After the latch, `set_wakeup` returns `WakeupError::ShuttingDown`
  (`scheduler.rs` 283-288). Batch's `set_arrival` (1735-1754) mapped any
  error to `ProcessorError`, and `accept_payload` (1047) and the
  retained-residual path (1218) propagate it with `?`.
- A processor keeps taking input after its `Shutdown` latches. When that
  input arrives into an empty buffer and stays below `min_size`, batch arms
  its timer, gets `ShuttingDown`, and `process()` fails. The processor loop
  ends (`processor.rs` 674-676) and the pipeline join stops on the error, so
  batch's buffered data and the rest of the drain are left to the deadline.
- The model missed it because its holding processor treated a refused
  re-arm as harmless. With the L9 behavior modeled (`Fixes` without
  `"L9"`), TLC finds it in 8 steps in T2.
- Temporal reaggregation already treats `ShuttingDown` as `Ok`
  (`schedule_wakeup`, 593-600). The durable buffer's `sync_armed_wakeup`
  (`deferred_retry_state.rs` 205) also returns the error, but its callers
  (`schedule_at`, `rearm_after_processing`) turn it into `false`, and the
  durable buffer only logs a warning; the bundle stays in storage. The
  exclusive routers handle it (L7). So batch was the only affected node.

## Behaviors that are intended, not violations

- **Queued timer ticks arrive late.** See S2 and D4.
- **Forced Nack does not prove non-delivery.** See D1.
- **A node may get one early wakeup during drain.** With the L7 fix every
  pending wakeup is delivered once at the latch. Batch skips a wakeup that
  arrives before `max_batch_duration` has passed (`batch_processor` 1108-1116)
  and cannot arm a new one; its buffer is flushed when `Shutdown` is
  released (1400-1403).
- **Processors still honor admission during drain.** This lets stateful
  processors take Acks and Nacks and reopen (`message.rs` 499-501). The cost
  is L5.
- **Exporters force-drain their input.** Data already in an exporter's
  channel is processed during drain even if the exporter closed admission.

## Assumptions about the environment

- Single-threaded runtime per pipeline. Each model action is one
  uninterrupted run of a node between two `await` points that can block.
- Every channel is FIFO. Which ready input a node takes next is
  nondeterministic, except where the code's control preference decides a
  drain outcome.
- Time is abstracted. The deadline is one nondeterministic event; delayed
  resumes and wakeups can fire at any time before the latch.
- An exporter answers each message it takes at most once.

## Out of scope

Completion routing details (Part 2), batch internals (Part 1), control-plane
fairness and ordering (Part 4, except where ordering decides a drain
outcome), live reconfiguration, extensions, quiver and the durable buffer
internals, node-specific shutdown bugs (D2), proofs, and CI.

## Fixes

Each fix is mirrored in `Shutdown.tla` behind the `Fixes` constant, so a
config can check the code before (`Fixes = {}`) or after the fix. The repo
configs list every fix that has landed.

| Lead | Fix | Rust test |
| --- | --- | --- |
| L2b, L2c | A closed `pdata` input no longer closes the inbox while the local scheduler holds retained payloads. In normal mode the scheduler is latched (`begin_shutdown`) and its due `ResumeData` are delivered first (`closed_pdata_shutdown_when_drained`); in drain mode a closed input releases `Shutdown` only when `shutdown_drain_complete()` holds. | `dst_retained_payloads_survive_input_closing_before_shutdown_seeded`, `dst_retained_payloads_survive_control_burst_during_drain_seeded` |
| L7 | `begin_shutdown` delivers each pending wakeup once (original deadline and revision) instead of dropping it, and the exclusive routers NACK a parked message (`NodeShutdown`, retryable) when the scheduler refuses to re-arm its wakeup (`nack_unprobeable_parked`). Admission reopens and the drain finishes before the deadline. | `dst_parked_payload_wakeup_survives_shutdown_latch_seeded`, `test_shutdown_latch_nacks_parked_routes_that_cannot_be_reprobed`; `processor_inbox_fires_pending_wakeups_once_on_shutdown_latch` (was `..._drops_pending_wakeups_...`) |
| L6, L8 | Receivers and exporters are wired to the pipeline forced-shutdown signal (`start_until_forced_shutdown`). After it fires they get `NODE_FORCED_SHUTDOWN_GRACE` (1 s) to finish their own deadline handling, then their task is cancelled. A receiver whose `DrainIngress` and `Shutdown` were lost in the manager's `pending_sends` (L8) stops the same way. | `exporter_stops_by_deadline_when_inflight_export_never_completes`, `receiver_stops_on_forced_shutdown_without_drain_or_shutdown` |
| L3 | Runtime and telemetry timers keep running while receivers drain and are frozen when downstream `Shutdown` is sent (`freeze_timers`); `StartTimer` is ignored only from then on. A fanout timeout can now answer a draining receiver when a destination never replies. | `test_timer_keeps_firing_until_downstream_shutdown` (fails with the freeze moved back to the shutdown request); TLC: T4 with `Fixes` without `"L3"` deadlocks in 12 steps, with it 8.2M states pass (`fto`, silent destination, no deadline) |
| L4 | Fanout NACKs every request in `inflight` and `slim_inflight` upstream at `Shutdown` (`NodeShutdown`, retryable) instead of dropping them (`nack_inflight_on_shutdown`). Late downstream answers are D6. | `shutdown_nacks_inflight_requests_upstream`, `shutdown_nacks_slim_inflight_requests_upstream` |
| L9 | Batch treats `WakeupError::ShuttingDown` from `set_wakeup` as `Ok` in `set_arrival`, as temporal reaggregation already does: the input stays buffered and is flushed when `Shutdown` is released. | `test_input_after_shutdown_latch_is_buffered_and_flushed_on_shutdown` |

## Decisions to revisit

### D1. A forced Nack does not guarantee the data was not delivered

- Decision: S6 accepts a forced Nack at the deadline as a valid outcome even
  if the data later reaches, or already reached, an exporter.
- Why: the receiver cannot know what downstream did after it stopped
  waiting. This matches the open "Nack followed by delivery" question in
  #4395 and Part 2 D6.
- Revisit when: #4395 settles whether a Nack must mean "not delivered".

### D2. Node-specific shutdown drops are recorded, not modeled or fixed

- Decision: the drops found in the shutdown inventory are kept in the
  appendix and listed as follow-up issue candidates. Only one is modeled,
  generically: a receiver that stops without `ReceiverDrained`. TLC shows
  that it does not stall a linear pipeline (S7); the error return still
  aborts the pipeline join, outside the model.
- Candidates for follow-up issues under #4395:
  - parquet exporter never sends Ack or Nack;
  - otap exporter drops pdata with no outcome on several paths;
  - clickhouse and kafka exporters drop writes or reports at the deadline;
  - azure_monitor exporter drops unsupported signals and error-path messages;
  - kafka receiver reports drained before in-flight Acks arrive;
  - topic receiver does not Nack tracked messages on `Shutdown`;
  - receivers that skip `ReceiverDrained` on error paths: internal
    telemetry, journald, otlp, otap, oracle (scraper), user_events;
  - syslog connection tasks outlive the receiver (S5b candidate).
- Revisit when: the owners of those nodes pick up the issues.

### D3. Closed admission strands data until the deadline (L5)

- Decision: L5 is an expected violation, reproduced as a separate config
  like Part 2's `T1_exit.cfg`, not a finding.
- Why: it is documented in `engine/src/testing/dst/closed_admission.rs` and
  follows from honoring admission during drain.
- Revisit when: the engine changes how processors are drained.

### D4. S2 constrains timer generation, not delivery

- Decision: S2 says no tick is generated after the freeze (the shutdown
  request before the L3 fix, downstream `Shutdown` after it). Ticks already
  queued may still be delivered.
- Why: they were generated before the request, and the node may act on
  them during drain.
- Revisit when: a node is found that misbehaves on a late tick.

### D5. S5b is expected to fail at `298846b57`

- Decision: S5b is checked and its violation recorded (L6). No production
  change was assumed while modeling.
- Resolved: the L6 fix (see "Fixes") makes S5b hold in every config.

### D6. Late outcomes for an answered request are still dropped silently

- Observation: after the L4 fix, fanout NACKs its in-flight requests at
  `Shutdown`, and a forced receiver NACKs its open slots at the deadline.
  A destination may still answer afterwards. That answer reaches a control
  channel that is already closed, and the dispatcher drops it with no
  signal (`pipeline_ctrl.rs` 1039, 1064, 1187). S8 still fails in
  `T1_late.cfg` and `T4_late.cfg`.
- Decision: recorded, not fixed. Every waiting subscriber has already been
  answered, so nothing hangs; the lost signal is diagnostic only. Counting
  these drops needs a completion-metric design (Part 2 D3).
- Revisit when: a dispatcher drop counter is added.

### D7. Partition and transform still drop split contexts at `Shutdown`

- Observation: `partition_processor` and `transform_processor` keep split
  contexts that wait for downstream outcomes and ignore `Shutdown`, like
  fanout did before the L4 fix.
- Decision: recorded, not fixed here (fix priority covered the fanout
  leads only). The same pattern applies: NACK the inbound request of every
  open split context at `Shutdown`.
- Revisit when: follow-up issue under #4395.

### D8. Forced cancellation can leave node cleanup unfinished

- Observation: with the L6 fix, a receiver or exporter that has not stopped
  1s after the deadline (`NODE_FORCED_SHUTDOWN_GRACE`) is cancelled. Its
  task's futures are dropped wherever they are, so cleanup it had not
  finished never runs. From the inventory (code reading, not reproduced):
  - Kafka receiver: offsets are committed synchronously when it handles
    `DrainIngress` or `Shutdown` (`kafka_receiver/receiver.rs` 653-722),
    so they are lost only if it is cancelled before it handles either (for
    example the L8 case). Uncommitted offsets are redelivered on restart:
    duplicates, not loss.
  - Parquet exporter: `flush_all` already races the deadline (277-324) and
    gives up with `TimedOut`, which can leave a partial file. This predates
    the fix; the cancel 1s later adds little.
  - Exporters that waited with no bound (OTLP gRPC/HTTP, OTAP, Azure
    Monitor): requests still in flight are dropped, and the server may
    already have them (D1).
- Decision: accepted. Before the fix these nodes kept the pipeline running
  forever; a bounded stop with possible duplicates or a partial file is the
  better failure. Waiting receivers were already force-Nacked at the
  deadline.
- Revisit when: a node needs guaranteed cleanup at shutdown. It should then
  bound its own cleanup by the `Shutdown` deadline, as parquet, file, Kafka
  exporter and ClickHouse already do, so it finishes before the grace runs
  out. Follow-up candidates for the node owners: Kafka receiver offsets on
  forced stop, parquet partial files.

## Appendix: shutdown-handling inventory

Every receiver, processor and exporter in `crates/core-nodes` and
`crates/contrib-nodes`, at `298846b57`. Built by searching non-test code for
`DrainIngress`, `Shutdown`, `notify_receiver_drained`, `TimerTick`,
`start_periodic_timer`, `set_wakeup`, `requeue_later`, `accept_pdata`,
`subscribe_to`, and the Ack/Nack calls, then reading each hit.

Timer codes: R = runtime `TimerTick`, W = local wakeup or `requeue_later`,
k = node-local tokio sleep or interval.

### Receivers

| Node | On `DrainIngress` | On `Shutdown` | Holds work | Timers | Leads and notes |
| --- | --- | --- | --- | --- | --- |
| otlp | Stop listeners, wait for slots, force-Nack at deadline, then notify (736-761, 779-792) | Force-Nack, await servers with no bound (793-800, 861-880) | Ack slots | k | Reference. L6. Skips notify if a server errors during drain (815-839). |
| otap | Same pattern, also waits for stream tasks (614-672) | Force-Nack, bounded join then cancel (674-680, 757-791) | Slots, stream tasks | k | An open bidi stream may block the notify (not verified). |
| topic | Nack unsent forward, wait for tracked ids, Nack at deadline, notify (223-292) | Return, no Nack (406-408) | Tracked ids | k | L4-like on `Shutdown`. |
| kafka (contrib) | Commit, notify at once, close (692-722) | Commit, close (653-691) | Offsets, replays | R, k | Late Acks lost; redelivery duplicates. Only receiver using `TimerTick`. |
| journald | Worker drain, notify when idle or at deadline (834-848, 750-773, 1108-1114) | Stop worker | Pending batches | k | Skips notify on worker stop with pending, failure, Nack failure. |
| internal_telemetry | Flush within deadline, then notify (272-286) | Same flush, no notify | Log batch | k | Skips notify on timeout (`?`). |
| oracle (contrib, scraper) | Wait pending page, cleanup, notify (760-766, 1027-1034) | Cleanup | One page | k | Skips notify on cleanup error. |
| host_metrics | Notify at once (584-588) | Return | One scrape, dropped | k | - |
| syslog_cef | Flush with `try_send` (error ignored), notify (653-685, 1187-1209) | Return | Builders | k | Silent drop when channel full; detached connection tasks. |
| etw (contrib) | Drain ring, `try_flush`, notify (791-832) | Return | Builder | k | Drop with metric when full. |
| user_events (contrib) | Blocking flush, notify (648-682) | Return | Builder | k | Flush can pass deadline; skips notify on error. |
| filelog, windows_perf_counters | Not wired as receivers yet | - | - | - | n/a |

### Processors

Processors never receive `DrainIngress`.

| Node | On `Shutdown` | Holds work | Timers | Gates admission | Leads |
| --- | --- | --- | --- | --- | --- |
| fanout | Ignore, drop in-flight state (1107-1109) | `inflight`, `slim_inflight` | R | Yes, at `max_inflight` (1052-1064) | L3, L4, L5 |
| retry | No-op (705); relies on `ResumeData` conversion | Local scheduler | W | No | L2b |
| batch | Flush (1400-1403) | Buffers, outbound map | W | No | Partial L4 (outbound); L9: a refused `set_wakeup` after the latch failed the node (`set_arrival` 1735-1754) |
| temporal_reaggregation | Flush (310-312) | In and out batches | W | Yes, by slots (331-349) | L5, partial L4 |
| content_router, signal_type_router | Nack parked messages (865-867; 515-517) | One message per port | W | Yes, backpressure mode | L7 |
| durable_buffer | Flush, drain until deadline (1643-1736) | Pending bundles | R, W | No | L3-like (data is durable); a refused retry wakeup only logs a warning (not affected by L9) |
| partition, transform | Ignore, drop split contexts | Split contexts | - | No | L4 |
| debug | Log only | - | - | No | - |
| attributes, filter, log_parser, log_sampling, condense_attributes, recordset_kql, resource_validator | Ignore | - | log_sampling R (counters only) | No | - |

### Exporters

Exporters see `Shutdown` only after their input is drained. They have no
local scheduler (`message.rs` 827).

| Node | On `Shutdown` | Holds work | Timers | Leads and notes |
| --- | --- | --- | --- | --- |
| otlp_grpc | Nack parked, await in-flight, no bound (420-443) | In-flight | k | L6 |
| otlp_http | Await in-flight, no bound (428-447) | In-flight | k | L6 |
| otap | Join workers, no bound (707-738) | Stream queues, pending | k | L6; several no-outcome paths (D2) |
| azure_monitor (contrib) | Flush, drain no bound, Nack the rest (429-454) | Batches | k | L6; no-outcome paths (D2) |
| topic | Nack blocked and pending (561-569, 167-199) | Pending | - | - |
| file | Finalize within deadline (135-141) | - | - | Sync happens after Ack. |
| parquet | `flush_all` within deadline (277-324) | Buffered rows | R | L3; never Acks or Nacks (D2) |
| clickhouse (contrib) | Wait to deadline, drop the rest with a warning (321-351) | In-flight | - | D2 |
| kafka (contrib) | Flush and purge to deadline, abandon reports (944-972, 1286-1318) | In-flight | - | D2 |
| console | Exit | - | - | Acks failed exports. |
| noop, geneva | Exit | - | - | - |
| azure_data_explorer, geneva_metrics | Stub, no exporter | - | - | n/a |
