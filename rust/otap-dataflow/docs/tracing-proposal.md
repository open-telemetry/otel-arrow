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

Note: In its current state, this document does not describe the propagation of
traces that could be initiated outside the dataflow engine. This topic will need
to be studied further at a later time.

> Note: An RFC on the same topic was issued for the Go Collector. This document
> is a refinement of
>
that [one](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).

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

> Note: Events are technically logs at the protocol level, distinguished by a
> mandatory name. However, at a higher abstraction level (e.g. in semantic
> conventions), they are treated as a separate signal type.

## Tracing Modes

- **No Tracing**: No spans emitted.
- **Tail Sampling**: Only selected traces (e.g. errors, random sample) are
  exported, but all nodes are instrumented.
- **Full Tracing**: Every dataflow activity is traced.

> Note: This list of tracing modes is not exhaustive. Other modes could be
> defined later. Regarding the tail tracing mode, it is worth noting that, as
> defined here, it is possible to envision an extremely efficient implementation
> since we control all the dataflow nodes involved in an end-to-end trace.

**Metrics are derived from spans before sampling**, ensuring high-fidelity
monitoring regardless of trace sampling.

## Mapping OTEL Tracing Primitives to the DAG

### Trace

- Definition: Represents a single PData flow journey through the DAG.
- Purpose: Enables full end-to-end flow and performance visibility.

### Span

- Definition: Captures a node's processing of a PData flow.
- Attributes:
  - otelcol.component.id: the node id.
  - otelcol.component.kind: receiver, processor, or exporter.
  - otelcol.signal: logs, metrics, events, or traces.
  - otelcol.signal.output: logs, metrics, events, or traces.
  - otelcol.pipeline.id: the pipeline id.
  - pdata.batch.size, out.port
  - stateful, error
- Purpose: Provides detailed, per-node insight into processing time, routing,
  state, and errors.

### Events

- Definition: Record significant in-node occurrences.
- Examples:
  - Control message handling (ack, time_tick, shutdown)
  - State transitions (batch full/flush/drop)
  - Output port selection, backpressure, errors

### Span Links

- Definition: Capture relationships between spans when batches are split (
  fan-out) or merged (fan-in) across nodes.
- Purpose: Enables lineage and parallel flow reconstruction.

### Control Plane Integration

- Control actions generate events or standalone spans, linked to affected
  data spans as needed, providing operational audit trails.

### Channel Utilization Tracking

- Utilization Metrics (either maintained as direct metrics or derived from span,
  events/attributes):
  - Queue depth at receive/send
  - Time spent in input/output channels
  - Rate of message arrival/departure per channel
  - Backpressure events (as span events/attributes)
- Purpose: Highlights overloaded or underutilized channels, informs
  concurrency and buffer size tuning.

## Metrics

Metrics are either maintained individually or derived from traces (pre-sampling)
and may include:

- Node and path-level latency, throughput, error rates
- Batch size distributions
- Channel utilization
- Frequency and success of control actions
- Dropped or delayed batches

The following attribute conventions are proposed for metrics:

- otelcol.component.outcome: success, failure, refused
- more to come...

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
  +- [Span] receiver/otlp/rcv-1
     Events: control/ack, out_port=main, channel_queue_depth=2
     |
     +- [Span] processor/batch/proc-2
        Events: batch_flush, backpressure, control/timer_tick
        |
        +- [Span] exporter/http/exp-1
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