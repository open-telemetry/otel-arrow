# OTAP Engine - Entity Model

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
| ---------- | --------------------------------------------- |
| Entity     | The observed thing                            |
| Signal     | The observation                               |
| Attributes | Properties describing the entity or the event |

Note: This document will be replaced by formal OpenTelemetry Semantic
Conventions in the future. For now, it serves as an internal reference for the
project.

## Project Entities

### Service

The logical service representing the OTAP Engine.

Attributes (resource level):
- `service.name`: The name of the service (e.g. "otap_engine").
- `service.instance.id`: A unique identifier for the service instance.

Note: `process.instance.id` is currently used in the project but will be
replaced by `service.instance.id` in the future. `service.name` is not yet set
but will be added later.

### Host

The physical or virtual machine where the OTAP Engine is running.

Attributes (resource level):
- `host.name`: The hostname of the machine.

Note: `host.id` is currently used in the project but will be replaced by
`host.name` in the future.

### Container

The container instance where the OTAP Engine is running (if applicable).

Attributes (resource level):
- `container.id`

### Process

The process instance of the OTAP Engine running on the host or in the container.

Attributes (resource level):
- `process.pid`
- `process.creation.time`

### OTAP Execution Engine

The OTAP pipeline execution engine running in the process.

Attributes:
- `numa_node.logical_number` (was named numa.node.id): NUMA node identifier.
- `cpu.logical_number` (was named core.id): Core CPU identifier.
- `thread.id`: Thread identifier.

Note: `numa.node.id` and `core.id` are currently used in the project but will be
replaced by `numa_node.logical_number` and `cpu.logical_number` in the future.
`thread.id` is not yet set but will be added later.

### Pipeline

A data processing pipeline running within the OTAP Execution Engine.

Attributes:
- `pipeline_group.id`: Pipeline group unique identifier.
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

### Channels

Channels connect nodes within a pipeline. There are two types of channels:
- Control Channel: Used for orchestration commands (e.g. config_update, ack,
  timer_tick, shutdown)
- PData Channel: Used for ingesting batches of telemetry signals (metrics, logs,
  events, spans)

Attributes:
- `channel.id`: Unique channel identifier (in scope of the pipeline).
- `channel.kind`: Channel payload kind ("control" or "pdata").
- `channel.mode`: Concurrency mode of the channel ("local" or "shared").
- `channel.type`: Channel type ("mpsc", "mpmc", "spsc", "spmc").
- `channel.impl`: Channel implementation ("tokio", "flume", "internal").
- `channel.sender.out.port`: Output port of the sender node.

The `channel.id` format depends on the channel kind:
- Control Channel: `control:{node.id}`
- PData Channel: `pdata:{source_node.id}:{out_port}`

Notes: `channel.sender.out.port` is not yet set but will be added later. The
`channel.id` format is not yet enforced but will be standardized in the future.

## Stability and Identity Guarantees

Unless noted otherwise, identifiers are stable for the lifetime of their entity
and may change on restart or reconfiguration.

- `service.instance.id`: Unique per process start (changes on restart).
- `service.name`: Stable per deployment; not guaranteed unique.
- `host.name`: Human-readable hostname; not guaranteed globally unique and may
  change if the host is renamed.
- `container.id`: Stable for the container lifetime.
- `process.pid`, `process.creation.time`: Stable for the process lifetime.
- `numa_node.logical_number`, `cpu.logical_number`: Stable for a host boot; may
  change with CPU or NUMA reconfiguration.
- `thread.id`: Stable for the thread lifetime; may be reused after thread exit.
- `pipeline_group.id`, `pipeline.id`, `node.id`: Stable across configuration
  reloads; intended to remain consistent for the same logical pipeline graph.
- `channel.id`: Identifies the source + output port only and is stable across
  configuration reloads as long as the source node id and port are unchanged.
- `channel.sender.out.port`: Stable across configuration reloads for a given
  pipeline graph.

## Entity Relationships

Relationships are implicit and expressed through co-located attribute sets on
the same signal. The entity model can be read as a containment chain plus a DAG
of channels.

Containment chain:
Service -> Process -> Execution Engine -> Pipeline Group -> Pipeline -> Node

Channels connect nodes:
- `channel.id` identifies the source node + output port only; fan-out receivers
  share the same `channel.id`.
- Node identity is carried by the `node.*` attributes on each signal.
- If explicit endpoint roles are needed, add a `channel.endpoint.role` attribute
  (sender|receiver) or derive role from metric names (send/recv).
