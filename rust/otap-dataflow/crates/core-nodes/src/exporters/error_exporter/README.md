# Error Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:error` (`urn:otel:exporter:error`)
- Feature gate: Default
- Stability: experimental

## Overview

The error exporter rejects every received message by returning a NACK with a
configured message. It is useful for tests, retry-path validation, and pipeline
experiments that need deterministic downstream failures.

## Getting Started

Configure the exporter with the NACK message that upstream nodes should
observe:

```yaml
type: exporter:error
config:
  message: "forced exporter failure"
```

## Configuration

```yaml
type: exporter:error
config:
  # Error message used in every NACK (required).
  message: "forced exporter failure"
```

## Telemetry

This node does not emit node-specific telemetry.

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
