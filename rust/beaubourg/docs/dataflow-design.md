# Dataflow Mechanics

## Introduction

This document describes the various components and operational mechanics of the dataflow runtime used in Phase 2 of this
project. We aim to unify the pipeline and connector concepts into a single dataflow concept consisting exclusively of
receivers, processors, and exporters. This design will:

1. Support a standard OpenTelemetry pipeline configuration with or without connectors (via an adaption layer).
2. Support more complex scenarios in a simpler and more uniform manner.

## Telemetry Signal Types and Streams

The telemetry signals supported by this dataflow runtime adheres to OpenTelemetry’s defined data model, encompassing:

- Metrics Stream (denoted as `M` in this document).
- Logs Stream `L`
- Traces Stream `T`
- Events Stream (`E` - typically embedded within Logs or Traces)

Throughout this document, the following notation will be used consistently:

- `A`: Represents a stream containing any combination of telemetry signal types (Metrics, Logs, Traces, Events).
  Example: `A = {M, L, T, E}` or any subset of this set.
- `M | L`: Denotes a single stream containing a mixture of Metrics and Logs.
- `M & L`: Represents two separate, parallel streams: one for Metrics and one for Logs.

These streams are composed of messages containing two parts: an envelope part and a data batch part. The envelope part
includes headers that characterize the message without interpreting the data batches themselves. These headers can
either be received externally (e.g., authorization tokens) or injected by components within the dataflow (e.g., deadline
header). The headers may carry metadata utilized by downstream components for processing decisions.

An unique ID is assigned to each incoming message, allowing the dataflow runtime to track the message throughout the
dataflow. This ID is used to correlate messages across different telemetry streams and to manage acknowledgments.

`Msg = {ID, Envelope, OTAP Data}`

## Control Signal Types and Propagation

A dataflow runtime utilizes internal control signals to manage system operations and enforce delivery, latency, or
resource constraints. The following signals are defined:

- **Acknowledgement Signal** (`ACK`): Indicates external systems have reliably received telemetry data.
- **Health Check Signal** (`HCS`): Indicates the operational status (Up/Down) of external dependencies (e.g., exporter
  backends).
- **Deadline Signal** (`DDL`): Emitted when a configured time limit for task completion (e.g., data storage) is exceeded.
- **Resource Budget Signal** (`REB`): Indicates the system’s capacity to accept additional telemetry data. A `REB` value
  of zero signifies no further acceptance, while a non-zero `REB` defines permissible data acceptance conditions (e.g.,
  message size, rate limits).
- **Timer Signal** (`TMR`): Emitted upon timer expiration, used to trigger scheduled tasks (e.g., batch emissions).
- **Error Signal** (`ERR`): Represents errors encountered by the dataflow components.
- **Configuration** Update Signal (`CFG`): Indicates a change in the configuration of a component.
- **Shutdown Signal** (`KIL`): Indicates the system is shutting down.

The control signals are summarized with the abbreviation `CTRL`. `CRTL` can be any of the control signals listed above.

### Reverse Propagation Mechanism

Components within the dataflow may subscribe to these signals to trigger specific behaviors or policies. When multiple
components subscribe to a single control signal, the dataflow runtime employs a **reverse propagation mechanism**. This
mechanism propagates signals starting from components nearest to the signal’s origin in the dataflow graph, moving
backward toward the receivers. Each triggered component may decide whether to propagate the signal further upstream or
terminate propagation at its level.

## Dataflow Components

Dataflows are represented as **Directed Acyclic Graphs** (DAGs) composed of interconnected nodes. Each node accepts 0 to n
input streams and produces 0 to m output streams. Dataflows are free from cycles.

Nodes are categorized into three types:

- **Receivers** (sources): Nodes interfacing dataflow runtime with external telemetry sources. All receivers must support
  handling of `REB` signals. Receivers are expected to reduce or halt acceptance of telemetry data when `REB` indicates
  insufficient resources. Example of receiver signatures:
  - Receiver producing any signal type: `[Receiver-ID] → A` (e.g., OTLP receiver). 
  - Receiver producing only metrics: `[Receiver-ID] → M` (e.g., Prometheus receiver).
- **Processors**: Nodes performing intermediate transformations, routing, filtering, or enrichment. Example signatures:
  - General-purpose processor: `A → [Processor-ID] → A`.
  - Metrics-filtering processor: `A → [Processor-ID] → M`.
  - Type-based router: `A → [Processor-ID] → M & L`.
- **Exporters** (sinks): Nodes interfacing the dataflow runtime with external data consumers or storage systems. Example
  signatures:
  - Any signal type: `A → [Exporter-ID]`.
  - Metrics only: `M → [Exporter-ID]`.

The component signatures provide a quick overview of the regular component’s input and output streams. However, all
dataflow components (receiver, processor, exporter) can receive, produce, or propagate control signals. Those control
signals are not part of the component signature to keep the notation simple.

### Processors

Three categories of processors have been identified so far.

#### Routing Processors

- **Type Router** (`TR`): Routes telemetry data based on signal type (Metrics, Logs, Traces).
  - Statefulness: Stateless
  - Default destination: Optional, handles signals not matching explicit routing rules.
- **Content Router** (`CR`): Routes telemetry data based on defined content conditions (e.g., attribute matching).
  - Statefulness: Stateless
  - Default destination: Optional.
- **Failover Router** (`FR`): Routes telemetry data to alternative destinations when primary paths fail or exceed deadlines.
  - Statefulness: Stateful; retains unacknowledged data until acknowledged or timeout.
  - Subscribed Signals: `ACK`, `HCS`, `DDL`
  - Propagated Signals: TBD.

#### Control Processors

- **Admission Controller** (`AC`): Determines acceptance of telemetry data based on resource availability or other conditions
  at receiver points. Propagates REB signals upstream to receivers, affecting data acceptance decisions.
- **Deadline Controller** (`DC`): Assigns deadlines to telemetry messages using defined policies. Deadlines are added as
  attributes ("deadline attribute envelope") within the telemetry data.
  - Subscribed Signals: REB
  - Propagated Signals: REB
- **Ack Controller** (`AK`): Determines the point of acknowledgment in dataflows:
  - Ingress: Immediate acknowledgment upon reception.
  - Pre-export: Acknowledgment just before exporting.
  - Downstream: Acknowledgment only after confirmed reception by the external system.
- **Batch Processor** (`BP`): Manages batching of telemetry signals, defining maximum batch size and timeout intervals for
  emission.

#### Data Processors

These processors directly modify the content of telemetry data. They can filter, aggregate, enrich, convert, ... The 
four main data processors are:
- **Filter Processor** (`FP`): A processor that allows only telemetry data meeting specific conditions to pass through. It is
  important to distinguish between a Content Router (CR) and a Filter Processor (FP):
  - A CR directs data to different destinations.
  - An FP discards data that does not meet the specified conditions.
- **Aggregation Processor** (`AP`): A processor that aggregates metrics data based on certain criteria.
- **Sampling Processor** (`SP`): A processor that preserves a subset of input data based on a sampling strategy.
- **Converter Processor** (`CP`): A processor that transforms telemetry data from one type to another.

## Dataflow Optimizations

### Pruning Rules

Receivers or exporters without at least one active connected path are considered inactive. During the compilation of the
dataflow, inactive nodes are automatically removed from the DAG.

### Processor Chain

Processors forming a chain without intermediate branches can be logically grouped together to form an aggregated
processor. The dataflow runtime is free to optimize such chains, for example, by removing intermediate channels.

> More optimization to come.