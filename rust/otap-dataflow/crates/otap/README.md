# OTAP Pipeline

The OTAP (OpenTelemetry Arrow Protocol) crate now primarily contains shared OTAP
and OTLP transport infrastructure, pdata types, TLS/compression helpers, and
test support used by node implementations in other crates.

Core node implementations live in `crates/core-nodes`.

Contrib components (for example Geneva and Azure Monitor exporters, and
optional contrib processors) live in `crates/contrib-nodes`.

## Shared Infrastructure

- OTAP/OTLP pdata types and conversions (`src/pdata.rs`, `src/pdata_conversions.rs`)
- OTAP gRPC transport support (`src/otap_grpc/`, `src/otap_grpc.rs`)
- OTLP gRPC transport support (`src/otlp_grpc.rs`)
- OTLP HTTP client/server support (`src/otlp_http/`, `src/otlp_http.rs`)
- Compression configuration (`src/compression.rs`)
- TLS and crypto helpers (`src/tls_utils.rs`, `src/crypto.rs`)
- Shared receiver metrics (`src/otlp_metrics.rs`)
- Test fixtures and mocks (`src/otap_mock.rs`, `src/otlp_mock.rs`, `src/testing/`)

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
