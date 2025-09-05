# OTAP Pipeline

The OTAP (OpenTelemetry Arrow Protocol) crate contains receivers, processors,
and exporters supporting natively the OTAP Pdata.

## Receivers

- OTLP Receiver: A receiver accepting OTLP messages over gRPC.
- OTAP Receiver: A receiver accepting OTAP messages over gRPC.
- Syslog CEF Receiver: A receiver accepting TCP and UDP Syslog messages
  formatted in the Common Event Format (CEF), RFC-3164, and RFC-5424.
- Fake Data Generator: A receiver generating fake data for testing
  purposes. Fake signals are generated from semantic convention registries.

## Processors

- Attribute Processor: A processor to rename and delete attributes on
  spans, metrics, and logs. Other operations such as inserting and updating
  attributes will be added in the future.
- Batch Processor: A processor to batch incoming data into batches of a
  specified size or timeout.
- Debug Processor: A processor to log incoming data for debugging
  purposes.
- Retry Processor (WIP): A processor to retry sending data on failure.
- Signal Type Router: A processor to route data based on signal type
  (traces, metrics, logs) to different downstream nodes.

## Exporters

- OTLP Exporter: An exporter sending OTLP messages over gRPC.
- OTAP Exporter: An exporter sending OTAP messages over gRPC.
- Noop Exporter: An exporter that drops all data.
- Parquet Exporter: An exporter that writes data to Parquet files.

## Generate Protobuf Stubs

In the root of the repository, run:

```bash
cargo xtask compile-proto
```
