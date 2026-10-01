# Batch Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:batch` (`urn:otel:processor:batch`)
- Feature gate: Always enabled
- Stability: Experimental

## Overview

The batch processor combines OTAP and OTLP payloads before forwarding them
downstream. It can preserve the inbound payload format or force output to OTAP
or OTLP, and it tracks ACK/NACK-sensitive request state across batch flushes.

Batches are sized against a range of acceptable sizes, `[min_size, max_size]`.
Inputs that already fall in that range are forwarded unchanged. Smaller inputs
are buffered and merged, and larger inputs are split, until the output falls in
range or `max_batch_duration` elapses.

## Getting Started

Configure format-specific sizing and the maximum time to hold pending data:

```yaml
type: processor:batch
config:
  max_batch_duration: 500ms
  format: preserve
  otap:
    min_size: 8192
    max_size: null
    sizer: items
  otlp:
    min_size: 1048576
    max_size: null
    sizer: bytes
```

## Configuration

```yaml
type: processor:batch
config:
  # Batch sizing for OTAP records (defaults are format-specific).
  otap:
    min_size: 8192     # Lower bound of the acceptable size range; defaults to max_size when null.
    max_size: null     # Optional upper bound.
    sizer: items       # OTAP supports "items" only.

  # Batch sizing for OTLP bytes (defaults are format-specific).
  otlp:
    min_size: 262144
    max_size: null
    sizer: bytes       # OTLP supports "bytes" only.
    max_split_fragments: 65536  # Cap on fragments per oversize entry (OTLP only).
    max_split_overhead_bytes: 8388608  # Cap on duplicated wrapper bytes per oversize entry (OTLP only).
    max_split_fragments_per_flush: 65536  # Greedy per-flush split threshold (OTLP only).

  # Maximum time before flushing pending data (default: 200ms).
  max_batch_duration: 500ms

  # Pending request tracking limits.
  inbound_request_limit: 1024
  outbound_request_limit: 512

  # Output format: "otap", "otlp", or "preserve" (default: preserve).
  format: preserve
```

Each format object contains:

- `min_size`: lower bound of the acceptable batch size, or `null`. When `null`
  it defaults to `max_size`. `0` is allowed and means any input up to
  `max_size` is forwarded as-is. Defaults: 8192 items (OTAP), 262144 bytes
  (OTLP).
- `max_size`: optional non-zero upper bound, or `null` for no upper bound. Must
  be `>= min_size` when both are set.
- `sizer`: the unit `min_size` and `max_size` are measured in. OTAP supports
  `items` (log records, spans, or metric data points); OTLP supports `bytes`
  (encoded request size).
- `max_split_fragments` (OTLP bytes only): non-zero cap on how many fragments a
  single oversize resource entry may split into, or `null` for unbounded
  (default 65536, a power-of-two backstop). Splitting an entry that exceeds
  `max_size` re-encodes the resource/scope
  headers around each fragment, so a tiny `max_size` relative to one indivisible
  input could fan out into very many fragments. When the projected fragment
  count exceeds this budget the entry is emitted whole (best-effort, possibly
  exceeding `max_size`) and counted by the `split.budget.fallbacks` metric.
- `max_split_overhead_bytes` (OTLP bytes only): non-zero cap on how many
  duplicated wrapper bytes a single oversize resource entry may amplify into, or
  `null` for unbounded (default 8 MiB). Because each fragment re-encodes the
  resource/scope headers, a large header split across many records can amplify
  output far beyond the input even when the fragment count stays under
  `max_split_fragments`. The amplification is measured from the *actual* greedy
  packing (the emitted fragments' total bytes minus one whole encoding of the
  entry), not a per-record worst case, so many small records under a large
  header are not falsely collapsed. When it exceeds this budget the entry is
  emitted whole (best-effort, possibly exceeding `max_size`) and counted by the
  same `split.budget.fallbacks` metric; emission also aborts early once the
  running amplification passes the budget, bounding transient memory. This is
  *measured* from the actual packing as the entry is split -- unlike
  `max_split_fragments`, it is not projected up front.
- `max_split_fragments_per_flush` (OTLP bytes only): non-zero greedy threshold on
  the number of output batches a single flush may build from splitting, or `null`
  for unbounded (default 65536). The two budgets above bound each entry
  individually, but a flush containing many large entries builds its entire
  output vector in memory before anything is sent, so their combined split
  fan-out could still amplify into a very large allocation. Once a flush has
  produced this many output batches, any further oversize entry is emitted whole
  (best-effort, counted by `split.budget.fallbacks`) instead of split. This is a
  simple greedy running threshold on *split fan-out*, not a strict total-output
  cap. The total output can exceed the threshold in two ways. First, the entry
  that crosses the threshold may still add its full per-entry fan-out, so split
  amplification is bounded by roughly this threshold plus one entry's
  `max_split_fragments`. Second, every remaining oversize or indivisible entry is
  still emitted whole, and each such entry contributes at least one output batch;
  the threshold bounds only the *additional* split fan-out, not this mandatory
  output floor of (at least) one batch per remaining top-level entry. It does not
  look ahead over later entries and is independent of Ack/Nack outbound-slot
  accounting (which governs *sending*, not up-front allocation).

### Validation rules

- At least one of `min_size` or `max_size` must be set.
- `max_size`, when set, must be non-zero and `>= min_size`.
- A non-zero `min_size` requires a non-zero `max_batch_duration`, otherwise
  buffered data below `min_size` could wait forever.
- With `max_batch_duration: 0s`, `max_size` must be set unless `min_size` is
  explicitly `0`.
- `min_size: 0` with no `max_size` is accepted but logs a
  `batch.config.forwards_everything` warning: every input is forwarded as-is and
  no batching is performed.

## Behavior

Sizes are measured in the format's sizer unit. Signals (logs, metrics, traces)
and formats (OTAP, OTLP) are buffered independently. Below, `min` is the
effective lower bound (`min_size`, or `max_size` if `min_size` is unset) and
`max` is `max_size` (infinite if unset).

### When an input arrives

1. **Empty input** (size 0): acknowledged immediately and dropped.
2. **Already acceptable** (`min <= size <= max`): forwarded immediately and
   unchanged, with its original request context. It is not buffered, merged, or
   split, and downstream Ack/Nack go straight to the original sender without
   using the processor's request-tracking slots. Any data already buffered is
   left in place.
3. **Otherwise** (smaller than `min`, or larger than `max`): appended to the
   pending buffer. If this is the first pending data, the flush timer starts.
   If the pending total reaches `min` (or `max_batch_duration` is `0s`), a size
   flush runs immediately.

### Flushing

A flush happens when pending data reaches `min` (size flush), when
`max_batch_duration` elapses after the oldest pending data arrived (timer
flush), or at shutdown.

- If exactly one whole input is pending and it fits within `max`, it is
  forwarded unchanged with its original context, as in step 2 above.
- Otherwise all pending data is merged in arrival order. If `max_size` is set,
  it is split so every output is at most `max_size`. Items are packed to fill
  each output to exactly `max_size` where the data allows.
- If `max_size` is unset, everything pending is merged into a single output with
  no upper bound.

### Held-back remainders

On a size flush with `max_size` set and a non-zero `max_batch_duration` that
produces more than one output, if the final output is smaller than `min`, it is
held back instead of sent. It becomes
the first data in the next batch and restarts the flush timer. A held-back
remainder therefore waits at most about `2 x max_batch_duration` in total.
Timer and shutdown flushes send everything, including remainders.

The original request is not acknowledged until every output carrying its data,
including a held-back remainder, has been acknowledged.

### Acknowledgements

- Forwarded-as-is inputs keep their own context, so downstream Ack/Nack route
  directly to the original sender.
- Merged or split data is tracked: an input is acknowledged only after every
  output containing part of it is acknowledged, and is negatively acknowledged
  as soon as any such output is rejected.
- When the tracking limits (`inbound_request_limit`, `outbound_request_limit`)
  are exhausted, affected requests are rejected with a NACK so the sender can
  retry.

### Guarantees and caveats

- Output order is not preserved relative to buffered data: an input forwarded
  as-is may be sent before older data that is still buffered.
- Batch sizes are best-effort. Metrics are only split at metric boundaries, so
  a metric with more data points than `max_size` is sent on its own and
  metric batches can fall below `min_size`. OTLP byte splitting may emit an
  entry whole when a split budget is exceeded.
- Timer and shutdown flushes send whatever is pending, regardless of size.
- Without `max_size`, merged OTAP output is limited only by the protocol's ID
  width; a merge that would overflow it fails and the affected requests are
  rejected. Set `max_size` for bounded output.

### Common configurations

| Goal | `min_size` | `max_size` | `max_batch_duration` | Behavior |
| --- | --- | --- | --- | --- |
| Uniform batches | `N` | `N` | `> 0` | Inputs of exactly `N` pass through; everything else is merged or split into batches of `N` (except timer/shutdown flushes). |
| Batches in a range | `L` | `H` | `> 0` | Inputs in `[L, H]` pass through; smaller inputs accumulate to at least `L`; larger inputs are split to at most `H`. |
| Accumulate up to a maximum | `null` | `H` | `> 0` | `min_size` defaults to `H`: smaller inputs accumulate to `H`, inputs of exactly `H` pass through. |
| Merge only | `L` | `null` | `> 0` | Inputs of at least `L` pass through; smaller inputs accumulate to at least `L` with no upper bound. |
| Split only | `0` or `null` | `H` | `0s` | Inputs up to `H` pass through; larger inputs are split to at most `H`. |
| Split only (any timeout) | `0` | `H` | any | As above; with `min_size: 0` nothing is ever buffered, so `max_batch_duration` has no effect. |

## Examples

Batches of exactly 1000 items (except timer/shutdown flushes):

```yaml
type: processor:batch
config:
  max_batch_duration: 200ms
  format: otap
  otap:
    min_size: 1000
    max_size: 1000
    sizer: items
```

Only split oversized requests, forwarding everything else unchanged:

```yaml
type: processor:batch
config:
  max_batch_duration: 0s
  format: otap
  otap:
    min_size: 0
    max_size: 8192
    sizer: items
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `otap.processor.batch`

| Metric | Unit | Description |
| --- | --- | --- |
| `otap.processor.batch.consumed_batches_logs` | `{item}` | Total batches consumed for logs signal. |
| `otap.processor.batch.consumed_batches_metrics` | `{item}` | Total batches consumed for metrics signal. |
| `otap.processor.batch.consumed_batches_traces` | `{item}` | Total batches consumed for traces signal. |
| `otap.processor.batch.produced_batches_logs` | `{item}` | Total batches produced for logs signal. |
| `otap.processor.batch.produced_batches_metrics` | `{item}` | Total batches produced for metrics signal. |
| `otap.processor.batch.produced_batches_traces` | `{item}` | Total batches produced for traces signal. |
| `otap.processor.batch.flushes_size` | `{flush}` | Number of flushes triggered by size threshold (all signals) |
| `otap.processor.batch.flushes_timer` | `{flush}` | Number of flushes triggered by timer (all signals) |
| `otap.processor.batch.passthrough_batches` | `{batch}` | Number of inputs forwarded as-is, without re-batching or completion tracking, because their size was already within `[min_size, max_size]` or they were flushed alone. |
| `otap.processor.batch.flush_pending_requests` | `{request}` | Number of input requests pending at flush time. |
| `otap.processor.batch.flush_pending_bytes` | `By` | Number of bytes pending at flush time when byte size is known. |
| `otap.processor.batch.flush_age_duration` | `ns` | Time from first pending input arrival to actual flush start. |
| `otap.processor.batch.flush_timer_lateness_duration` | `ns` | Delay between scheduled timer wakeup and actual timer flush start. |
| `otap.processor.batch.flush_output_batches` | `{batch}` | Number of output batches emitted by each flush. |
| `otap.processor.batch.flush_output_bytes` | `By` | Number of bytes emitted by each flush when byte size is known. |
| `otap.processor.batch.dropped_conversion` | `{msg}` | Number of messages dropped due to conversion failures. |
| `otap.processor.batch.batching_errors` | `{error}` | Number of batches for which errors encountered. |
| `otap.processor.batch.nacked_inbound_slots` | `{msg}` | Number of requests nacked due to inbound slot exhaustion. |
| `otap.processor.batch.nacked_outbound_slots` | `{msg}` | Number of requests nacked due to outbound slot exhaustion. |
| `otap.processor.batch.split_budget_fallbacks` | `{entry}` | Number of oversize resource entries emitted whole because splitting would have exceeded `max_split_fragments`, `max_split_overhead_bytes`, or the per-flush `max_split_fragments_per_flush` threshold. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `batch.config.forwards_everything` | WARN | Emitted at startup for each format configured with `min_size: 0` and no `max_size`; every input is forwarded as-is and no batching is performed. |

## Limits

- `max_size`, when set, must be non-zero. `min_size` may be `0`.
- `bytes` sizing depends on payload formats that can report encoded size.
- `max_batch_duration: 0s` disables time-based accumulation and flushes
  immediately.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
