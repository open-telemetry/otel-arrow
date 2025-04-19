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

These design principles focus on achieving high performance, predictability, and
maintainability in an observability gateway implemented in Rust. Targeting a
single-threaded async runtime reduces complexity, enhances cache locality, and
lowers overhead. Favoring `!Send` futures and declaring async traits as `?Send`
maximizes flexibility and allows performance gains by avoiding unnecessary
synchronization. Minimizing synchronization primitives prevents contention and
overhead, thus ensuring consistently low latency. Avoiding unbounded channels
and data structures protects against unpredictable resource consumption,
maintaining stable performance. Finally, limiting external dependencies reduces
complexity, security risks, and maintenance effort, further streamlining the
gateway’s operation and reliability.

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

## ThreadMode

### Concept and Purpose

The `ThreadMode` trait is a core abstraction that defines the thread-safety
behavior for pipeline components (receivers, processors, and exporters). It
allows components to specify whether they can be safely sent across thread
boundaries by implementing one of two modes:

1. **LocalMode (!Send)**: Thread-local components that are restricted to the
   thread they were created on. These components use `Rc` internally for
   reference counting and can leverage non-`Send` dependencies.

2. **SendableMode (Send)**: Thread-safe components that can be sent across
   thread boundaries. These components use `Arc` internally and require all
   state to be `Send + Sync`.

### Why ThreadMode Was Introduced

The ThreadMode abstraction was introduced to address several challenges:

1. **Balancing Performance and Flexibility**: The engine is designed to be
   high-performance using a share-nothing, thread-per-core approach with a
   single-threaded async runtime. This works best with `!Send` components that
   don't require synchronization overhead. However, some components (like those
   based on Tonic GRPC services) require `Send` traits.

2. **Library Integration**: Some external libraries (e.g. Tonic) don't yet
   support `?Send` trait declarations (
   see [Tonic issue #2171](https://github.com/hyperium/tonic/issues/2171)).
   ThreadMode provides a pathway to integrate such libraries.

3. **Unified Interface**: By parameterizing components with their threading
   behavior, we maintain a consistent API across all components while allowing
   them to declare their thread-safety requirements.

### Preferred Implementation Approach

For this project, **`LocalMode` is the preferred and recommended approach** for
most components:

- **Receivers**: Implement using `LocalMode` unless integrating with libraries
  that require `Send` traits.
- **Processors**: Use `LocalMode` for optimal performance with the
  single-threaded runtime.
- **Exporters**: Similarly, prefer `LocalMode` unless specific external
  integration requires `Send`.

`SendableMode` exists primarily as an escape hatch for specific implementations
that cannot use `LocalMode`, such as OTLP Receivers based on Tonic GRPC
services.

### Running Both !Send and Send Nodes in a Single Runtime

One of the key benefits of the ThreadMode abstraction is that it allows a
single-threaded async runtime to seamlessly run both `!Send` and `Send` nodes:

- **Uniform API**: All components implement the same traits and interfaces
  regardless of their thread-safety characteristics.
- **Dynamic Dispatch**: The pipeline engine can treat all nodes uniformly at
  runtime, dispatching messages to them via their effect handlers.
- **Zero-Cost Abstraction**: For `LocalMode` components, there's no
  synchronization overhead.
- **Future Compatibility**: As libraries evolve to support `?Send`, components
  can be migrated to `LocalMode` without changing the API.

This flexibility enables the pipeline to maintain high performance for most
components while accommodating special cases that require thread-safety
guarantees, all within a single cohesive architecture.
