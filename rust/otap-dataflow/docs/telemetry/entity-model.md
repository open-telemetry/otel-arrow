# OTAP Dataflow Engine - Entity Model

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

## Attribute Ownership

- Resource attributes describe the producing service/process/host/container and
  MUST be attached at the resource level.
- Entity attributes identify in-process entities (pipelines, nodes, channels)
  and MUST be stable for the entity lifetime.
- Signal-specific attributes (when used) MUST be bounded and documented
  alongside the signal.

Project-specific entity attributes use the `otelcol.*` prefix to avoid
collisions with existing and future semantic conventions.

## Project Entities

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

### OTAP Execution Engine

The OTAP pipeline execution engine running in the process.

Attributes:

- `otelcol.numa_node.logical_number`: NUMA node identifier.
- `cpu.logical_number` (was named core.id): Core CPU identifier.
- `thread.id`: Thread identifier.

### Pipeline

A data processing pipeline running within the OTAP Execution Engine.

Attributes:

- `otelcol.pipeline_group.id`: Pipeline group unique identifier.
- `otelcol.pipeline.id`: Pipeline unique identifier.

### Node

A processing unit within a pipeline. There are three types of nodes:

- Receiver: Ingests and translates data from external sources
- Processor: Transforms, filters, batches, or enriches data
- Exporter: Delivers processed data to external systems

Attributes:

- `otelcol.node.id`: Node unique identifier (in scope of the pipeline).
- `otelcol.node.urn`: Node plugin URN.
- `otelcol.node.type`: Node type (e.g. "receiver", "processor", "exporter").

### Channels

Channels connect nodes within a pipeline. There are two types of channels:

- Control Channel: Used for orchestration commands (e.g. config_update, ack,
  timer_tick, shutdown)
- PData Channel: Used for ingesting batches of telemetry signals (metrics, logs,
  events, spans)

Channels are observed via two endpoint perspectives: sender and receiver.

- Sender-side signals attach the sender node identity plus `otelcol.channel.*`
  attributes.
- Receiver-side signals attach the receiver node identity plus
  `otelcol.channel.*` attributes.
- `otelcol.channel.id` connects sender and receiver signals that belong to the
  same channel.

Attributes:

- `otelcol.channel.id`: Unique channel identifier (in scope of the pipeline).
- `otelcol.channel.kind`: Channel payload kind ("control" or "pdata").
- `otelcol.channel.mode`: Concurrency mode of the channel ("local" or "shared").
- `otelcol.channel.type`: Channel type ("mpsc", "mpmc", "spsc", "spmc").
- `otelcol.channel.impl`: Channel implementation ("tokio", "flume", "internal").
- `otelcol.channel.sender.out.port`: Output port of the sender node.

The `otelcol.channel.id` format depends on the channel kind:

- Control Channel: `control:{node.id}`
- PData Channel: `pdata:{source_node.id}:{out_port}`

## Stability and Identity Guarantees

Unless noted otherwise, identifiers are stable for the lifetime of their entity
and may change on restart or reconfiguration.

- `service.instance.id`: Unique per process start (changes on restart).
- `service.name`: Stable per deployment; not guaranteed unique.
- `host.name`: Human-readable hostname; not guaranteed globally unique and may
  change if the host is renamed.
- `container.id`: Stable for the container lifetime.
- `process.pid`, `process.creation.time`: Stable for the process lifetime.
- `otelcol.numa_node.logical_number`, `cpu.logical_number`: Stable for a host
  boot; may change with CPU or NUMA reconfiguration.
- `thread.id`: Stable for the thread lifetime; may be reused after thread exit.
- `otelcol.pipeline_group.id`, `otelcol.pipeline.id`, `otelcol.node.id`: Stable
  across configuration reloads; intended to remain consistent for the same
  logical pipeline graph.
- `otelcol.channel.id`: Identifies the source + output port only and is stable
  across configuration reloads as long as the source node id and port are
  unchanged.
- `otelcol.channel.sender.out.port`: Stable across configuration reloads for a
  given pipeline graph.

## Entity Relationships

Relationships are implicit and expressed through co-located attribute sets on
the same signal. The entity model can be read as a containment chain plus a DAG
of channels.

Containment chain:
Service -> Process -> Execution Engine -> Pipeline Group -> Pipeline -> Node

Channels connect nodes:

- `otelcol.channel.id` identifies the source node + output port only; fan-out
  receivers share the same `otelcol.channel.id`.
- Node identity is carried by the `otelcol.node.*` attributes on each signal.
- Endpoint role is implied by the metric set (e.g. `channel.sender` vs
  `channel.receiver`), not by a channel attribute.
