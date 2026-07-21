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

Input PData message volume is reported by the engine through
`channel.receiver.recv.count` on the PData input channel and is not duplicated
by the processor. The metrics below describe retry-specific item outcomes and
delivery attempts.

#### `processor.retry.items`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `processor.retry.items.consumed` | `{item}` | `signal`, `outcome` | Number of items whose retry processing reached a terminal outcome. |
| `processor.retry.items.produced` | `{item}` | `signal`, `outcome` | Number of items returned by downstream delivery attempts. |

The `consumed` metric records one terminal outcome per non-empty input. The
`produced` metric records each downstream delivery result, so a retried input
can contribute refused items from intermediate attempts and successful items
from its final attempt.

#### `processor.retry.attempts`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `processor.retry.attempts.scheduled` | `{retry}` | `signal` | Number of retry attempts scheduled after a downstream refusal. |

Attribute values are bounded: `signal` is `traces`, `metrics`, or `logs`, and
`outcome` is `success`, `failure`, or `refused`.

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
