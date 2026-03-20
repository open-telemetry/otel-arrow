# Log Subsampling Processor

Status: **Design** -- not yet implemented.

URN: `urn:otel:processor:log_subsampling`

## Overview

The Log Subsampling processor reduces log volume by discarding a portion of
incoming log records according to a configurable subsampling policy. The
processor treats all incoming log records as equal and makes no attempt to
classify them further. Pipeline administrators are expected to configure their
pipeline such that all logs reaching a processor instance can be considered
equivalent.

In a typical deployment, a Router Processor using OPL-based classification
sits upstream of this processor. The router classifies incoming telemetry
and directs logs to the appropriate subsampling instance based on those
classifications.

## Signal Handling

| Signal  | Behavior               |
|---------|------------------------|
| Logs    | Apply subsampling      |
| Metrics | Pass through unchanged |
| Traces  | Pass through unchanged |

Non-log signals are forwarded downstream without modification. This makes
the processor resilient to pipeline misconfiguration where metrics or traces
are inadvertently routed through it.

## Configuration

The processor configuration uses an externally tagged enum on the `policy`
field. Exactly one policy must be specified.

### Zip Sampling

Emit at most N log records per time window.

```yaml
config:
  policy:
    zip:
      interval: 60s
      max_items: 100
```

| Field       | Type     | Required | Description                    |
| ----------- | -------- | -------- | ------------------------------ |
| `interval`  | duration | yes      | Length of the sampling window  |
| `max_items` | integer  | yes      | Max records to emit per window |

Constraints: `interval > 0`, `max_items > 0`.

### Ratio Sampling

Emit a fixed fraction of log records.

```yaml
config:
  policy:
    ratio:
      emit: 1
      out_of: 10
```

| Field    | Type    | Required | Description                          |
|----------|---------|----------|--------------------------------------|
| `emit`   | integer | yes      | Numerator of the sampling fraction   |
| `out_of` | integer | yes      | Denominator of the sampling fraction |

Constraints: `emit > 0`, `out_of > 0`, `emit <= out_of`.

The ratio `emit / out_of` determines the fraction of records to keep. For
example, `emit: 1, out_of: 10` keeps approximately 10% of records.

## Subsampling Policy Details

### Record Selection

Policies are evaluated against each incoming otap batch. Some number of records
are selected to be kept based on the policy and the rest are discarded. 
Which logs are kept and which are discarded is considered arbitrary and either 
policy may simply keep the first N that fits into the current budget from each
batch.

### Zip Sampling Algorithm

The zip sampler tracks a `count` of records emitted in the current
window and resets it to zero when the window elapses. We'll implement the
timer via the engine's TimerTick message.

At startup (or on first message), the processor calls
`effect_handler.start_periodic_timer(interval)` to register a
per-node periodic timer with the engine. The engine delivers
`NodeControlMsg::TimerTick` at a fixed, drift-free cadence.

On `TimerTick`: reset `count = 0`.

On each incoming log batch of size B:

1. Compute `budget = max_items - count`.
2. Compute `to_keep = min(budget, B)`.
3. Update `count += to_keep`.
4. If `to_keep > 0`, slice and forward the first `to_keep` rows.
5. If `to_keep == 0`, ack the batch immediately (see Ack/Nack section).

Windows follow a fixed wall-clock cadence regardless of traffic
patterns. Ticks continue to fire during idle periods (harmless -- the
reset on an already-zero count is a no-op). The processor must handle
the `TimerTick` control message. There is no `window_expiry` state to
manage since the engine handles the scheduling.


#### Option 2: Lazy Window 

State: `window_expiry` (Instant), `count` (usize).

No background timer. The window is checked and reset on demand when a
batch arrives. On each incoming log batch of size B:

1. If `now >= window_expiry`, reset `count = 0` and set
   `window_expiry = now + interval`.
2. Compute `budget = max_items - count`.
3. Compute `to_keep = min(budget, B)`.
4. Update `count += to_keep`.
5. If `to_keep > 0`, slice and forward the first `to_keep` rows.
6. If `to_keep == 0`, ack the batch immediately (see Ack/Nack section).

Windows are anchored to traffic arrival, not the wall clock. If no data
arrives for several intervals, only a single reset occurs on the next
arrival. Unused budget from past windows does not carry forward. No
timer lifecycle or control message handling is required.

### Ratio Sampling Algorithm

The ratio sampler keeps M (`emit`) out of every N (`out_of`) records.

The algorithm is based on integer arithmetic roughly as follows:

- `emitted` -- records emitted in the current cycle (initialized to 0)
- `seen` -- records observed in the current cycle (initialized to 0)

Conceptually, for each log record: increment `seen`, and if
`emitted < emit` then emit the record and increment `emitted`. When
`seen` reaches `out_of`, reset both counters to 0. Within each cycle
of `out_of` records, the first `emit` are kept and the remaining
`out_of - emit` are dropped.

At the batch level, the processor computes how many records to keep
from the batch in O(1) using a closed-form formula rather than iterating
per record. Given state `(emitted, seen)`, config `(emit, out_of)`, and
batch size `B`:

```text
remaining_in_cycle = out_of - seen
from_current       = min(B, remaining_in_cycle)
keep_from_current  = min(max(emit - emitted, 0), from_current)

after_current = B - from_current
full_cycles   = after_current / out_of
leftover      = after_current % out_of

to_keep = keep_from_current + (full_cycles * emit) + min(emit, leftover)
```

After processing, update state:

```text
new_seen    = (seen + B) % out_of
new_emitted = min(emit, new_seen)
```

If `to_keep > 0`, slice and forward the first `to_keep` rows.
If `to_keep == 0`, ack the batch immediately (see Ack/Nack section).

The `to_keep` count determines how many contiguous rows to take from the
front of the batch. This does not correspond to the exact row positions
the per-record counter logic would select -- it is only the count that
matches. This keeps the slicing simple and consistent with zip sampling.

**Implementation note:** The O(1) formula will be tested against an O(B)
per-record loop.  The per-record algorithm is:

```text
to_keep = 0
for _ in 0..B:
    seen += 1
    if emitted < emit:
        emitted += 1
        to_keep += 1
    if seen == out_of:
        seen = 0
        emitted = 0
```

Example with `emit: 2, out_of: 5`:

| Batch | Size | State before      | to_keep | State after       |
| ----- | ---- | ----------------- | ------- | ----------------- |
| 1     | 12   | emitted=0, seen=0 | 6       | emitted=2, seen=2 |
| 2     | 4    | emitted=2, seen=2 | 1       | emitted=1, seen=1 |
| 3     | 5    | emitted=1, seen=1 | 2       | emitted=1, seen=1 |
| 4     | 4    | emitted=1, seen=1 | 1       | emitted=2, seen=4 |

Total records in: 12 + 4 + 5 + 4 = 25
Total records emitted = 6 + 1 + 2 + 1 = 10 (exactly 2 out of 5)

## Row Slicing Implementation

When slicing some number of rows from the root record batch, we have to recursively
process the child record batches. We have two implementations of this:

- A private implementation as a part of batching code in `pdata::otap::transform::split`
- A maybe public implementation in `pdata::otap::filter`.

We need to determine the right approach for this processor and if something 
can be unified here.

## Ack/Nack

By deploying a subsampling processor, the pipeline operator has explicitly
opted into data loss as a volume reduction strategy. The dropped records
are intentionally discarded, not failed. As such, the ack/nack semantics
for this processor are less about delivery guarantees for dropped data
and more about preventing behavioral anomalies on retry.

There are two design options for how the processor interacts with the
ack/nack chain.

### Common Behavior (Both Options)

**All records dropped (to_keep == 0):** The processor immediately acks
the inbound request by calling `notify_ack(AckMsg::new(pdata))`. No
data is sent downstream. This matches the batch processor's convention
for empty records.

**Empty incoming batch (num_items == 0):** Ack immediately, same as
the all-records-dropped case.

### Option 1: Transparent (No Ack/Nack Participation)

The processor does not subscribe to downstream ack/nack interests and
does not maintain any correlation state between inbound and outbound
messages.

When `to_keep > 0`, the processor constructs a new `OtapPdata` with
the sliced Arrow records and the original `Context`, then sends it
downstream via `send_message_with_source_node`. Because the original
context is preserved, downstream acks and nacks propagate transparently
to the original sender. The processor is invisible in the ack/nack chain
and does not handle `Ack` or `Nack` control messages.

**The double-subsampling problem:** If a downstream node nacks the
subsampled data, the nack propagates upstream past the subsampling
processor (since it did not subscribe). If a retry processor upstream
re-sends the data, it re-sends its own saved copy of the original
records. Those records arrive at the subsampling processor as new input
and are subsampled again. For example, with a 1:10 ratio, 100 original
records become 10 on the first attempt. On retry, those 10 are
subsampled again to 1. The effective ratio on retry becomes 1:100
instead of 1:10.

**Mitigation without Option 2:** Pipeline operators can place the
retry processor after the subsampling processor so that retried data
is not re-subsampled. However, this constrains pipeline topology.

### Option 2: Ack/Nack Participation

The processor subscribes to downstream ack/nack interests and maintains
a slot-based correlation between inbound and outbound messages. This
prevents double-subsampling on retry at the cost of moderate additional
complexity.

When `to_keep > 0`:

1. If the incoming pdata has subscribers (`context.has_subscribers()`),
   allocate a slot storing the original `Context`.
2. Call `effect.subscribe_to(Interests::ACKS | Interests::NACKS,
   slot_key.into(), &mut pdata)` to push a frame onto the outgoing
   context.
3. Send the subsampled data downstream.

The processor does NOT subscribe with `Interests::RETURN_DATA`. Only
the original `Context` is stored -- not the Arrow record payload. This
keeps memory overhead minimal.

On `Ack`: look up the slot by calldata, retrieve the original `Context`,
and call `notify_ack` with an empty-payload `OtapPdata` constructed
from the original context.

On `Nack`: same as ack, but call `notify_nack` instead. The upstream
retry processor holds its own copy of the original data (via its own
`RETURN_DATA` subscription) and will re-send the full original records.
The subsampling processor then re-subsamples from the full original
rather than from the already-subsampled result.

**Counter state on retry:** Because the subsampling counters (zip window
count, ratio emitted/seen) continue to evolve between the original send
and the retry, the retry may produce a different `to_keep` count than
the original attempt. This is inherent to a stateful processor and is
acceptable -- the key property is that the input to the retry is the
full original data, not the previously-subsampled subset.

**Shutdown:** On shutdown, all in-flight slots are nacked upstream to
ensure the context stack unwinds cleanly.

**1:1 simplification:** Unlike the batch processor, which has N:M
mappings between inbound and outbound messages (due to merging and
splitting), the subsampling processor has a strict 1:1 mapping. Each
inbound request produces at most one outbound message. This means a
single `State<Context>` slot map is sufficient -- no `BatchPortion`
vectors or multi-counter outbound tracking is needed.

## Telemetry

```rust
#[metric_set(name = "log_subsampling.processor.pdata.metrics")]
#[derive(Debug, Default, Clone)]
pub struct LogSubsamplingMetrics {
    /// Total log records received
    #[metric(unit = "{log}")]
    pub log_signals_consumed: Counter<u64>,

    /// Log records dropped by subsampling
    #[metric(unit = "{log}")]
    pub log_signals_dropped: Counter<u64>,

    /// Batches where all records were dropped (acked immediately)
    #[metric(unit = "{batch}")]
    pub batches_fully_dropped: Counter<u64>,
}
```

These metrics are reported via the `CollectTelemetry` control message
handler.

## Control Messages

The Option 1 / Option 2 columns refer to the ack/nack options described
above. The `TimerTick` behavior depends on the zip sampling window
option and is the same regardless of ack/nack choice.

| Message            | Option 1       | Option 2                              |
|--------------------|----------------|---------------------------------------|
| `CollectTelemetry` | Report metrics | Report metrics                        |
| `TimerTick`        | See note       | See note                              |
| `Shutdown`         | No-op          | Nack all in-flight slots              |
| `Ack`              | No-op          | Look up slot, propagate ack upstream  |
| `Nack`             | No-op          | Look up slot, propagate nack upstream |
| `Config`           | No-op          | No-op                                 |

`TimerTick` handling depends on the zip sampling window option: with
the TimerTick window option, the handler resets the zip counter to
zero. With the lazy window option, `TimerTick` is not used and can be
ignored. This is independent of the ack/nack option choice.

The processor does not use `DelayedData`.

## State and Restarts

No state is persisted across restarts. On restart:

- Zip sampling starts with count = 0. With the lazy window option,
  window_expiry is in the past so the first batch triggers a reset.
  With the TimerTick option, the timer is re-registered at startup.
- Ratio sampling starts with emitted = 0, seen = 0.
- Ack/nack Option 2 only: the in-flight slot map is empty (any
  in-flight requests from before the restart are lost).

This is acceptable for a volume-reduction processor where exact
long-term ratios are not critical across restart boundaries.

## Edge Cases

- **Single record batch with zero budget**: Acked immediately, nothing
  forwarded.
- **Batch larger than zip max_items**: Only the first `max_items` (or
  remaining budget) records are forwarded. The rest are dropped.
- **Ratio emit equals out_of (1:1)**: All records are forwarded. The
  processor becomes a pass-through for logs.
- **OTLP bytes format**: The processor operates on OtapArrowRecords. If
  the incoming payload is OTLP bytes, we convert to Arrow format first 
  (handled by the standard payload conversion path).

