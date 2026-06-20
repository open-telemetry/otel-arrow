# Batch Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:batch` (`urn:otel:processor:batch`)
- Feature gate: Default
- Stability: Experimental

## Overview

The batch processor combines OTAP and OTLP payloads before forwarding them
downstream. It can preserve the inbound payload format or force output to OTAP
or OTLP, and it tracks ACK/NACK-sensitive request state across batch flushes.

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
    min_size: 8192      # Flush threshold; null disables size flushing.
    max_size: null     # Optional upper bound.
    sizer: items       # "requests", "items", or "bytes".

  # Batch sizing for OTLP bytes (defaults are format-specific).
  otlp:
    min_size: 1048576
    max_size: null
    sizer: bytes

  # Maximum time before flushing pending data (default: 200ms).
  max_batch_duration: 500ms

  # Pending request tracking limits.
  inbound_request_limit: 1024
  outbound_request_limit: 512

  # Output format: "otap", "otlp", or "preserve" (default: preserve).
  format: preserve
```

Each format object contains:

- `min_size`: non-zero flush threshold, or `null` to disable size flushing.
- `max_size`: optional non-zero upper bound, or `null`.
- `sizer`: one of `requests`, `items`, or `bytes`.

## Examples

Flush every incoming message:

```yaml
type: processor:batch
config:
  max_batch_duration: 0s
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

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- `min_size` and `max_size`, when set, must be non-zero.
- `bytes` sizing depends on payload formats that can report encoded size.
- `max_batch_duration: 0s` disables time-based accumulation and flushes
  immediately.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
