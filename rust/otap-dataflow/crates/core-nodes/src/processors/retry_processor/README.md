# Retry Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:retry` (`urn:otel:processor:retry`)
- Feature gate: Default
- Stability: Experimental

## Overview

The retry processor retries downstream delivery when it receives a NACK. It
uses exponential backoff with a maximum per-attempt interval and an overall
elapsed-time limit.

Retry state is held in processor memory and call data, not in durable storage.
Use `processor:durable_buffer` when retries must survive process restarts.

## Getting Started

Tune the retry backoff window around the downstream exporter or processor:

```yaml
type: processor:retry
config:
  initial_interval: 1s
  max_interval: 30s
  max_elapsed_time: 5m
  multiplier: 2.0
```

## Configuration

```yaml
type: processor:retry
config:
  # Delay before the first retry (default: 5s).
  initial_interval: 1s

  # Maximum delay between retry attempts (default: 30s).
  max_interval: 30s

  # Maximum total retry window (default: 300s).
  max_elapsed_time: 5m

  # Exponential backoff multiplier (default: 1.5).
  multiplier: 2.0
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

Message outcomes are reported by the engine through
`node.consumer.consumed.messages` and `node.producer.produced.messages`.
Per-signal item outcomes are available through
`node.consumer.consumed.items` and `node.producer.produced.items` when detailed
runtime metrics are enabled or the node opts in with
`policies.telemetry.item_counts`. The processor does not duplicate these
engine-owned metrics.

#### `processor.retry`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `processor.retry.retries.scheduled` | `{retry}` | `signal` | Number of retries successfully scheduled after a downstream refusal. |
| `processor.retry.messages.recovered` | `{message}` | `signal` | Number of PData messages accepted downstream after at least one retry. |

The `signal` attribute is bounded to `traces`, `metrics`, or `logs`.

#### `processor.retry.messages`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `processor.retry.messages.terminated` | `{message}` | `signal`, `reason` | Number of PData messages the retry processor stopped retrying. |

Both message lifecycle counters use the `{message}` unit and the `messages`
namespace.

The `reason` attribute is bounded to:

| Value | Description |
| --- | --- |
| `invalid_state` | Retry state in call data was absent or malformed. |
| `permanent_refusal` | Downstream permanently refused the request. |
| `payload_missing` | Downstream did not return the payload required for a retry. |
| `retry_limit` | The retry-count safety limit was reached. |
| `deadline` | The next retry would exceed the configured elapsed-time deadline. |
| `send_failure` | The processor could not send the PData message or convert the failure into a NACK. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- Retry state is not durable across process restart.
- The implementation rejects configurations that would require absurd retry
  growth simulations.
- The processor retries NACK outcomes; it does not make an exporter idempotent.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Durable buffer](../durable_buffer_processor/README.md)
- [Core node catalog](../../../README.md)
