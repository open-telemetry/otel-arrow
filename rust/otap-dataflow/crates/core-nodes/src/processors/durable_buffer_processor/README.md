# Durable Buffer

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:processor:durable_buffer`
- Type shortcut: `processor:durable_buffer`
- Feature gate: Default
- Stability: Experimental

## Overview

The Durable Buffer provides crash-resilient buffering using a
write-ahead log (WAL) and segment storage. Data is persisted before forwarding
downstream, enabling recovery after crashes or network outages.

## Configuration

```yaml
type: processor:durable_buffer
config:
  # Directory for persistent storage (required)
  path: /var/lib/otap/buffer

  # Maximum disk space (default: 10 GiB)
  retention_size_cap: 10 GiB

  # Maximum age of data to retain (optional, no default)
  # When set, data older than this becomes eligible for removal.
  # Can be combined with retention_size_cap for dual-constraint retention.
  # max_age: 24h

  # Policy when size cap is reached (default: backpressure)
  # - backpressure: Block ingestion (no data loss)
  # - drop_oldest: Remove oldest segments (controlled data loss)
  size_cap_policy: backpressure

  # Interval for polling for available bundles (default: 100ms)
  poll_interval: 100ms

  # Maximum time a segment stays open before finalization (default: 1s)
  # Lower values reduce latency but increase I/O overhead.
  max_segment_open_duration: 1s

  # OTLP handling mode (default: pass_through)
  # - pass_through: Store OTLP as opaque binary, very CPU efficient
  # - convert_to_arrow: Convert to Arrow format, enables querying but higher CPU
  otlp_handling: pass_through

  # Initial retry delay after first NACK (default: 1s)
  initial_retry_interval: 1s

  # Maximum retry delay cap (default: 30s)
  max_retry_interval: 30s

  # Backoff multiplier (default: 2.0)
  retry_multiplier: 2.0

  # Maximum bundles in-flight to downstream (default: 1000)
  max_in_flight: 1000
```

## Architecture

Each processor instance (one per CPU core) has its own isolated storage engine:

```text
{path}/
+-- core_0/
|   +-- wal/
|   +-- segments/
+-- core_1/
|   +-- wal/
|   +-- segments/
+-- ...
```

## Message Flow

1. **Ingest**: Incoming data is written to the WAL, then ACK sent upstream
2. **Segment Finalization**: When segment reaches size/time threshold, it's
   written to disk
3. **Forward**: Timer tick polls for finalized bundles, sends downstream
4. **ACK/NACK**: On ACK, bundle marked complete; on NACK, deferred for retry
5. **Cleanup**: Fully-consumed segments are deleted to reclaim disk space

## Examples

See the configuration example above.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `otap.processor.durable_buffer`

| Metric | Unit | Description |
| --- | --- | --- |
| `otap.processor.durable_buffer.bundles_acked` | `{bundle}` | Number of bundles acknowledged by downstream. |
| `otap.processor.durable_buffer.bundles_nacked_deferred` | `{bundle}` | Number of bundles deferred for retry after transient downstream failures. |
| `otap.processor.durable_buffer.bundles_nacked_permanent` | `{bundle}` | Number of bundles permanently rejected by downstream (not retried). These indicate data loss due to permanent failures (e.g., malformed data). |
| `otap.processor.durable_buffer.rejected_log_records` | `{log_record}` | Number of log records rejected. |
| `otap.processor.durable_buffer.rejected_metric_points` | `{data_point}` | Number of metric data points rejected. |
| `otap.processor.durable_buffer.rejected_spans` | `{span}` | Number of spans rejected. |
| `otap.processor.durable_buffer.consumed_log_records` | `{log_record}` | Number of log records consumed (ingested to durable storage). For OTLP bytes, counted by scanning the protobuf wire format without full deserialization. |
| `otap.processor.durable_buffer.consumed_metric_points` | `{data_point}` | Number of metric data points consumed (ingested to durable storage). For OTLP bytes, counted by scanning the protobuf wire format without full deserialization. |
| `otap.processor.durable_buffer.consumed_spans` | `{span}` | Number of spans consumed (ingested to durable storage). For OTLP bytes, counted by scanning the protobuf wire format without full deserialization. |
| `otap.processor.durable_buffer.produced_log_records` | `{log_record}` | Number of log records produced (sent downstream). For OTLP bytes, counted by scanning the protobuf wire format without full deserialization. |
| `otap.processor.durable_buffer.produced_metric_points` | `{data_point}` | Number of metric data points produced (sent downstream). For OTLP bytes, counted by scanning the protobuf wire format without full deserialization. |
| `otap.processor.durable_buffer.produced_spans` | `{span}` | Number of spans produced (sent downstream). For OTLP bytes, counted by scanning the protobuf wire format without full deserialization. |
| `otap.processor.durable_buffer.ingest_errors` | `{error}` | Number of ingest errors (excludes backpressure/capacity rejections). |
| `otap.processor.durable_buffer.ingest_backpressure` | `{rejection}` | Number of ingest rejections due to storage backpressure (soft cap exceeded). |
| `otap.processor.durable_buffer.read_errors` | `{error}` | Number of read errors. |
| `otap.processor.durable_buffer.storage_bytes_used` | `By` | Current bytes used by persistent storage (WAL + segments). |
| `otap.processor.durable_buffer.storage_bytes_cap` | `By` | Configured storage capacity cap. |
| `otap.processor.durable_buffer.dropped_segments` | `{segment}` | Total segments force-dropped due to DropOldest retention policy. Non-zero values indicate data loss. |
| `otap.processor.durable_buffer.dropped_bundles` | `{bundle}` | Total bundles lost due to force-dropped segments (DropOldest policy). Non-zero values indicate data loss. |
| `otap.processor.durable_buffer.dropped_items` | `{item}` | Total individual items (log records, data points, spans) lost due to force-dropped segments (DropOldest policy). Non-zero values indicate data loss. |
| `otap.processor.durable_buffer.expired_bundles` | `{bundle}` | Total bundles lost due to expired segments (max_age retention). Non-zero values indicate data aged out before delivery. |
| `otap.processor.durable_buffer.expired_items` | `{item}` | Total individual items (log records, data points, spans) lost due to expired segments (max_age retention). Non-zero values indicate data aged out before delivery. |
| `otap.processor.durable_buffer.retries_scheduled` | `{retry}` | Number of retry attempts scheduled. |
| `otap.processor.durable_buffer.in_flight` | `{bundle}` | Current number of bundles in-flight to downstream. |
| `otap.processor.durable_buffer.requeued_log_records` | `{log_record}` | Number of individual log records requeued for retry after NACK. |
| `otap.processor.durable_buffer.requeued_metric_points` | `{data_point}` | Number of individual metric data points requeued for retry after NACK. |
| `otap.processor.durable_buffer.requeued_spans` | `{span}` | Number of individual spans requeued for retry after NACK. |
| `otap.processor.durable_buffer.queued_log_records` | `{log_record}` | Current number of log records queued (ingested but not yet ACKed). |
| `otap.processor.durable_buffer.queued_metric_points` | `{data_point}` | Current number of metric data points queued (ingested but not yet ACKed). |
| `otap.processor.durable_buffer.queued_spans` | `{span}` | Current number of spans queued (ingested but not yet ACKed). |
| `otap.processor.durable_buffer.flush_failures` | `{error}` | Number of segment finalization (flush) failures. Non-zero values indicate data at risk - check logs for root cause. Data may still be recoverable via WAL replay on restart. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `durable_buffer.engine.*` | `info`, `error` | Engine initialization, readiness, and unavailable states. |
| `durable_buffer.ingest.*` | `warn`, `error` | Storage backpressure and ingest failures. |
| `durable_buffer.otlp.*` | `error` | OTLP adapter and conversion failures. |
| `durable_buffer.flush.failed` | `warn` | Segment flush failure during timer processing. |
| `durable_buffer.maintenance.failed` | `warn` | Durable storage maintenance failure during timer processing. |
| `durable_buffer.drain.*` | `debug`, `error` | Drain loop budget, capacity, backpressure, and poll-failure states. |
| `durable_buffer.bundle.*` | `debug`, `warn`, `error` | Bundle forward, ACK/NACK, duplicate, conversion, and permanent-rejection states. |
| `durable_buffer.retry.*` | `debug`, `warn` | Retry scheduling, deferred retry, backpressure, claim, rearm, and reschedule states. |
| `durable_buffer.shutdown.*` | `info`, `warn`, `error` | Shutdown flush, drain, deadline, and engine termination progress or failure. |
| `durable_buffer.timer.started` | `debug` | Periodic poll timer started on first processed message. |
| `durable_buffer.config.update` | `debug` | Runtime config update received. |
| `durable_buffer.delayed_data.unexpected` | `warn` | Unexpected delayed data was received and discarded. |

See [telemetry.md](telemetry.md) for maintenance notes and the expanded event inventory.

## Limits

- `path` is required and each core writes to an isolated subdirectory.
- Retention size is divided across assigned pipeline cores.
- `max_age` is based on segment finalization time, not telemetry timestamps.
- `convert_to_arrow` mode can increase CPU cost compared with `pass_through`.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
