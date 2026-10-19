# Graceful shutdown and drain model

A TLA+ model of how one pipeline shuts down: the runtime-control manager's
receiver-first sequence, the receivers' drain contract, each node inbox's
drain by channel closure, the node-local scheduler at the shutdown latch,
the completion dispatcher, and the deadline. The TLC model checker explores
every interleaving within small bounds and checks the properties in
[spec.md](spec.md).

Line numbers in the model and in spec.md refer to commit `298846b57`, before
the fixes listed in spec.md "Fixes". The model contains both behaviors: the
constant `Fixes` selects which fixes are applied, and every repo config
applies all of them.

## Files

| File | Purpose |
| --- | --- |
| [spec.md](spec.md) | Properties in plain language, traced to the code; leads; fixes; decisions to revisit; shutdown inventory. |
| `Shutdown.tla` | The model. |
| `T1.cfg` | receiver -> stateless processor -> exporter. |
| `T2.cfg` | receiver -> holding processor (local wakeup flush, batch-like) -> exporter. |
| `T3.cfg` | receiver -> retry (`requeue_later`) -> exporter, admission gating off and on. |
| `T4.cfg` | receiver -> fanout (`await_ack: all`) -> exporters A, B; `max_inflight` off and on. |
| `T4_timeout.cfg` | T4 with a destination timeout and a destination that never answers (L3). |
| `T4_live.cfg` | T4 liveness (S5a, S5b), fire-and-forget receivers, timeouts off, deadline on. |
| `T5.cfg` | receiver -> exclusive router in backpressure mode -> exporter (L7). |
| `T1_skip.cfg` | T1 with a receiver that stops without `ReceiverDrained`. |
| `T1_overrun.cfg` | T1 with an exporter whose shutdown waits forever for in-flight work (L6). |
| `T1_late.cfg`, `T4_late.cfg` | S8 only: late outcomes for nodes that already stopped (D6). |
| `T5_closed.cfg` | A processor whose admission closes for good (L5, expected to fail). |
| `T1_large.cfg` to `T5_large.cfg` | The same topologies at larger bounds (see "Bounds"). |
| `Coverage.tla`, `coverage.sh` | Vacuity check: every path the properties and leads are about is reachable. |

## What is checked

| Property (spec.md) | TLA+ name | Kind |
| --- | --- | --- |
| S1 Ingress closes first | `IngressFirst` | invariant |
| S2 Timers freeze | `TimersFreeze` | invariant |
| S3 Drain by closure | `DrainByClosure` | invariant |
| S4 Retained payloads come back | `RetainedComeBack` | invariant |
| S5a Processors stop by the deadline | `ProcessorsStopByDeadline` | liveness |
| S5b Every node stops after the deadline | `EveryNodeStopsAfterDeadline` | liveness |
| S6 Exactly one outcome per waiting request | `OneOutcome`, `AnsweredAtEnd` | invariants |
| S7 Graceful when possible | `GracefulOutcomes` and TLC deadlock detection | invariant + deadlock |
| S8 No late outcome lost silently | `NoSilentLateOutcome` | invariant |
| Model sanity | `TypeOK` | invariant |

S1-S4 and S8 are checked at the step where they could break and recorded in
the ghost variable `flags`; the invariant names the label. Each main config
lets TLC pick, per behavior, whether the deadline can fire (`Deadlines =
{TRUE, FALSE}`). Behaviors without a deadline are restricted to S7's premise
(`GracefulPremise`), and CHECK_DEADLOCK is on, so a stuck state before "all
stopped" is a drain stall. S5a and S5b use weak fairness of every node, the
manager and the dispatcher; the shutdown request, the deadline, admissions
and the gate closing get none.

## How the model maps to the code

Paths are relative to `rust/otap-dataflow/crates/`.

| Model | Rust |
| --- | --- |
| `MgrRequest`, `MgrRtc`, `MgrPending`, `MgrTick`, `MgrDeadline`, `MSendAll`, `MDrain` | `RuntimeCtrlMsgManager::run`, `send`, `drain_pending_sends`, `freeze_timers` (`engine/src/pipeline_ctrl.rs`) |
| `DispDeliver`, `DispRetry`, `DispExit`, `DDrainR` | `PipelineCompletionMsgDispatcher::run`, `send`, `drain_pending_sends` (same file) |
| `Recv`, `RecvResult`, `Release` | `InboxCore::recv_with_policy`, `shutdown`, `closed_pdata_shutdown(_when_drained)` (`engine/src/message.rs`) |
| `BeginShutdown`, `DelayedFire`, `WakeFire` | `NodeLocalScheduler::begin_shutdown`, `pop_due` (`engine/src/node_local_scheduler.rs`) |
| `NodeExit`, `ForcedStop`, `ExpStop` | processor loop end (`engine/src/processor.rs` 657), forced shutdown (`processor.rs` 812, `start_until_forced_shutdown` in `exporter.rs` / `receiver.rs`) |
| `Admit`, `RCtrl`, `RDrained`, `RSkip`, `RDeadline`, `RForcedStop`, `ForceNack` | OTLP receiver drain (`core-nodes/src/receivers/otlp_receiver/mod.rs` 736-855), `AckRegistry::force_shutdown` (`otap/src/otap_grpc/common.rs` 74) |
| `PDataH`, `CtrlH`, `DueH`, `OnShutP` | the processors of the topologies: retry (`retry_processor`), fanout (`fanout_processor`), exclusive routers (`exclusive_router_admission.rs`, `content_router`, `signal_type_router`), and a batch-like holding processor |
| `Answer`, `Fates` | any exporter: Ack or Nack, dropped with no outcome (`silent`), or stuck forever (`unb`) |
| `Report`, `Target` | `notify_ack` / `notify_nack` to the nearest subscriber (frames are Part 2) |

Abstractions:

- Time is abstracted. The deadline is one nondeterministic event; delayed
  resumes and wakeups may fire at any step before the latch; a fanout
  timeout may expire on any tick.
- `BurstLimit` stands for `CONTROL_BURST_LIMIT` (32). The model keeps
  `consecutive_control`, capped at the limit, so the fairness path is
  reachable with a few messages.
- A node handler runs until a send would block; the rest waits in the
  node's `out` queue and the node takes no other input.
- The receiver's `NODE_FORCED_SHUTDOWN_GRACE` is abstracted: after the
  deadline a receiver or exporter may finish on its own or be force-stopped,
  in any order.
- Node-specific shutdown bugs (spec.md D2) are not modeled, except a
  receiver that stops without `ReceiverDrained` (`T1_skip.cfg`).

## Bounds

| Configs | Requests | Capacity (all channels) | Retries |
| --- | --- | --- | --- |
| `T1.cfg` to `T4.cfg`, `T4_timeout.cfg` and the variants | 2 | 1 | 1 (T3) |
| `T5.cfg` | 3 | 1 | - |
| `T1_large.cfg` to `T3_large.cfg` | 3 | 2 | 1 (T3) |
| `T4_large.cfg` | 2 | 2 | - |
| `T5_large.cfg` | 4 | 2 | - |

T5 needs one more request than the others so that one message can sit in
the exporter channel, one in the router's parked slot and one in its input.
`T4.cfg` and `T4_large.cfg` check safety and deadlock only, with timeouts
off and answering exporters, and `T4_large.cfg` keeps 2 requests: with 3
requests and capacity 2 the run passed 600M distinct states and filled the
disk (see "Known limits").

## Results

All properties hold in every main config with all fixes applied. The three
configs that isolate a known gap fail as expected.

| Config | Result | Distinct states | Depth | Time |
| --- | --- | --- | --- | --- |
| `T1.cfg` | pass (incl. S5a, S5b) | 48,262 | 26 | 6s |
| `T2.cfg` | pass (incl. S5a, S5b) | 62,344 | 28 | 7s |
| `T3.cfg` | pass (incl. S5a, S5b) | 832,049 | 44 | 14m44s |
| `T4.cfg` | pass (safety, deadlock) | 71,567,156 | 43 | 11m07s |
| `T4_timeout.cfg` | pass (safety, deadlock) | 8,177,602 | 44 | 1m45s |
| `T4_live.cfg` | pass (S5a, S5b) | 14,920,453 | 38 | 53m00s |
| `T5.cfg` | pass (incl. S5a, S5b) | 570,815 | 33 | 1m06s |
| `T1_skip.cfg` | pass | 837 | 24 | 0s |
| `T1_overrun.cfg` | pass (S5b) | 24,039 | 26 | 3s |
| `T1_late.cfg` | S8 fails, as expected (D6) | 4,556 (violation) | 11 | 0s |
| `T4_late.cfg` | S8 fails, as expected (D6) | 22,452 (violation) | 10 | 1s |
| `T5_closed.cfg` | deadlock, as expected (D3, L5) | 322 (violation) | 10 | 0s |

At the larger bounds:

| Config | Result | Distinct states | Depth | Time |
| --- | --- | --- | --- | --- |
| `T1_large.cfg` | pass (incl. S5a, S5b) | 682,588 | 33 | 1m21s |
| `T2_large.cfg` | pass (incl. S5a, S5b) | 812,218 | 35 | 1m56s |
| `T3_large.cfg` | pass (incl. S5a, S5b) | 26,998,891 | 60 | 1h43m |
| `T4_large.cfg` | pass (safety, deadlock) | 61,963,861 | 43 | 9m08s |
| `T5_large.cfg` | pass (incl. S5a, S5b) | 10,095,185 | 39 | 31m52s |

Times are for 10 workers on a 10-core Apple silicon laptop, with `-Xmx8g`
for the large configs. `T1_skip.cfg` passing is a finding: a receiver that
stops without `ReceiverDrained` does not stall a linear pipeline, because
the next node drains by channel closure anyway.

Before the fixes (`Fixes` without the lead), TLC finds each finding again:

| Lead | Config | Counterexample |
| --- | --- | --- |
| L2b | `T3.cfg` | `DrainByClosure`, 12 steps: a fire-and-forget receiver drains and stops, retry's inbox sees its input closed before `Shutdown` and closes with a parked retry inside. |
| L7 | `T5.cfg`, no deadline | deadlock, 15 steps: the router latches `Shutdown` with a message parked, its wakeup is dropped, admission stays closed with input buffered. |
| L6 | `T1_overrun.cfg` | `EveryNodeStopsAfterDeadline`: the exporter waits forever for a stuck export after the deadline. |
| L8 | `T1.cfg` | deadlock, 15 steps: the deadline fires while the receiver's `DrainIngress` is parked in the manager's `pending_sends`; the manager exits and the receiver never learns about shutdown. |
| L3 | `T4_timeout.cfg` | deadlock, 12 steps: fanout's `StartTimer` arrives after the shutdown request and is ignored, a destination drops the message, and the receiver waits forever. |
| L9 | `T2.cfg` | `RetainedComeBack`, 8 steps: batch takes an input into an empty buffer after the latch, its timer is refused, `process()` fails and the buffer is lost. Found in review; the model gained this behavior afterwards. |
| L4 | `T4_late.cfg`, no deadline | `NoSilentLateOutcome`, 12 steps: fanout stops with a request in flight and the destination's later Ack is dropped. With the fix the request is NACKed first; the late Ack is still dropped silently (D6). |

Two further checks guard against a vacuous pass:

- **Coverage.** `coverage.sh` checks that each path is reachable in its
  config: real Acks and Nacks, forced and shutdown Nacks, fanout timeout
  Nacks, the deadline, a full stop, both `pending_sends` buffers, the
  fairness burst path while draining, retained payloads converted at the
  latch, the scheduler latched by a closed input, a wakeup delivered after
  the latch, a router parked at the latch, a tick during ingress drain, a
  receiver stopped without `ReceiverDrained`, fanout in flight at
  `Shutdown`, and batch taking input into an empty buffer after the latch.
  Expected output: every line starts with `reached` (31 lines, about 25
  seconds).
- **Mutation.** Removing `"L2b"` from `Fixes` in `T3.cfg` makes TLC report
  `DrainByClosure` in 12 steps.

## Known limits

- The results hold within the bounds above. They are exhaustive there, but
  they are not a proof for more requests or larger channels.
- T4 with every option at once (timeouts, silent destinations, deadline,
  `max_inflight`) is too large: a safety run was stopped at 537M distinct
  states, and T4 with 3 requests and capacity 2 passed 600M before its
  state files filled the disk. `T4.cfg` (timeouts off) and
  `T4_timeout.cfg` (the L3 scenario) split it, and `T4_live.cfg` checks
  liveness separately with fire-and-forget receivers and timeouts off.
- S8 fails by design in `T1_late.cfg` and `T4_late.cfg` (spec.md D6): a
  destination may answer after the request was already answered, and the
  dispatcher drops that answer with no signal.
- `T5_closed.cfg` fails by design (spec.md D3, L5).
- Partition and transform still drop split contexts at `Shutdown` (D7); the
  model has no split processor.
- Batch internals (Part 1), completion routing (Part 2) and control-plane
  fairness (Part 4) are out of scope.

## Running TLC

TLC needs Java 11 or later and `tla2tools.jar`. The results above used
`TLC2 Version 2026.10.06`, rev `94d0c50`. Keep the jar and TLC's working
files outside the repository.

```bash
cd rust/otap-dataflow/formal/shutdown
java -XX:+UseParallelGC -cp /path/to/tla2tools.jar tlc2.TLC \
    -workers auto -noGenerateSpecTE -metadir /path/to/tlc-state/T1 \
    -config T1.cfg Shutdown.tla
```

Run it once per `.cfg` file; add `-Xmx8g` for the `*_large.cfg` files.
Without `-metadir`, TLC writes under `states/`, which is ignored by git.

To check coverage (about 30 seconds):

```bash
cd rust/otap-dataflow/formal/shutdown
./coverage.sh /path/to/tla2tools.jar /path/to/tlc-state/coverage
```

To check a fix, remove its lead from `Fixes` in a copy of a config and run
TLC again; the matching counterexample comes back (see "Results").
