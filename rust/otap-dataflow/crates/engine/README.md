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
gateway's operation and reliability.

## Control Messages

Each node in a pipeline can receive control messages, which must be handled with
priority. These control messages are issued by a control entity (e.g. a pipeline
engine) and are used to orchestrate the behavior of pipeline nodes. For example,
configuring or reconfiguring nodes, coordinating acknowledgment mechanisms,
stopping a pipeline, and more.

## Effect Handlers

### Concept and Purpose

Effect handlers are a core abstraction that provide a clean interface for
pipeline components (receivers, processors, and exporters) to perform side
effects such as sending messages, opening sockets, or managing other resources.
They hide the implementation details of these operations, abstracting away the
specific async runtime and platform details from the implementers of these
nodes.

This abstraction allows the engine to control how these side effects are
actually performed, while providing a consistent interface for component
developers. There are two implementations of effect handlers:

1. **NotSendEffectHandler (!Send)**: The default and preferred implementation.
   These handlers use `Rc` internally for reference counting and work with
   components that don't need to be sent across thread boundaries. This aligns
   with the project's single-threaded, thread-per-core design philosophy.

2. **SendEffectHandler (Send)**: An alternate implementation for components that
   need to integrate with libraries requiring `Send` bounds. These handlers use
   `Arc` internally.

### Why Different Effect Handlers Were Introduced

The dual effect handler approach was introduced to address several challenges:

1. **Abstracting Engine Implementation**: Effect handlers decouple the node
   implementations from the specifics of how messages are sent between nodes,
   allowing the engine to evolve independently from the components.

2. **Library Integration**: Some external libraries (e.g. Tonic) don't yet
   support `?Send` trait declarations
   (see [Tonic issue #2171](https://github.com/hyperium/tonic/issues/2171)).
   The type-level declaration with `SendEffectHandler` provides a pathway to
   integrate such libraries. For nodes that don't need to be `Send`, there's no
   synchronization overhead.

3. **Unified Interface with Type-Level Requirements**: Components parameterize
   their effect handler type in their interface, allowing them to declare their
   specific requirements at the type level while maintaining a consistent API.

### Preferred Implementation Approach

For this project, **`NotSendEffectHandler` is the preferred and recommended
approach** for most components.

`SendEffectHandler` exists primarily as an escape hatch for specific
implementations that must interact with libraries requiring `Send` traits, such
as OTLP Receivers based on Tonic GRPC services.

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
