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

Batching is optional and off by default. With an empty config each log record is
emitted as its own `ExportLogsRequest`, exactly as before:

```yaml
type: receiver:internal_telemetry
config: {}
```

To batch records, set `batch_size`. Records that share a scope are then combined
into one `ScopeLogs` message instead of being emitted one per message:

```yaml
type: receiver:internal_telemetry
config:
  # Records to accumulate before emitting one batched ExportLogsRequest.
  # Omit to disable batching. Must be greater than 0.
  batch_size: 100

  # Upper bound on how long a record waits in a partial batch before it is
  # flushed. Only applies when batch_size is set (default: 200ms).
  max_batch_duration: 200ms
```

- `batch_size`: omit (the default) to emit each record immediately, or set a
  positive count to group records sharing a scope into one `ScopeLogs` message.
- `max_batch_duration`: how long a partial batch may wait before flushing. Only
  relevant when `batch_size` is set. The pending batch is also flushed on drain,
  shutdown, and channel close, and large batches are split so every emitted
  message stays well under the transport size limit.

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
