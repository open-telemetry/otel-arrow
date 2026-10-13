# Batch processor Ack/Nack model

A TLA+ model of how the batch processor
(`crates/core-nodes/src/processors/batch_processor/mod.rs`) aggregates
downstream Acks and Nacks into one outcome per inbound request. The TLC model
checker explores every interleaving of arrivals, flushes, and downstream
answers within small bounds, and checks the properties in
[spec.md](spec.md).

## Files

| File | Purpose |
| --- | --- |
| [spec.md](spec.md) | The properties in plain language, traced to the code. |
| `BatchAggregation.tla` | The model. |
| `logs_traces.cfg` | OTAP items, aligned splitting (logs and traces). |
| `metrics.cfg` | OTAP items, split on metric rows (metrics). |
| `bytes.cfg` | OTLP bytes, whole entries and header-repeating fragments. |
| `Coverage.tla`, `coverage.sh` | Vacuity check: every risky path is reachable. |

## What is checked

| Property (spec.md) | TLA+ name | Kind |
| --- | --- | --- |
| P1 No premature Ack | `NoPrematureAck` | invariant |
| P2 At most one outcome | `AtMostOneOutcome` | invariant |
| P3 Exact reference counts | `ExactCounts` | invariant |
| P4 Full table yields a Nack | `NoSilentDrop` | invariant |
| P5 Residual from last input | `ResidualFromLastInput` | invariant |
| P6 Drained means empty | `DrainedMeansEmpty` | invariant |
| P7 Guards never fail | `GuardsHold` | invariant |
| P8 Slot-free paths | `SlotFreePaths` | action property |
| Model sanity | `OwnershipConsistent`, `TypeOK` | invariant |

Guards and P5 are checked through the ghost variable `failed`: when a modeled
`debug_assert!` condition (or P5's content check) is false, its label is
added, the model continues as a release build would, and the matching
invariant reports the state.

## How the model maps to the code

Each action is one message handled by `BatchProcessor::process`.

| Model | Rust (mod.rs unless noted) |
| --- | --- |
| `AcceptPayload` | `accept_payload` 972, via `process_signal_impl` 823 |
| `TimerFlush` | `Wakeup` 1424 -> `flush_signal_impl` with `Timer` |
| `Shutdown` | `flush_shutdown` 756 |
| `Respond` | `handle_ack` / `handle_nack` 1320 -> `handle` 1300 |
| `Flush`, `FlushOk` | `flush_signal_impl` 1083 |
| `FlushErr` | `make_batches` error branch 1151-1164 |
| retention in `FlushOk` | `take_remaining` 1589 |
| `Drain` | `drain_context` 1651 |
| `SendLoop` | send loop 1236-1290 |
| `NackParts` | outbound slots exhausted 1257-1278 |
| `HandleParts` | `handle_partial_responses` 1702 |
| `MakeBatches` | `Batcher::make_batches` 890, 931 |
| `AlignedPieces` | `plan_split` (`crates/pdata/src/otap/transform/split.rs`) |
| `RowSplit` | `plan_metrics_split` (same file, 152) |
| `Concat` | `generic_concatenate` (`crates/pdata/src/otap/groups.rs` 244) |
| `BytesPack`, `SplitEntry` | `make_bytes_batches_owned`, `push_resource_entry`, `rescale_ownership` (`crates/pdata/src/otlp/batching.rs`) |

Abstractions:

- Data is counted in abstract units. Unit `<<r, k>>` is the k-th unit of
  request r. Header units repeated by byte splitting are `<<0, k>>`.
- Ghost variables (`confirmed`, `lost`, `failed`, `arrived`, `trk`,
  `reqSize`) record ground truth that the code does not keep.
- Clocks are not modeled. A timer flush may happen whenever data is
  buffered, which covers early and late timers.
- Split budgets in bytes mode are a nondeterministic choice per flush
  (`AllowFallback`).
- `make_batches` failure is a nondeterministic choice per flush
  (`AllowBatchError`).
- Slot keys are never reused, as with the generational slot map.

Bounds (each `.cfg` explores all combinations in one run): 3 requests of
0 to 4 units, tracked or not; `min_size` 2 or 3; `max_size` unset, 3, or 4;
inbound limit 1 to 3; outbound limit 1 or 2; header overhead 0 or 1 (bytes
mode only).

## Known limits

- The results hold within the bounds above. They are exhaustive there, but
  they are not a proof for larger inputs or more requests.
- One signal and one format at a time. The three sizing modes are checked
  separately.
- Bytes mode: a split fragment is retained only when its even ownership share
  happens to equal its real size. That needs an entry larger than 4 units, so
  this path is modeled but not reached at these bounds.
- Bytes mode: only resource entries are modeled. Unparseable or non-resource
  top-level fields (`push_opaque`) are not.
- Metric rows always carry at least one data point.
- The early-timer check and wakeup scheduling are abstracted into "a timer
  flush may happen whenever data is buffered".

## Results

All properties hold in all three modes, at the bounds above, with the
metrics `debug_assert!` fix applied (see "Findings" in [spec.md](spec.md)).

| Config | Distinct states | Depth | Time |
| --- | --- | --- | --- |
| `logs_traces.cfg` | 6,709,806 | 13 | 48s |
| `metrics.cfg` | 25,403,565 | 13 | 4m45s |
| `bytes.cfg` | 76,303,186 | 13 | 23m46s |

Times are for 10 workers on a 10-core Apple silicon laptop with `-Xmx6g`.

Two further checks guard against a vacuous pass:

- **Coverage.** `coverage.sh` checks that each risky path is reachable in
  each mode: residuals, requests split across batches and Acked, outbound
  and inbound slot exhaustion, batching errors, downstream Nacks, tracked
  pass-through, and byte fragments. Expected output: every line starts with
  `reached` (28 lines, about 30s).
- **Mutation.** Removing the residual's reference in `FlushOk` (replace the
  `!.cnt = IF holds ...` update with `!.cnt = @,`) makes TLC report a
  violation of `NoPrematureAck` even with `ExactCounts` disabled. The ground
  truth alone catches the resulting data loss.

## Running TLC

TLC needs Java 11 or later and `tla2tools.jar`. The results above used the
jar from the [v1.8.0 release](https://github.com/tlaplus/tlaplus/releases/tag/v1.8.0)
(`TLC2 Version 2026.10.06`, rev `94d0c50`). Keep the jar outside the
repository.

```bash
cd rust/otap-dataflow/formal/batch_processor
java -Xmx6g -XX:+UseParallelGC -cp /path/to/tla2tools.jar tlc2.TLC \
    -workers auto -noGenerateSpecTE -config logs_traces.cfg \
    BatchAggregation.tla
```

Run it once per `.cfg` file. Expected output ends with `Model checking
completed. No error has been found.` TLC writes its working files under
`states/`, which is ignored by git.

To check coverage:

```bash
cd rust/otap-dataflow/formal/batch_processor
./coverage.sh /path/to/tla2tools.jar
```

To read a counterexample, look at the last state of the trace: the violated
invariant is named in the output, and `failed` names any guard that fired.
