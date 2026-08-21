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
- Shared component boundary metrics (`src/metrics.rs`)
- Shared OTLP receiver metrics (`src/otlp_metrics.rs`)
- Test fixtures and mocks (`src/otap_mock.rs`, `src/otlp_mock.rs`, `src/testing/`)

## Shared Component Boundary Metrics

Universal node metrics describe internal PData delivery, while receivers and
exporters own the external boundaries:

```text
wire -> receiver.ingress -> node.producer -> ... -> node.consumer -> exporter.egress -> wire
```

The shared contracts are:

```text
receiver.ingress.{messages,wire_bytes,duration}{signal,outcome}
exporter.egress.{messages,items,wire_bytes,duration}{signal,outcome}
```

### Receiver ingress

One observation represents one external message after signal classification.
`success` means receiver-local handoff completed, `refused` means the classified
message was rejected, and `failure` means receiver-local processing failed.
Duration ends at that receiver-local result and excludes downstream processing
and Ack/Nack completion. Rejections before signal classification remain
component-specific diagnostics.

Receiver ingress omits `items` because decoded items are measured by
`node.producer.produced.items`.

### Exporter egress

One observation represents one exporter dequeue or invocation through one
terminal external result. Component-internal retries belong to the same
observation, while redelivery by a retry processor is a new observation.
Duration excludes Ack/Nack notification propagation. Items inherit the
whole-message terminal outcome.

### Wire bytes and internal size

`wire_bytes` means encoded application payload bytes visible at the external
boundary. It excludes protocol headers, TLS overhead, and storage amplification.
Exporter wire bytes accumulate the bytes submitted across every attempt within
the logical export and are attributed to its terminal outcome. An export that
fails before submitting bytes records zero. Component-specific diagnostics may
add per-attempt protocol details.

Instrumentation must not serialize PData solely to populate `wire_bytes`.

The internal `size` measurement is separate: it describes the PData
representation at `node.producer` and `node.consumer`, not the external encoded
representation.

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
