# OTel-Arrow Protocol Implementation in Rust

[![build](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-ci.yml)
[![build](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-audit.yml/badge.svg)](https://github.com/open-telemetry/otel-arrow/actions/workflows/rust-audit.yml)
[![codecov](https://codecov.io/gh/open-telemetry/otel-arrow/graph/badge.svg?token=tmWKFoMT2G&component=otap-dataflow)](https://codecov.io/gh/open-telemetry/otel-arrow)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Slack](https://img.shields.io/badge/Slack-OTEL_Arrow-purple)](https://cloud-native.slack.com/archives/C07S4Q67LTF)

----

Rust reference implementation for the [OpenTelemetry-Arrow Protocol
(OTAP)](../../README.md). Here are some of the main data use-cases:

## OTAP records to OTLP bytes

The OTAP Records type is `otel_arrow_rust::otap::OtapArrowRecords`.

In the Logs signal, for example,
`otel_arrow_rust::otlp::logs::LogsProtoBytesEncoder` encodes OTLP
bytes from OTAP records to a `&mut ProtoBuffer` output, likewise for
`metrics::Metrics` and `traces::Traces`.

In the OTAP-Dataflow engine, see
`otap_df_otap::pdata::TryFrom<OtapArrowRecords>` for example, to
translate from OTLP bytes into OTAP records.

## OTLP bytes to OTAP records

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
3. `otel_arrow_rust::encode::record::logs::LogsRecordBatchBuilder`
  is a OTel-Arrow builder which assembles Arrow arrays as the output of traversal.

## OTAP records to OTAP stream

The OTAP stream data type is
`otel_arrow_rust::proto::opentelemetry::arrow::v1::BatchArrowRecords`.

To translate a stream of OTAP Records to `BatchArrowRecords`, use
`otel_arrow_rust::encode::producer::Producer`. The producer manages
underlying `arrow::ipc::writer::StreamWriter` instances following the
OTel-Arrow Phase 1 Golang reference implementation.

Note that the `otel_arrow_rust::otap::OtapArrowRecords` type supports
both transport-optimized and memory-optimized representations, using
Arrow metadata for the indicator.

See more documentation on this process in our [OTAP
basics](../../docs/otap-basics.md) documentation.

## OTAP stream to OTAP records

To convert from `BatchArrowRecords` to `OtapArrowRecords`, use a
`otel_arrow_rust::decode::decoder::Consumer`. The consumer manages
underlying `arrow::ipc::reader::StreamReader` instances following the
OTel-Arrow Phase 1 Golang reference implementation.

After passing through an intermediate representation,
`otel_arrow_rust::decode::record_message::RecordMessage`, an assembly process
`otel_arrow_rust::otap::from_record_messages(Vec<RecordMessage>) -> T` yielding
`OtapArrowRecords` of the correct signal.

## Reference implementation and verification

The production OTAP-Dataflow engine uses the `otap_df_pdata_views`
crate to encode and decode OTLP bytes directly to and from OTAP
records.

The Prost crate (`prost_build`) is used to derive OpenTelemetry
Protocol (OTLP) message objects which are used as a reference
implementation for test and validation steps in this repository.

See the repository-level [CONTRIBUTING](../../CONTRIBUTING.md) for
details on building and testing this software.
