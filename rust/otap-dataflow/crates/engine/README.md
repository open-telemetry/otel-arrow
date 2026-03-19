# Pipeline Engine

## Introduction

The `otap-df-engine` crate is the in-core execution engine for OTAP Dataflow.
It is responsible for running pipeline nodes, wiring bounded channels, routing
runtime messages, and enforcing the engine's drain and shutdown behavior inside
one pipeline runtime.

The engine is not the whole process. In this project, the process-level
controller and the in-core engine have distinct responsibilities:

| Component | Role |
|-----------|------|
| Controller | Resolves configuration, allocates cores, creates topic bindings, spawns per-core pipeline runtimes, and drives pipeline lifecycle from admin or process-level events. |
| Engine | Runs receivers, processors, exporters, channels, timers, completion unwinding, and graceful drain/shutdown inside one pipeline runtime on one core. |

This README documents the current `OtapPdata`-based engine used by OTAP
Dataflow. It focuses on the runtime model that contributors and advanced users
need to understand when working on the engine or reasoning about pipeline
behavior.

## Core Concepts

| Term | Meaning |
|------|---------|
| DAG | A directed acyclic graph of nodes and connections describing one pipeline. |
| Pipeline | The configured DAG itself. |
| Pipeline runtime | One instantiated copy of a pipeline running on one assigned core. A pipeline configured on `n` cores produces `n` pipeline runtimes. |
| Receiver / ingress | A node that admits external work into the DAG and produces `pdata` for downstream nodes. |
| Processor | A node that consumes `pdata`, transforms it, and may emit zero, one, or many downstream `pdata` messages. |
| Exporter / egress | A node that consumes `pdata` and terminates the in-process DAG path. |
| `pdata` | The data unit flowing on the forward path. In this project, `pdata` means `OtapPdata`. |
| Ack/Nack / completion | The completion state of an in-flight `pdata` request. Completion traffic travels on the return path and is surfaced as `Ack` or `Nack` control messages. |
| Hyper-edge | A runtime wiring unit that groups compatible logical connections onto one bounded underlying `pdata` channel. |
| Topic | A named in-process transport used to connect pipelines without a direct DAG edge. Topic receiver/exporter nodes bridge between a pipeline runtime and the topic runtime. |

## Architecture

A list of the design principles followed by this project can be found in
[`../../docs/design-principles.md`](../../docs/design-principles.md). More
specifically, the engine implemented in this crate follows a share-nothing,
thread-per-core approach:

- one single-threaded async runtime per assigned core
- one pipeline runtime per deployed pipeline per assigned core
- no implicit work stealing or cross-core scheduling in the hot data path
- bounded channels and explicit backpressure instead of unbounded buffering
- `?Send`-friendly traits so local, `!Send` implementations can avoid
  synchronization when possible
- listener setup that can rely on `SO_REUSEPORT` where the platform supports it

This split matters operationally. The controller decides where and when
pipelines run; the engine decides how a single pipeline runtime behaves once it
is running on a core.

One configured pipeline can therefore produce several independent pipeline
runtimes. Each runtime owns its own nodes, bounded channels, timers, runtime
control state, and completion unwinding state. Immutable resources can be shared,
but hot mutable data-path state stays local to the runtime.

The controller can also assign multiple pipeline runtimes to the same core in
some circumstances. This can happen intentionally when different pipeline
configurations are consolidated onto the same core set, and it can also happen
transiently during rollout-style lifecycle events such as live reconfiguration
when overlapping old and new runtimes are expected to coexist for a period.

### Topics Across Pipelines

Topics are the engine-supported way to connect pipelines without direct DAG
edges. Topic declaration, validation, mode inference, capability checks, and
cycle checks happen during controller startup. Each pipeline runtime then
receives a `TopicSet` with pipeline-scoped bindings, and topic exporter/receiver
nodes bridge between the DAG and the topic runtime. See
[`../../docs/topic-architecture.md`](../../docs/topic-architecture.md) for the
full topic-specific architecture.

Operationally, topics play three main roles in the system:

- they decouple pipelines that together form one larger logical flow, which
  makes it possible to evolve or reconfigure one part of that flow without
  necessarily interrupting another part. A common pattern is to separate an
  ingress-oriented pipeline from a downstream processing-and-export pipeline so
  the latter can change independently while ingress listeners and network
  connections stay stable
- they provide the cross-pipeline delivery pattern required by the topology,
  whether that means balanced delivery, broadcast delivery, or a mixed topic
  serving both kinds of consumers. The example configurations use this for
  scenarios such as multitenant isolation, best-effort tap pipelines, and
  mixed-criticality processing paths
- they can carry tracked publish outcomes across the topic hop, so Ack/Nack
  propagation can be bridged across pipelines when the topic binding and node
  configuration enable it

## OtapPdata

`OtapPdata` is the `pdata` type used by the engine in this project.

At a high level, `OtapPdata` can carry:

- OTLP bytes batches, typically represented as `OtlpProtoBytes`
- OTAP Arrow batches, typically represented as `OtapArrowRecords`

This distinction is important for performance. The engine does not require
every ingress path to deserialize OTLP bytes into the OTAP in-memory
representation immediately. Some receivers, processors, and exporters can route,
forward, inspect, or translate requests while still operating on OTLP bytes.
Conversion to OTAP records happens on demand when a component actually needs the
decoded representation.

That lets the runtime preserve a zero- or low-deserialization path where
possible while still supporting components that need OTAP Arrow batches for
batching, transformation, encoding, or export.

## Runtime Channels and Message Families

The engine uses bounded channels only.

At runtime, compatible DAG connections are grouped into hyper-edges, and each
hyper-edge is wired onto one bounded underlying `pdata` channel. With ordinary
multi-destination `one_of` wiring, multiple consumers on the same hyper-edge
compete on that shared channel. Generic hyper-edge broadcast fan-out is not
implemented at the channel-wiring level, but explicit broadcast-style fan-out
inside a pipeline is supported through the dedicated fanout processor.

Each pipeline runtime uses three channel families:

1. **`pdata` channels**
   These carry only `pdata`. Receivers and processors produce onto them;
   processors and exporters consume them.
2. **Node-control channels**
   There is one bounded node-control channel per node. Receivers consume this
   channel in competition with their external ingress sources and are expected
   to prioritize control. Processors and exporters consume node control together
   with `pdata` through role-specific message channels.
3. **Pipeline runtime channels**
   Each pipeline runtime owns two bounded shared MPSC channels:
   - a **runtime-control channel** for timers, delayed-data requests,
     receiver-drain notifications, and shutdown requests, consumed by
     `RuntimeCtrlMsgManager`
   - a **pipeline-completion channel** for `DeliverAck` / `DeliverNack`, consumed by
     `PipelineCompletionMsgDispatcher`

`ProcessorMessageChannel` and `ExporterMessageChannel` both prefer control over
`pdata`, but neither gives control absolute priority. After a bounded burst of
control messages, the channel forces one `pdata` receive attempt when node-level
admission allows it, so control storms do not starve the forward data path.

`recv_when(...)` is the receive-side primitive that enforces that admission
control. When the guard is `false`, `pdata` stays queued while control messages
continue to be delivered, which lets a node reduce in-flight state and surface
backpressure upstream.

Processors and exporters use that mechanism differently. Processors do not
usually call `recv_when(...)` themselves because the engine owns their receive
loop. Instead, a processor exposes `accept_pdata()`, and the engine feeds that
policy into `ProcessorMessageChannel::recv_when(...)` on the processor's
behalf. Exporters own their run loops directly, so they call
`ExporterMessageChannel::recv()` or `ExporterMessageChannel::recv_when(...)`
themselves. The two mechanisms therefore serve the same admission-control goal
at different layers: `accept_pdata()` is the processor-side readiness hook,
while `recv_when(...)` is the channel primitive used by self-driven exporter
loops.

The shutdown contract is also role-specific. `ProcessorMessageChannel`
continues to honor closed admission during shutdown, so a processor that keeps
`accept_pdata() == false` until the deadline may still strand buffered `pdata`.
`ExporterMessageChannel` is different: once shutdown is latched, it still
force-drains already buffered channel data even if the exporter has temporarily
closed normal admission.

The current message families are:

- **Node control messages**: `Ack`, `Nack`, `Config`, `TimerTick`,
  `CollectTelemetry`, `DelayedData`, `DrainIngress`, `Shutdown`
- **Runtime control messages**: `StartTimer`, `CancelTimer`,
  `StartTelemetryTimer`, `CancelTelemetryTimer`, `DelayData`,
  `ReceiverDrained`, `Shutdown`
- **Pipeline completion messages**: `DeliverAck`, `DeliverNack`

## Runtime Message Dynamics

The runtime has five concurrent message paths that together explain most engine
behavior:

1. **Forward `pdata` flow**
   Receivers admit external work and emit `OtapPdata` on `pdata` channels.
   Processors and exporters then consume that `pdata` through
   `ProcessorMessageChannel` and `ExporterMessageChannel`.

2. **Node-control delivery**
   Receivers consume node control in competition with external ingress. By
   contrast, processors and exporters consume node control and `pdata` through
   their role-specific message channels, which give control preferred but
   bounded-fair treatment.

3. **Runtime-control flow**
   Nodes send timer requests, delayed-data requests, `ReceiverDrained`, and
   runtime `Shutdown` requests to the runtime-control channel. That channel is
   consumed by `RuntimeCtrlMsgManager`, which handles orchestration and turns
   due work back into node-control messages.

4. **Ack/Nack completion flow**
   Nodes that complete or reject work send `DeliverAck` and `DeliverNack` on the
   pipeline-completion channel. `PipelineCompletionMsgDispatcher` consumes that channel,
   unwinds the caller subscription stack stored in `pdata`, and delivers
   `Ack`/`Nack` node-control messages to the closest interested upstream node.

5. **Drain and shutdown flow**
   Graceful shutdown enters through `RuntimeControlMsg::Shutdown`. The runtime
   first sends `DrainIngress` to receivers, waits for `ReceiverDrained`, then
   sends downstream `Shutdown`. Drain propagates as receivers stop admitting new
   work, drop their forward senders, and downstream `pdata` channels gradually
   empty and close. The later sections on Ack/Nack delivery and graceful
   shutdown cover this in more detail.

## Runtime Properties

The runtime is organized around a small set of guarantees:

- **Isolated control paths:** runtime orchestration and Ack/Nack unwinding use
  separate bounded runtime channels and dedicated runtime components.
- **Bounded progress under load:** control remains responsive while `pdata`,
  timer expiry, telemetry collection, and delayed-data resumption still make
  progress under sustained control traffic.
- **Explicit node-level admission control:** processors can temporarily pause
  `pdata` delivery through `accept_pdata()`, and exporters can apply the same
  pattern in their run loops with `ExporterMessageChannel::recv_when(false)`.
  The engine uses two entry points because processor receive loops are
  engine-owned while exporter receive loops are node-owned. During shutdown,
  exporters still drain already buffered channel data, while processors keep
  their current stricter admission semantics.
- **Receiver-first graceful drain:** graceful shutdown drains ingress first,
  then shuts down downstream consumers after receivers report
  `ReceiverDrained`.
- **Producer-visible completion semantics for `wait_for_result`:** receivers
  that expose downstream completion to telemetry producers can only provide a
  useful contract if those producers act on the result. Upstream senders are
  expected to treat temporary failures as retryable and resend with an
  appropriate backoff policy, while permanent refusals should not be retried.
  The engine reports completion; it does not persist or replay those upstream
  requests on the producer's behalf.
- **Bounded memory and surfaced backpressure:** all communication paths remain
  bounded, so pressure appears in the relevant channel family rather than being
  hidden behind unbounded queues.

## Effect Handlers

Effect handlers are the node-facing abstraction for performing engine-managed
side effects. They let receivers, processors, and exporters interact with the
runtime without owning the runtime internals directly.

In practice, effect handlers are how nodes:

- send `pdata` to downstream ports
- subscribe to Ack/Nack interests on the forward path
- emit Ack/Nack outcomes onto the pipeline-completion channel
- schedule or cancel timers on the runtime-control channel
- return delayed data
- report `ReceiverDrained`
- create listeners and sockets with engine-defined socket options

The codebase exposes two families of effect handlers:

1. **Local effect handlers (`!Send`)**
   These are the default and preferred path. They align with the engine's
   single-threaded runtime model and avoid synchronization overhead where the
   component and its futures do not need to be `Send`.

2. **Shared effect handlers (`Send`)**
   These exist for integrations that require `Send`-bound components or futures,
   such as Tonic-based receivers and similar libraries.

The important design point is that the effect handler is not just a message
sender. It is the node's capability object for interacting with the engine's
forward path, completion path, runtime-control path, and listener setup.

## Ack/Nack Delivery Mechanism

The engine includes a built-in mechanism for returning the success or failure of
data requests through the pipeline. Components can opt in to Ack (positive
acknowledgement) or Nack (negative acknowledgement) control messages.

Ack/Nack unwinding is intentionally separated from timer and shutdown
orchestration. Nodes publish return-path events on the pipeline-completion channel,
and `PipelineCompletionMsgDispatcher` unwinds the caller subscription stack and
forwards `Ack` / `Nack` node-control messages to the closest interested
upstream node.

The data layer above the engine is responsible for the physical representation
of request context. The engine effect handler and `OtapPdata` cooperate through
the following calling convention:

1. A producer subscribes to a set of `Interests` before sending `pdata`.
2. Those subscriptions are recorded as frames in the `pdata` context stack.
3. A downstream consumer later calls `notify_ack` or `notify_nack`.
4. The dispatcher unwinds the context stack until it finds the closest frame
   interested in that outcome.

### Interests, Route Data, and Return Data

`Interests` is an 8-bit flag set. The most important flags are:

- `ACKS`
- `NACKS`
- `RETURN_DATA`

Each subscription frame carries:

- the interested node id
- `RouteData` for the forward path
- user-defined `CallData`

When the return path is formed, the engine creates `UnwindData`, which combines
the route metadata with the return-path timestamp.

By default, the payload is dropped on the return path to avoid holding request
memory longer than necessary. If `RETURN_DATA` is set, the caller asks for the
payload to be preserved on the return path. Even then, callers should still
reason in terms of bounded graceful behavior rather than guaranteed payload
recovery under every failure mode.

### Caller Subscription

When a producer wants to be informed of the outcome via Ack/Nack, it uses a
call sequence like:

```rust
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};

async fn process(
    msg: Message<OtapPdata>,
    effect: &mut local::processor::EffectHandler<OtapPdata>,
) -> Result<(), EngineError> {
    match msg {
        Message::PData(mut pdata) => {
            let call_state = SomeCallData::new(...);
            effect.subscribe_to(
                Interests::ACKS | Interests::NACKS,
                call_state.into(),
                &mut pdata,
            );
            effect.send_message(pdata).await
        }
        Message::Control(_) => Ok(()),
    }
}
```

### Caller Return

When a consumer finishes processing `pdata` and wants to return an outcome, it
uses the consumer-side effect-handler extension:

```rust
use otap_df_engine::ConsumerEffectHandlerExtension;

async fn export(
    msg: Message<OtapPdata>,
    effect: &local::exporter::EffectHandler<OtapPdata>,
) -> Result<(), EngineError> {
    match msg {
        Message::PData(pdata) => {
            effect.notify_ack(AckMsg::new(pdata)).await
        }
        Message::Control(_) => Ok(()),
    }
}
```

`notify_nack` works the same way, but returns a `NackMsg`. The engine does not
invent the next unwind frame on its own; the data-layer-aware extension methods
build the next Ack/Nack message using the context stored in `pdata`.

### Handling Ack and Nack Messages

A processor that subscribes to Ack/Nack will typically both consume and produce
Ack/Nack:

```rust
async fn process(
    msg: Message<OtapPdata>,
    effect: &mut local::processor::EffectHandler<OtapPdata>,
) -> Result<(), EngineError> {
    match msg {
        Message::PData(_) => Ok(()),
        Message::Control(ctrl) => match ctrl {
            NodeControlMsg::Ack(ack) => {
                let my_state: SomeCallData = ack.unwind.route.calldata.clone().try_into()?;
                // update local state
                effect.notify_ack(ack).await
            }
            NodeControlMsg::Nack(mut nack) => {
                let my_state: SomeCallData = nack.unwind.route.calldata.clone().try_into()?;
                // update local state
                nack.reason = format!("more info: {}", nack.reason);
                effect.notify_nack(nack).await
            }
            _ => Ok(()),
        },
    }
}
```

Without a matching interest, a node is skipped on the return path. For
topic-specific Ack/Nack bridging across topic hops, see
[`../../docs/topic-architecture.md`](../../docs/topic-architecture.md).

## Graceful Shutdown Sequence

The engine supports graceful shutdown of pipeline runtimes and their nodes.
Shutdown is initiated by sending `RuntimeControlMsg::Shutdown` to the
runtime-control channel; it is not implemented by broadcasting `Shutdown`
directly to every node at once.

In practice, that shutdown request can enter the runtime through:

- direct admin/API shutdown requests handled by the controller
- broader process shutdown flows triggered by OS signals and translated into
  pipeline shutdown requests

When graceful shutdown starts, the runtime control manager:

1. Enters ingress-draining mode.
2. Cancels recurring timers.
3. Flushes queued delayed data back to the originating nodes as
   `NodeControlMsg::DelayedData`.
4. Sends `NodeControlMsg::DrainIngress` to every receiver.

Each receiver is then responsible for stopping admission of new external work
while keeping receiver-local drain state alive long enough to finish local
cleanup. For example, an RPC receiver can keep its wait-for-result registry,
transport shutdown, and telemetry finalization alive until it has no more
admitted requests to resolve.

Once ingress is closed and receiver-local drain work is complete, the receiver
reports `RuntimeControlMsg::ReceiverDrained`.

After all receivers have reported `ReceiverDrained`, the control manager sends
`NodeControlMsg::Shutdown` to processors and exporters. `ProcessorMessageChannel`
continues delivering control messages while only draining `pdata` when the
processor reopens admission. `ExporterMessageChannel` also continues delivering
control messages, but it force-drains already buffered input-channel `pdata`
even if the exporter has temporarily closed normal admission. In both cases,
the channel returns final shutdown once inputs are drained or the shutdown
deadline is reached.

As receivers exit and drop their `pdata` senders, downstream channels drain and
close progressively toward exporters. Once a downstream input is fully drained
or closed, the corresponding consumer receives `Shutdown` and exits its run
loop.

If the shutdown deadline expires, receivers may force-resolve remaining
receiver-local waiters and the runtime control manager forces the remaining
nodes to exit.

When no fatal error occurs and nodes honor the shutdown deadline, this sequence
allows the runtime to stop gracefully while preserving bounded memory and
backpressure semantics.

## Testability

All node types, as well as the pipeline engine itself, are designed for
isolated testing. A receiver, processor, exporter, or runtime component can be
tested without constructing a full deployed pipeline.

The engine provides a substantial `testing` surface, including:

- component wrappers and dedicated test runtimes for receivers, processors, and
  exporters
- helper channels and control-message harnesses
- validation contexts for asserting forward-path and control-path behavior
- single-threaded async test utilities aligned with the engine's local runtime
  model

This keeps runtime behavior testable at the level where bugs usually occur:
channel interaction, Ack/Nack unwinding, runtime-control handling, and
component-local state transitions.

## Deterministic Simulation Testing

In addition to ordinary unit and integration tests, the engine includes a
deterministic simulation testing (DST) layer for concurrency-sensitive runtime
behavior.

The DST approach combines:

- a deterministic clock (`SimClock`) so deadlines, timer expiry, and delayed
  data wakeups can be advanced explicitly
- seeded interleavings so the same scenario can be replayed via `DST_SEED` or
  expanded into a larger sweep via `DST_SEEDS`
- real engine actors and channels rather than parallel mock semantics

This is important for the control plane because many of the interesting failure
classes are about ordering and bounded progress, not only about local business
logic. The DST harness therefore runs real `ProcessorMessageChannel`,
`ExporterMessageChannel`, `RuntimeCtrlMsgManager`, and
`PipelineCompletionMsgDispatcher` logic inside the engine's single-threaded
runtime model.

The current DST scenarios cover five main families:

- role-specific channel fairness and drain behavior: bounded-fair control vs
  `pdata`, processor shutdown draining after admission reopens, exporter
  shutdown draining of buffered `pdata`, and deadline-forced shutdown when a
  processor keeps admission closed
- runtime-control vs pipeline-completion progress under load: timer and delayed
  data delivery, Ack/Nack unwinding, `RETURN_DATA`, and receiver-first shutdown
  ordering under mixed runtime noise
- heavy ingress / backpressure / interblock scenarios: sustained receiver
  traffic, bounded `pdata` channels, processor admission gating and reopen,
  mixed Ack/Nack completions, and clean drain propagation
- explicit known-limitation coverage for processor closed admission through the
  shutdown deadline: the DST suite documents the current case where buffered
  `pdata` remains stranded if a processor never reopens admission before the
  deadline
- receiver `wait_for_result` behavior during drain and shutdown: the OTLP
  receiver DST suite exercises Ack, temporary Nack, permanent Nack, and
  shutdown/unavailable completion at the deadline

The seed controls are:

- `DST_SEED=<u64>` to replay one specific deterministic run
- `DST_SEEDS=<n>` to append `n` generated seeds after the fixed regression
  seeds

The current DST coverage is intentionally scoped. It does not yet model
topic-based routing, cross-pipeline Ack/Nack across topic hops, or fatal
process failure. Those cases still rely on ordinary tests, design review, and
future targeted coverage.

## Compile-Time Plugin System

Receivers, processors, and exporters are registered through a compile-time
plugin system built on [`linkme`](https://docs.rs/linkme). Each node type
publishes a factory under its URN, and the engine's factory tables use those
distributed slices to instantiate nodes at runtime.

This gives the engine a stable runtime lookup model without requiring dynamic
loading. The current system is compile-time only: built-in plugins are
registered into the binary, and the controller/engine use those factory tables
when building pipeline runtimes.

## Telemetry

The engine emits telemetry in terms of process, core, pipeline runtime, and
node identity. These attributes let operators and contributors correlate engine
metrics and events with the runtime structure described above.

### Predefined Attributes

| Scope    | Attribute           | Type    | Description                                                  |
|----------|---------------------|---------|--------------------------------------------------------------|
| Resource | process_instance_id | string  | Unique process instance identifier (base32-encoded UUID v7). |
| Resource | host_id             | string  | Host identifier (e.g. hostname).                             |
| Resource | container_id        | string  | Container identifier (e.g. Docker/containerd container ID).  |
| Engine   | core_id             | integer | Core identifier.                                             |
| Engine   | numa_node_id        | integer | NUMA node identifier.                                        |
| Pipeline | pipeline_id         | string  | Pipeline identifier.                                         |
| Node     | node_id             | string  | Node unique identifier (in scope of the pipeline).           |
| Node     | node_type           | string  | Node type (e.g. "receiver", "processor", "exporter").        |

## Failure Modes and Engine Responses

The engine is designed to address a small number of important runtime failure
classes without pretending that every failure becomes harmless:

- **Bounded memory and explicit backpressure**
  All forward-path and control-path communication remains bounded. Pressure is
  surfaced on the relevant channel family instead of being hidden behind
  unbounded buffering.

- **Separation of orchestration and result traffic**
  Runtime-control work and Ack/Nack unwinding travel on separate runtime paths,
  so completion traffic does not share the same queue as timers, delayed data,
  and receiver-drain coordination.

- **Bounded-fair progress**
  The runtime avoids absolute control priority. Control remains responsive, but
  `pdata`, due timers, telemetry collection, and delayed-data resumption still
  make progress under sustained control load.

- **Receiver-first graceful drain**
  Shutdown begins by stopping ingress rather than abruptly terminating the whole
  DAG. This gives receivers a place to finish admitted work, unwind late
  Ack/Nack outcomes, and then let downstream drain naturally.

- **Explicit drain-time behavior**
  During graceful shutdown, recurring timers are canceled, delayed data is
  returned explicitly, and downstream shutdown is gated by `ReceiverDrained`.

- **Invalid cross-pipeline topic topologies are rejected early**
  Topic declaration and cycle validation happen before runtimes start, so the
  engine does not need to discover topic feedback loops at runtime.

These protections still live within explicit limits:

- graceful shutdown is bounded by a deadline
- fatal process failure bypasses graceful behavior
- correctness depends on nodes honoring the runtime contracts and continuing to
  poll while draining

Those limits are intentional. The engine favors explicit contracts, bounded
resource use, and predictable runtime behavior over hidden retries or
unbounded buffering.
