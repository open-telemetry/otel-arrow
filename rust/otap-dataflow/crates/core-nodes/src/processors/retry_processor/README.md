# Retry Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:processor:retry`
- Type shortcut: `processor:retry`
- Feature gate: Default
- Stability: Experimental

## Overview

The retry processor retries downstream delivery when it receives a NACK. It
uses exponential backoff with a maximum per-attempt interval and an overall
elapsed-time limit.

Retry state is held in processor memory and call data, not in durable storage.
Use `processor:durable_buffer` when retries must survive process restarts.

## Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `initial_interval` | duration | `5s` | Delay before the first retry. |
| `max_interval` | duration | `30s` | Maximum delay between retry attempts. |
| `max_elapsed_time` | duration | `300s` | Maximum total retry window. |
| `multiplier` | number | `1.5` | Exponential backoff multiplier. |

## Examples

```yaml
type: processor:retry
config:
  initial_interval: 1s
  max_interval: 30s
  max_elapsed_time: 5m
  multiplier: 2.0
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `processor.retry`

| Metric | Unit | Description |
| --- | --- | --- |
| `processor.retry.consumed_items_logs_success` | `{item}` | Number of items consumed (logs) with outcome=success. |
| `processor.retry.consumed_items_metrics_success` | `{item}` | Number of items consumed (metrics) with outcome=success. |
| `processor.retry.consumed_items_traces_success` | `{item}` | Number of items consumed (traces) with outcome=success. |
| `processor.retry.consumed_items_logs_failure` | `{item}` | Number of items consumed (logs) with outcome=failure. |
| `processor.retry.consumed_items_metrics_failure` | `{item}` | Number of items consumed (metrics) with outcome=failure. |
| `processor.retry.consumed_items_traces_failure` | `{item}` | Number of items consumed (traces) with outcome=failure. |
| `processor.retry.consumed_items_logs_refused` | `{item}` | Number of items consumed (logs) with outcome=refused. |
| `processor.retry.consumed_items_metrics_refused` | `{item}` | Number of items consumed (metrics) with outcome=refused. |
| `processor.retry.consumed_items_traces_refused` | `{item}` | Number of items consumed (traces) with outcome=refused. |
| `processor.retry.produced_items_logs_success` | `{item}` | Number of items produced (logs) with outcome=success. |
| `processor.retry.produced_items_metrics_success` | `{item}` | Number of items produced (metrics) with outcome=success. |
| `processor.retry.produced_items_traces_success` | `{item}` | Number of items produced (traces) with outcome=success. |
| `processor.retry.produced_items_logs_refused` | `{item}` | Number of items produced (logs) with outcome=refused (downstream error) |
| `processor.retry.produced_items_metrics_refused` | `{item}` | Number of items produced (metrics) with outcome=refused (downstream error) |
| `processor.retry.produced_items_traces_refused` | `{item}` | Number of items produced (traces) with outcome=refused (downstream error) |
| `processor.retry.retry_attempts_logs` | `{event}` | Number of retry attempts scheduled as a result of NACKs, logs. |
| `processor.retry.retry_attempts_traces` | `{event}` | Number of retry attempts scheduled as a result of NACKs, traces. |
| `processor.retry.retry_attempts_metrics` | `{event}` | Number of retry attempts scheduled as a result of NACKs, metrics. |

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
