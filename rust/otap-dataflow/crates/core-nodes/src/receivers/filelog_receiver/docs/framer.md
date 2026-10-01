# Filelog physical-line framer

`LineFramer` consumes source bytes through the [decoder](decoder.md), emitting
owned physical lines, bounded split fragments or validated truncated prefixes.
Consumed input can run ahead of emitted frames because of lookahead. Bodies
hold text or exact bytes; frame ranges also cover omitted delimiters and tails.
This portable primitive registers no receiver; the [Phase 1 spec][spec] defines
its surrounding contracts.

## Bounds and byte measures

Construct with `LineConfig` and `LineStart`. The effective body bound is
`B = min(max_line_bytes, max_record_bytes)`; equality fits. Bounds must accommodate
the largest encoding unit and fit allocation arithmetic.

| Measure | Meaning and use |
| --- | --- |
| Decoded UTF-8 length | Text bytes, including replacements; bounds text encodings with `Replace` or `Fail`. |
| Source-body length | Original bytes in `body_range`; bounds `Raw` mode under every decode policy. |
| Emitted body length | Actual `Text(String)` or `Bytes(Vec<u8>)` length, at most `B`; preserve-raw prospective sizing can require an earlier boundary. |
| Frame range | `frame_range`: half-open source ownership, including LF, stripped BOM and discarded tail as applicable; not bounded by `B`. |
| Allocation capacity | Reserved storage, including spare capacity; determines memory charges, not framing. |

For text encodings with `PreserveRaw`, prospective sizing checks
`max(decoded UTF-8 body bytes, exact source body bytes)` before appending each
unit, for both split and truncate. Thus body **lengths**, including the decoded
shadow of malformed input, determine limits even when output will be bytes.

All ranges are exact, half-open source-byte ranges. Body measures exclude
terminal LF and a stripped matching initial BOM; frame ownership includes both.
Raw mode retains BOM bytes as body data. Conflicting or later BOM-shaped content
follows [decoder policy](decoder.md#malformed-unit-and-bom-contract).
CR and NUL remain body data; empty lines still own LF.

## Step results and ownership

```text
next(input_offset, input) -> Result<LineStep, LineFailure>
complete_partial(Idle | PermanentEof) -> Result<CompletionStep, LineFailure>
```

| Field | Caller contract |
| --- | --- |
| `LineStep.consumed` | Accepted fresh bytes, at most four per call. Advance input slice/offset by this amount. |
| `LineStep.frame` | At most one owned output, with positive or zero consumption. Retain it. |
| `LineStep.advanced` | True if bytes were consumed or an event processed, even without output. Continue with remaining/empty input; false means wait for bytes. |
| `CompletionStep.frame` | At most one earlier or partial output. Retain it even when `complete` is true. |
| `CompletionStep.complete` | True ends this operation, not necessarily the split sequence. Otherwise repeat with the same reason. |

`next` handles at most one decoded event or retained overflow unit. Empty input
drains work without completing an incomplete line. On `Err(LineFailure)`, advance
by `consumed` (zero for completion), retain previous outputs, and follow
[error recovery](#error-recovery). Failure returns no frame but may consume input,
including lookahead.

Stop at any step for backpressure or the scheduler's turn budget, retaining the
framer and unconsumed input. The framer holds no input borrow or output queue;
callers must bound both stepping and completion loops.

Input position, decoder-delivered boundary and returned frame boundary differ.
For UTF-16 `AB`, a high surrogate, then `C`, replacement with a four-byte limit
can consume through offset 8 while emitting `AB` through offset 4. The framer
retains the replacement unit at `[4,6)`; the decoder retains `C` at `[6,8)`.
Continue live input at 8. Do not discard either pending unit or resume live
input at 4. `pending_source_start()` identifies the first consumed byte not
owned by output, not a committed frontier.

## States and completion

### Oversized lines

`LineConfig.oversize` selects a policy. Runtime state starts at `Line`, or
`Split` for continuation recovery; crossing the bound activates that policy:

| Active state | Body unit | LF |
| --- | --- | --- |
| `Line` | Append if it fits; otherwise apply the configured policy | Emit line |
| `Split` | Append if it fits; otherwise emit a nonfinal fragment and retain the overflow unit | Emit final fragment |
| `Truncate` | Validate and count; retain no tail bytes | Emit prefix owning the complete frame |

LF or final partial output resets to `Line`; terminal errors stop every state.
Boundaries never cut a [source unit](decoder.md#malformed-unit-and-bom-contract).

### Representation and evidence

Every preserve-raw split fragment uses exact bytes, including clean fragments.
Clean unsplit/truncated prefixes can remain text; `source_body` retains exact
bytes for multiline representation changes without rereading. Byte bodies need
no shadow. Malformed truncate tails count in `malformed_units` without changing
a clean prefix's representation.
`discarded_source_bytes` excludes LF. Output transfers body and shadow ownership;
consumers charge both buffers by capacity.

### Failure ordering

Truncate emits only after validating the complete tail through LF or authorized
completion. A fail-policy error suppresses that record's prefix. Earlier frames
remain caller-owned and must resolve before quarantine. A malformed unit that
would overflow an unfinished split prefix does not authorize that prefix:
decoding fails before a safe boundary is established.

### Completion and lifecycle

After supplying all input, only caller-established [idle eligibility][idle] or
[permanent rotation EOF][eof] authorizes `complete_partial`. It drains earlier
events before resolving incomplete input. While `complete` is false, `next`
calls and reason changes return `CompletionInProgress`; retain the reason across
pauses. Final partial output accompanies `complete: true`, marked `Idle` or
terminal-unterminated `PermanentEof`. Retain it, then stop the completion loop.

After `complete: true` with **either** reason, `next` accepts input at
`next_expected_input_offset()`: `PermanentEof` does not close the primitive.
After partial emission, later bytes start a new line/unit. Receiver lifecycle
still owns permanent-EOF finalization; API acceptance does not authorize intake.
Completion never re-enables BOM probing. An empty completion before any input
leaves the initial new-stream probe unchanged; `ResumeAt` and `Continuation`
never probe, even at offset zero.

A resumed split with no new bytes returns `frame: None, complete: true` for
either reason, retaining continuation state. The **split sequence** remains
pending; this completion operation is finished. Await source/lifecycle progress
instead of retrying completion or fabricating a final fragment/clean checkpoint.
Empty completion is idempotent; a BOM-only range can complete with an empty body.
Shutdown, read pauses, empty chunks and ordinary EOF never authorize completion.

### Error recovery

`CompletionInProgress` and `next`'s input-offset `Decode(OffsetDiscontinuity)`
leave state usable: finish the active completion or correct the offset, then
retry. Malformed-input, source-offset overflow, allocation and fragment-index
failures latch; subsequent input/completion calls return the same error with
zero consumption. `terminal_error()` reports this status. Recovery requires
caller-authorized reconstruction; constructor errors create no instance.

`Decode` preserves decoder evidence but follows framer recovery rules.
Completion drains first, so `DrainRequired` indicates an internal violation and
latches rather than allowing retry.

## Continuation and consumer boundaries

Nonfinal fragments carry the original frame start and next `u32` index.
Projection combines origin and file identity/epoch for correlation IDs. Index
overflow fails before nonfinal emission; a final `u32::MAX` fragment is allowed.

Delivery proposes ranges/continuation only after accepting output into its
bookkeeping; applied/durable advancement requires Ack, transaction and sync rules.
`LineContinuation` maps to scan-to-LF (`record_end_offset == 0`), without changing
checkpoint formats.

For [restart][restart], validate identity, continuity, profile compatibility and
a safe source-unit boundary. Construct `LineStart::Continuation` at the returned
frame end; reread surviving bytes without re-emitting the prefix or re-probing
BOM. This authorized replay differs from retaining live lookahead across pauses:
a returned boundary does not prove the decoder empty. Reconstruction never
authorizes dropping pending failure state.

Multiline grouping preserves internal LF separators in the configured encoding
and resolves earlier buffered content before oversized-line fragments or later
failure. It owns patterns, aggregate record bounds, known-end continuation and
grouping metadata. Scheduling/lifecycle own source turns and rotation; receiver
memory accounting owns aggregate retention.

## Resource bounds

Text decoding uses one text buffer, plus an exact-source buffer for preserve-raw;
raw mode uses one byte buffer. Geometric growth caps each at `B`; retained
payload is at most `2 * B`, plus inline decoder state and one event. Truncate
memory does not grow with the tail. There is no per-unit heap allocation,
rescanning, synchronization or retained input borrow.

`retained_capacity()` reports owned heap capacities; add `size_of::<LineFramer>()`
for inline storage. Caller scratch, transferred output, allocator overhead and
transient old/new growth allocations are separate charges. Reservation and
aggregate admission belong to the receiver.

Emitting a preserve-raw byte body clears text length but retains capacity, even
at an empty line boundary. A later text output transfers that storage, possibly
far larger than its body length. Charge returned `String`/`Vec` and `source_body`
capacities; assume no fixed capacity-to-length ratio. Retained allocations remain
charged until transferred or dropped. Allocation failure returns no frame.
Source offsets and fragment indices use checked arithmetic; malformed counts
cannot exceed distinct source bytes in a `u64` range. Append bounds use subtraction.

[Framer tests](../framer/tests.rs) cover independent bodies/ranges, exhaustive
small-fixture partitions, continuation replay and bounded long-line retention.
The [allocation test](../../../../tests/filelog_framer_allocations.rs) checks
allocation-free truncate scanning. These are primitive checks, not full receiver
performance qualification.

[spec]: ../../../../../../docs/filelog-receiver-phase1-spec.md#source-decoding-and-framing
[idle]: ../../../../../../docs/filelog-receiver-phase1-spec.md#idle-partial-flush
[eof]: ../../../../../../docs/filelog-receiver-phase1-spec.md#permanent-eof-and-terminal-framing
[restart]: ../../../../../../docs/filelog-receiver-phase1-spec.md#split-continuation-restart
