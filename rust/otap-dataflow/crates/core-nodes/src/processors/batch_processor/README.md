# Batch Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:processor:batch`
- Type shortcut: `processor:batch`
- Feature gate: Default
- Stability: Experimental

## Overview

The batch processor combines OTAP and OTLP payloads before forwarding them
downstream. It can preserve the inbound payload format or force output to OTAP
or OTLP, and it tracks ACK/NACK-sensitive request state across batch flushes.

## Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `otap` | object | OTAP defaults | Batch sizing for OTAP records. |
| `otlp` | object | OTLP defaults | Batch sizing for OTLP bytes. |
| `max_batch_duration` | duration | `200ms` | Maximum time before flushing pending data. |
| `inbound_request_limit` | non-zero integer | `1024` | Pending inbound request tracking limit. |
| `outbound_request_limit` | non-zero integer | `512` | Pending outbound request tracking limit. |
| `format` | enum | `preserve` | Output format: `otap`, `otlp`, or `preserve`. |

Each format object contains:

| Field | Type | Description |
| --- | --- | --- |
| `min_size` | non-zero integer or null | Flush threshold. |
| `max_size` | non-zero integer or null | Optional upper bound. |
| `sizer` | enum | `requests`, `items`, or `bytes`. |

## Examples

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
