# Delay Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:delay` (`urn:otel:processor:delay`)
- Feature gate: Default
- Stability: experimental

## Overview

The delay processor sleeps for a configured duration before forwarding each
message. It is intended for tests, timeout validation, and simple rate-shaping
experiments.

## Getting Started

Set the sleep duration applied to each incoming message:

```yaml
type: processor:delay
config:
  delay: 250ms
```

## Configuration

```yaml
type: processor:delay
config:
  # Sleep duration before forwarding each message (required).
  # Durations use humantime syntax such as "100ms", "1s", or "2m".
  delay: 250ms
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

- Delay is applied per message on the processor task.
- This node intentionally adds latency and should be used carefully in
  production pipelines.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
