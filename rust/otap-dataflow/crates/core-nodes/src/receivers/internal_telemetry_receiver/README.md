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

```yaml
type: receiver:internal_telemetry
config: {}
```

## Configuration

This receiver has no node-specific configuration.

```yaml
type: receiver:internal_telemetry
config: {}
```

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
