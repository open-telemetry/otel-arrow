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
- gRPC settings, proxy, and middleware (`src/grpc/`)
- OTAP streaming server and ACK/NACK subscriptions (`src/grpc/otap/`)
- OTLP gRPC client and server (`src/grpc/otlp/`)
- HTTP client/server settings (`src/http/settings/`)
- OTLP HTTP server (`src/http/otlp/server.rs`)
- Cross-transport request acknowledgement tracking (`src/receiver/ack.rs`)
- Transport-independent OTLP metrics and exporter helpers (`src/otlp/`)
- Compression configuration (`src/compression.rs`)
- TLS and crypto helpers (`src/tls_utils.rs`, `src/crypto.rs`)
- Shared node boundary metrics (`src/metrics.rs`)
- Test fixtures and mocks (`src/otap_mock.rs`, `src/otlp_mock.rs`, `src/testing/`)

HTTP and gRPC are sibling transports, each owning its settings and wire
implementations. Neither transport depends on the other's server.
OTLP helpers that do not depend on transport live under `otlp`; receiver
acknowledgement state shared across OTLP transports lives under `receiver::ack`.
OTAP stream subscriptions live alongside their server in `grpc::otap::ack`.
Typed protobuf services used by fixtures live in `grpc::otlp::typed`.

## Shared Node Boundary Metrics

This crate provides the shared `ReceiverMetrics` and `ExporterMetrics` helpers
for external node boundaries. These metrics count external messages and
node-local export attempts independently from engine-managed PData node
metrics.

The complete contract, including 1:1, fan-out, aggregation, many-to-many,
retry, timing, payload-size, and outcome guidance, is in the
[Internal Telemetry Metrics Guide](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/docs/telemetry/metrics-guide.md#shared-receiver-and-exporter-boundary-metrics).
See [Node and Flow Metrics](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/docs/node-and-flow-metrics.md)
for operator interpretation.

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
