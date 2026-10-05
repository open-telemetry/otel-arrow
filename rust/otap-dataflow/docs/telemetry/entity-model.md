# OTAP dataflow engine entity model

## Introduction

This document describes the entity model used by this project to organize and
categorize the collected telemetry data.

As a reminder, an entity is a stable, identifiable subject of observation to
which telemetry signals relate. OpenTelemetry SDKs and protocols do not define
a first-class Entity object. Instead, entities are implicitly modeled through
sets of attributes. OpenTelemetry Semantic Conventions can be used to define the
entities that exist for a given project. A single signal can involve multiple
entities.

| Concept    | Role                                          |
|------------|-----------------------------------------------|
| Entity     | The observed thing                            |
| Signal     | The observation                               |
| Attributes | Properties describing the entity or the event |

Note: This document will be replaced by formal OpenTelemetry Semantic
Conventions in the future. For now, it serves as an internal reference for the
project.

## Related guides

- Attribute policy: [attributes-guide.md](attributes-guide.md)
- Stability
  rules: [stability-compatibility-guide.md](stability-compatibility-guide.md)
- Implementation status: [implementation-gaps.md](implementation-gaps.md)

## Attribute ownership

- Resource attributes describe the producing service/process/host/container and
  MUST be attached at the resource level.
- Entity attributes identify in-process entities (pipelines, nodes, channels)
  and MUST be stable for the entity lifetime.
- Signal-specific attributes (when used) MUST be bounded and documented
  alongside the signal.

Project-specific entity attributes use the stable namespace of the entity they
describe, such as `pipeline.*`, `node.*`, or `channel.*`. They reuse upstream
OpenTelemetry attributes when the semantics match and do not require a
universal project prefix.

## Project entities

### Service

The logical service representing the OTAP Engine.

Attributes (resource level):

- `service.name`: The name of the service (e.g. "otap_engine").
- `service.instance.id`: A unique identifier for the service instance.

### Host

The physical or virtual machine where the OTAP Engine is running.

Attributes (resource level):

- `host.id`: A unique identifier for the host machine.
- `host.name`: The hostname of the machine.

### Container

The container instance where the OTAP Engine is running (if applicable).

Attributes (resource level):

- `container.id`

### Process

The process instance of the OTAP Engine running on the host or in the container.

Attributes (resource level):

- `process.pid`
- `process.creation.time`

### OTAP execution engine

The OTAP pipeline execution engine running in the process.

Attributes:

- `numa.node.id`: NUMA node identifier.
- `core.id`: Core CPU identifier.
- `thread.id`: Thread identifier.

### Pipeline

A data processing pipeline running within the OTAP Execution Engine.

Attributes:

- `pipeline.group.id`: Pipeline group unique identifier.
- `pipeline.id`: Pipeline unique identifier.

### Node

A processing unit within a pipeline. There are three types of nodes:

- Receiver: Ingests and translates data from external sources
- Processor: Transforms, filters, batches, or enriches data
- Exporter: Delivers processed data to external systems

Attributes:

- `node.id`: Node unique identifier (in scope of the pipeline).
- `node.urn`: Node plugin URN.
- `node.type`: Node type (e.g. "receiver", "processor", "exporter").

The tracing target and exported `InstrumentationScope.name` identify the static
component type that produced an event. They do not identify a configured node
instance. A component scope is derived from its canonical URN by removing the
`urn:` prefix and replacing colons with dots. Component events use the target
convention defined in the
[telemetry crate README](../../crates/telemetry/README.md#logging-macros), while
`node.id` identifies the runtime instance and `node.urn` carries the complete
canonical component identity.

### Channels

Channels connect nodes within a pipeline. There are two types of channels:

- Control Channel: Used for orchestration commands (e.g. config_update, ack,
  timer_tick, shutdown)
- PData Channel: Used for ingesting batches of telemetry signals (metrics, logs,
  events, spans)

Channels are observed via two endpoint perspectives: sender and receiver.

- Sender-side signals attach the sender node identity plus `channel.*`
  attributes.
- Receiver-side signals attach the receiver node identity plus `channel.*`
  attributes.
- `channel.id` connects sender and receiver signals that belong to the same
  channel.

Attributes:

- `channel.id`: Unique channel identifier (in scope of the pipeline).
- `channel.kind`: Channel payload kind ("control" or "pdata").
- `channel.mode`: Concurrency mode of the channel ("local" or "shared").
- `channel.type`: Channel type ("mpsc" or "mpmc").
- `channel.impl`: Channel implementation ("tokio", "flume", "internal").
- `node.port`: Port associated with this channel endpoint.

The `channel.id` format depends on the channel kind:

- Control Channel: `{owner}:control`, where the owner is the node or extension
  that owns the channel.
- PData Channel: `hyperedge:<hash>`, where the stable hash covers the complete
  source, destination, and dispatch-policy signature.

## Stability and identity guarantees

Unless noted otherwise, identifiers are stable for the lifetime of their entity
and may change on restart or reconfiguration.

- `service.instance.id`: Unique per process start (changes on restart).
- `service.name`: Stable per deployment; not guaranteed unique.
- `host.name`: Human-readable hostname; not guaranteed globally unique and may
  change if the host is renamed.
- `container.id`: Stable for the container lifetime.
- `process.pid`, `process.creation.time`: Stable for the process lifetime.
- `numa.node.id`, `core.id`: Stable for a host boot; may change with CPU or NUMA
  reconfiguration.
- `thread.id`: Stable for the thread lifetime; may be reused after thread exit.
- `pipeline.group.id`, `pipeline.id`, `node.id`: Stable across configuration
  reloads; intended to remain consistent for the same logical pipeline graph.
- `channel.id`: Stable across configuration reloads while its owner or complete
  PData hyperedge signature remains unchanged.
- `node.port`: Stable across configuration reloads for a given pipeline graph.

## Entity relationships

Relationships are implicit and expressed through co-located attribute sets on
the same signal. The entity model can be read as a containment chain plus a DAG
of channels.

Containment chain:
Service -> Process -> Execution Engine -> Pipeline Group -> Pipeline -> Node

Channels connect nodes:

- `channel.id` identifies one control-channel owner or one complete PData
  hyperedge; all endpoints of a PData hyperedge share the same `channel.id`.
- Node identity is carried by the `node.*` attributes on each signal.
- Endpoint role is implied by the metric set (e.g. `channel.sender` vs
  `channel.receiver`), not by a channel attribute.
