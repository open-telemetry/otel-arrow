# Proposal: OpenTelemetry-based Tracing for our dataflow engine

## Problem Statement

Modern observability pipelines are increasingly complex. In our system, these
pipelines are defined as Directed Acyclic Graphs (DAGs) of interconnected
processing nodes (receivers, processors, and exporters). In such a system,
diagnosing data loss, performance bottlenecks, misrouted signals, stateful
processing delays, or the impact of control-plane actions becomes challenging.

**Traditional metrics and logs alone are insufficient** for root-cause analysis,
end-to-end flow visibility, and operational insights, especially under dynamic
configuration and sampling regimes.

**Goal**:
Leverage OpenTelemetry tracing primitives (traces, spans, events, links)
to enable robust debugging, troubleshooting, monitoring, and optimization of the
entire dataflow engine, **without imposing excessive overhead or losing critical
information through sampling.**

## System Overview

The dataflow engine is structured as a DAG of the following node types:

- **Receivers**: Ingest and translate data from external sources (may have
  multiple output ports).
- **Processors**: Transform, filter, batch, or enrich data (may have multiple
  output ports).
- **Exporters**: Deliver processed data to external systems.

Each node features:

- A **unique id** and **type** (i.e. urn).
- A **control channel** for orchestration commands (e.g. config_update, ack,
  timer_tick, shutdown).
- A **pdata channel** (processors/exporters) for ingesting batches of signals (
  metrics, logs, events, spans).

**PData messages** are batches containing a single signal type.

## Tracing Modes

- **No Tracing**: No spans emitted.
- **Tail Sampling**: Only selected traces (e.g. errors, random sample) are
  exported, but all nodes are instrumented.
- **Full Tracing**: Every dataflow activity is traced.

> **Metrics are derived from spans before sampling**, ensuring high-fidelity
> monitoring regardless of trace sampling.

## Mapping OTEL Tracing Primitives to the DAG

**Trace**

- Definition: Represents a single PData flow journey through the DAG.
- Purpose: Enables full end-to-end flow and performance visibility.

**Span**

- Definition: Captures a node’s processing of a PData flow.
- Attributes:
    - node.id, node.type, signal.type
    - pdata.batch.size, out.port
    - stateful, latency.ms, error
- Purpose: Provides detailed, per-node insight into processing time, routing,
  state, and errors.

**Span Events**

- Definition: Record significant in-node occurrences.
- Examples:
    - Control message handling (ack, time_tick, shutdown)
    - State transitions (batch full/flush/drop)
    - Output port selection, backpressure, errors

**Span Links**

- Definition: Capture relationships between spans when batches are split (
  fan-out) or merged (fan-in) across nodes.
- Purpose: Enables lineage and parallel flow reconstruction.

**Control Plane Integration**

- Control actions generate events or standalone spans, linked to affected
  data spans as needed, providing operational audit trails.

**Channel Utilization Tracking**

- Utilization Metrics (derived from span events/attributes or direct metrics):
    - Queue depth at receive/send
    - Time spent in input/output channels
    - Rate of message arrival/departure per channel
    - Backpressure events (as span events/attributes)
- Purpose: Highlights overloaded or underutilized channels, informs
  concurrency and buffer size tuning.

## Metrics Extraction

Metrics are derived from traces (pre-sampling) and may include:

- Node and path-level latency, throughput, error rates
- Batch size distributions
- Channel utilization
- Frequency and success of control actions
- Dropped or delayed batches

## Possible Visualization & Analysis

- Trace views show the full DAG traversal for any batch, including processing
  time, routing, control actions, and errors.
- Metrics dashboards visualize node/channel performance, bottlenecks, and the
  operational impact of control actions and configuration changes.

> Idea: A connector like the service graph connector could perhaps be used to
> graphically represent the dataflow.

## Example Trace Structure

```
Trace: batch-1234
  └─ [Span] receiver/otlp/rcv-1
     Events: control/ack, out_port=main, channel_queue_depth=2
     |
     └─ [Span] processor/batch/proc-2
        Events: batch_flush, backpressure, control/timer_tick
        |
        └─ [Span] exporter/http/exp-1
           Events: export_success, control/shutdown
```

Fan-out/fan-in cases are modeled with span links.

## Benefits

- **End-to-end visibility**: Complete flow and lineage for any batch.
- **Root-cause analysis**: Diagnose drops, misconfigurations, and performance
  regressions.
- **Performance tuning**: Optimize batching, buffering, and concurrency using
  trace-derived metrics.
- **Operational insight**: Track the impact and results of control-plane
  actions.

## Implementation Details

**Not yet defined**