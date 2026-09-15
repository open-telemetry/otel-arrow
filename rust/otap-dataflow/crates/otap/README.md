# OTAP Common Runtime

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

The OTAP (OpenTelemetry Arrow Protocol) crate is the common runtime layer used
by Dataflow node crates. It provides the shared OTAP and OTLP transport
infrastructure, pdata types, TLS and compression helpers, metrics, and test
support that core, contrib, development, and custom nodes build on.

Core node implementations live in `crates/core-nodes`.

Development-only test, fault-injection, and benchmark nodes live in
`crates/dev-nodes`.

Contrib nodes (for example Geneva and Azure Monitor exporters, and
optional contrib processors) live in `crates/contrib-nodes`.

## Shared Infrastructure

- OTAP/OTLP pdata types and conversions (`src/pdata.rs`, `src/pdata_conversions.rs`)
- OTAP gRPC transport support (`src/otap_grpc/`, `src/otap_grpc.rs`)
- OTLP gRPC transport support (`src/otlp_grpc.rs`)
- OTLP HTTP client/server support (`src/otlp_http/`, `src/otlp_http.rs`)
- Compression configuration (`src/compression.rs`)
- TLS and crypto helpers (`src/tls_utils.rs`, `src/crypto.rs`)
- Shared node boundary metrics (`src/metrics.rs`)
- Shared OTLP receiver metrics (`src/otlp_metrics.rs`)
- Test fixtures and mocks (`src/otap_mock.rs`, `src/otlp_mock.rs`, `src/testing/`)

## Shared Node Boundary Metrics

This crate provides the shared `ReceiverMetrics` and `ExporterMetrics` helpers
for external node boundaries. These metrics count external messages and
submissions independently from engine-managed PData node metrics.

The complete contract, including 1:1, fan-out, aggregation, many-to-many,
retry, timing, payload-size, and outcome guidance, is in the
[Internal Telemetry Metrics Guide](../../docs/telemetry/metrics-guide.md#shared-receiver-and-exporter-boundary-metrics).
See [Node and Flow Metrics](../../docs/node-and-flow-metrics.md) for operator
interpretation.

## Node Implementations Using This Crate

The following core OTAP/OTLP nodes now live in `crates/core-nodes` and reuse
shared functionality from this crate:

- OTAP Receiver (`crates/core-nodes/src/receivers/otap_receiver/`)
- OTLP Receiver (`crates/core-nodes/src/receivers/otlp_receiver/`)
- OTAP Exporter (`crates/core-nodes/src/exporters/otap_exporter/`)
- OTLP gRPC Exporter (`crates/core-nodes/src/exporters/otlp_grpc_exporter/`)
- OTLP HTTP Exporter (`crates/core-nodes/src/exporters/otlp_http_exporter/`)

## Generate Protobuf Stubs

In the repository root, run:

```bash
cargo xtask compile-proto
```
