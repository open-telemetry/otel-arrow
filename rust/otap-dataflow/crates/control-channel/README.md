# Control-Aware Bounded Channel

This crate defines a bounded control channel for OTAP dataflow node-control
traffic.

Its purpose is to provide a control-plane primitive with semantics that are
stronger and more explicit than a generic FIFO MPSC queue:

- lifecycle delivery must not depend on ordinary control backlog
- high-frequency completion traffic must remain efficient
- low-value control noise should be coalesced instead of competing with
  correctness-critical work
- shutdown progress must remain bounded and explicit
- the public API should make invalid lifecycle operations hard to express

Today this crate is developed standalone so its queue policy, fairness, and
shutdown behavior can be evaluated in isolation. The design is intended for
future engine integration.

## Goal

The control channel is meant for node-control traffic, not for pdata transport.

It targets the specific needs of the OTAP engine:

- thread-per-core execution
- single-threaded async runtimes on the hot path
- frequent `Ack` and `Nack` traffic when `wait_for_result` is enabled
- node-local lifecycle transitions such as `DrainIngress` and `Shutdown`
- a need for bounded memory, bounded-fairness, and explicit terminal progress

The design separates policy classes that behave differently:

- retained and backpressured completion traffic
- best-effort coalesced control work
- latest-wins configuration updates
- reserved lifecycle tokens

## Design

The channel keeps one queue core with role-specific public APIs.

### Role-specific APIs

There are two channel families:

- `receiver_channel(...)`
  - sender type: `ReceiverControlSender`
  - receiver event type: `ReceiverControlEvent`
  - supports both `accept_drain_ingress(...)` and `accept_shutdown(...)`
- `node_channel(...)`
  - sender type: `NodeControlSender`
  - receiver event type: `NodeControlEvent`
  - supports `accept_shutdown(...)` only

This split is intentional. `DrainIngress` is receiver-specific lifecycle
control, so non-receiver nodes cannot express it through the public API.

The implementation is available in both:

- local form for `!Send` execution
- shared form for `Send` execution

Both variants are designed to expose the same semantics.

### Traffic classes

The channel currently supports:

- lifecycle:
  - `DrainIngress`
  - `Shutdown`
- retained completion traffic:
  - `Ack`
  - `Nack`
- latest-wins normal control:
  - `Config`
- coalesced best-effort normal control:
  - `TimerTick`
  - `CollectTelemetry`

Delayed resume is intentionally out of scope. It is expected to be handled by a
different mechanism than the control channel.

### Internal model

The queue core stores different control classes separately instead of forcing
everything through one FIFO:

- reserved lifecycle slots for `DrainIngress` and `Shutdown`
- a bounded completion deque for `Ack` and `Nack`
- one replaceable `Config` slot
- one coalesced `TimerTick` flag
- one coalesced `CollectTelemetry` flag

This lets the channel apply class-specific admission and receive-side policy
without requiring unbounded buffering or sender-side queue surgery.

## Behaviors And Guarantees

### Bounded memory

Ordinary retained traffic is bounded by configuration:

- `completion_msg_capacity`
- `completion_batch_max`
- `completion_burst_limit`

Lifecycle tokens do not consume ordinary bounded completion capacity.

### Type-safe lifecycle control

The sender API separates lifecycle acceptance from generic control sends:

- `accept_drain_ingress(...)`
- `accept_shutdown(...)`
- `try_send(...)`
- `send(...).await`

This prevents forced lifecycle traffic from being accidentally routed through
the ordinary bounded send path.

### Backpressure vs non-blocking send

For non-lifecycle traffic, the sender exposes two modes:

- `try_send(...)`
  - non-blocking
  - returns the original command on `Full` or `Closed`
- `send(...).await`
  - waits for bounded capacity when needed
  - returns only when the command is accepted or the channel closes

This supports both explicit backpressure and opportunistic best-effort usage,
depending on the caller.

### Completion batching

`Ack` and `Nack` are retained in a bounded FIFO and emitted as
`CompletionBatch(Vec<CompletionMsg<_>>)`.

Properties:

- arrival order is preserved within completion traffic
- batching reduces receive-side overhead under heavy completion load
- `completion_batch_max` bounds the size of a single emitted batch
- completion traffic remains eligible after `DrainIngress`
- completion traffic remains eligible after `Shutdown` until terminal progress
  occurs

### Latest-wins config

`Config` is not queued as an unbounded sequence.

Properties:

- only the most recent pending config is kept
- a new config replaces the previously pending one
- config is dropped once drain or shutdown begins

### Best-effort coalescing

`TimerTick` and `CollectTelemetry` are coalesced per channel.

Properties:

- only one pending tick of each kind is retained
- duplicate sends are reported as coalesced
- these events are dropped once drain or shutdown begins

### Bounded fairness

The receive side is not a pure priority queue and not a pure FIFO.

It enforces bounded fairness between completion traffic and normal control
traffic:

- completions are emitted in batches
- `completion_burst_limit` bounds how many completion messages can be delivered
  consecutively before one pending normal event must be surfaced
- `Config`, `TimerTick`, and `CollectTelemetry` are rotated fairly when
  multiple normal events are pending

This prevents long completion runs from starving all other control activity in
normal operation.

### Drain and shutdown semantics

`DrainIngress` and `Shutdown` are distinct lifecycle states.

Properties:

- `DrainIngress` and `Shutdown` are accepted through reserved lifecycle slots
- if both are present, `DrainIngress` is delivered first
- once drain begins, normal control work is no longer accepted
- once shutdown begins, normal control work is no longer accepted
- pending normal control state is cleared when drain or shutdown is accepted
- completion traffic may continue draining after shutdown is accepted

### Deadline-bounded terminal progress

`Shutdown` carries a deadline and the receiver wait path is deadline-aware.

Properties:

- if retained completion traffic drains before the deadline, `Shutdown` is
  emitted after retained work
- if the deadline expires first, terminal progress is forced
- when forced shutdown fires, remaining retained completions are abandoned and
  the channel closes
- after final shutdown delivery, new sends are rejected

This is the key liveness property needed to avoid shutdown being postponed
indefinitely by continued completion traffic.

### Matching local and shared semantics

The crate provides both local and shared implementations, but the design goal is
semantic parity:

- same role split
- same lifecycle behavior
- same batching and fairness rules
- same terminal shutdown behavior

## Observability

The channel exposes a `stats()` snapshot on both senders and receivers.

The snapshot includes:

- lifecycle state:
  - `phase`
  - `drain_ingress_recorded`
  - `shutdown_recorded`
  - `shutdown_forced`
  - `closed`
- current occupancy / pending state:
  - `completion_len`
  - `has_pending_drain_ingress`
  - `has_pending_shutdown`
  - `has_pending_config`
  - `has_pending_timer_tick`
  - `has_pending_collect_telemetry`
  - `completion_burst_len`
- cumulative counters:
  - `completion_batch_emitted_total`
  - `completion_message_emitted_total`
  - `config_replaced_total`
  - `timer_tick_coalesced_total`
  - `collect_telemetry_coalesced_total`
  - `normal_event_dropped_during_drain_total`
  - `completion_abandoned_on_forced_shutdown_total`

These snapshots are intended to map cleanly to a future engine metric set such
as `channel.control`, attached to the existing control-channel entity rather
than to pipeline-global runtime-control telemetry.

## Integration With The Engine

The channel is designed to integrate into the engine as a per-node control
channel primitive.

### Intended fit

The natural engine mapping is:

- one control channel per node
- receiver nodes use the receiver-role API
- non-receiver nodes use the node-role API
- control-channel telemetry is attached to the existing control-channel entity

### What the channel should own

The channel should own per-node queue semantics:

- lifecycle acceptance for that node
- bounded admission
- batching
- coalescing
- fairness
- deadline-bounded terminal shutdown

### What the channel should not own

The channel should not own pipeline-wide shutdown orchestration.

That remains an engine concern:

- deciding when receivers should enter `DrainIngress`
- waiting for `ReceiverDrained`
- deciding when downstream nodes should receive `Shutdown`
- reporting pipeline-wide lifecycle events

If the engine adopts this channel, a thin convenience layer such as
`begin_receiver_shutdown(deadline, reason)` should live in the engine, not in
the channel. That helper would:

- accept `DrainIngress` on receiver channels
- track receiver drain completion
- accept downstream `Shutdown` only after receiver drain completes

This keeps the channel focused on per-node semantics while preserving the
engine's receiver-first shutdown contract.

## Current Scope

This crate currently exists as a standalone implementation so its behavior can
be validated before integration, but the design itself is intentionally
long-lived:

- the public API is already role-specific
- the queue semantics are aligned with engine shutdown and fairness needs
- the observability surface is shaped for future engine telemetry integration

The remaining work is not about redefining the model; it is about wiring this
channel into the engine in a controlled way.
