# OTel-Arrow Pipeline Data

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

Rust reference implementation for the [OpenTelemetry-Arrow Protocol
(OTAP)](../../../../README.md). Here are some of the main data use-cases:

## High-level interfaces

### OTAP records to OTLP bytes

The OTAP Records type is `otap_df_pdata::otap::OtapArrowRecords`.

In the Logs signal, for example,
`otap_df_pdata::otlp::logs::LogsProtoBytesEncoder` encodes OTLP
bytes from OTAP records to a `&mut ProtoBuffer` output, likewise for
`metrics::Metrics` and `traces::Traces`.

In the OTAP-Dataflow engine, see
`otap_df_otap::pdata::TryFrom<OtapArrowRecords>` for example, to
translate from OTLP bytes into OTAP records.

### OTLP bytes to OTAP records

An OTAP Records struct varies by signal.

The OTAP-Dataflow engine represents OTLP bytes in their original form,
translating into OTAP Records on demand. Continuing with Logs, for
example, the `OtapArrowRecords::Logs` struct consists of 4 record
batches corresponding with tables of Logs, Log Attributes, Scope
Attributes, and Resource Attributes.

This translation into records is:

1. `otap_df_pdata_views::otlp::bytes::logs::RawLogsData`: Construct a
  view over the OTLP bytes for zero-copy traversal
2. `otap_df_otap::encoder::encode_logs_otap_batch<T: LogsDataView>()`
  is a function to build OTAP records in a traversal
3. `otap_df_pdata::encode::record::logs::LogsRecordBatchBuilder`
  is a OTel-Arrow builder which assembles Arrow arrays as the output of traversal.

### OTAP records to OTAP stream

The OTAP stream data type is
`otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords`.

To translate a stream of OTAP Records to `BatchArrowRecords`, use
`otap_df_pdata::encode::producer::Producer`. The producer manages
underlying `arrow::ipc::writer::StreamWriter` instances following the
OTel-Arrow Phase 1 Golang reference implementation.

Note that the `otap_df_pdata::otap::OtapArrowRecords` type supports
both transport-optimized and memory-optimized representations, using
Arrow metadata for the indicator.

See more documentation on this process in our [OTAP
basics](../../docs/otap-basics.md) documentation.

### OTAP stream to OTAP records

To convert from `BatchArrowRecords` to `OtapArrowRecords`, use a
`otap_df_pdata::decode::decoder::Consumer`. The consumer manages
underlying `arrow::ipc::reader::StreamReader` instances following the
OTel-Arrow Phase 1 Golang reference implementation.

After passing through an intermediate representation,
`otap_df_pdata::decode::record_message::RecordMessage`, an assembly process
`otap_df_pdata::otap::from_record_messages(Vec<RecordMessage>) -> T` yielding
`OtapArrowRecords` of the correct signal.

## Sub-Modules

### Schema

The `otap_df_pdata::schema` module provides constants related to the
OTAP records payload representation, defining the OpenTelemetry
Protocol with Apache Arrow.

### OTAP

This module defines the `OtapArrowRecords` enum, for representing
OTel-Arrow signal data in memory. This is generally how
`otap-dataflow` carries OTAP protocol data, and there are optimized
code paths for encoding to and decoding from `OtlpProtoBytes`.

### OTLP

This module defines the `OtlpProtoBytes` enum, for representing
OpenTelemetry protocol data without decoding into message objects.
This is generally how `otap-dataflow` carries OTLP protocol data,
and in the ordinarily case no OTLP message objects are constructed.

### Proto

The `otap_df_pdata::proto` module provides access to the original OTLP
and OTAP protocol message objects, for reference and testing.  This
also exposes constants such as protocol tag numbers used for directly
encoding and decoding OTLP bytes. These use the Prost and Tonic crates
to generate structs and gRPC client/server stubs.

This module defines the `OtlpProtoMessage` type, an enum of OTLP
protocol message objects by signal. This types is mostly used in
testing, because it has human-readable correspondence with
`OtapArrowRecords`, however it can be used anywhere Prost message
objects are required.

### Testing

The `otap_df_pdata::testing::equiv` module contains equivalence tests
for OTLP data. This compares two slices of `OtlpProtoMessage` to ensure
that they contain equivalent data. This internally canonicalizes the
two sets of messages and compares them with human-readable output.

### Views

The `otap_df_pdata::views` provides view abstractions and utilities
for working with OTLP pdata (protocol data) structures without
constructing protocol message objects, enabling efficient translation
to and from the OTAP records data format

### Validation

This module runs simple end-to-end tests using the OTel-Arrow Go
Collector components built in the top-level of this repository.
