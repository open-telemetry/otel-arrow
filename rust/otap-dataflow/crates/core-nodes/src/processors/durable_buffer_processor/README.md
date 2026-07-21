# Durable Buffer

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:durable_buffer` (`urn:otel:processor:durable_buffer`)
- Feature gate: Default
- Stability: Experimental

## Overview

The Durable Buffer provides crash-resilient buffering using a
write-ahead log (WAL) and segment storage. Data is persisted before forwarding
downstream, enabling recovery after crashes or network outages.

## Getting Started

Set a persistent storage path before placing the durable buffer ahead of an
exporter:

```yaml
type: processor:durable_buffer
config:
  path: /var/lib/otap/buffer
  retention_size_cap: 10 GiB
  size_cap_policy: backpressure
```

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

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

| Scope | Instrument(s) | Dimensions |
| --- | --- | --- |
| `processor.durable_buffer` | `read.errors`, `storage.bytes.used`, `storage.bytes.cap`, `retries.scheduled`, `in.flight`, `flush.failures`, `storage.utilization` | None |
| `processor.durable_buffer.bundles` | `resolved` | `outcome=acked\|deferred\|permanently_rejected` |
| `processor.durable_buffer.ingest` | `failures` | `failure=error\|backpressure` |
| `processor.durable_buffer.items` | `rejected`, `consumed`, `produced`, `requeued`, `queued` | `signal=traces\|metrics\|logs` |
| `processor.durable_buffer.loss` | `segments`, `bundles`, `items` | `reason=drop_oldest\|expired` |
| `processor.durable_buffer.item_loss` | `items` | `signal=traces\|metrics\|logs`, `reason=drop_oldest\|expired` |

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
