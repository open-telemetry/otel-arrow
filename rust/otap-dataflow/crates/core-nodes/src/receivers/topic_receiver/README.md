# Topic Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:topic` (`urn:otel:receiver:topic`)
- Feature gate: Default
- Stability: Experimental

## Overview

The topic receiver subscribes to a named in-process topic and forwards received
pdata into its pipeline. It supports broadcast subscriptions and balanced
consumer groups.

## Getting Started

Subscribe to a declared topic with the default broadcast mode:

```yaml
type: receiver:topic
config:
  topic: raw_signals
  subscription:
    mode: broadcast
```

The `raw_signals` topic must be declared in the surrounding runtime
configuration.

## Configuration

```yaml
type: receiver:topic
config:
  # Topic name to subscribe to (required).
  topic: raw_signals

  # Subscription mode and options (default: broadcast).
  subscription:
    # "broadcast" gives each subscriber every message.
    mode: broadcast

    # Use "balanced" with a group to share messages across subscribers in the
    # same group.
    # mode: balanced
    # group: workers
```

Subscription modes:

- `broadcast`: each subscriber receives each message.
- `balanced`: subscribers in the same `group` share the stream.

## Examples

Balanced subscription:

```yaml
type: receiver:topic
config:
  topic: raw_signals
  subscription:
    mode: balanced
    group: workers
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.topic`

| Metric | Unit | Description |
| --- | --- | --- |
| `receiver.topic.forwarded_messages` | `{item}` | Number of messages forwarded to downstream. |
| `receiver.topic.forward_failures` | `{item}` | Number of forward failures to downstream channel. |
| `receiver.topic.lagged_notifications` | `{event}` | Number of lag notifications emitted by broadcast subscriptions. |
| `receiver.topic.lagged_messages` | `{item}` | Total messages missed across lag notifications. |
| `receiver.topic.lag_disconnects` | `{event}` | Number of broadcast subscriptions disconnected because of lag. |
| `receiver.topic.downstream_backpressure_events` | `{event}` | Number of downstream backpressure events (>= 500ms blocked). |
| `receiver.topic.downstream_blocked_ms` | `ms` | Total milliseconds blocked while forwarding to downstream. |
| `receiver.topic.bridged_downstream_acks` | `{item}` | Number of downstream ACK controls successfully bridged to topic ack. |
| `receiver.topic.bridged_downstream_nacks` | `{item}` | Number of downstream NACK controls successfully bridged to topic nack. |
| `receiver.topic.bridge_controls_ignored_propagation_disabled` | `{event}` | Number of downstream ACK/NACK controls ignored because topic Ack/Nack propagation is disabled for this receiver. |
| `receiver.topic.bridge_missing_calldata` | `{event}` | Number of downstream ACK/NACK controls missing the bridged topic message id in calldata. |
| `receiver.topic.bridge_invalid_or_untracked_id` | `{event}` | Number of downstream ACK/NACK controls carrying an id that is not currently tracked by the topic runtime. With the current raw `message_id` bridge this also includes invalid or forged ids; those causes are not distinguishable yet. |
| `receiver.topic.bridge_runtime_failures` | `{event}` | Number of downstream ACK/NACK controls that failed to bridge for some runtime reason other than an unknown message id. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `topic_receiver.start` | `info` | Receiver startup with topic, subscription, and ACK propagation mode. |
| `topic_receiver.drain_ingress_pending_forward_nack_failed` | `warn` | A pending forward could not be NACKed during ingress drain. |
| `topic_receiver.drain_ingress_drop_pending_forward` | `warn` | A pending forward was dropped during ingress drain. |
| `topic_receiver.drain_ingress.timeout` | `warn` | Ingress drain timed out. |
| `topic_receiver.drain_ingress_force_nack_failed` | `warn` | A forced NACK failed during ingress drain. |
| `topic_receiver.bridge_ack_untracked_or_invalid_id` | `warn` | A downstream ACK referenced an invalid or untracked topic message id. |
| `topic_receiver.bridge_ack_failed` | `warn` | A downstream ACK failed to bridge to the topic runtime. |
| `topic_receiver.bridge_ack_missing_calldata` | `warn` | A downstream ACK did not include the required bridged message id calldata. |
| `topic_receiver.bridge_nack_untracked_or_invalid_id` | `warn` | A downstream NACK referenced an invalid or untracked topic message id. |
| `topic_receiver.bridge_nack_failed` | `warn` | A downstream NACK failed to bridge to the topic runtime. |
| `topic_receiver.bridge_nack_missing_calldata` | `warn` | A downstream NACK did not include the required bridged message id calldata. |
| `topic_receiver.downstream_backpressure` | `warn` | Forwarding to downstream was blocked long enough to count as backpressure. |
| `topic_receiver.forward_failed` | `warn` | Forwarding a topic message to downstream failed. |
| `topic_receiver.lag_disconnect` | `warn` | The receiver disconnected from the broadcast topic because it lagged too far behind. |
| `topic_receiver.lagged` | `warn` | The receiver missed one or more topic messages due to broadcast lag. |

## Limits

- The named topic must be declared and visible to the pipeline.
- Topic wiring must remain acyclic across topic hops.
- Queue capacity, lag policy, and ack propagation limits are configured on the
  topic declaration.

## Related Docs

- [Configuration model topics](../../../../../docs/configuration-model.md#topic-declarations)
- [Core node catalog](../../../README.md)
