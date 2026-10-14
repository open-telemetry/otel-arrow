# Ack/Nack completion protocol model

A TLA+ model of how the engine returns one Ack or Nack to whoever waits for
a request: frames pushed on the message context, reports from nodes, and the
completion dispatcher that unwinds them. The TLC model checker explores
every interleaving of admissions, sends that block, node steps, dispatcher
steps, and exporter answers within small bounds, and checks the properties
in [spec.md](spec.md).

Line numbers in the model and in spec.md refer to commit `298846b57`, before
the fixes listed in spec.md "Findings".

## Files

| File | Purpose |
| --- | --- |
| [spec.md](spec.md) | Properties in plain language, traced to the code; findings; decisions to revisit. |
| `Completion.tla` | The model, with the fixes applied. |
| `T1.cfg` | receiver -> processor -> exporter, every processor behavior. |
| `T2.cfg` | receiver -> fanout -> 2 exporters, `await_ack` `none`, `primary`, `all`. |
| `T3.cfg` | receiver -> retry -> exporter, up to 2 retries. |
| `T4.cfg` | receiver -> retry -> fanout `none` -> 2 exporters, up to 2 retries. |
| `T5.cfg` | receiver -> debug processor -> 0, 1 or 2 main ports, Detailed off and on. |
| `T1_exit.cfg` | T1 with the dispatcher exit. Expected to fail (spec.md D3). |
| `Coverage.tla`, `coverage.sh` | Vacuity check: every path the properties are about is reachable. |

## What is checked

| Property (spec.md) | TLA+ name | Kind |
| --- | --- | --- |
| Q1 At most one delivery per frame | `AtMostOneDelivery` | invariant |
| Q2 Deliveries reach only the lineage's subscribers | `LineageOnly` | invariant |
| Q3 Most recent interested subscriber first | `LifoDelivery` | invariant |
| Q4 Ack only after delivery | `AckOnlyAfterDelivery` | invariant |
| Q5 Drained means answered | `DrainedMeansAnswered` | invariant |
| Q6 No silent drop in the dispatcher | `NoSilentDispatcherDrop` | invariant |
| Q7 Returned payload only where asked | `ReturnDataOnlyWhereAsked` | invariant |
| Q8 Fanout `none` answers once, with Ack | `FanoutNoneAnswersOnceWithAck` | invariant |
| Model sanity | `TypeOK` | invariant |

Q1-Q4 and Q6-Q8 are checked at the moment of delivery and recorded in the
ghost variable `flags`; the invariant names the label. Q3 holds by
construction (the model's unwind is the same loop as `unwind_frames`); its
check guards the model against itself.

## How the model maps to the code

Paths are relative to `rust/otap-dataflow/crates/`.

| Model | Rust |
| --- | --- |
| `Admit`, `RecvResume`, `RecvSend` | OTLP export handler, `otap/src/otap_grpc/otlp/server_new.rs` 470-547 |
| `Cancel` | `SlotGuard::drop`, same file 368-378 |
| `ReceiverHandleOutcome` | `handle_ack` / `handle_nack`, `core-nodes/src/receivers/otlp_receiver/mod.rs` 398-421, and `AckSlot::route_response` |
| `Subscribe` | `Context::subscribe_to`, `otap/src/pdata.rs` 431 |
| `Entry` | `Context::push_entry_frame`, `pdata.rs` 705 |
| `CloneWithoutContext` | `OtapPdata::clone_without_context`, `pdata.rs` 1064 |
| `Report`, `RunR` | `notify_ack` / `notify_nack` (`pdata.rs` 1311-1323) -> `route_ack` / `route_nack` (`engine/src/effect_handler.rs` 364-410) |
| `Send`, `Continue` | `send_message_to` / `send_message_with_source_node(_to)`, awaited |
| `Export` | any exporter: one Ack, transient Nack, or permanent Nack per message |
| `P_ProcessPData`, `P_ProcessControl` | patterns in spec.md "Behavior patterns" (partition, transform, log_sampling, debug) |
| `F_ProcessPData`, `F_ProcessControl` | fanout `process` (`core-nodes/src/processors/fanout_processor/mod.rs` 1066-1160) and its ack/nack handlers |
| `Y_ProcessPData`, `Y_ProcessControl`, `Y_Resume` | retry `process`, `handle_ack`, `handle_nack`, `handle_resumed` (`core-nodes/src/processors/retry_processor/mod.rs` 454-672) |
| `D_ProcessPData`, `D_ProcessControl` | debug `process` (`core-nodes/src/processors/debug_processor/mod.rs` 200-350) |
| `DispatcherReceive`, `UnwindR` | dispatcher run loop, `unwind_ack` / `unwind_nack`, `unwind_frames`, `send` (`engine/src/pipeline_ctrl.rs` 1011, 1160-1325) |
| `DispatcherRetryPending`, `DrainR` | `drain_pending_sends`, same file 1044 |
| `DispatcherExit` | loop exit on completion channel close, same file 1187 |

Abstractions:

- A node handler computes a list of effects (sends and reports) and runs them
  until one would block; the rest waits in `todo[n]` and the node takes no
  other input. This models a suspended `await` on a full channel.
- Every frame has a ghost identity. A subscription always gets a fresh one;
  a `clone()` keeps the identities it copied. This is how Q1 sees a shared
  frame without relying on the receiver's slot map.
- Each request has two data units, so a detached split can cut it. Lineage
  (`lin`) is ghost data.
- Timers are abstracted: a parked retry resumes at any time, and a transient
  Nack below the retry limit may still hit the deadline.
- Slot keys are never reused, as with the generational slot maps.
- The `consume` behavior (a node that takes a message and never answers) is
  the debug processor's zero-port path before its fix. It stays in the model
  for experiments but is not in `T1.cfg`.

Bounds: 2 requests, capacity 1 on every data channel, control channel, and
the completion channel; up to 2 retries.

## Known limits

- The results hold within the bounds above. They are exhaustive there, but
  they are not a proof for more requests or larger channels.
- Q6 is only reachable through the dispatcher exit, which needs every node
  to have stopped (shutdown, Part 3). A failed node never subscribes in this
  model, so the closed-channel drop is not reached, and the missing-sender
  drop cannot happen because every node has a control sender.
- In T4, after the fix, fanout `none` only answers Ack, so the retry
  processor never sees a Nack there. Retry paths are covered by T3.
- Fanout sequential mode, fallbacks, timeouts and `max_inflight` are not
  modeled. Neither are router parking, the durable buffer, and batch
  internals (Part 1).
- The Nack-before-Ack race found in T2 has no Rust test (spec.md D5).

## Results

All properties hold in `T1.cfg` to `T5.cfg` with the fixes applied.
`T1_exit.cfg` fails `NoSilentDispatcherDrop` as expected (D3).

| Config | Distinct states | Depth | Time |
| --- | --- | --- | --- |
| `T1.cfg` | 12,068 | 26 | 1s |
| `T2.cfg` | 26,309 | 23 | 1s |
| `T3.cfg` | 18,773 | 32 | 1s |
| `T4.cfg` | 5,490 | 21 | 1s |
| `T5.cfg` | 3,110 | 18 | 1s |
| `T1_exit.cfg` | 49 (violation) | 12 | 0s |

Times are for 10 workers on a 10-core Apple silicon laptop.

Two further checks guard against a vacuous pass:

- **Coverage.** `coverage.sh` checks that each path is reachable in its
  topology: Ack and Nack answers, blocked sends, the dispatcher's pending
  buffer, late deliveries to a cancelled request, abandonment and error
  answers after a node failure, a split half answered, fanout `all` half
  answered, fanout `primary` ignoring the secondary, retries and an Ack after
  a retry, detached copies answered by an exporter, and the debug sink Ack.
  Expected output: every line starts with `reached` (25 lines, about 15
  seconds).
- **Mutation.** Undoing the fanout fix in `F_ProcessPData` (send `m`
  instead of `CloneWithoutContext(m)` to A and B) makes TLC report
  `AtMostOneDelivery` for `T2.cfg` and `T4.cfg`. With `AtMostOneDelivery`
  removed from `T2.cfg`, it reports `FanoutNoneAnswersOnceWithAck`.

## Running TLC

TLC needs Java 11 or later and `tla2tools.jar`. The results above used
`TLC2 Version 2026.10.06`, rev `94d0c50`. Keep the jar outside the
repository.

```bash
cd rust/otap-dataflow/formal/completion
java -XX:+UseParallelGC -cp /path/to/tla2tools.jar tlc2.TLC \
    -workers auto -noGenerateSpecTE -config T1.cfg Completion.tla
```

Run it once per `T*.cfg` file. Expected output ends with `Model checking
completed. No error has been found.`, except for `T1_exit.cfg`. TLC writes
its working files under `states/`, which is ignored by git.

To check coverage:

```bash
cd rust/otap-dataflow/formal/completion
./coverage.sh /path/to/tla2tools.jar
```

To read a counterexample, look at the last state of the trace: the violated
invariant is named in the output, and `flags` names the property that
failed.
