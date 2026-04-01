# Control-Aware Bounded Channel

This crate defines a bounded control channel for OTAP dataflow node-control
traffic.

Its purpose is to provide a control-plane primitive with semantics that are
stronger and more explicit than a generic FIFO MPSC queue:

- lifecycle control such as `DrainIngress` and `Shutdown` must still be
  accepted and delivered even when ordinary control traffic is backlogged
- high-frequency completion traffic must remain efficient through bounded batching
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
- frequent `Ack` and `Nack` traffic when `wait_for_result` is enabled,
  with batching to amortize receive-side overhead
- node-local lifecycle transitions such as `DrainIngress` and `Shutdown`
- a need for bounded memory, bounded-fairness, and explicit termination progress

The design separates policy classes that behave differently:

- retained, backpressured, and batched completion traffic
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

There are two channel families used between the engine controller and the individual nodes. Receivers have a different interface than the other node types, because they begin the shutdown sequence.

- For sending messages to receivers, a controller uses `receiver_channel(...)`
  - channel type: `ReceiverControlSender`, `ReceiverControlReceiver`
  - receiver event type: `ReceiverControlEvent`
  - channel receiver supports both `accept_drain_ingress(...)` and `accept_shutdown(...)`
- For sending messages to other nodes, a controller uses `node_channel(...)`
  - channel type: `NodeControlSender`, `NodeControlReceiver`
  - receiver event type: `NodeControlEvent`
  - channel receiver supports `accept_shutdown(...)` only

This split is intentional. `DrainIngress` is a receiver-specific lifecycle
control, so non-receiver nodes cannot express it through the public channel receiver API.

The crate exposes a single-owner implementation for use by a single engine core. 
This design fits our thread-per-core execution model and keeps the control
state machine behind one receiver-owned or node-owned **queue core**.

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

### Backpressure vs non-blocking send

For non-lifecycle traffic, the sender exposes two modes:

- `try_send(...)`
  - non-blocking
  - returns the original command on `Full` or `Closed`
- `send(...).await`
  - waits for bounded capacity when needed
  - returns only when the command is accepted or the channel closes
  - wakes blocked completion senders in FIFO order as completion capacity is released

This supports both explicit backpressure and opportunistic best-effort usage,
depending on the caller.

### Completion batching

`Ack` and `Nack` are retained in a bounded FIFO and emitted as
`CompletionBatch(Vec<CompletionMsg<_>>)`.

Properties:

- arrival order is preserved within completion traffic
- batching reduces receive-side overhead under heavy completion load
- protocols that support completion signals such as `Ack` and `Nack` can take
  advantage of this batching to reduce control-plane churn
- completion messages can optionally carry explicit metadata through
  `AckMsg<PData, Meta>` / `NackMsg<PData, Meta>`; standalone usage defaults to
  `Meta = ()`, while future engine integration can preserve unwind state there
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
- once drain begins, ordinary non-completion control work such as `Config`,
  `TimerTick`, and `CollectTelemetry` is no longer accepted
- the `DrainIngress` deadline is carried to the receiver event loop so the
  receiver can bound its own ingress-drain phase; it does not make the
  control-channel queue itself deadline-driven
- once shutdown begins, ordinary non-completion control work such as `Config`,
  `TimerTick`, and `CollectTelemetry` is no longer accepted
- pending ordinary non-completion control state is cleared when drain or
  shutdown is accepted
- completion traffic may continue draining after shutdown is accepted

### Deadline-bounded terminal progress

Only `Shutdown` carries an active queue-level deadline. `DrainIngress` may also
carry a deadline field, but that field is for receiver-local ingress-drain
behavior after delivery rather than for forced progress inside the control
channel itself.

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

### Single-owner execution model

The channel is implemented as a single-owner state machine:

- the queue core is mutated only by the channel owner
- sender clones share that owner through local single-threaded handles
- blocked completion senders wait in a keyed FIFO waiter queue so
  capacity release can wake only the senders that can now make progress
- receiver waiting remains deadline-aware so forced shutdown does not
  depend on a later producer wakeup

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

## Future Work

### Engine integration

The intended engine integration is:

- one control channel per node
- receiver nodes use the receiver-role API
- non-receiver nodes use the node-role API
- control-channel telemetry is reported as `channel.control` from the existing
  `stats()` surface

Pipeline-wide shutdown orchestration should remain in the engine, not in the
channel. In particular, an engine-side helper such as
`begin_receiver_shutdown(deadline, reason)` should:

- accept `DrainIngress` on receiver channels
- wait for `ReceiverDrained`
- accept downstream `Shutdown` only after receiver drain completes

After integration, behavior and performance should be revalidated under
realistic engine workloads.

### Admin UI

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
