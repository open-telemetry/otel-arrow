# Control-Aware Bounded Channel

This crate defines a bounded control channel for OTAP dataflow node-control
traffic.

Its purpose is to provide a control-plane primitive with semantics that are
stronger and more explicit than a generic FIFO MPSC queue:

- lifecycle control such as `DrainIngress` and `Shutdown` must still be
  accepted and delivered even when ordinary control traffic is backlogged
- high-frequency completion traffic must remain efficient
- low-value control noise should be coalesced instead of competing with
  correctness-critical work
- shutdown progress must remain bounded and explicit
- the public API should make invalid lifecycle operations hard to express

Here, "lifecycle control" means the per-node shutdown transitions carried by
`DrainIngress` and `Shutdown`, as opposed to ordinary control traffic such as
`Ack`, `Nack`, `Config`, `TimerTick`, and `CollectTelemetry`.

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
- best-effort coalesced control work, where duplicate signals collapse into one
  pending token
- latest-wins configuration updates, where the newest pending value replaces
  the older one
- reserved lifecycle tokens

## Design

The channel uses one internal queue implementation with role-specific public
APIs.

### Operational overview

```text
receiver sender                                  receiver-side delivery
---------------                                  ----------------------
accept_drain_ingress() ----------------------->  DrainIngress
accept_shutdown(deadline) -------------------->  Shutdown
try_send/send(Ack | Nack) -------------------->  CompletionBatch(...)
try_send/send(Config) ------------------------>  Config
try_send/send(TimerTick) --------------------->  TimerTick
try_send/send(CollectTelemetry) -------------->  CollectTelemetry

                     +------------------------------------------------+
                     |              Control channel core              |
                     |------------------------------------------------|
                     | lifecycle slots: drain_ingress, shutdown       |
                     | retained queue: completion deque               |
                     | latest-wins slot: config                       |
                     | coalesced flags: timer_tick, collect_telemetry |
                     |------------------------------------------------|
                     | phases: Normal                                 |
                     |         IngressDrainRecorded                   |
                     |         ShutdownRecorded                       |
                     |------------------------------------------------|
                     | stats / metrics surface:                       |
                     |   completion_len                               |
                     |   completion_batch_emitted_total               |
                     |   completion_message_emitted_total             |
                     |   config_replaced_total                        |
                     |   timer_tick_coalesced_total                   |
                     |   collect_telemetry_coalesced_total            |
                     |   normal_event_dropped_during_drain_total      |
                     |   shutdown_forced                              |
                     +------------------------------------------------+
```

The sender side submits lifecycle operations and ordinary control commands into
separate internal classes. The receiver side observes normalized control events
whose ordering is governed by the channel's fairness and shutdown policy.

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
- latest-wins normal control, where a new pending value replaces the previous
  one:
  - `Config`
- coalesced best-effort normal control, where duplicate signals merge into one
  pending token:
  - `TimerTick`
  - `CollectTelemetry`

### Internal model

Internally, that queue implementation stores control classes in separate
slots/queues instead of forcing everything through one FIFO:

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

### Future admin UI work

Once the control channel is integrated into the engine and starts emitting a
dedicated `channel.control` metric set, the admin UI should be updated in a
separate change.

The intended UI changes are:

- graph model:
  - associate `channel.control` metric sets with existing control-channel edges
    using `channel.id`
- selection details:
  - render a control-specific metrics block for control channels
  - use `completion.queued` as the queue-depth fallback when generic
    `queue.depth` is not available
- optional charting:
  - add selected-channel views for control-specific gauges and counters only
    after the emitted metric names are stable

Those updates are intentionally deferred until integration time so this branch
stays focused on the standalone channel design rather than on dormant UI code.

## Future Work And Integration Plan

The design is intended to stay stable while the remaining work focuses on
integration and operationalization.

The main follow-up work is:

- integrate this channel into the engine as the node-control channel
  implementation for receiver and non-receiver nodes
- add engine-side wiring for the `channel.control` metric set derived from the
  existing `stats()` surface
- add the deferred admin UI work described above once those metrics are emitted
- validate behavior and performance again after integration under realistic
  engine workloads

The crate remains standalone until that integration work happens, but the goal
is not to keep a separate model forever. The goal is to use this crate to pin
down the channel semantics first, then wire those semantics into the engine with
minimal ambiguity.
