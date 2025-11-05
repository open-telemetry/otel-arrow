## OTel-Arrow Rust

PData refers generally to the pipeline data type used within an
OpenTelemetry pipeline.  This package is the location of our core
utilities for converting between several representations.

This crate contains the low-level reference implementation used in
OTel-Arrow, similar in nature to the [OTel-Arrow Golang
library](../../../../go/README.md) used by the project's Golang collector
components.  This library translates between the following
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

## Schema

The `otap_df_pdata::schema` module provides constants related to the
OTAP records payload representation, defining the OpenTelemetry
Protocol with Apache Arrow.

## Proto

The `otap_df_pdata::proto` module provides access to the original OTLP
and OTAP protocol message objects, for reference and testing.  This
also exposes constants such as protocol tag numbers used for directly
encoding and decoding OTLP bytes. These use the Prost and Tonic crates
to generate structs and gRPC client/server stubs.

## Views

The `otap_df_pdata::views` provides view abstractions and utilities
for working with OTLP pdata (protocol data) structures without
constructing protocol message objects, enabling efficient translation
to and from the OTAP records data format
