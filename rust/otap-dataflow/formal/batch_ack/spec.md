# Batch processor Ack/Nack: specification

Part 1 of the experiment tracked in
[#4395](https://github.com/open-telemetry/otel-arrow/issues/4395):
verify the Ack/Nack delivery guarantees of the batch processor with an
executable Quint model.

This document states the intended behavior in plain language, traces each
property to the Rust functions that implement it, and records findings and
open decisions. It was written against `main` at commit `36a5c4ff3`.

Paths are relative to `rust/otap-dataflow/`. Function names are the stable
reference; line numbers drift and are omitted.

## Scope

In scope:

- The batch processor node in
  `crates/core-nodes/src/processors/batch_processor/mod.rs`: input admission,
  buffering, flushing, tail retention, inbound and outbound slot tracking,
  downstream Ack/Nack handling and the upstream answer.
- The range contract between the node and the batchers, as enforced by
  `validate_input_ranges` in `crates/pdata-codec/src/batching.rs`.
- The interaction with the processor-local wakeup scheduler during a
  shutdown drain (`crates/engine/src/node_local_scheduler.rs` and
  `crates/engine/src/message.rs`).

Out of scope:

- Sizes, sizers, signal types, codecs and payload contents. Batchers are
  abstracted to "any valid list of input ranges" (see
  [Model abstraction](#model-abstraction)).
- What upstream nodes do with an answer (retry, timeout). Part 3 covers the
  engine-level shutdown protocol.
- Metrics and `peer_addr` merging, which do not affect completion.

## Vocabulary

- **Input**: one `OtapPdata` delivered to `process`. A *tracked* input has
  `Context::needs_completion_tracking()` and owns an inbound slot while the
  node holds it.
- **Answer**: the single upstream Ack or Nack for a tracked input, sent with
  `notify_ack` or `notify_nack`.
- **Output**: one batch sent downstream by a flush. A *tracked* output names
  at least one live tracked input and owns an outbound slot.
- **Range**: the inclusive range of buffered-input indices an output was
  built from (`BatchingResult::batches`).
- **Completed input**: an input the batcher consumed without producing any
  output (`BatchingResult::completed_inputs`).
- **Reference**: a unit of `BatchContext::outbound`, the refcount that keeps
  a tracked input unanswered. One reference is held per pending buffer
  entry and one per emitted output that names the input.
- **Tail**: the last output of a size flush when it is undersized and is
  moved back into the buffer instead of being sent.
- **Drain**: the window after the inbox latches `Shutdown` with a future
  deadline and before it returns `Shutdown` to the processor. Pdata and
  Ack/Nack are still delivered; the local scheduler rejects new wakeups.
- **Settled**: no further input arrives, every emitted output has been
  answered by downstream, and every armed wakeup has fired. Settling is
  evaluated before the final `Shutdown` is delivered (see D3).

## Batching contract

From #4347, as implemented on `main`:

1. The inputs contributing to an output are contiguous: each output names one
   inclusive range.
2. Ranges are explicit and reported by the batcher while batching. They are
   never derived from sizes, weights or item counts.
3. Output sizes are best effort and may fall outside `[min_size, max_size]`.

`validate_input_ranges` enforces the structural part of the contract on every
batcher result before the node sees it:

- ranges are non-empty, in bounds and ordered;
- two consecutive outputs may share one boundary input (a split), and no
  other overlap is allowed;
- every input index is covered by an output range or listed in
  `completed_inputs`, never both.

A violation turns into `Err` from `BatchPlan::batch`, which the node answers
with a retryable Nack to every input of that flush. The validator does not
bound the number of outputs and cannot detect a range that is structurally
valid but names the wrong inputs.

The Rust test `every_batcher_honours_the_batching_contract`
(`crates/pdata-codec/src/batching.rs`) covers that gap for the OTAP, OTLP
and syslog batchers: it marks every record with its input, decodes every
output, and checks that each range equals the recovered provenance, that
inner inputs contribute all their records, that every record appears once,
and that outputs outside `[min_size, max_size]` are accepted.

## Invariants

Each invariant is stated as intent, then traced to the code that maintains
it. `S` marks safety properties, `L` marks liveness properties.

### S1. Every tracked input is answered at most once

Intent: an upstream subscriber never receives two answers for one input.

Trace:

- Tracked inputs are answered only when their inbound slot is removed:
  `SignalBuffer::handle_partial_responses` (Ack or Nack) and the
  outbound-exhaustion branch of `BatchProcessorSignal::flush_signal_impl`
  (Nack).
- A slot is removed at most once. `SlotState` keys are generational
  (`slotmap`), so a late response that names a freed or reused slot finds
  nothing and is ignored.
- Inputs that never get a slot are answered inline, once, at the point where
  they are rejected or completed: `process_signal_impl` (no route,
  `prepare_batch` failure), `accept_payload` (size 0, inbound exhaustion) and
  `forward_as_is` (`finish_batch` failure).

### S2. A Nack on any output carrying an input makes the answer a Nack

Intent: an input is Acked only if every output that carries any of its data
was Acked downstream. Any failure makes the answer a Nack.

Rule confirmed from the code: the **first** Nack answers immediately. The
node does not wait for sibling outputs; later responses for the same input
are ignored. Retryable and permanent classifications are preserved.

Trace:

- `handle_partial_responses` removes the slot on any `Err` and sends
  `NackMsg::new` or `NackMsg::new_permanent`.
- On `Ok` it only decrements; the Ack is sent when the last reference is
  released. Because every output holds a reference until it is answered,
  an Ack cannot precede a Nack for the same input.
- Other failure sources that Nack: batcher `Err` (retryable, all inputs of
  the flush), `finish_batch` failure on one output (permanent, that output's
  inputs) and outbound slot exhaustion (retryable, that output's inputs).

### S3. Reference conservation

Intent: the refcount of a live tracked input always equals the number of
places that still owe it a result.

For every live inbound slot `k`:

```text
outbound(k) = pending entries naming k
            + outbound slots whose portion list names k
            + outputs of the current flush that name k and are not yet sent
```

Trace:

- `accept_payload` allocates the slot with `outbound: 1` (the pending entry).
- `flush_signal_impl` adds one reference per output range that names the
  input before releasing anything, then releases one pending reference per
  drained entry with `handle_partial_responses(Ok)`.
- `take_remaining` moves a tail back into pending with the same
  `BatchPortion`; the tail output's reference becomes its pending reference.
- `output_context` and the outbound slot carry the portion list;
  `BatchProcessorSignal::handle` releases it on the downstream response.

S3 is the inductive invariant that implies S1, S2 and S5. The flush is a
single atomic step in the model except where it awaits (see F3).

### S4. A retained tail is attributed to exactly the inputs its range names

Intent: a tail carries the delivery owner of the data it contains, and only
that owner.

Trace:

- Retention in `flush_signal_impl` requires that the last output range is a
  single input (`start == end`) and that this input is the last buffered
  input.
- `take_remaining` copies that input's `BatchPortion` (same `inkey`) into the
  new pending entry.
- Multi-input tails are never retained; they are sent with normal
  multi-owner tracking.

### S5. Slots do not leak

Intent: once the node is settled, it holds no inbound or outbound slots, and
slot exhaustion produces an explicit answer instead of a stall.

Trace:

- Inbound slots are freed by `handle_partial_responses` (last reference or
  any error) and by the outbound-exhaustion branch.
- Outbound slots are freed only by `BatchProcessorSignal::handle` when
  downstream answers.
- Inbound exhaustion: `accept_payload` Nacks the input ("inbound routes
  exhausted"). Outbound exhaustion: `flush_signal_impl` Nacks every owner of
  that output ("outbound routes exhausted") and still sends the output.

### S6. The node does not fail on a reachable scheduler state

Intent: no input sequence that the engine can deliver makes `process` return
`Err`. A returned error ends the processor loop and drops every held context
without an answer.

Trace: `SignalBuffer::set_arrival` maps every `set_wakeup` error to
`EngineError::ProcessorError`. `WakeupError::ShuttingDown` is reachable
during a drain. This invariant is **violated on `main`** (F1, F2).

### L1. No tracked input is left unanswered once the node is settled

Intent: when the node is settled, every tracked input it admitted has been
answered.

Trace: requires S3, that every retained tail is eventually flushed (timer,
next size flush or shutdown), and that no flush exits early. Tails are
flushed by `flush_signal_impl` with `FlushReason::Timer` (never retains) or
`FlushReason::Shutdown` (never retains). The property is broken by F1, F2
and F3, and stops at the shutdown boundary described in D3.

### L2. Buffered data is flushed within a bounded time

Intent: data does not stay buffered indefinitely while the node is running.

Trace: `set_arrival` arms one wakeup per buffer when it goes from empty to
non-empty. A retained tail re-arms it. Retention requires the tail to come
from the last input, so a tail is never retained twice and waits at most
about `2 * max_batch_duration`. During a drain, wakeups are not available;
buffered data waits for the final `Shutdown` flush.

## Model abstraction

The Quint model abstracts payloads to input identities and lets the batcher
return any result that `validate_input_ranges` accepts, within a bound.

| Code | Model |
| --- | --- |
| `Inputs.pending` and `Inputs.context` | ordered list of portions, each `{ input, tracked }` |
| `inbound: SlotState<BatchContext>` | map from input to refcount, with a capacity bound |
| `outbound: SlotState<Vec<BatchPortion>>` | map from output id to the list of owner inputs, with a capacity bound |
| `BatchPlan::batch` | nondeterministic choice, per input, of one token (completed, keep open, close, split, split and close), which yields every validator-accepted result with at most one split boundary per input; or a batcher error. Output count is bounded by `2 * n` |
| size threshold for a size flush | nondeterministic: any admitted input may trigger a size flush |
| tail retention | nondeterministic, guarded by the structural conditions (size flush, more than one output, single-owner last range on the last input) |
| `forward_as_is` (passthrough) | not modeled; it takes no slot and its context goes downstream unchanged |
| `finish_batch` failure | nondeterministic: none, the first or the second output of a flush |
| downstream | nondeterministic Ack or Nack per tracked output, in any order, until `Shutdown` |
| local scheduler | one live wakeup per buffer, plus a shutdown latch that clears it and makes `set_wakeup` fail |

This extends the range abstraction in two ways that the code requires:

- Holes are allowed when they are listed in `completed_inputs`. Such inputs
  release their pending reference at flush and are Acked if they hold no
  other reference.
- Only single-owner tails on the last input can be retained.

Each model step is one `process()` call, which the engine runs to completion
before the next message. Helper names in the model follow the Rust
functions they mirror: `accept_payload`, `flush_signal_impl`,
`take_remaining`, `handle_partial_responses`, `set_arrival`. The actions are
one per message kind: `accept_buffer` and `accept_flush` (PData), `wakeup`,
`downstream` (Ack or Nack), `begin_drain` (the inbox latches `Shutdown`)
and `shutdown` (`flush_shutdown`). See `README.md` for the checks.

## Findings

### F1. Input arriving during a drain fails the node (L9, path A)

Status: fixed. Reproduced by the model (instance `nofix`, scenario
`f1CrashOnMainTest`) and by the Rust test
`test_input_during_shutdown_drain_is_buffered_until_shutdown`, which fails
on `36a5c4ff3` with "could not set wakeup". Violates S6 and L1.

When the inbox latches `Shutdown` with a future deadline, it calls
`NodeLocalScheduler::begin_shutdown`, which clears all wakeups and makes
`set_wakeup` return `WakeupError::ShuttingDown`. The inbox keeps delivering
pdata during the drain.

Path: `accept_payload` on an empty buffer with a pending size below the
lower limit calls `set_arrival`, which returns `EngineError::ProcessorError`
("could not set wakeup"). `process` returns `Err`, the processor loop exits,
and the input that was just buffered (and its inbound slot) is never
answered.

Reproduction: `min_size = 5`, `max_size = 10`, items sizer,
`max_batch_duration = 50ms`. Latch scheduler shutdown, then send one
subscribed log record. Expected: `process` returns `Ok`, the input stays
buffered, `Shutdown` flushes it and the Ack reaches upstream. Today:
`process` returns `Err`.

This is the same behavior reported as L9 against the previous batcher in
Part 3. The Part 3 fix (branch `experiment/tla-part3`, commit `84af6bbe3`)
treats `ShuttingDown` as success with no armed wakeup, and adds a test-only
`TestContext::begin_local_shutdown` hook. Neither is on `main`.

### F2. Size flush with a retained tail during a drain loses the flush

Status: fixed. Reproduced by the model (instance `nofix`, invariant
`no_crash_in_flush`, scenario `f2CrashOnMainTest`) and by the Rust tests
`test_otap_tail_retained_during_shutdown_drain_is_flushed_on_shutdown` and
`test_otlp_tail_retained_during_shutdown_drain_is_flushed_on_shutdown`,
which fail on `36a5c4ff3`. Violates S3, S6 and L1. New relative to Part 3.

Path: during a drain, a size flush whose last output qualifies as a tail
calls `set_arrival` after `take_remaining`. The error returns from
`flush_signal_impl` after the inputs were drained and output references
were added, but before any pending reference was released and before any
output was sent. Every input of that flush is stranded and its data is
dropped.

Reproduction, using the existing residual-test layout: `min_size = max_size
= 4` units, `max_batch_duration = 1s`. Buffer input A (3 units) before the
latch, latch scheduler shutdown, then send input B (6 units). The size flush
produces outputs of 4 and 4 units and a 1-unit tail from B. Expected: two
outputs sent, the tail flushed on `Shutdown`, A and B answered. Today:
`process` returns `Err`, no output is sent and neither input is answered.

The shortest counterexample found by TLC needs only one input: during a
drain, a single input larger than `max_size` (for example 6 units with
`min_size = max_size = 4`) is split into 4 and 2, and the 2-unit remainder
qualifies as a tail. The input does not need to be tracked; its data is
dropped either way.

### Fix for F1 and F2

`SignalBuffer::set_arrival` treats `WakeupError::ShuttingDown` as success: it
records the arrival, leaves `wakeup_armed` false and returns `Ok`. The input
or tail stays buffered and `flush_shutdown` sends it when `Shutdown` is
delivered. Other wakeup errors still fail the node; neither is reachable
(`Unsupported` needs a processor without local wakeups and `Capacity` needs
more live slots than the three per buffer that `local_wakeup_requirements`
reserves). The test-only hook `TestContext::begin_local_shutdown` latches
the scheduler the same way the inbox does. This is the Part 3 fix, ported;
the model's `FIX_SHUTDOWN_WAKEUP` toggle mirrors it.

### F3. Early returns inside a flush strand inputs

Status: confirmed by reading; not modeled (D5). Violates S3 and L1 on
engine-error paths.

After the reference loop in `flush_signal_impl`, several `?` returns exit
before all pending references are released and before all outputs are sent:
the tail `batch_size` measurement, `set_arrival` (F2), the `notify_*` calls
inside `handle_partial_responses` and the outbound-exhaustion branch, and
`send_message_with_source_node`. The send failure also leaks the outbound
slot allocated for that output. These are engine-level errors after which
the node stops, so the practical effect is that held contexts are dropped
without an answer. Whether the model treats them as violations is D5.

### F4. Timer early-skip clears the armed flag without re-arming

Status: latent, not a violation on `main`. With early wakeups enabled
(instance `early`) the model breaks L2.

On a timer flush that arrives before `arrival + max_batch_duration`,
`flush_signal_impl` sets `wakeup_armed = false` and returns without
re-arming. Buffered data would then wait for the next size flush or for
shutdown. This is safe today only because the scheduler keeps a single live
wakeup per slot, `set_arrival` sets `when = arrival + max_batch_duration`
exactly, and a wakeup is popped and delivered in one step, so a stale early
wakeup is not delivered in production. The model checks L2 with and without
this assumption (D9).

## Behaviors that are intended, not violations

- **Passthrough reorders output.** An input already within
  `[min_size, max_size]` is forwarded as-is by `forward_as_is` and can
  overtake buffered data. Its context goes downstream unchanged, so the
  engine routes its answer directly; the node takes no slot.
- **Untracked inputs are not answered.** Inputs without completion
  interests take no slot; the node drops their context.
- **Size-0 inputs are Acked immediately** in `accept_payload`.
- **Completed inputs are Acked at flush.** An input the batcher consumed
  without output (for example, every record removed by integrity repair)
  holds no output reference, so releasing its pending reference Acks it.
- **The first Nack answers immediately** (S2), even while sibling outputs
  of the same input are still in flight downstream.
- **Inbound slot exhaustion Nacks the new input** with a retryable Nack and
  drops it; the caller is expected to retry.
- **No downstream timeout in this node.** Slots stay held until downstream
  answers. Timeouts are the responsibility of other nodes.
- **No wakeups during a drain.** Data buffered when the drain starts is
  flushed by the final `Shutdown`, not by its timer.
- **A tail can wait up to about `2 * max_batch_duration`** (documented in the
  module header).
- **Duplicates on retry (provisional, see D4).** Three paths deliver data
  for an input that is Nacked upstream: outbound slot exhaustion sends the
  output while Nacking its owners; outputs already sent before the first
  Nack still deliver; and a retained tail of an input that was already
  Nacked is still sent later, untracked. An upstream retry duplicates that
  data. The code comments describe the first path as a deliberate choice.
- **Outputs of the shutdown flush are never answered by this node
  (provisional, see D3).** After `Shutdown` is returned to the processor,
  the inbox drops its channels, so Acks and Nacks for outputs emitted by
  `flush_shutdown` (and for outputs still in flight at a forced deadline)
  never reach the node.

## Decisions to revisit

Each decision lists the default this spec and the model use until you
decide otherwise.

- **D1. Folder name.** Default: `formal/batch_ack/`. Part 3 used
  `formal/shutdown/` on its own branch; `main` has no `formal/` yet.
- **D2. Fix F1 and F2 in this PR.** Resolved: fixed in `set_arrival` (see
  [Fix for F1 and F2](#fix-for-f1-and-f2)), with failing-first Rust tests.
- **D3. Where "settled" ends.** Default: L1 is checked up to, but not
  including, delivery of the final `Shutdown`. Answers for outputs of the
  shutdown flush are left to the engine shutdown protocol (Part 3).
- **D4. Duplicates on retry.** Default: intended at-least-once behavior,
  modeled and counted but not an invariant violation. Making them
  violations would require changing outbound-exhaustion handling.
- **D5. Engine-error exits (F3).** Default: modeled as a `crashed` state.
  Reaching it from a reachable scheduler state (F1, F2) is a violation;
  channel and notify failures are environment faults and out of scope.
- **D6. Model scope.** Default: model tracked and untracked buffered inputs,
  completed inputs, batcher errors and `finish_batch` failures. Passthrough
  and the inline rejects in `process_signal_impl` are summarized as "no slot
  taken" and not modeled in detail.
- **D7. Branch.** Default: keep working on the existing `quint/part1`
  branch, which is identical to `main` at `36a5c4ff3`.
- **D8. Tooling.** Resolved: Quint 0.33.1 is installed, with its Rust
  evaluator and Apalache 0.62.4 in `~/.quint`. TLC (through Quint) is the
  exhaustive backend; Apalache does not accept this model (see `README.md`).
- **D9. Stale wakeups (F4).** Default: the model delivers only the live
  wakeup. A toggle allows stale early wakeups to show what F4 would cost if
  the scheduler guarantee changed.

## Appendix A. Function-level flow

1. `process` dispatches `PData` to `process_signal_impl`, which selects the
   format route and calls `prepare_batch`.
2. `accept_payload`:
   - size 0: complete without output and Ack;
   - within the passthrough range: `forward_as_is`;
   - otherwise, if tracked, allocate an inbound slot with one reference,
     then `Inputs::accept`;
   - below the lower limit: `set_arrival` if the buffer was empty;
     otherwise `flush_signal_impl(Size)`.
3. `flush_signal_impl`:
   1. drain the buffer and call `BatchPlan::batch` (batcher plus
      `validate_input_ranges`); on error, Nack every input;
   2. add one reference per output range that names each input;
   3. if a tail qualifies, `take_remaining` and `set_arrival`;
   4. release one pending reference per drained entry with `Ok`;
   5. for each output: `finish_batch`, `output_context`, allocate an
      outbound slot, subscribe with calldata `[outkey, wakeup_slot]`, send.
4. Downstream answers arrive as `Ack` or `Nack` control messages:
   `handle_ack` or `handle_nack`, then `handle_response` (decodes the
   buffer from calldata), then `BatchProcessorSignal::handle`
   (`outbound.take`), then `handle_partial_responses`.
5. `Wakeup` runs `flush_signal_impl(Timer)`. `Shutdown` runs
   `flush_shutdown`, which flushes every buffer with
   `FlushReason::Shutdown`.

## Appendix B. Inventory

### B1. Where ranges come from

| Batcher | Returns | Where the contract could break |
| --- | --- | --- |
| OTAP `make_item_batches_with_ranges` (`crates/pdata/src/otap/batching.rs`) | Origins tracked through `split_with_origins` and `generic_concatenate` (`groups.rs`); zero-item outputs filtered out; uncovered inputs reported as completed | Filtering can leave holes, reported as completed. No cap on output count (about total items over `max_size`). |
| OTLP `make_bytes_batches_owned` (`crates/pdata/src/otlp/batching.rs`) via `OtlpBatcher` | `BatchWriter` sets `first..=last` on append; splits and `emit_top` produce `i..=i`; fast paths for one input and for no `max_size` | Output count bounded by `max_split_fragments_per_flush` (default 65536), with whole-entry fallback beyond it. `EmptyBatch` only when total bytes are 0, which the node prevents. |
| Syslog `SyslogBatcher` (`crates/pdata-codec/src/codecs/syslog/codec.rs`) | Logs and items only; an oversize input is split in isolation (`i..=i`) and its last fragment may merge forward (`i..=j`); never reports completed inputs | Bounded by `MAX_BATCH_ITEMS`, `MAX_BATCH_BYTES` and the split budgets. |
| `validate_input_ranges` (`crates/pdata-codec/src/batching.rs`) | Central check on every batcher result | No length bound. A violation becomes a retryable Nack to every input of the flush. Cannot detect valid but wrong ranges. |

### B2. Tail retention

| Aspect | Behavior |
| --- | --- |
| When | `FlushReason::Size`, `max_batch_duration != 0`, `max_size` set, more than one output, last range is one input and that input is the last buffered input, and `0 < size < lower_limit` |
| Attribution | `take_remaining` pops the last output and re-accepts it with the same `BatchPortion`; that output's reference becomes the pending reference |
| Timer | `set_arrival` resets `arrival` and re-arms the wakeup |
| Flushed by | timer flush, the next size flush, or shutdown; none of the latter two can retain the same tail again |

### B3. Slot and context tracking

| Event | Behavior |
| --- | --- |
| Input admitted | Inbound slot allocated only if `needs_completion_tracking()`. If full: retryable Nack "inbound routes exhausted", input dropped. |
| Flush | One reference added per output naming the input, then one removed per drained pending entry. A completed input reaches zero and is Acked. |
| Downstream Ack | `outbound.take`, one reference released per named input; Ack upstream on the last one. |
| Downstream Nack | `outbound.take`; each named live input is removed and Nacked immediately, keeping the classification. Later answers are ignored. |
| `finish_batch` failure | Permanent Nack to that output's owners. |
| Outbound slots full | Every owner of that output is removed and Nacked "outbound routes exhausted"; the output is still sent without a subscription. |
| Timeout | None in this node. |
| Shutdown | `flush_shutdown` emits everything; responses to those outputs never reach the node. |
| Slot release | Inbound: last reference, any error, or outbound exhaustion. Outbound: downstream answer only. |

### B4. Sites that create, copy or drop a context or slot

| # | Function | Effect |
| --- | --- | --- |
| 1 | `process_signal_impl`, no route | context answered with a permanent Nack |
| 2 | `process_signal_impl`, `prepare_batch` error | context answered with a permanent Nack |
| 3 | `accept_payload`, size 0 | `complete_processor_without_output`, then Ack |
| 4 | `forward_as_is` | permanent Nack on `finish_batch` error, otherwise the context is sent downstream unchanged |
| 5 | `accept_payload`, untracked | context dropped; `peer_addr` copied into the `BatchPortion` |
| 6 | `accept_payload`, inbound allocation | context moved into an inbound slot, or answered with a Nack if full |
| 7 | `flush_signal_impl`, reference loop | refcount incremented once per output |
| 8 | `take_remaining` | `BatchPortion` copied back into pending |
| 9 | `handle_partial_responses` | slot removed, context taken, Ack or Nack sent |
| 10 | `output_context` | portions with a live slot copied into the output's list |
| 11 | `flush_signal_impl`, outbound allocation | outbound slot created, or inbound slots taken and Nacked if full |
| 12 | `flush_signal_impl`, output pdata | new `Context::default()` subscribed with `[outkey, wakeup_slot]` |
| 13 | `handle_response` | returned pdata dropped |
| 14 | any `?` early return, or processor drop | every context still held in a slot is dropped without an answer |
