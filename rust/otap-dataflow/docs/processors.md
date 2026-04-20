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
- `filter_processor`
- `log_sampling_processor`
- `delay_processor`
- `debug_processor`

### Stateful Single-Route Schedulers

Stateful single-route schedulers also have one logical downstream path, but they
retain local state across calls. They may batch, retry, persist, reaggregate, or
defer work before forwarding it.

Examples:

- `batch_processor`
- `retry_processor`
- `durable_buffer_processor`
- `temporal_reaggregation_processor`

### Exclusive Routers

Exclusive routers expose multiple output ports, but each inbound message selects
at most one output route. Route-local admission matters because one blocked
selected route should not automatically stall unrelated routes.

Examples:

- `content_router`
- `signal_type_router`

See [Exclusive Router Guarantees](#exclusive-router-guarantees) for the current
runtime contract.

### Replicating Multi-Route Processors

Replicating multi-route processors clone one inbound message to multiple
destinations. Completion is usually aggregated across those destinations and may
depend on Ack/Nack policy, fallback chains, and timeout handling.

Example:

- `fanout_processor`

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
| `filter_processor` | Inline single-route | `single-route`, `inline`, `may-drop` | Filters signals according to configured rules. |
| `log_sampling_processor` | Inline single-route | `single-route`, `inline`, `may-drop` | Samples logs to reduce volume. |
| `delay_processor` | Inline single-route | `single-route`, `inline` | Adds artificial delay for testing and rate-shaping scenarios. |
| `debug_processor` | Inline single-route | `single-route`, `inline` | Observes or emits debug output while preserving simple forward flow. |
| `batch_processor` | Stateful single-route scheduler | `single-route`, `stateful`, `batching`, `wakeup-driven`, `ack-aware` | Batches by size or time and tracks Ack/Nack-sensitive request state. |
| `retry_processor` | Stateful single-route scheduler | `single-route`, `stateful`, `retrying`, `ack-aware` | Retries failed downstream delivery using exponential backoff. |
| `durable_buffer_processor` | Stateful single-route scheduler | `single-route`, `stateful`, `buffering`, `retrying`, `wakeup-driven` | Persists data before forwarding and retries from durable state. |
| `temporal_reaggregation_processor` | Stateful single-route scheduler | `single-route`, `stateful`, `buffering`, `admission-gated` | Reaggregates metrics at lower frequency. |
| `content_router` | Exclusive router | `multi-route`, `exclusive-routing`, `wakeup-driven`, `admission-gated` | Routes by resource attribute value to one selected output. |
| `signal_type_router` | Exclusive router | `multi-route`, `exclusive-routing`, `wakeup-driven`, `admission-gated` | Routes by signal type to one selected output. |
| `fanout_processor` | Replicating multi-route processor | `multi-route`, `replicating`, `ack-aware`, `admission-gated` | Clones data to configured destinations and aggregates completion. |
| `transform_processor` | Transforming routed emitter | `multi-route`, `stateful`, `ack-aware` | Runs transformation/query logic that may emit routed outputs. |
<!-- markdownlint-enable MD013 -->

## Exclusive Router Guarantees

`content_router` and `signal_type_router` are exclusive-routing processors:
each inbound message selects at most one downstream output route.

This section defines the runtime guarantees they provide once a route has been
selected.

### Shared Contract

For both routers:

- selected-route admission never awaits the downstream send in the main router
  task
- `Closed` on the selected route always produces an immediate route-local
  retryable NACK
- `default_output` uses the same admission policy as matched or named routing
  once the default route has been selected
- the router returns success after route-local rejection, so the node stays
  live and control traffic continues to flow

The routers choose among two policies for selected-route `Full`:

- `reject_immediately`: default policy; emit an immediate route-local
  retryable NACK with `cause = RouteFull`; unrelated healthy routes continue to
  flow
- `backpressure`: keep at most one parked message per blocked output port; keep
  admitting pdata while at least one selectable route is still making progress;
  close pdata admission only when every selectable route currently has a parked
  full message; reopen pdata admission once at least one parked route forwards;
  later messages for an already parked route are retryable-NACKed with
  `cause = RouteFull`

These policies are implemented with explicit router-local state and
processor-local wakeups. They do not reintroduce the old head-of-line blocking
bug caused by awaiting the selected route send inside the main router task.

### Shutdown Behavior

If a router has locally parked messages when `NodeControlMsg::Shutdown` starts:

- every parked message is retryable-NACKed locally
- those NACKs use `cause = NodeShutdown`
- the router clears its parked state instead of leaving messages stranded in a
  local wait path

This applies only to work still owned by the router. Work already admitted to a
downstream channel is outside the router's scope.

### Router-Specific Behavior

These guarantees apply only after an output route has been selected. Route
selection itself remains processor-specific.

#### `content_router`

- routes by configured resource attribute values
- if no configured route matches, `default_output` is used when present
- if the routing key is missing or no route matches and there is no default
  output, the message is rejected as before
- mixed-batch rejection and conversion-error rejection are unchanged

#### `signal_type_router`

- prefers the signal-type-specific named output (`logs`, `metrics`, `traces`)
  when that port is connected
- falls back to the node default output only when the type-specific named port
  is not connected
- existing drop behavior when no type-specific port is connected and no default
  output exists is unchanged

### Ack/Nack Propagation Across Topic Hops

These router guarantees are local to the processor. Whether a router-generated
NACK is bridged farther upstream depends on the topic's
`ack_propagation.mode`.

- with `ack_propagation.mode: disabled`, a router-generated NACK remains local
  to the downstream side of the topic hop
- with `ack_propagation.mode: auto`, that same NACK is bridged upstream across
  the topic hop

This means the admission policy is the same in both modes, but the visibility
of its NACKs to upstream publishers differs. See
[Configuration Model](./configuration-model.md#topics).

### Observability

Both routers expose separate telemetry for route rejection caused by:

- selected route full
- selected route closed

`signal_type_router` reports those counters per signal type. These counters are
distinct from unmatched-route, missing-key, conversion, and drop telemetry.

Routers also stamp a machine-readable `NackCause` alongside the existing
human-readable `reason` string:

- `RouteFull`
- `RouteClosed`
- `NodeShutdown`

### Non-Goals and Future Direction

These routers do not currently guarantee either of the following:

- draining or NACKing work that has already been admitted to downstream
  channels when a route later closes
- a generic engine-wide admission policy shared by all multi-output processors
  such as `fanout_processor` or `transform_processor`

The current policy surface is intentionally local to exclusive routers.

The next useful extensions would be:

- richer blocked-route scheduling only if a new exclusive router needs the same
  contract
- bounded per-route queueing beyond one parked message per blocked output port
- downstream lifecycle semantics for flushing already-admitted work when a
  route closes or a pipeline shuts down
- more selective pause conditions when some selected routes are unavailable
  rather than merely full

Those extensions should keep the same `NackCause` meanings instead of
reinterpreting `RouteFull`, `RouteClosed`, or `NodeShutdown`.
