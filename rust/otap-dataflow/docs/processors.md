# Processors

Processors are the middle nodes in an OTAP Dataflow pipeline DAG. Receivers
ingest telemetry and create `pdata`, processors transform or route that `pdata`,
and exporters deliver it outside the pipeline.

Processors can have one or more input sources and one or more output ports. The
engine owns their input loop, delivers both data and control messages, and asks
each processor whether new `pdata` should be admitted through `accept_pdata()`.
Processors use effect handlers to emit downstream data, Ack/Nack outcomes,
telemetry, wakeups, and other runtime effects.

This document describes the main processor classes currently represented in
`core-nodes`. The taxonomy is descriptive; it is not a Rust trait hierarchy or a
configuration schema.

## Primary Processor Classes

Each current core processor fits one primary class. Secondary traits, described
later, capture cross-cutting behavior.

### Inline Single-Route Processors

Inline single-route processors have one logical downstream path and handle each
inbound message in the main `process()` call. They may mutate, observe, delay,
or drop messages, but they do not maintain router-style multi-output state.

Examples:

- `attributes_processor`
- [`filter_processor`](../crates/core-nodes/src/processors/filter_processor/README.md)
- [`log_sampling_processor`](../crates/core-nodes/src/processors/log_sampling_processor/README.md)
- `delay_processor`
- [`debug_processor`](../crates/core-nodes/src/processors/debug_processor/README.md)

### Stateful Single-Route Schedulers

Stateful single-route schedulers also have one logical downstream path, but they
retain local state across calls. They may batch, retry, persist, reaggregate, or
defer work before forwarding it.

Examples:

- `batch_processor`
- `retry_processor`
- [`durable_buffer_processor`](../crates/core-nodes/src/processors/durable_buffer_processor/README.md)
- [`temporal_reaggregation_processor`](../crates/core-nodes/src/processors/temporal_reaggregation_processor/README.md)

### Exclusive Routers

Exclusive routers expose multiple output ports, but each inbound message selects
at most one output route. Route-local admission matters because one blocked
selected route should not automatically stall unrelated routes.

Examples:

- [`content_router`](../crates/core-nodes/src/processors/content_router/README.md)
- [`signal_type_router`](../crates/core-nodes/src/processors/signal_type_router/README.md)

### Replicating Multi-Route Processors

Replicating multi-route processors clone one inbound message to multiple
destinations. Completion is usually aggregated across those destinations and may
depend on Ack/Nack policy, fallback chains, and timeout handling.

Example:

- [`fanout_processor`](../crates/core-nodes/src/processors/fanout_processor/README.md)

### Transforming Routed Emitters

Transforming routed emitters run transformation logic that may produce zero,
one, or many output batches. Routing is computed by transformation/query
execution rather than by a static exclusive route selector.

Example:

- `transform_processor`

## Secondary Behavior Traits

Secondary traits describe behavior that cuts across the primary classes.

- `single-route`: the processor has one logical downstream path.
- `multi-route`: the processor can emit to more than one output port.
- `exclusive-routing`: each inbound message selects at most one output.
- `replicating`: one inbound message may be cloned to multiple outputs.
- `inline`: processing completes in the current `process()` call.
- `stateful`: processing depends on local state retained across calls.
- `ack-aware`: the processor subscribes to downstream Ack/Nack outcomes.
- `wakeup-driven`: the processor uses local wakeups to resume deferred work.
- `admission-gated`: the processor can return `accept_pdata() == false`.
- `buffering`: the processor retains data locally before forwarding.
- `batching`: the processor combines or splits data by size or time.
- `retrying`: the processor reschedules failed delivery attempts.
- `may-drop`: the processor can intentionally drop input data.
- `must-explicitly-ack-or-nack-deferred-work`: the processor owns deferred work
  and must resolve it explicitly during success, failure, or shutdown paths.

Port count alone is not enough to classify a processor. For example,
`content_router` and `fanout_processor` are both multi-output, but
`content_router` selects one route while `fanout_processor` replicates to
multiple destinations. `transform_processor` can emit routed outputs, but those
routes are produced by transformation execution rather than a static router.

## Current Core Processor Classification

<!-- markdownlint-disable MD013 -->
| Processor | Primary class | Notable secondary traits | Notes |
| --- | --- | --- | --- |
| `attributes_processor` | Inline single-route | `single-route`, `inline` | Mutates OpenTelemetry attributes before forwarding. |
| [`filter_processor`](../crates/core-nodes/src/processors/filter_processor/README.md) | Inline single-route | `single-route`, `inline`, `may-drop` | Filters signals according to configured rules. |
| [`log_sampling_processor`](../crates/core-nodes/src/processors/log_sampling_processor/README.md) | Inline single-route | `single-route`, `inline`, `may-drop` | Samples logs to reduce volume. |
| `delay_processor` | Inline single-route | `single-route`, `inline` | Adds artificial delay for testing and rate-shaping scenarios. |
| [`debug_processor`](../crates/core-nodes/src/processors/debug_processor/README.md) | Inline single-route | `single-route`, `inline` | Observes or emits debug output while preserving simple forward flow. |
| `batch_processor` | Stateful single-route scheduler | `single-route`, `stateful`, `batching`, `wakeup-driven`, `ack-aware` | Batches by size or time and tracks Ack/Nack-sensitive request state. |
| `retry_processor` | Stateful single-route scheduler | `single-route`, `stateful`, `retrying`, `ack-aware` | Retries failed downstream delivery using exponential backoff. |
| [`durable_buffer_processor`](../crates/core-nodes/src/processors/durable_buffer_processor/README.md) | Stateful single-route scheduler | `single-route`, `stateful`, `buffering`, `retrying`, `wakeup-driven` | Persists data before forwarding and retries from durable state. |
| [`temporal_reaggregation_processor`](../crates/core-nodes/src/processors/temporal_reaggregation_processor/README.md) | Stateful single-route scheduler | `single-route`, `stateful`, `buffering`, `admission-gated` | Reaggregates metrics at lower frequency. |
| [`content_router`](../crates/core-nodes/src/processors/content_router/README.md) | Exclusive router | `multi-route`, `exclusive-routing`, `wakeup-driven`, `admission-gated` | Routes by resource attribute value to one selected output. |
| [`signal_type_router`](../crates/core-nodes/src/processors/signal_type_router/README.md) | Exclusive router | `multi-route`, `exclusive-routing`, `wakeup-driven`, `admission-gated` | Routes by signal type to one selected output. |
| [`fanout_processor`](../crates/core-nodes/src/processors/fanout_processor/README.md) | Replicating multi-route processor | `multi-route`, `replicating`, `ack-aware`, `admission-gated` | Clones data to configured destinations and aggregates completion. |
| `transform_processor` | Transforming routed emitter | `multi-route`, `stateful`, `ack-aware` | Runs transformation/query logic that may emit routed outputs. |
<!-- markdownlint-enable MD013 -->
