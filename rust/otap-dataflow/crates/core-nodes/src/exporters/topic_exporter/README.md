# Topic Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:topic` (`urn:otel:exporter:topic`)
- Feature gate: `topic`
- Stability: Experimental

## Overview

The topic exporter publishes pdata to a named in-process topic declared in the
runtime configuration. It can bridge end-to-end ACK/NACK outcomes when the
topic declaration enables ack propagation.

## Getting Started

Publish to a topic by naming the topic in the node-level config:

```yaml
type: exporter:topic
config:
  topic: raw_signals
  queue_on_full: drop_newest
```

The `raw_signals` topic must be declared in the surrounding runtime
configuration.

## Configuration

```yaml
type: exporter:topic
config:
  # Topic name to publish to (required).
  topic: raw_signals

  # Optional local full-queue behavior. When omitted, the topic policy applies.
  # Supported values include "block" and "drop_newest".
  queue_on_full: drop_newest
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `exporter.attempted`

Each attempt begins when the exporter receives the batch and ends when topic
publication succeeds, fails, or is refused. It includes publisher-side decoding
when required and any wait for queue space. Downstream acknowledgments are
tracked separately. Duration and item accounting follow the node telemetry policy.

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.attempted.messages` | `{message}` | `signal`, `outcome` | Number of topic admission attempts. |
| `exporter.attempted.duration` | `s` | `signal`, `outcome` | Time spent preparing and admitting a batch. |
| `exporter.attempted.items` | `{item}` | `signal`, `outcome` | Signal items belonging to the attempted publish. |

#### `exporter.topic.rejections`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.topic.rejections.messages` | `{message}` | `signal`, `reason` | Publishes refused because the queue was full, outcome capacity was exhausted, or shutdown interrupted admission. |

The bounded `reason` values are `queue_full`, `outcome_capacity`, and
`shutdown`.

#### `exporter.topic.tracked`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.topic.tracked.messages` | `{message}` | `signal`, `result` | Admitted tracked publishes grouped by their downstream terminal result. |

The bounded `result` values are `ack`, `nack`, `timeout`, `topic_closed`, and
`shutdown`.

#### `exporter.topic`

| Metric | Unit | Description |
| --- | --- | --- |
| `exporter.topic.tracked.in.flight` | `{message}` | Current number of admitted tracked publishes waiting for a terminal result. |

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
