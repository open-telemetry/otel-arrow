# OTAP Pipeline

The OTAP (OpenTelemetry Arrow Protocol) crate contains OTAP-native receivers,
processors, and exporters for OtapPdata pipelines.

Contrib components (for example Geneva and Azure Monitor exporters, and
optional contrib processors) live in `crates/contrib-nodes`.

## Receivers

- OTLP Receiver (`src/otlp_receiver.rs`)
- OTLP/HTTP Receiver (`src/otlp_http.rs`)
- OTAP Receiver (`src/otap_receiver.rs`)
- Internal Telemetry Receiver (`src/internal_telemetry_receiver.rs`)
- Syslog CEF Receiver (`src/syslog_cef_receiver/`)

## Processors

- Attributes Processor (`src/attributes_processor/`, `src/attributes_processor.rs`)
- Content Router (`src/content_router.rs`)
- Durable Buffer Processor (`src/durable_buffer_processor/`)
- Retry Processor (`src/retry_processor.rs`)
- Transform Processor (`src/transform_processor/`, `src/transform_processor.rs`)

## Exporters

- OTAP Exporter (`src/otap_exporter/`, `src/otap_exporter.rs`)
- OTLP gRPC Exporter (`src/otlp_grpc_exporter.rs`)
- OTLP HTTP Exporter (`src/otlp_http_exporter/`)
- Parquet Exporter (`src/parquet_exporter/`, `src/parquet_exporter.rs`)
- Perf Exporter (`src/perf_exporter/`)

## Generate Protobuf Stubs

In the repository root, run:

```bash
cargo xtask compile-proto
```
