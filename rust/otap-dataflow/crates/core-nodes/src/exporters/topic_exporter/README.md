# Topic Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:exporter:topic`
- Type shortcut: `exporter:topic`
- Feature gate: Default
- Stability: Experimental

## Overview

The topic exporter publishes pdata to a named in-process topic declared in the
runtime configuration. It can bridge end-to-end ACK/NACK outcomes when the
topic declaration enables ack propagation.

## Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `topic` | string | Required | Topic name to publish to. |
| `queue_on_full` | enum | Topic policy | Optional local full-queue behavior. |

`queue_on_full` accepts the topic queue policies supported by the topic model,
such as `block` and `drop_newest`.

## Examples

```yaml
type: exporter:topic
config:
  topic: raw_signals
  queue_on_full: drop_newest
```

The `raw_signals` topic must be declared in the surrounding runtime
configuration.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `exporter.topic`

| Metric | Unit | Description |
| --- | --- | --- |
| `exporter.topic.published_messages` | `{item}` | Number of messages published to the topic. |
| `exporter.topic.dropped_messages_on_full` | `{item}` | Number of messages dropped due to queue full policy. |
| `exporter.topic.end_to_end_acks` | `{item}` | Number of end-to-end acks bridged back to upstream. |
| `exporter.topic.end_to_end_nacks` | `{item}` | Number of end-to-end nacks bridged back to upstream. |
| `exporter.topic.dropped_messages_on_outcome_capacity` | `{item}` | Number of messages rejected because tracked outcome capacity was exhausted. |
| `exporter.topic.tracked_in_flight` | `{item}` | Current number of tracked publishes waiting for a terminal outcome. Future: add a pending-bytes gauge once retained payload size accounting is available for tracked publishes. |
| `exporter.topic.outcome_timeouts` | `{item}` | Number of tracked publishes that resolved by timeout. Future: add an outcome-latency histogram once histogram instruments are available in the telemetry layer. |
| `exporter.topic.shutdown_nacks` | `{item}` | Number of pending end-to-end messages nacked during shutdown. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `topic_exporter.start` | `info` | Exporter startup with topic name and effective publish policy. |
| `topic_exporter.drop_newest` | `warn` | A publish was dropped because the topic queue was full and policy dropped newest. |
| `topic_exporter.outcome_capacity_full` | `warn` | A publish requiring end-to-end outcome tracking was rejected because tracking capacity was exhausted. |

## Limits

- The named topic must be declared and visible to the pipeline.
- Queue capacity and ack propagation limits are configured on the topic
  declaration, not on this exporter.
- Broadcast topic ack propagation currently resolves on the first subscriber
  outcome, as described in the configuration model.

## Related Docs

- [Configuration model topics](../../../../../docs/configuration-model.md#topic-declarations)
- [Core node catalog](../../../README.md)
