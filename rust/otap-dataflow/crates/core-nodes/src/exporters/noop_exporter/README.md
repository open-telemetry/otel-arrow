# Noop Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:noop` (`urn:otel:exporter:noop`)
- Feature gate: Default
- Stability: Experimental

## Overview

The noop exporter acknowledges every received message without processing or
exporting it. It is useful for validating pipeline wiring, measuring upstream
cost, and terminating test pipelines.

## Getting Started

Use an empty config object when the pipeline only needs a terminal ACKing
exporter:

```yaml
type: exporter:noop
config: {}
```

## Configuration

This exporter has no node-specific configuration.

```yaml
type: exporter:noop
config: {}
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

- All pdata is discarded after the exporter ACKs it.
- The exporter does not validate payload content.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
