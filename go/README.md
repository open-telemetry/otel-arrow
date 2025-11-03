# OTel-Arrow Go libraries

This folder contains the OTel-Arrow Go reference implementation.  This
implementation was built around the OpenTelemetry Collector in Golang,
therefore it targets the `pdata` representation used in that system's
pipeline, with top-level types known as `ptrace.Traces`,
`pmetric.Metrics`, and `plog.Logs` corresponding with the payload of
an OpenTelemetry Traces, Metrics, or Logs export request.

The primary use for this library involves converting between two
primary representations:

- OTLP records: the Collector's in-memory data representation
- OTAP stream: the OTel-Arrow batch of Arrow IPC stream records

The intermediate representation between the OTLP records and OTAP
stream forms, known as "OTAP records", exists here, however its design
was not emphasized. Refer to the
[Otel-Arrow-Rust](../rust/otel-arrow-rust/README.md) reference
implementation for more details about handling the OTAP records format
in memory.

## OpenTelemetry Collector Producer to OTAP stream

This library produces the OTel-Arrow OTAP stream representation of
OpenTelemetry data from the standard representation, for use in
OpenTelemetry Collector pipelines. [The `otelarrowexporter` component
in the OpenTelemetry Collector-Contrib repository][OTELARROWEXPORTER]
is the primary user of this feature, which first converts PData to
OTAP records, then to OTAP bytes using an Arrow IPC writer.

The main Producer entry point for converting from OpenTelemetry
Collector records into OTAP streams is found in
[./pkg/otel/arrow_record/producer.go](./pkg/otel/arrow_record/producer.go)

[OTELARROWEXPORTER]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/otelarrowexporter/README.md

## OTAP stream to OpenTelemetry Collector Consumer

This library consumes the OTel-Arrow OTAP stream representation of
OpenTelemetry data and produces the standard representation, for use
in OpenTelemetry Collector pipelines.  [The `otelarrowreceiver`
component in the OpenTelemetry Collector-Contrib
repository][OTELARROWRECEIVER] is the primary user of this feature,
which first converts OTAP bytes into OTAP records using an Arrow IPC
reader, then converts the records back into the standard
representation.

The main Consumer entry point for converting from OTAP streams into
OpenTelemetry Collector records is found in
[./pkg/otel/arrow_record/consumer.go](./pkg/otel/arrow_record/consumer.go)

[OTELARROWRECEIVER]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/receiver/otelarrowreceiver/README.md
