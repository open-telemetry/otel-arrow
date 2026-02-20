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
- Fake Data Generator (`src/fake_data_generator/`)

## Processors

- Attributes Processor (`src/attributes_processor/`, `src/attributes_processor.rs`)
- Batch Processor (`src/batch_processor.rs`)
- Content Router (`src/content_router.rs`)
- Debug Processor (`src/debug_processor/`, `src/debug_processor.rs`)
- Durable Buffer Processor (`src/durable_buffer_processor/`)
- Fanout Processor (`src/fanout_processor/`, `src/fanout_processor.rs`)
- Filter Processor (`src/filter_processor/`, `src/filter_processor.rs`)
- Retry Processor (`src/retry_processor.rs`)
- Signal Type Router (`src/signal_type_router.rs`)
- Transform Processor (`src/transform_processor/`, `src/transform_processor.rs`)

## Exporters

- Console Exporter (`src/console_exporter.rs`)
- Error Exporter (`src/error_exporter.rs`)
- Noop Exporter (`src/noop_exporter.rs`)
- OTAP Exporter (`src/otap_exporter/`, `src/otap_exporter.rs`)
- OTLP Exporter (`src/otlp_exporter.rs`)
- Parquet Exporter (`src/parquet_exporter/`, `src/parquet_exporter.rs`)
- Perf Exporter (`src/perf_exporter/`)

## Generate Protobuf Stubs

In the repository root, run:

```bash
cargo xtask compile-proto
```
