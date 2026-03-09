# Topic Module

This module provides the engine-facing topic abstraction used to decouple
pipeline nodes through named topics.

## Goals

- Backend-agnostic API for topic publish/subscribe.
- Decouple publisher and subscriber nodes through named topics.
- Support both balanced and broadcast subscriptions behind one contract.
- Keep engine-facing code stable while allowing multiple backends
  (in-memory now, persistent/distributed later).

## Architecture

The topic module is organized in three layers:

1. Public API layer

- `TopicBroker<T>`: create, lookup, list, remove, and close topics.
- `TopicHandle<T>`: publish and subscribe entry point.
- `Subscription<T>`: receive + ack/nack interface.
- `TopicSet<T>`: per-pipeline resolved map of local topic alias ->
  `TopicHandle<T>`.

1. Backend abstraction layer

- `TopicBackend<T>`: factory creating per-topic state.
- `TopicState<T>`: shared per-topic operations (`publish`,
  `try_publish`, subscribe, close).
- `SubscriptionBackend<T>`: per-subscription receive + ack/nack backend.

1. Backend implementation layer (current)

- `InMemoryBackend` creates `TopicInner<T>`.
- `TopicInner<T>` selects one variant from `TopicOptions`:
  - `BalancedOnly`
  - `BroadcastOnly`
  - `Mixed`

High-level flow:

```text
Publisher/Subscriber Node
    -> TopicSet (local alias resolution)
    -> TopicHandle
    -> dyn TopicState (backend-agnostic contract)
    -> backend implementation (in-memory today)
```

## YAML Mapping

Runtime YAML declares topics under `topics.<name>` or
`groups.<group>.topics.<name>`:

```yaml
topics:
  ingress:
    backend: in_memory
    impl_selection: auto
    policies:
      balanced:
        queue_capacity: 1024
        on_full: block
      broadcast:
        queue_capacity: 4096
        on_lag: drop_oldest
      ack_propagation: disabled
```

Mapping from YAML to runtime behavior:

- `receiver:topic` with `subscription.mode: balanced` uses the
  balanced delivery path.
- `receiver:topic` with `subscription.mode: broadcast` uses the
  broadcast delivery path.
- The controller infers `TopicOptions::BalancedOnly`,
  `BroadcastOnly`, or `Mixed` from actual topic usage.
- `impl_selection: force_mixed` disables that optimization and always
  selects `TopicOptions::Mixed`.
- `balanced.queue_capacity` and `balanced.on_full` govern balanced
  consumer-group queues.
- `broadcast.queue_capacity` and `broadcast.on_lag` govern the
  broadcast ring and slow-subscriber behavior.
- `policies.ack_propagation` controls whether topic hops can bridge
  Ack/Nack across pipelines.
- `exporter:topic.config.queue_on_full` is a per-publisher override
  for balanced full-queue behavior; it does not override broadcast lag
  policy.

## Backend Capability Contract

Current minimal behavior:

- The controller validates topic declarations before broker creation.
- Unsupported backend, mode, or policy combinations fail fast at
  startup with explicit errors.
- Capability checks are startup-only; publish/recv hot paths are unchanged.

Recommended full engine-level contract for a future second backend:

1. Each backend should declare a `TopicBackendCapabilities` contract in
   the engine topic layer.
1. Capabilities should cover backend availability plus support for the
   selected runtime mode (`BalancedOnly`, `BroadcastOnly`, `Mixed`) and
   policy families such as `broadcast.on_lag` and
   `ack_propagation`.
1. Topic creation should validate the selected backend, mode, and
   policies against that contract before instantiating backend state.
1. Failures should use explicit errors such as
   `UnsupportedTopicBackend`, `UnsupportedTopicMode`, and
   `UnsupportedTopicPolicy` rather than generic internal errors.
1. Capability validation should remain on the topic creation path only;
   it should not add work to publish or receive hot paths.

That fuller contract is not implemented in the engine layer yet. Today
the controller owns the startup validation, while the broker/backend
API stays unchanged and infallible for the built-in in-memory backend.

## Example Use Cases

### 1. Work distribution (balanced)

```rust
use otap_df_engine::topic::{
    SubscriberOptions, SubscriptionMode, TopicBroker, TopicOptions,
};
use otap_df_config::SubscriptionGroupName;

let broker = TopicBroker::<u64>::new();
let topic = broker.create_in_memory_topic(
    "ingress",
    TopicOptions::BalancedOnly { capacity: 1024 },
)?;

let mut worker_a = topic.subscribe(
    SubscriptionMode::Balanced {
        group: SubscriptionGroupName::from("workers"),
    },
    SubscriberOptions::default(),
)?;
let mut worker_b = topic.subscribe(
    SubscriptionMode::Balanced {
        group: SubscriptionGroupName::from("workers"),
    },
    SubscriberOptions::default(),
)?;

// Each published message is delivered to exactly one subscriber in
// "workers".
```

### 2. Fan-out analytics (broadcast)

```rust
use otap_df_engine::topic::{
    SubscriberOptions, SubscriptionMode, TopicBroadcastOnLagPolicy,
    TopicBroker, TopicOptions,
};

let broker = TopicBroker::<u64>::new();
let topic = broker.create_in_memory_topic(
    "audit",
    TopicOptions::BroadcastOnly {
        capacity: 4096,
        on_lag: TopicBroadcastOnLagPolicy::DropOldest,
    },
)?;

let mut sink_a = topic.subscribe(
    SubscriptionMode::Broadcast,
    SubscriberOptions::default(),
)?;
let mut sink_b = topic.subscribe(
    SubscriptionMode::Broadcast,
    SubscriberOptions::default(),
)?;

// Both subscribers receive each message from their subscribe point onward.
```

### 3. Mixed criticality (balanced + broadcast on one topic)

```rust
use otap_df_engine::topic::{
    SubscriberOptions, SubscriptionMode, TopicBroadcastOnLagPolicy,
    TopicBroker, TopicOptions,
};
use otap_df_config::SubscriptionGroupName;

let broker = TopicBroker::<u64>::new();
let topic = broker.create_in_memory_topic(
    "events",
    TopicOptions::Mixed {
        balanced_capacity: 1024,
        broadcast_capacity: 4096,
        on_lag: TopicBroadcastOnLagPolicy::DropOldest,
    },
)?;

let mut primary = topic.subscribe(
    SubscriptionMode::Balanced {
        group: SubscriptionGroupName::from("primary"),
    },
    SubscriberOptions::default(),
)?;
let mut security = topic.subscribe(
    SubscriptionMode::Broadcast,
    SubscriberOptions::default(),
)?;
let mut anomaly = topic.subscribe(
    SubscriptionMode::Broadcast,
    SubscriberOptions::default(),
)?;
```

### 4. Backend selection per topic

```rust
use otap_df_engine::topic::{
    InMemoryBackend, TopicBroker, TopicOptions,
};

let broker = TopicBroker::<u64>::new();
let _topic = broker.create_topic(
    "t1",
    TopicOptions::default(),
    InMemoryBackend,
)?;

// Another backend can be plugged in later via the same TopicBackend trait.
```

## Public Contracts

### Runtime lifecycle

1. Topics are created with `TopicBroker::create_topic` /
   `create_topics` (or in-memory convenience methods).
2. Nodes obtain a `TopicHandle` via broker lookup or from a `TopicSet`.
3. Publishers use `publish`/`try_publish`; subscribers use `subscribe`
   -> `Subscription::recv`.
4. `TopicHandle::close` or `TopicBroker::remove_topic` closes the topic.
5. After close:

- publish operations return `Error::TopicClosed`.
- receive side eventually observes `Error::SubscriptionClosed`.
- balanced subscribe attempts are rejected with `Error::TopicClosed`.

### Publish contract

- `publish(msg).await`
  - Accepts a message or returns an error.
  - May await under balanced backpressure.
- `try_publish(msg)`
  - Never awaits.
  - Returns `PublishOutcome::Published` or `PublishOutcome::DroppedOnFull`.
- In `Mixed` mode, `try_publish` can return `DroppedOnFull` because
  balanced queues are full while broadcast delivery may still succeed.
- Messages are not retained for future subscribers: subscribing later
  does not replay pre-subscribe history.

### Delivery + ack/nack contract

- Balanced mode (`SubscriptionMode::Balanced { group }`)
  - One consumer group receives one logical stream.
  - Within a group, each delivered message goes to exactly one subscriber.
  - Different groups receive independently.
- Broadcast mode (`SubscriptionMode::Broadcast`)
  - Each subscriber receives each message from its subscribe point.
  - With `TopicBroadcastOnLagPolicy::DropOldest`, slow subscribers may
    receive `RecvItem::Lagged { missed }` and continue from the oldest
    retained message.
  - With `TopicBroadcastOnLagPolicy::Disconnect`, slow subscribers
    receive a final `RecvItem::Lagged { missed }` and the next
    `recv()` returns `Error::SubscriptionClosed`.
- Ack/Nack
  - Subscribers ack/nack by message ID (`Subscription::ack` /
    `Subscription::nack`).
  - Ack routing is per publisher handle when `with_ack_sender` is used
    (directly or via `TopicSet::with_ack_sender`).
  - If no ack sender is registered, ack/nack returns `Error::AckNotEnabled`.

## Guarantees

- One stable engine-facing contract for topic operations, independent
  of backend implementation.
- Strongly-typed topic and balanced group identifiers (`TopicName`,
  `SubscriptionGroupName`) at API boundaries.
- Explicit mode compatibility errors:
  - `Error::SubscribeBalancedNotSupported`
  - `Error::SubscribeBroadcastNotSupported`
  - `Error::SubscribeSingleGroupViolation` (for balanced-only topics
    with a different group)
- Broker duplicate checks are atomic in `create_topics`: if any
  duplicate exists, no topic from that batch is inserted.
- `TopicSet::remove` only detaches local alias mapping; it does not
  close the underlying topic.

## Current Limits

- Current integrated backend is in-memory; no built-in persistent or
  distributed backend is wired yet.
- Delivery is bounded by configured capacities:
  - balanced channels are bounded and can backpressure (`publish`) or
    report drop-on-full (`try_publish`)
  - broadcast uses a bounded ring buffer and slow subscribers either
    continue after `RecvItem::Lagged` or disconnect, depending on
    `TopicBroadcastOnLagPolicy`
- No replay/history for late subscribers: delivery starts from subscribe time.
- `SubscriberOptions` is currently empty (no per-subscriber tuning
  knobs exposed yet).

## Non-Goals

- No guarantee that all backends provide identical internal behavior
  for every policy.
- No persistence/distributed semantics in this abstraction layer itself
  (in the current impl).
- No exactly-once semantics promised by this module.
