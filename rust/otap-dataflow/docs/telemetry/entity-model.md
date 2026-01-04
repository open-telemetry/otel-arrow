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

The `channel.id` format depends on the channel kind:
- Control Channel: `{node.id}:control`
- PData Channel: `{source_node.id}:{out_port}`