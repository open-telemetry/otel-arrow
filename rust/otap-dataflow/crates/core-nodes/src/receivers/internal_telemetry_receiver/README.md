# Internal Telemetry Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:internal_telemetry` (`urn:otel:receiver:internal_telemetry`)
- Feature gate: Default
- Stability: Experimental

## Overview

The internal telemetry receiver consumes internal engine log events from the
pipeline context and emits them as OTLP log pdata. It is intended for the engine
observability pipeline rather than normal user ingest.

## Getting Started

Declare it with an empty config inside `engine.observability.pipeline`:

The observability pipeline is defined under `engine.observability` rather than
inside a user pipeline group, so it is owned by the engine and cannot be
referenced from groups.

```yaml
type: receiver:internal_telemetry
config: {}
```

## Configuration

Batching is on by default: records accumulate until `batch_size` bytes are
reached (or `max_batch_duration` elapses), and those sharing a scope are
combined into one `ScopeLogs` message instead of being emitted one per message.
The empty config shown above uses the defaults below; set either field to
override them:

```yaml
type: receiver:internal_telemetry
config:
  # Bytes to accumulate (estimated per-record size) before emitting one
  # batched ExportLogsRequest. Default 64 KiB. Must be greater than 0.
  batch_size: 65536

  # Upper bound on how long a record waits in a partial batch before it is
  # flushed. Default: 200ms.
  max_batch_duration: 200ms
```

- `batch_size`: bytes to accumulate before flushing, estimated from each
  record's pre-encoded body plus framing overhead. Default 64 KiB. Set it to
  `1` to flush every record immediately (each record's own size always
  exceeds a 1-byte threshold).
- `max_batch_duration`: how long a partial batch may wait before flushing.
  The pending batch is also flushed on drain, shutdown, and channel close. A
  `batch_size` set well above the transport size limit is still split
  automatically once accumulated records reach an internal ~2 MiB budget. That
  budget is a size estimate (body plus framing, not full protobuf overhead),
  so it's a safety margin rather than a guarantee against oversized messages.

This receiver is normally declared inside `engine.observability.pipeline`, not
inside a user ingest pipeline.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

| Metric | Unit | Description |
| --- | --- | --- |
| *None* | N/A | This node does not register a node-specific metric set. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | This receiver consumes internal telemetry and does not emit additional node-specific events during normal operation. |

## Limits

- The receiver requires internal telemetry settings in the pipeline context.
- It is normally used under `engine.observability.pipeline`.
- It emits internal logs; metric export is configured separately through engine
  telemetry settings.

## Related Docs

- [Configuration model observability pipeline](../../../../../docs/configuration-model.md#observability-pipeline)
- [Telemetry guide](../../../../../docs/telemetry/README.md)
- [Core node catalog](../../../README.md)
