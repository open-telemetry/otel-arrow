## OTel-Arrow Phase-2 Design

This is a blueprint for an end-to-end OTel-Arrow pipeline written in
Rust. We take this as an opportunity to keep what we like best about
OpenTelemetry pipelines in Golang and to redesign those aspects that
we find could be improved for performance, reliability,
serviceability, and other concerns.

### Choice of Apache Arrow

The Apache Arrow project is a widespread standard for efficient
column-oriented data processing. Like OpenTelemetry, it has a language-neutral
specification and compatible implementations in most programming
languages. Arrow enables the use of memory-sharing across process
boundaries and supports access to structured data without copying
through the use of flatbuffers and position-relative offsets.

To achieve a high-performance telemetry pipeline, we believe that
a column-oriented approach is necessary. Arrow is the obvious solution
for this in the open-source space.

### Choice of Rust

The Rust programming language is rapidly becoming dominant in the
same industries where Arrow has already taken hold. The `arrow-rs`
crates are widely used to exchange data across API boundaries for
databases, analysis engines, and storage engines. It is no accident
that Rust and Arrow go hand-in-hand. Data-intensive applications
demand control over memory allocation and synchronization in ways
that are difficult to achieve in garbage-collected languages, and
Rust provides the required low-level facilities with unrivaled safety.

Rust enables the kind of high-performance data processing that Arrow
provides, and these two technologies have become self-reinforcing. The
Rust/Arrow ecosystem is strong, and in particular, we are interested
in applying the DataFusion query engine towards OpenTelemetry data. The
potential here is great, both to create more efficient telemetry processors
and to create new applications inside OpenTelemetry. Therefore, we choose
Rust as the basis for an end-to-end OTel-Arrow pipeline.

## Essential Components

With the goal to create an end-to-end [OTAP][] pipeline covering a range 
of standard pipeline behaviors, here are the set of core features we
believe are needed. We follow telemetry pipeline terminology developed
in the OpenTelemetry Collector.

[OTAP]: ../README.md

### OTAP Receiver

As the single built-in receiver, the OTAP receiver accepts the OTAP
protocol as developed during OTel-Arrow Phase 1. The analogous Golang
component is our [`otelarrowreceiver`](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md),
which receives gRPC streams conveying the OTAP protocol. Note, however,
this receiver will not directly support OTLP/gRPC the way `otelarrowreceiver`
does, nor will it support OTLP/HTTP during this stage of our investigation.

Note that in general, conversion from OTLP to OTAP protocols requires
a substantial amount of logic, therefore we will continue to use the
Golang implementation as required to convert OTLP to OTAP. This
"adapter" component computes a dynamic Arrow schema from the arriving
stream of structured but not strongly-typed data. Since we are focused on
an end-to-end OTel-Arrow pipeline, we are not concerned with directly
receiving OTLP data at this stage.

The essential features of the OTAP receiver will be:

- Decompression with Zstd codec
- Admission control limits (e.g., in-flight request size, waiting request size)
- Related-data size limits (e.g., total memory used for stream state)
- Network-level settings (e.g., TLS, keepalive)

### OTAP Exporter

Because the underlying pipeline data representation will be OTAP data
frames, the OTAP exporter is not required to perform a substantial
data translation, the way the Golang [`otelarrowexporter`](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md)
is required to do. Like the OTAP exporter described above, this component
will not directly send telemetry using the OTLP/gRPC or OTLP/HTTP protocols.

The essential features of the OTAP exporter are:

- Compression with Zstd codec
- Network-level settings (e.g., resolver, balancer)
- Configurable number of concurrent streams

### Pipeline Construction Kit

A pipeline is required to support a flexible and configurable arrangement
of components, and it is responsible for constructing the component graph at runtime. The
OpenTelemetry Collector component model, including receiver, processor, exporter,
connector, and extension components, will be followed. We expect to follow the Collector's
configuration model for the pipeline, for example with `service::pipelines` and
`service::telemetry` sections.

Pipeline construction includes a number of common supports, including retry and
back-off, timeout and cancellation features, and self-observability.

During pipeline construction, a number of built-in accessories are required
including fan-in and fan-out operations, failover, and routing connector
support, and no-op terminals.

### Batch Processor

A batch processor performs an essential form of regulation in a
pipeline by limiting batch sizes in both directions.  The batch
processor can be configured to combine requests or split requests to
maintain minimum and maximum bounds on the number of items or the size
of the encoding.

The essential features of the batch processor are:

- Support assembling minimum and maximum size requests
- Support input request cancellation
- Support deadline propagation
- Do not restrict concurrency
- Link diagnostic contexts
- Error transmission.

The OTAP protocol computes a dedicated schema for every distinct
stream. The central challenge for the batching processor, aside from
the responsibilities listed above, is to combine OTAP streams. This
is a central area for investigation.

### Queue Processor

The Queue Processor's primary goal is to quickly write requests into
storage, typically (but not always) a persistent volume. The Queue Processor
allows the pipeline to return to the caller quickly, as a way to avoid
dropping data when the backend is unavailable. While there is storage available
in the queue, we expect incoming requests to be limited by storage throughput,
as opposed to by pipeline latency.

Apache Arrow includes a format for [Serialization and Inter-process Communication](https://arrow.apache.org/docs/format/Columnar.html#format-ipc)"
as a chapter in its columnar data specification. The Arrow IPC format
defines a one-way stream of Arrow record batches, with conventions for
interleaving schema, metadata, dictionary, and data frames. This format
is the basis of the OTAP protocol, therefore the pipeline data for a
single OTAP stream can be easily and naturally written to a file
using a writer method in the `arrow-rs` crate. We will adopt this format
in the OTel-Arrow Queue Processor.

Likewise, a reader method in the `arrow-rs` crate will support replay
of OTAP streams from the queue processor. Otherwise, the essential
features of the queue processor are:

- Support input request cancellation
- Support in-memory or persistent queue
- Recognize request timeout.

## Essential Features

The OTel-Arrow end-to-end pipeline prioritizes security and reliability, starting
with the choice of a memory-safe, non-garbage-collected language. Here
other ways we prioritize these qualities.

### Managing memory

Whereas the Golang pipeline has several approaches to limiting memory,
including a `memorylimiterprocessor` and `memorylimiterextension`. Aside from
being not fine-grained enough, practically, we observe problems with that approach:

- `memorylimiterprocessor`: a processor is "too late" in the pipeline to
effectively restrict memory usage, because the memory has to be allocated in
order to reach the processor
- `memorylimiterextension`: The approach is based on garbage collection statistics,
which are a lagging indicator, a noisy one, and also prone to error (e.g., this component
responds to memory leaks the same as it does to overload).

The [OTel-Arrow Receiver component configures admission control][ADMISSION]
via two parameters, `request_limit_mib` and `waiting_limit_mib` which govern the amount
of data (measured in bytes) pending in a call to `Consume()` in the pipeline
or waiting in the receiver, across active steams. In addition, there are auxiliary
per-stream limits on Arrow stream state and compression-related memory (e.g., `memory_limit_mib`).

OTel-Arrow Rust pipeline receivers will implement explicit admission control,
along these lines, as a mandatory feature instead of an optional one. In general,
all large users of memory, not only receivers, should account for allocations
using limits prescribed in configuration.

We envision using a new section, `service::memory`, used to configure memory
assignments for each pipeline. We anticipate the need to configure both broad memory
limits (e.g., per component, per pipeline) and fine-grain ones (e.g., per tenant).
We see the admission controller also as typical location for other sorts of throttling
(e.g., rate limits) and prioritization (e.g., by signal, by tenant).

[ADMISSION]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md#admission-control-configuration

### Managing back-pressure

A reliable telemetry pipeline requires a set of features described as
"back-pressure," which covers a range of built-in supports.

- Coordinated admission limits for receivers
- Timeout and deadline propagation
- Automatic retry
- Cancellation support
- Error transmission with status codes
- Durable and in-memory queue configuration.

Following the reasoning given above, because the OTel-Arrow pipeline
is focused exclusively on OTAP support, we find there are few reasons
to bundle batching and queuing support into an `exporterhelper`
module. Instead, we will provide queuing and batching support using
dedicated channels and processors. Likewise, retry, failover, and timeout support
will be provided as primitive features.

### Self-observability

The pipeline is required to be instrumented using the OTel-Rust API.
However, we also require an end-to-end OTAP pipeline. Therefore, we will
work with the OTel-Rust language group to find a solution. We are interested
in using telemetry APIs generated from schema definitions defined in the
OpenTelemetry Weaver project that have strong type information. For this
project, we will develop an OTel-Rust SDK that supports strongly-typed
OTAP-friendly instrumentation at first. The new SDK will yield OTAP
streams directly, and the pipeline will be capable of directing its
telemetry to itself for processing.

The pipeline will provide a couple of essential built-in observability features:

#### Debug/Console Exporter

A simple exporter to print telemetry data to the console will be
supplied. By default, the console exporter will export line-oriented
OTLP/JSON data for INFO-level-and-above logging statements.

#### Push-based Self-Diagnostic Exporter

Using standard OTel-Rust SDK features, the pipeline can be configured
to push self-diagnostic OTLP and/or OTAP telemetry data to a configurable
destination.

Push-based self-diagnostics will be disabled by default for all signals.

#### Pull-based Self-Diagnostic Exporter

For its own metrics, the pipeline will offer a configurable built-in
Prometheus endpoint enabling a fallback HTTP endpoint for diagnostic
information.

For its own logs, the pipeline will offer a configurable built-in
HTTP endpoint displaying a fixed portion of info/warning/error logs.

For security reasons, because static port assignment is not always
possible, and in case of short-lived, these features will be disabled
by default.

## Immediate Areas of Exploration

### Pipeline CPU/Memory Affinity

We are interested in exploring a configurable runtime execution
model, including the ability to specify thread-per-core and
multi-threaded pipelines. Through configuration, pipelines can
be assigned to CPU groups and segregated by signal type and
tenant identity. Pipelines can have independent priorities and be
mapped to CPUs through configuration.

For this aspect of the project, we are likely to introduce a new
section in the configuration, `service::runtime` where CPU affinities,
CPU limits, memory assignments/limits, and tenant assignments are
described. These details will be parallel in the configuration with
the components that they impact, so that configuration otherwise
maintains parity with OpenTelemetry Collector. In NUMA scenarios,
we expect `service::runtime` and `service::memory` configurations
to be interrelated.

### Operating-System and Platform Optimized

We expect to explore platform and OS-specific optimizations. We
will, for example, explore using the best available low-level
asynchronous framework (e.g., io_uring, Windows IOCP), making it
possible to use the best-available allocators and work schedulers
on each platform.

### Multi-tenancy

We are aware of a number of mechanisms through which multi-tenancy
can be supported. Users should be able to configure CPU, memory, and
other shared pipeline resources to be tenant-aware.

Tenants may identify themselves using dedicated HTTP headers or through
field values inside OTLP resources. We envision a configuration section
named `service::tenant` where tenant CPU, memory, and rate-limit assignments
are located. We expect to support both dedicated CPU resources (e.g.,
using a thread-per-core architecture) and shared CPU resources (e.g.,
using a prioritization scheme).

As a primary consideration, the pipeline should not crash or become unavailable
for all tenants because of bad data or buggy code experienced by one tenant.

### Dynamic Telemetry

As a telemetry agent, the pipeline has more than an average
level of sensitivity to risk in the telemetry SDK. We are
worried about feedback created by instrumenting a pipeline that
exports telemetry to itself. We are interested in ensuring that
the pipeline offers very-low-cost instrumentation and we are
willing to incorporate advanced receivers to make this possible.

As described above, we require type-safe instrumentation APIs in
order to directly generate OTAP data. To address the risk of a
complex new OTAP SDK impacting the pipeline, we will use an
"empty" SDK. The empty SDK is an OpenTelemetry SDK with empty methods
compiled to have minimal overhead on the application.

We propose to use the cross-platform User-events framework, as embodied
in the Microsoft One-collect dynamic instrumentation framework, to receive
self-diagnostic telemetry from the pipeline itself. Thus, telemetry exported
by the pipeline will be in most regards treated on the same footing as ordinary
pipeline data.

We anticipate that the One-collect project will be donated to OpenTelemetry,
eventually, and we believe this project will be a great demonstration of its value.

## Example Components

As part of the first stage of this investigation, we will investigate
implementing a few but not all of the following components. The requirements
to be included in this list are:

- Relatively easy to implement
- Expecting major performance improvement

### Probabilistic Span/Log Sampler

Emulating the `probabilisticsamplingprocessor`, this component
will apply a sampling decision in a column-oriented way. Whereas
the Golang implementation of this component must perform certain
logic for every span and log record, the OTAP pipeline will compute
sampling decisions at the column level. We expect the resulting
reconstruction of sampled data to be significantly faster and use
less CPU.

### Attributes Processor

We are interested in demonstrating that attribute removal
and attribute renaming are low-cost operations in an OTAP pipeline, because
these operations simply mutate the Arrow schema. We would
supply a processor with features corresponding to the
Golang `attributesprocessor`.

### Resources Processor

Similar to the attributes processor, but for resources. These
two components can easily share a common code base.

### Resource Detection Processor

This is a processor that infers information from environment
variables and stables them onto the request. This should be
extremely low cost when done in a column-oriented fashion.

### K8s Attributes Processor

This processor implements the logic of the `k8sattributesprocessor`.
We expect this to be substantially less expensive and also
safer than its Golang equivalent.

## Future Ideas

Here are some ideas we expect to explore in the future.

- Pipeline mechanism support for multiple built-in data types (e.g., OTLP/proto, OTLP/json). We would like to support raw content in its original form, sometimes.
- Support Golang components through FFI, WASM, etc. We would like to be able to compile certain components from the Golang implementation (e.g., Prometheus, hostmetrics, etc.).
- Dynamic configuration. We would like to ensure that components have a minimum level of support for dynamic reconfiguration from the start, but this is not an explicit requirement at this stage.
