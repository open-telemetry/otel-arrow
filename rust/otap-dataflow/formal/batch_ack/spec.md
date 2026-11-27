# Batch processor Ack/Nack: specification

Part 1 of [#4395](https://github.com/open-telemetry/otel-arrow/issues/4395):
the Ack/Nack delivery guarantees of the batch processor node
(`crates/core-nodes/src/processors/batch_processor/mod.rs`), traced to the
Rust functions that implement them and checked by a Quint model
([README.md](README.md)). Also covered: `validate_input_ranges`
(`crates/pdata-codec/src/batching.rs`) and the local wakeup scheduler during
a shutdown drain (`crates/engine/src/node_local_scheduler.rs`, `message.rs`).

Not covered: sizes, codecs and payload contents (batchers are abstracted to
"any valid list of input ranges"); what upstream does with an answer; the
engine shutdown protocol (F5); metrics and metadata merging.

Written against `main` at `108a4ff87`. Paths are relative to
`rust/otap-dataflow/`; function names are the stable reference.

## Vocabulary

- **Input**: one `OtapPdata` delivered to `process`. A *tracked* input has
  `Context::needs_completion_tracking()`, owns an inbound slot while held,
  and gets one **answer**: an upstream Ack or Nack.
- **Output**: one batch sent downstream by a flush. A *tracked* output names
  a live tracked input and owns an outbound slot.
- **Range**: the inclusive range of buffered-input indices an output was
  built from. A **completed input** is consumed with no output
  (`BatchingResult::completed_inputs`).
- **Reference**: a unit of `BatchContext::outbound`, the refcount that keeps
  a tracked input unanswered: one per pending buffer entry, one per emitted
  output that names the input.
- **Tail**: the undersized last output of a size flush, moved back into the
  buffer instead of being sent.
- **Drain**: from the inbox latching `Shutdown` with a future deadline until
  it returns `Shutdown`. Pdata and Ack/Nack still arrive, but
  `NodeLocalScheduler::begin_shutdown` has cleared all wakeups and
  `set_wakeup` returns `WakeupError::ShuttingDown`.
- **Settled**: no further input arrives, every output has been answered
  downstream, and every armed wakeup has fired (bounded by D3).

## Batching contract

From #4347: each output names one contiguous inclusive range; ranges are
reported by the batcher, never derived from sizes; output sizes are best
effort and may fall outside `[min_size, max_size]`.

`validate_input_ranges` checks every batcher result: ranges are non-empty,
in bounds and ordered; consecutive outputs may share one boundary input (a
split) and nothing else; every input is covered by a range or listed as
completed, never both. A violation becomes `Err` from `BatchPlan::batch`,
answered with a retryable Nack to every input of the flush. The validator
does not bound the output count and cannot detect a valid range that names
the wrong inputs.

`every_batcher_honours_the_batching_contract` checks the OTAP and OTLP
batchers for logs, traces and metrics, and the syslog batcher for logs,
against per-record provenance markers.

## Invariants

`S` marks safety, `L` liveness.

### S1. Every tracked input is answered at most once

- Tracked inputs are answered only when their inbound slot is removed:
  `SignalBuffer::handle_partial_responses` and the outbound-exhaustion branch
  of `BatchProcessorSignal::flush_signal_impl`.
- `SlotState` keys are generational (`slotmap`): a slot is removed at most
  once, and a late response for a freed or reused slot is ignored.
- Inputs that never get a slot are answered inline, once:
  `process_signal_impl` (no route, `prepare_batch` failure), `accept_payload`
  (size 0, inbound exhaustion), `forward_as_is` (`finish_batch` failure).

### S2. A Nack on any output carrying an input makes the answer a Nack

An input is Acked only if every output carrying its data was Acked. The
**first** Nack answers at once, without waiting for sibling outputs; later
responses are ignored. Retryable and permanent are preserved.

- `BatchProcessorSignal::handle` takes the outbound slot and calls
  `handle_partial_responses` for each named input. On `Err` that removes the
  slot and sends `NackMsg::new` or `NackMsg::new_permanent`; on `Ok` it
  decrements and Acks on the last reference. Every output holds a reference
  until answered, so an Ack cannot precede a Nack.
- Other Nack sources: batcher `Err` (retryable, whole flush), `finish_batch`
  failure (permanent, that output's inputs), outbound exhaustion (retryable,
  that output's inputs).

### S3. Reference conservation

For every live inbound slot, `outbound` equals the pending entries naming
it, plus the outbound slots naming it, plus the outputs of the current flush
that name it and are not yet sent.

- `accept_payload` allocates the slot with `outbound: 1`.
- `flush_signal_impl` adds one reference per output range naming the input
  before releasing anything, then releases one pending reference per drained
  entry. A completed input reaches zero and is Acked. A retained tail keeps
  its output reference as a pending one (B2).
- `output_context` copies the portions with a live slot into the output's
  list, which the outbound slot holds until downstream answers.

S3 is the inductive invariant behind S1, S2 and S5. The flush is one atomic
model step except where it awaits (F3).

### S4. A retained tail is attributed to exactly the inputs its range names

Only a single-input last range on the last buffered input is retained, and
`take_remaining` keeps that input's `BatchPortion` (same `inkey`); see B2.

### S5. Slots do not leak

Once settled, the node holds no slots, and exhaustion produces an answer.

- Inbound slots are freed by `handle_partial_responses` (last reference or
  any error) and the outbound-exhaustion branch. Outbound slots are freed
  only by `BatchProcessorSignal::handle`.
- Inbound exhaustion: `accept_payload` Nacks and drops the input ("inbound
  routes exhausted"). Outbound exhaustion: `flush_signal_impl` Nacks every
  owner of the output ("outbound routes exhausted") and still sends it,
  without a subscription.

### S6. The node does not fail on a reachable scheduler state

A `process` error ends the processor loop and drops every held context
unanswered. Before the fix, `SignalBuffer::set_arrival` maps every
`set_wakeup` error to `EngineError::ProcessorError`, and `ShuttingDown` is
reachable during a drain (F1, F2, now fixed).

### L1. No tracked input is left unanswered once the node is settled

Requires S3, that every tail is eventually flushed (`FlushReason::Timer` and
`FlushReason::Shutdown` never retain), and that no flush exits early. Broken
by F1, F2 and F3. Checked only up to the shutdown boundary (D3); past it,
see F5.

### L2. Buffered data is flushed within a bounded time

`set_arrival` arms one wakeup when a buffer goes from empty to non-empty; a
retained tail re-arms it. A tail comes from the last input, so it is never
retained twice and waits at most about `2 * max_batch_duration`. During a
drain no wakeup can be armed, and buffered data waits for `Shutdown`.

## Model abstraction

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
| Nack classification | not modeled: an answer is only Ack or Nack, so S2's "retryable and permanent are preserved" is not checked |

A `process` error sets a `crashed` flag. In the model, F1 and F2 therefore
show up as S6 violations (`no_crash`); S3 and L1 are checked only while the
node hasn't crashed.

## Findings

### F1. Input arriving during a drain fails the node

Fixed. Violates S6 and L1. During a drain, `accept_payload` buffers an
undersized input into an empty buffer and calls `set_arrival`, which fails
with "could not set wakeup"; the input is never answered. Tests: model
`nofix`, `f1CrashOnMainTest`; Rust
`test_input_during_shutdown_drain_is_buffered_until_shutdown`.

### F2. Size flush with a retained tail during a drain loses the flush

Fixed. Violates S3, S6 and L1. During a drain, a size flush that retains a
tail calls `set_arrival` after `take_remaining`. The error leaves
`flush_signal_impl` after output references were added but before any
pending reference was released or output sent, stranding every input of the
flush. One input larger than `max_size` is enough. Tests: model `nofix`,
`no_crash_in_flush`, `f2CrashOnMainTest`; Rust
`test_otap_tail_retained_during_shutdown_drain_is_flushed_on_shutdown` and
`test_otlp_tail_retained_during_shutdown_drain_is_flushed_on_shutdown`.

### Fix for F1 and F2

`SignalBuffer::set_arrival` treats `WakeupError::ShuttingDown` as success
with no armed wakeup, so the input or tail stays buffered until
`flush_shutdown`. Other wakeup errors still fail the node and are
unreachable (`Unsupported` needs a processor without local wakeups;
`Capacity` needs more than the three slots per buffer that
`local_wakeup_requirements` reserves). `TestContext::begin_local_shutdown`
(test only) latches the scheduler as the inbox does; the model's
`FIX_SHUTDOWN_WAKEUP` toggle mirrors the fix.

### F3. Early returns inside a flush strand inputs

Confirmed by reading; not modeled (D5). After the reference loop in
`flush_signal_impl`, `?` returns exit before all pending references are
released and outputs sent: the tail `batch_size` measurement, `set_arrival`
(F2), the `notify_*` calls, and `send_message_with_source_node`, which also
leaks the output's outbound slot. The node stops and drops held contexts.

### F4. Timer early-skip clears the armed flag without re-arming

Latent. A timer flush before `arrival + max_batch_duration` sets
`wakeup_armed = false` and returns without re-arming. Safe today only
because the scheduler keeps one live wakeup per slot at exactly that
deadline and delivers it in one step. With early wakeups on (instance
`early`, D9) the model breaks L2.

### F5. Inputs still held at Shutdown are never answered

Confirmed by reading; not modeled (D3). Violates L1 past the shutdown
boundary. Handed to the engine shutdown work under #4395; not fixed here.

1. `InboxCore::recv_with_policy` latches `Shutdown`. The drain ends when
   `shutdown_drain_complete` holds (pdata channel closed and empty, local
   scheduler drained) or the deadline sleep fires; it does not wait for
   outstanding Acks or Nacks.
2. `InboxCore::shutdown` drops `control_rx` and `pdata_rx`, with any answers
   already queued, and only then is `Shutdown` returned.
3. `process` runs `flush_shutdown`, which calls
   `flush_signal_impl(FlushReason::Shutdown)` on every buffer. Each output is
   subscribed back to this node and its owners' inbound slots stay live,
   including inputs buffered during the drain, which the F1/F2 fix keeps.
4. The next `recv_when` returns `RecvError::Closed`; the loop in
   `ProcessorWrapper::start_with_completion_metrics` exits and the processor
   is dropped with its `inbound` slots and their upstream contexts.
5. A later downstream answer reaches
   `PipelineCompletionMsgDispatcher::unwind_ack` or `unwind_nack`, whose
   `send` gets `SendError::Closed` and drops it, with no signal.

At a forced deadline, the `forced_shutdown_signal` branch of
`start_with_completion_metrics` cancels the run future and drops the
processor where it is, possibly before `flush_shutdown`: buffered inputs are
neither sent nor answered.

Result: every tracked input with a live inbound slot when `Shutdown` is
released gets no Ack or Nack from this node. That covers inputs with outputs
in flight, inputs flushed by `flush_shutdown` and, at a forced deadline,
inputs still buffered.

Possible fixes, to decide with the engine shutdown work: the drain waits for
outstanding answers, or a node Nacks (retryable) what it still holds at
`Shutdown`. The second option runs into D4. The choice also decides
drain-time buffering: if the drain waits for answers, the node must flush
during the drain (it can do so when `set_arrival` gets `ShuttingDown`); if
held inputs are Nacked at `Shutdown`, buffering until then is fine.

## Behaviors that are intended, not violations

- **Passthrough reorders output.** An input already within
  `[min_size, max_size]` goes out through `forward_as_is` and can overtake
  buffered data. Its context goes downstream unchanged; no slot is taken.
- **Untracked inputs are not answered**; the node drops their context.
- **Size-0 inputs are Acked immediately** in `accept_payload`.
- **Completed inputs are Acked at flush** (S3), for example when integrity
  repair removed every record.
- **No downstream timeout in this node.** Slots stay held until answered.
- Also intended: first-Nack answers (S2), inbound-exhaustion Nacks (S5), and
  the drain and tail waits in L2.

## Decisions to revisit

Each lists the default the spec and model use. Numbers stay stable for
references from the model and README.

- **D1. Folder name.** Default: `formal/batch_ack/`; `main` has no
  `formal/` yet.
- **D3. Where "settled" ends.** Default: L1 is checked up to, but not
  including, delivery of the final `Shutdown`. Past that point, see F5.
- **D4. Nack followed by delivery.** The open maintainer question "Nack
  followed by delivery" in #4395, not a settled choice. Default: the model
  counts these deliveries but does not treat them as violations. Three paths
  deliver data of an input already Nacked upstream, which a retry then
  duplicates:
  - outbound exhaustion sends the output while Nacking its owners (S5); the
    code comments call this deliberate;
  - outputs sent before the first Nack still deliver (S2);
  - a retained tail of an already Nacked input is sent later, untracked.
    This is the weakest of the three: the data goes out after the node has
    reported it failed, with no owner left to answer.
- **D5. Engine-error exits (F3).** Default: a `crashed` state. Reaching it
  from a reachable scheduler state (F1, F2) is a violation; channel and
  notify failures are environment faults, out of scope.
- **D6. Model scope.** Default: tracked and untracked buffered inputs,
  completed inputs, batcher errors and `finish_batch` failures. Passthrough
  and the inline rejects in `process_signal_impl` are "no slot taken".
- **D9. Stale wakeups (F4).** Default: only the live wakeup is delivered. A
  toggle allows stale early wakeups, to show what F4 would cost if the
  scheduler guarantee changed.

## Appendix. Inventory

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

### B4. Sites that create, copy or drop a context or slot

| # | Function | Effect |
| --- | --- | --- |
| 1 | `process_signal_impl`, no route | context answered with a permanent Nack |
| 2 | `process_signal_impl`, `prepare_batch` error | context answered with a permanent Nack |
| 3 | `accept_payload`, size 0 | `complete_processor_without_output`, then Ack |
| 4 | `forward_as_is` | permanent Nack on `finish_batch` error, otherwise the context is sent downstream unchanged |
| 5 | `accept_payload`, untracked | context dropped; its metadata kept as a `MergeInput` beside the `BatchPortion` |
| 6 | `accept_payload`, inbound allocation | context moved into an inbound slot, or answered with a Nack if full |
| 7 | `flush_signal_impl`, reference loop | refcount incremented once per output |
| 8 | `take_remaining` | `BatchPortion` and `MergeInput` copied back into pending |
| 9 | `handle_partial_responses` | slot removed, context taken, Ack or Nack sent |
| 10 | `output_context` | portions with a live slot copied into the output's list; the range's metadata merged |
| 11 | `flush_signal_impl`, outbound allocation | outbound slot created, or inbound slots taken and Nacked if full |
| 12 | `flush_signal_impl`, output pdata | new context built from the merged metadata (no upstream subscriptions), subscribed with `[outkey, wakeup_slot]` |
| 13 | `handle_response` | returned pdata dropped |
| 14 | any `?` early return, or processor drop | every context still held in a slot is dropped without an answer (F3, F5) |
