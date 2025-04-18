# Pipeline Engine

Status: **WIP**

## Introduction

The term pipeline is used here to represent an interconnection of nodes forming
a Directed Acyclic Graph (DAG). The inputs of a pipeline are called receivers,
the intermediate processing nodes are called processors, and the output nodes
are referred to as exporters.

Messages flowing through a pipeline are generically referred to as pdata (i.e.
Pipeline Data). An OTLP message or an OTAP message are examples of pdata types.
This pipeline framework is generic over pdata, which means:

- It is possible to instantiate an OTLP pipeline, an OTAP pipeline, or even a
  pipeline designed to support another type of pdata.
- It is not possible to support multiple pdata types within a single pipeline.
  However, a fourth type of component, called a connector, can be used to bridge
  two pipelines and enable interoperability between different pdata types.

This terminology aligns well with the concepts introduced in the OTEL Go
Collector.

## Architecture

A list of the design principles followed by this project can be found
[here](../../docs/design-principles.md). More specifically, the pipeline engine
implemented in this crate follows a share-nothing, thread-per-core approach.
In particular, one instance of the pipeline engine is created per core. This
engine:

- Is based on a single-threaded async runtime
- Avoids synchronization mechanisms whenever possible
- Declares async traits as `?Send`, providing `!Send` implementations and
  futures whenever practical
- Relies on listening sockets configured with the `SO_REUSEPORT` option,
  allowing the OS to handle connection load balancing
- May share immutable data between cores, but ideally only within a single NUMA
  node

## Control Messages

Each node in a pipeline can receive control messages, which must be handled with
priority. These control messages are issued by a control entity (e.g. a pipeline
engine) and are used to orchestrate the behavior of pipeline nodes. For example,
configuring or reconfiguring nodes, coordinating acknowledgment mechanisms,
stopping a pipeline, and more.

## Testability

All node types, as well as the pipeline engine itself, are designed for isolated
testing. Practically, this means it's possible to test components like an OTLP
Receiver independently, without needing to construct an entire pipeline. This
approach facilitates rapid and precise identification of issues such as memory
overconsumption, bottlenecks, or logical errors within individual nodes.

The engine provides an extensive `testing` module containing utilities tailored
to each pipeline component:

- Defined test message types and control message counters for monitoring
  component behavior.
- Dedicated test contexts and runtimes specifically built for receivers,
  processors, and exporters.
- Single-threaded asynchronous runtime utilities aligned with the engine's
  non-Send design philosophy.
- Convenient helper functions for establishing and managing communication
  channels between components.

These utilities streamline the process of validating individual component
behaviors, significantly reducing setup effort while enabling comprehensive
testing.
