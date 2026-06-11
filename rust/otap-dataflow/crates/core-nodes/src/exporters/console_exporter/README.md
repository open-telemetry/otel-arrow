# Console Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:console` (`urn:otel:exporter:console`)
- Feature gate: Default
- Stability: Experimental

## Overview

The console exporter prints OTLP logs, metrics, and traces using a hierarchical
text formatter. It ACKs each message after writing the formatted view.

This node is intended for local inspection, demos, and debugging pipelines. It
is not a production exporter, durable export path, or stable machine-readable
format.

## Getting Started

Use the console exporter when you want to inspect pdata directly from the
engine process:

```yaml
type: exporter:console
config:
  color: true
  unicode: true
```

## Configuration

```yaml
type: exporter:console
config:
  # Enables ANSI color output (default: true).
  color: true

  # Enables Unicode box-drawing output (default: true).
  unicode: true
```

## Examples

ASCII-only output:

```yaml
type: exporter:console
config:
  color: false
  unicode: false
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
| `console.logs_view.otlp_create_failed` | `error` | Failed to create an OTLP logs view for console output. |
| `console.logs_view.otap_create_failed` | `error` | Failed to create an OTAP logs view for console output. |
| `console.traces.not_implemented` | `error` | The exporter received traces, which are not currently rendered. |
| `console.metrics.not_implemented` | `error` | The exporter received metrics, which are not currently rendered. |
| `console.write_failed` | `error` | Failed to write rendered output to stdout. |

## Limits

- Output is written to the process console and is not persisted.
- Large or high-rate telemetry streams can produce substantial console output.
- Formatting is best-effort diagnostic output, not a stable machine-readable
  export format.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
