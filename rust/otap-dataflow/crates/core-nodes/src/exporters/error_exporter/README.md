# Error Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:exporter:error`
- Type shortcut: `exporter:error`
- Feature gate: Default
- Stability: Experimental

## Overview

The error exporter rejects every received message by returning a NACK with a
configured message. It is useful for tests, retry-path validation, and pipeline
experiments that need deterministic downstream failures.

## Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `message` | string | Required | Error message used in every NACK. |

## Examples

```yaml
type: exporter:error
config:
  message: "forced exporter failure"
```

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
| *None* | N/A | No node-specific events are emitted. |

## Limits

- This exporter never ACKs pdata.
- It is meant for test and validation scenarios, not production export.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
