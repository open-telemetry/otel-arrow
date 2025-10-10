# OTel-Arrow Rust libraries

This folder contains the OTel-Arrow Rust sub-projects listed below.

## OTAP Dataflow

**[Sub-project README](./otap-dataflow/README.md)**

The `otap-dataflow` folder contains the project's primary dataflow
engine for building OpenTelemetry pipelines with an Arrow-first
approach. This component supports building and running the engine as a
software library, suitable for embedding in other telemetry agents.

This crate includes a CLI tool named `df_engine` for test and
demonstration purposes including a set of core components. In this
form, the engine is configured with YAML configuration expression the
set of nodes and edges in the graph. The core components: OTLP
receiver and exporter, OTAP receiver and exporter, batch and retry
processors, debug processor, fake data generator, Parquet exporter,
and a few more.

The primary data type of the OTAP dataflow engine is OTAP records
format, consisting of a set of Arrow record batches corresponding with
elements in the OpenTelemetry data model, by signal. The OTAP pipeline
also supports passing through OTLP bytes as literal data, with
**direct conversion** between the OTAP records and OTLP bytes models.

## OTel-Arrow Rust

**[Sub-project README](./otel-arrow-rust/README.md)**

The `otel-arrow-rust` folder contains the project's Rust reference
implementation for OTel-Arrow, similar in nature to the [OTel-Arrow
Golang library](../go/README.md) used by the project's Golang
collector components.  This library translates between the following
representations of OpenTelemetry:

- OTAP records: represented using [Apache Arrow (arrow-rs)][ARROW_RS]
  record batches
- OTLP records: represented using [Prost][PROST_RS] message objects
- OTAP stream: represented as batches of [Arrow IPC][ARROW_IPC] stream
- OTLP bytes: represented as bytes of [OpenTelemetry Protocol
  (OTLP)][OTLP] data

[ARROW_RS]: https://github.com/apache/arrow-rs/blob/main/README.md
[PROST_RS]: https://github.com/tokio-rs/prost/blob/master/README.md
[ARROW_IPC]: https://arrow.apache.org/docs/format/IPC.html
[OTLP]: https://opentelemetry.io/docs/specs/otel/protocol/

This library a low-level interface for producing and consuming OTAP
records.  This library includes built-in support for batching and
splitting of OTAP records.  While this library is recommended any time
you are converting between the representations listed above, note that
the OTAP Dataflow engine includes an alternative that avoids
materializing intermediate OTLP records.  We recommend [PData
Views](./otap-dataflow/crates/pdata-views/README.md) for producing and
consuming OTLP bytes in the OTAP-Dataflow engine.

## Experimental

Here, find our experimental projects. As part of the OTel-Arrow Phase
2 project scope ([project-phases](../docs/project-phases.md)), we are
developing transform and filter capabilities based around the OTAP
records representation.

- [Query abstraction: intermediate representation for common OTTL and
  KQL phrases](./experimental/query_abstraction/README.md)
- [Query engine: reference implementation for the abstraction
  layer](./experimental/query_engine/README.md)
- [Parquet query examples: querying OTel-Arrow data in Parquet
  files using DataFusion](./parquet_query_examples/README.md)
