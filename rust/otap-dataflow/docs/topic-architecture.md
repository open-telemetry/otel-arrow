# Topic Architecture

This document summarizes how the topic system fits into the engine and how the
current in-memory topic runtime is structured internally.

## Scope

The diagrams below focus on four things:

- startup-time integration with config and controller logic
- runtime layering from pipeline nodes down to backend state
- in-memory delivery structures for balanced, broadcast, and mixed topics
- tracked publish and Ack/Nack propagation across a topic hop

## 1. Startup Integration

```mermaid
flowchart LR
    A[YAML topic declarations<br/>+ topic exporter/receiver nodes]
    B[Controller]
    C[Topic validation<br/>mode inference<br/>capability checks<br/>cycle checks]
    D[TopicBroker]
    E[Declared topics<br/>TopicHandle per declared topic]
    F[PipelineTopicBinding]
    G[TopicSet per pipeline]
    H[PipelineContext]
    I[OTAP exporter:topic]
    J[OTAP receiver:topic]

    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
    F --> G
    G --> H
    H --> I
    H --> J
```

### Startup Highlights

- The controller owns topic declaration and startup validation.
- Topic mode is inferred from actual topic usage and then mapped into
  `TopicOptions`.
- Capability validation and topic-wiring cycle detection happen before topic
  creation.
- Each pipeline receives a `TopicSet` containing `PipelineTopicBinding`
  instances, not raw broker state.

## 2. Runtime Layering

```mermaid
flowchart TD
    subgraph Pipeline["Pipeline runtime"]
        A[exporter:topic / receiver:topic]
        B[TopicSet]
        C[PipelineTopicBinding]
        D[TopicHandle]
        E[TrackedTopicPublisher]
        F[Subscription]
    end

    subgraph Engine["Topic runtime"]
        G[TopicBroker]
        H[Arc<dyn TopicState<T>>]
        I[TopicInner]
        J[BalancedOnlyTopic]
        K[BroadcastOnlyTopic]
        L[MixedTopic]
        M[TrackedPublishTracker]
    end

    A --> B --> C --> D
    D --> E
    D --> F
    G --> D
    D --> H --> I
    I --> J
    I --> K
    I --> L
    J --> M
    K --> M
    L --> M
```

### Runtime Highlights

- `TopicHandle` is the pure runtime API.
- `PipelineTopicBinding` adds pipeline-scoped defaults such as
  `queue_on_full` and `ack_propagation.mode`.
- `TrackedTopicPublisher` is layered on top of `TopicHandle` and adds
  bounded in-flight tracked publishes.
- The broker stores backend-erased `TopicState<T>` instances.
- The current in-memory backend is selected through `TopicInner`, which picks
  one of the three runtime implementations.
- `TrackedPublishTracker` is the shared tracked-outcome mechanism used by the
  in-memory topic variants.

## 3. In-Memory Topic Structures

```mermaid
flowchart LR
    subgraph Balanced["BalancedOnlyTopic"]
        B1[Single balanced group]
        B2[async_channel bounded queue]
        B3[BalancedSub]
        B1 --> B2 --> B3
    end

    subgraph Broadcast["BroadcastOnlyTopic"]
        C1[FastBroadcastRing]
        C2[BroadcastSub A]
        C3[BroadcastSub B]
        C1 --> C2
        C1 --> C3
    end

    subgraph Mixed["MixedTopic"]
        D1[Broadcast ring]
        D2[group_senders snapshot]
        D3[Balanced group queue 1]
        D4[Balanced group queue 2]
        D5[BroadcastSub]
        D1 --> D5
        D2 --> D3
        D2 --> D4
    end

    T[TrackedPublishTracker]
    T --- Balanced
    T --- Broadcast
    T --- Mixed
```

### Structure Highlights

- Balanced delivery uses bounded async queues per consumer group.
- Broadcast delivery uses a single ring buffer with per-subscriber cursors.
- Mixed topics combine both structures in one topic instance.
- All three variants share the same tracked publish tracker for tracked
  outcome resolution.

## 4. Tracked Publish and Ack/Nack Flow

```mermaid
sequenceDiagram
    participant U as Upstream node
    participant TE as exporter:topic
    participant TP as TrackedTopicPublisher
    participant TT as Topic runtime
    participant TR as TrackedPublishTracker
    participant RR as receiver:topic
    participant D as Downstream node

    U->>TE: PData with Ack/Nack interest
    TE->>TP: publish(data)
    TP->>TT: publish_tracked(msg, timeout, permit)
    TT->>TR: register(message_id, timeout, permit)
    TT-->>TP: TrackedPublishReceipt
    TP-->>TE: receipt
    TT->>RR: deliver Envelope { id, tracked=true, payload }
    RR->>D: forward PData and subscribe_to(ACKS | NACKS, message_id)
    D-->>RR: Ack or Nack control
    RR->>TT: subscription.ack(id) / nack(id, reason)
    TT->>TR: resolve(message_id, outcome)
    TR-->>TE: receipt resolves
    TE-->>U: upstream Ack or Nack
```

### Flow Highlights

- The exporter keeps the original upstream `PData` until the tracked receipt
  resolves.
- The topic runtime, not the exporter, owns tracked publish outcome state.
- `max_in_flight` is enforced before entering the topic runtime.
- The timeout belongs to the tracked publish contract and is applied after the
  topic accepts the publish.

## Notes and Current Limits

- Topic wiring across pipelines must remain acyclic. Startup rejects both
  same-pipeline feedback through topics and multi-pipeline topic loops.
- In broadcast mode, `ack_propagation.mode: auto` still uses
  first-subscriber-wins semantics today; it does not wait for all broadcast
  subscribers to Ack/Nack.
- Topic-owned gauges for balanced group count and broadcast subscriber count
  are still future work. Current metrics live on the topic exporter and topic
  receiver nodes.
