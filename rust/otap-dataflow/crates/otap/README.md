# OTAP Pipeline

The OTAP (OpenTelemetry Arrow Protocol) crate now primarily contains shared OTAP
and OTLP transport infrastructure, pdata types, TLS/compression helpers, and
test support used by node implementations in other crates.

Core node implementations live in `crates/core-nodes`.

Development-only test, fault-injection, and benchmark nodes live in
`crates/dev-nodes`.

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
wire -> receiver.received -> node.output -> ... -> node.input
node.input -> exporter.attempted -> wire
```

The shared contracts are:

```text
receiver.received.{messages,payload.size,duration}{signal,outcome}
exporter.attempted.{messages,items,payload.size,duration}{signal,outcome}
```

### Receiver received

One observation represents one external message after signal classification.
`success` means receiver-local handoff was accepted and completed. `refused`
means validation, policy, admission, capacity, or pipeline handoff explicitly
rejected the message. `failure` means receiver-local processing was attempted
but did not complete because of an internal error.
Duration ends at that receiver-local result and excludes downstream processing
and Ack/Nack completion. Rejections before signal classification remain
component-specific diagnostics.

Receiver received omits `items` because decoded items are measured by
`node.output.items`.

### Exporter attempted

One observation represents one attempt to submit an encoded application
payload to an external backend or storage boundary. Component-internal retries
produce additional observations, as does redelivery by a retry processor.
`success` means the attempt was accepted and completed by the external boundary,
`refused` means validation, policy, admission, or capacity at that boundary
explicitly rejected it, and `failure` means an encoding, transport, timeout,
backend, or other processing error prevented completion. Retryability is
independent of the outcome.

Messages and duration are recorded for every attempt. Payload size and items
use separately registered optional metric sets under the same
`exporter.attempted` namespace. Components register payload size only when an
encoded application payload exists and its size is naturally available at the
attempt boundary. They register items only when cached item counts are enabled.
Instrumentation must not encode, parse, or traverse PData solely to populate
either optional metric.

The legacy `exporter.exports` set remains during migration but will be
deprecated. Attempt-level external behavior belongs to `exporter.attempted`,
while `node.input` owns the logical message's terminal pipeline outcome.

### Payload size and internal size

`payload.size` means encoded application payload bytes visible immediately
before receiver decoding or submitted by an exporter attempt. It excludes
protocol headers, framing, TLS overhead, and storage amplification. Exporters
without an encoded application payload, or whose encoder does not expose its
size, omit the optional metric set rather than reporting zero.

The internal `size` measurement remains separate: it describes the PData
representation at `node.output` and `node.input`.

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
