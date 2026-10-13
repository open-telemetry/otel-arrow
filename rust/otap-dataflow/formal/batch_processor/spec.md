# Batch processor Ack/Nack aggregation: intended invariants

This document states, in plain language, what the batch processor must
guarantee about Ack/Nack outcomes for the requests it batches. Each property
is traced to the code that is responsible for it. The TLA+ model in this
directory (`BatchAggregation.tla`) checks these properties with the TLC model
checker.

Unless noted, line numbers point into
`crates/core-nodes/src/processors/batch_processor/mod.rs` (called `mod.rs`
below), as of the commit that fixes the metrics `debug_assert!` described in
"Findings". That commit sits on top of `298846b57`; lines after 1190 are 5
higher than in `298846b57`.

## Why this needs care

The batch processor repacks N inbound requests into M outbound batches. One
request can be split across several batches, and one batch can carry pieces of
several requests. Each request that asked for completion tracking must get
exactly one outcome:

- Ack, only after every piece of its data has been confirmed downstream.
- Nack, as soon as any piece fails, or when the processor cannot track it.

The bookkeeping is a reference count per request. A counting error does not
crash. It silently produces one of four failures:

| Failure | Effect |
| --- | --- |
| Premature Ack | Sender drops data that was never delivered (data loss). |
| Missing outcome | Sender waits forever (hung request). |
| Duplicate outcome | Sender sees two answers for one request. |
| Leaked slot | Capacity shrinks until every new request is Nacked. |

## Vocabulary

- **Request**: one inbound `OtapPdata`. It is *tracked* when its context asks
  for completion tracking (`ctx.needs_completion_tracking()`, line 1002),
  otherwise *untracked*.
- **Inbound slot**: entry in `SignalBuffer::inbound`, one per tracked request
  that is being batched. It holds the request context and a counter
  `outbound` (struct `BatchContext`, line 517).
- **Portion**: a contiguous run of one request's data, described by a
  `BatchPortion` (line 525) with an optional inbound key and a weight.
- **Batch**: one outbound `OtapPdata` built by a flush.
- **Outbound slot**: entry in `SignalBuffer::outbound`, one per sent batch that
  carries at least one tracked portion. It lists those portions.
- **Ownership weight**: the number of input units a batch represents. The
  batcher returns it next to each batch. It differs from the batch's real size
  only for split OTLP fragments (see "Sizing modes").
- **Residual**: the last batch of a size flush when it is below the lower limit
  and is put back into the buffer instead of being sent (`take_remaining`,
  line 1589).
- **Reference**: one unit of the inbound `outbound` counter.

Slot keys come from a generational slot map (`crates/otap/src/accessory/
slots.rs`). A key that was removed never matches a later entry, so a late
response for a finished request is ignored (`take` and `mutate` return `None`).

## Life of a request

| Step | Code | Effect on slots |
| --- | --- | --- |
| Size 0 | `accept_payload`, 978-987 | Ack at once. No slot. |
| Already a good batch | 991-993, `forward_as_is` 1064 | Sent unchanged with its own context. No slot. |
| Tracked, inbound full | 1002-1020 | Nack ("inbound routes exhausted"). Request dropped. |
| Tracked, slot free | 1002-1021 | Inbound slot with `outbound = 0`. Data buffered. |
| Untracked | 1023-1024 | Data buffered with no slot. |
| Buffer reaches lower limit | 1041-1058 | Size flush. |
| Timer fires, data buffered | 1424-1445, 1105-1116 | Timer flush, unless the timer is early. |
| Shutdown | `flush_shutdown`, 756 | Flush everything. |
| Batcher fails | 1151-1164 | Nack every buffered tracked request; slots removed. |
| Size flush, small last batch | 1174-1227 | Last batch becomes a residual; its request gets +1. |
| Each sent batch | `drain_context` 1651 | +1 for each tracked request it carries (residual hands over its +1). |
| Outbound slot free | 1279-1285 | Batch subscribes for Ack/Nack with the outbound key. |
| Outbound slot full | 1257-1278 | Every carried request still live: slot removed, Nack. Batch still sent. |
| Downstream Ack | `handle` 1309, 1702-1735 | -1 per carried request; at 1 -> remove slot, Ack. |
| Downstream Nack | same | Remove slot of each carried request, Nack (first failure wins). |

## Sizing modes

The batching code differs per format and signal. The model covers all three
modes because the counting logic relies on how batches are cut.

1. **Items, aligned** (OTAP logs and traces). Each log record or span is one
   item. `make_item_batches` (`crates/pdata/src/otap/batching.rs`) first splits
   each input at boundaries aligned to the running item total
   (`plan_split`, `crates/pdata/src/otap/transform/split.rs`), then joins
   pieces greedily up to `max_size` (`generic_concatenate`,
   `crates/pdata/src/otap/groups.rs`). Every batch except the last is exactly
   `max_size`. Ownership weight equals real size.
2. **Items, metric rows** (OTAP metrics). The item is a data point, but a
   metric row (with all its points) is indivisible. `plan_metrics_split`
   cuts each input on row boundaries and does not align to the running total.
   A row larger than `max_size` is emitted on its own, above `max_size`.
   Ownership weight equals real size.
3. **Bytes** (OTLP). `make_bytes_batches_owned`
   (`crates/pdata/src/otlp/batching.rs`) packs whole resource entries in
   order. An entry that does not fit starts a new batch. An entry that is
   larger than `max_size` on its own is split into standalone fragments
   (`split_resource_entry`): whole scopes are packed first, and only a scope
   that is too large is split into records. Each fragment repeats the
   resource header, or the resource and scope headers, so the fragments'
   real sizes add up to more than the entry, and they need not be equal.
   Their ownership weights are reset to an even share of the entry size
   (`rescale_ownership`), so a fragment can own more or less than its real
   size. An entry that would exceed the split budgets is emitted whole,
   above `max_size`.

In all modes the batches come out in input order, and ownership weights add
up to the total input weight. `drain_context` relies on both facts: it walks
the buffered portions in order and assigns each batch a window of exactly its
ownership weight.

## Ground truth

The model keeps hidden ("ghost") variables that the code does not have. For
every request and every unit of its data, they record where that unit really
is: buffered, in a sent batch that has not been answered, confirmed by an Ack,
or lost (in a batch that got a Nack). The ghost state follows the *content* of
each batch, not its ownership weight. This lets the model detect a batch that
is attributed to one request while it carries another request's data.

## Properties

Each property says what must hold, why, and which code is responsible.
"Tracked request" means a tracked request that received an inbound slot.

### P1. No premature Ack

A tracked request is Acked only when every unit of its data has been
confirmed downstream: no unit is still buffered, in flight, or lost.

- Code: the Ack is sent only when the counter reaches the last reference
  (1714-1729). The counter must therefore cover every batch and residual
  that holds the request's data.
- Depends on: `drain_context` adding a reference for every batch that carries
  the request (1682-1690); `take_remaining` adding one for the residual
  (1632-1637); ownership windows matching batch content (see "Ground truth").

### P2. At most one outcome per request

No request receives two outcomes (Ack and Nack, or two of the same).

- Code: every outcome for a buffered request is sent right after its inbound
  slot is removed (`inbound.take` at 1260, `inbound.mutate` returning the
  removed entry at 1711-1721, the batcher-error path at 1159-1161 which reuses
  1711-1721). A removed key never matches again (generational keys).
- Requests that never get a slot (size 0, pass-through, inbound full) receive
  their single outcome on the spot (985, downstream directly, 1014).

### P3. Exact reference counts

At every step between events, for every live inbound slot:

```text
outbound = (number of portions naming it, over all live outbound slots)
         + (1 if a residual of this request is buffered, else 0)
```

Buffered data that has never been flushed holds no reference, so a freshly
accepted request has `outbound = 0`. This is safe because no batch can be
answered for it yet.

- Code: increments at 1635 and 1686, hand-over at 1680, decrements at
  1714-1715, removals at 1260, 1311, 1706.
- Note: the count is per portion. `drain_context` adds one reference per
  portion it routes, and `handle_partial_responses` releases one per portion,
  so a batch that carries two portions of one request stays consistent.

### P4. A full slot table always produces a Nack

If a tracked request cannot be tracked because a slot table is full, it gets a
Nack. It is never left without an outcome.

- Inbound full: Nack at 1010-1015.
- Outbound full: for each carried request whose slot is still live, the slot
  is removed and a Nack is sent (1257-1277). The batch is still sent, because
  it may also carry untracked data.

### P5. A residual comes only from the last input

When a size flush keeps a residual, all of its data belongs to the input that
triggered the flush (the last buffered input).

- Code: `debug_assert!` at 1623, which justifies attributing the residual to
  `last_input.inkey` (1627).
- Why it should hold: the flush is triggered by the input that arrives last.
  Everything buffered before it totals less than the lower limit, which is at
  most `max_size`, so it all fits in the first batch. With more than one batch,
  the last batch can only hold data from the last input.

### P6. Drained means empty

Whenever nothing is buffered and no batch is waiting for an answer, both slot
tables are empty, and every tracked request that was accepted has received
exactly one outcome.

- This is a safety property (a state predicate). It detects leaked slots and
  hung requests without needing fairness assumptions.
- Code: every removal path in P2 sends an outcome; `drain_context` and
  `take_remaining` are the only places that add references.

### P7. Guards on modeled paths never fail

No `expect`, `unreachable!`, or `debug_assert!` on a modeled path can fail.

| Line | Guard | Claim |
| --- | --- | --- |
| 442 | `expect("valid")` in `lower_limit` | `min_size` or `max_size` is set. |
| 1189-1202 | `debug_assert!` | Logs and traces, more than one batch: the first batch reaches the lower limit (metrics are exempt, see "Findings"). |
| 1200 | `debug_assert!` | Bytes mode: the last batch has a known size. |
| 1598 | `expect("has last")` | There is a last batch to retain. |
| 1599 | `expect("has last")` | There is a last input. |
| 1600 | `expect("known size")` | The residual has a known size. |
| 1623-1626 | `debug_assert!` | P5. |
| 1664 | `expect("valid")` | `contexts.pos` is in range. |
| `rescale_ownership` | `debug_assert!` | Each split fragment owns at least one input unit. |

### P8. Size 0 and pass-through never touch the slot tables

A size-0 request and a pass-through request never allocate, read, or remove an
inbound or outbound slot, and the processor never subscribes to their
responses.

- Code: both return before the slot code (978-993); `forward_as_is` sends the
  original context (1064-1074).

## Behaviors that are intended, not violations

- **Untracked data is sent without bookkeeping.** An untracked request has no
  slot, and its data can share batches with tracked data.
- **Data can be delivered after a Nack.** If an outbound slot is full, the
  request is Nacked but its batch is still sent (1273-1275). A residual of a
  Nacked request is later sent untracked. Delivery is therefore at least once:
  a sender that retries after the Nack may cause duplicates downstream.
- **A batcher error ends the flush with an engine error** after Nacking every
  buffered tracked request (1151-1163).
- **Late responses are ignored.** A response for a batch whose requests were
  already answered finds no live slot and changes nothing.

## Findings

TLC found one violation, of P7 in metric-rows mode. With
`min_size = 2` and `max_size = 3`, a single metrics request with two metrics
of 1 and 3 data points is split into batches of 1 and 3. The first batch is
below the lower limit, so the `debug_assert!` at 1189 failed: tests and
debug builds panicked. Release builds sent the short batch, and Ack/Nack
counting stayed correct (all other properties hold in this mode).

The assert assumed every items-mode split fills the first batch. That holds
for logs and traces, whose split is aligned to the running item total, but
not for metrics, whose split keeps metric rows whole and is not aligned.

- Reproduced by `test_metrics_size_flush_first_batch_below_min_size` in
  `mod.rs` (two metrics of 2 points, `min_size = max_size = 3`).
- Fixed by exempting metrics from the assert at 1189. The model mirrors
  the fix, and all properties now hold in all three modes.

## Assumptions about the environment

- Downstream answers each subscribed batch exactly once, with Ack or Nack, in
  any order and at any time.
- The engine delivers each outcome the processor sends. A failure to send an
  outcome is an engine error and is out of scope.
- Messages are processed one at a time (the processor is single-threaded per
  instance). Each action in the model is one call to the processor.

## Out of scope

Multiple signals or formats at once, encoding details, the processor's own
telemetry metrics, peer addresses, wakeup mechanics beyond "a timer flush can
happen when data is buffered", configuration validation, immediate-flush mode
(`max_batch_duration = 0`), other processors, proofs, and CI integration.
