# OTAP Dataflow Library

[![build](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml)
[![build](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-audit.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-audit.yml)
[![codecov](https://codecov.io/gh/open-telemetry/otel-arrow/graph/badge.svg?token=tmWKFoMT2G&component=otap-dataflow)](https://codecov.io/gh/open-telemetry/otel-arrow)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Slack](https://img.shields.io/badge/Slack-OTEL_Arrow-purple)](https://cloud-native.slack.com/archives/C07S4Q67LTF)

----

## Overview

The OTAP Dataflow library is a set of core Rust crates which combine
to produce an OpenTelemetry pipeline support, for use as an embedded
software component, providing a framework for collecting OpenTelemetry
data.

> [!NOTE] These Rust libraries are the main deliverable of Phase 2 of
> the OTel-Arrow project, as defined in the [project
> phases](../../docs/project-phases.md).  The `df_engine` main
> program built through `cargo` in [`src/main.rs`](./src/main.rs) is
> provided as a means to test and validate OTAP pipelines built using
> the dataflow engine.
>
> The OTel-Arrow Rust library in `../otel-arrow-rust`, which implements
> low-level details of the conversion between OTAP and OTLP, will be
> incorporated into this set of crates.

## Features

The OTAP Dataflow engine consists of a number of major pieces. Here
are the major aspects of its design:

- OTel-Arrow first. The Apache Arrow record batch is the underlying
  data type used to represent "OTAP records", at the lowest level in
  our pipeline. The OTAP records format consists of a varying number
  of Arrow record batches per unit of data, representing hierarchical
  OpenTelemetry data in a column-oriented form, under a "star schema".
- Zero-copy to and from OTLP bytes. OTLP bytes are the standard for
  representing OpenTelemetry data on the wire. Through a custom
  implementation of Google Protocol buffers, we convert OTAP records
  directly to and from OTLP bytes without constructing intermediate
  data objects.
- Thread-per-core approach. Our design aims to support single-threaded
  _nothing-shared_ pipelines as our first priority. We make use of a
  [Local async
  runtime](https://docs.rs/tokio/latest/tokio/runtime/struct.LocalRuntime.html)
  freeing the pipeline from synchronizing instructions.
  Multi-threaded components are possible using `shared` adapters, but
  we choose single-threaded `local` components when possible.

The basic unit of data in an OTAP Dataflow pipeline is the OTAP
pipeline data object, `otap_df_otap::pdata::OtapPdata`. In the
hierarchy of types a pipeline component interacts with, this crate
`otap_df_otap::pdata` crate is a focal point. The `OtapPdata` data
type is a struct consisting of "context" and "payload", where context
is used for routing "Ack" and "Nack" responses, and payload has two
equivalent, signal-specific representations:

- OTLP bytes (Logs, Traces, Metrics): A signal-specific enum of
  `Vec<u8>` corresponding with one of the export requests (e.g.,
  `ExportLogsServiceRequest`).
- OTAP records (Logs, Traces, Metrics): A signal-specific array of
  [Arrow
  `RecordBatch`](https://docs.rs/arrow/latest/arrow/record_batch/struct.RecordBatch.html)
  objects defining aspects of the OpenTelemetry data model, where
  unused columns are omitted. For example, The Logs form of OTAP
  records consists of four record batches, corresponding with Logs,
  Log Attributes, Scope Attributes, and Resource Attributes.

Refer to the [OTAP basics](../../docs/otap_basics.md) documentation.

The [OTAP data model](../../docs/data_model.md) contains diagrams of
the many N-to-1 relationships expressed within an OTAP request.

## Major components

### Engine

[See crate README.](./crates/engine/README.md)

The `otap_df_engine` crate is located in `crates/engine`, here we
find the engine's overall architecture expressed:

- Local (unsynchronized) and shared (`Sync + Send`) code paths
- Queue-oriented message passing
- Separate control and data plane
- Effect handler for interacting with pipeline.

The engine's main entry point,
`otap_df_engine::PipelineFactory<PData>`, supports building a
single-thread pipeline for generic type `PData`. Generally, users do
not construct these, as they are managed by a controller instance.
Here are the key files to know about:

```text
crates/engine/lib.rs:    Effect handler extensions, pipeline factory
|-- attributes.rs:       Host, process/container IDs
|-- context.rs:          CPU, NUMA, thread context
|-- control.rs:          NodeControlMsg, AckMsg, NackMsg
|-- effect_handler.rs:   Component interfaces (network, clock, ack/nack)
|-- error.rs:            Structured errors
|-- exporter.rs:         Pipeline component (output only)
|-- message.rs:          The data and control plane messages
|-- node.rs:             The basic NodeId type
|-- pipeline_ctrl.rs:    Timer state, channel to all nodes
|-- processor.rs:        Pipeline component (input/input)
|-- receiver.rs:         Pipeline component (input only)
|-- runtime_pipeline.rs: Builds the graph of component channels
```

### OTAP: OTel-Arrow Protocol pipline data

[See crate README.](./crates/otap/README.md)

The OTAP pipeline data type is defined here, therefore all of our
built-in components are provided here as well.  The main entry point
into this crate is the `otap_df_otap::pdata::OtapPdata` type with its
two alternate representations, OTAP records and OTLP bytes, specific
by signal type.

The PData type also facilitates datatype-aware aspects of interacting
with the pipeline engine, including `ProducerEffectHandlerExtension`,
for receivers and processors to subscribe to the `NodeControlMsg::Ack`
and `NodeControlMsg::Nack` messages, and
`ConsumerEffectHandlerExtension` for processors and exporters to
notify the next recipient in the chain of subscribers.

Here are the key files to know that support the components in this
crate:

```text
crates/otap/lib.rs:      OTAP Dataflow pipeline factory
|-- compression.rs:      OTLP and OTAP compression settings
|-- encoder.rs:          Computes OTAP from OTLP view representations
|-- metrics.rs           Metrics logic shared by several components
|-- pdata.rs             The OtapPdata type, effect handler extensions
|-- otap_grpc/           OTLP and OTAP shared gRPC support
|-- fixtures.rs          Test support
|-- otap_mock.rs         Test support
|-- testing/             Test support
```

All gRPC services are implemented using
[Tonic](https://github.com/hyperium/tonic).

The major OTAP Dataflow components of `otap_df_otap` are listed next.

#### Attributes processor

This component supports efficient low-level manipulation of OTAP
records. For example, this component supports O(1) column renaming for
OpenTelemetry data.

#### Debug processor

A simple component that prints information about the data passing
through, with configurable level of detail.

#### Error exporter

A simple component that returns a constant error message. All requests
fail.

#### Noop exporter

A simple component that returns success. All requests succeed.

#### Fake Data Generator

A simple component to produce synthetic data from semantic convention registries.

#### Batch processor

An batching processor that works directly with OTAP records. This is
[based on lower-level support in the `otal_arrow_rust`
crate](../otel-arrow-rust/src/otap/batching.rs).

#### OTAP exporter

The OTAP streaming gRPC exporter. This corresponds with the
[`otelarrowexporter` Collector-Contrib exporter component][EXPORTER]
(i.e., this project's Phase 1 deliverable), based on Arrow IPC
streams over gRPC streams.

#### OTAP receiver

The OTAP streaming gRPC receiver. This corresponds with the
[`otelarrowreceiver` Collector-Contrib receiver component][RECEIVER],
this project's Phase 1 deliverable based on Arrow IPC streams over
gRPC streams.

#### OTLP exporter

The OTLP unary gRPC exporter. This corresponds with the `otlp`
Collector exporter component, exports standard OTLP bytes.

[RECEIVER]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md
[EXPORTER]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md

#### OTLP receiver

The OTLP unary gRPC exporter. This corresponds with the `otlp`
Collector receiver component, receives standard OTLP bytes using
[a Tonic server](https://github.com/hyperium/tonic).

#### Parquet exporter

The parquet exporter records the OTel-Arrow representation using
Parquet. While there is a direct translation between the OTel-Arrow
representation and Parquet, it requires changing several data types to
be compatible with Parquet. This component uses 32-bit identifiers, as
opposed to 16-bit identifiers used in OTel-Arrow batches, making large
batches of telemetry available for external engines to process.

#### Performance exporter

A simple component that collects and prints statistics about the
number of requests and items it sees, used for monitoring our
benchmarks.

#### Retry processor

The retry processor supports a configurable number of retries and
exponential back-off.

#### Signal type router

Supports routing OTAP data by signal type, enabling signal-specific
route destinations.

#### Syslog/CEF receiver

The Syslog/CEF receiver is considered a core component used to
establish the performance of the OTAP Dataflow system.

### Controller

[See crate README.](./crates/controller/README.md)

The `otap_df_controller` crate is located in `crates/controller` is
the main entry point to construct an OTAP Dataflow pipeline instance. The
controller type, `otap_df_controller::Controller<PData>`, manages building
and running one or more pipelines.

This component is responsible for making the assignment between OTAP
dataflow pipeline and individually-numbered CPU instances. The
`Controller::run_forever()` method is called to execute the pipeline.
Like the engine, the pipeline datatype `PData` is opaque to this crate.

### Config

[See crate README.](./crates/config/README.md)

Here, the configuration model for the OTAP Dataflow engine defines the
structs and conventions used to configure as well as observe the
pipeline, the engine, and the pipeline components.

A number of example configurations are listed in
[`./configs`](./configs). These are deserialized into the
`otap_df_config::engine::EngineConfig` structs, defined in this crate.

### Channel

[See crate README.](./crates/channel/README.md)

Defines the low-level queues used in the OTAP dataflow pipeline,
`otap_df_channel::mpsc` and `otap_df_channel::mpmc`.

Defines a standard `SendError<T>` used to return failures throughout
the codebase to enable recovering from `try_send()`.

### Admin

[See crate README.](./crates/admin/README.md)

Defines an administrative portal for operators, an HTTP service
capable of displaying the current pipeline state, pipeline
configuration, debugging logs, and Prometheus metrics. This supports
primitive controls such as the ability to shut down the pipeline.

### State

[See crate README.](./crates/state/README.md)

Low-level library supporting the state transition diagram, enabling
observability for the state of the `Controller`.

### Telemetry

[See crate README.](./crates/telemetry/README.md)

The OTAP Dataflow system is built using a bespoke telemetry system, as
we needed to ensure NUMA-awareness from the start. Moreover, this
project is taking up a charter to investigate an OTel-Arrow first
approach to telemetry, hence we are working with the experimental
telemetry SDK here.

### PData

[See crate README.](./crates/pdata/README.md)

TODO(#1218) The OTel-Arrow-Rust crates are moving here. The
`otap_df_otap::pdata` types including `OtapPdata` are moving here.

This is the future location for [OTel-Arrow Rust](../otel-arrow-rust/README.md).

The `views` sub-module contains zero-copy machinery for:

- interpreting OTLP bytes using views to build OTAP records
- interpreting OTAP records using views to encode OTLP bytes

## Development Setup

**Requirements**:

- Rust >= 1.86.0
- Cargo

**Clone & Build**:

```bash
git clone https://github.com/open-telemetry/otel-arrow
cd otel-arrow/rust/otap-dataflow
cargo build --workspace
```

**Run Tests**:

```bash
cargo test --workspace
```

**Run Examples**:

```bash
cargo run --example <example_name>
```

**With Docker**:

```bash
docker build --build-context otel-arrow=../../ -f Dockerfile -t df_engine .
```

## Contributing

- [Contribution Guidelines](CONTRIBUTING.md)
- Code of Conduct (TBD)

Before submitting a PR, please run the following commands:

```bash
# Prepare and check the entire project before submitting a PR or a commit
cargo xtask check
```

## :memo: License

Licensed under the [Apache License, Version 2.0](LICENSE).

## :telephone_receiver: Support & Community

CNCF Slack channel: [#otel-arrow](https://slack.cncf.io/)

## :star2: Roadmap

See our detailed [Roadmap](ROADMAP.md) for upcoming features and improvements.

## :white_check_mark: Changelog

- [CHANGELOG.md](CHANGELOG.md)
