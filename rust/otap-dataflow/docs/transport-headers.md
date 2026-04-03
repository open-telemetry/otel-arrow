# Transport Headers

This document describes the policy-based transport header capability that allows
receivers to extract selected inbound transport headers into the pipeline
context, and allows exporters to propagate selected headers on egress.

## Overview

Protocols like OTLP and OTAP carry request-scoped metadata outside the payload
itself—HTTP headers or gRPC metadata. This metadata can represent tenant IDs,
trace context, authorization tokens, routing hints, or correlation IDs that need
to survive the pipeline.

The transport headers feature provides:

- A **protocol-neutral abstraction** (`TransportHeader`, `TransportHeaders`)
  that preserves duplicate names, binary values, and original wire names
- A **policy-driven capture engine** for receivers to extract matching headers
- A **policy-driven propagation engine** for exporters to filter headers on
  egress
- **Context integration** so headers flow through processors transparently

Extraction and propagation are explicit and opt-in. By default, no headers are
captured or forwarded.

## Data Flow

```text
    ┌─────────────────────────────────────────────────────────────────────────┐
    │                           Pipeline                                      │
    │                                                                         │
    │  ┌──────────┐      ┌─────────────┐      ┌─────────┐      ┌──────────┐  │
    │  │ Receiver │      │  Processor  │      │Processor│      │ Exporter │  │
    │  │          │      │             │      │         │      │          │  │
    │  │ Capture  │─────>│ (headers    │─────>│  ...    │─────>│Propagate │  │
    │  │ Engine   │      │  pass thru) │      │         │      │ Engine   │  │
    │  └──────────┘      └─────────────┘      └─────────┘      └──────────┘  │
    │       ▲                                                       │         │
    └───────│───────────────────────────────────────────────────────│─────────┘
            │                                                       │
            │                                                       ▼
    ┌───────┴───────┐                                       ┌───────────────┐
    │ Inbound       │                                       │ Outbound      │
    │ - gRPC meta   │                                       │ - gRPC meta   │
    │ - HTTP headers│                                       │ - HTTP headers│
    └───────────────┘                                       └───────────────┘
```

Headers are captured at the receiver, stored in `OtapPdata.Context`, preserved
through all processors (they don't touch routing context), and propagated at the
exporter according to policy.

## 1. Core Types

All core types are defined in `crates/otap/src/transport_headers.rs`.

### TransportHeader

A single captured header with both normalized and original names:

```rust
pub struct TransportHeader {
    /// Normalized logical name used for matching and policy lookup.
    pub name: String,
    /// Original header or metadata name observed on ingress.
    pub wire_name: String,
    /// Whether the value is text or binary.
    pub value_kind: ValueKind,
    /// Raw value bytes.
    pub value: Vec<u8>,
}
```

**Why two names?**

- `name`: Lowercased/normalized for case-insensitive policy matching and
  `store_as` renaming (e.g., `"tenant_id"`)
- `wire_name`: Original casing as seen on ingress, used for lossless round-trip
  (e.g., `"X-Tenant-Id"`)



### TransportHeaders

An ordered collection that preserves duplicate names:

```rust
pub struct TransportHeaders {
    headers: Vec<TransportHeader>,  // Vec, not HashMap!
}
```

**Why `Vec`?** HTTP and gRPC both allow multiple headers with the same name
(e.g., multiple `X-Forwarded-For` entries in a proxy chain). A map would lose
duplicates.

**Key methods:**

```rust
impl TransportHeaders {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    pub fn push(&mut self, header: TransportHeader);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = &TransportHeader>;
    pub fn find_by_name(&self, name: &str) -> impl Iterator<Item = &TransportHeader>;
}
```

## 2. Engines

### CaptureEngine

Applies a `HeaderCapturePolicy` to extract matching headers from inbound
protocol metadata:

```rust
pub struct CaptureEngine {
    policy: HeaderCapturePolicy,
}

impl CaptureEngine {
    pub fn new(policy: HeaderCapturePolicy) -> Self;

    /// Capture headers from any iterator of (wire_name, value) pairs.
    /// Works with gRPC metadata, HTTP headers, or raw key-value sources.
    pub fn capture_from_pairs<'a>(
        &self,
        pairs: impl Iterator<Item = (&'a str, &'a [u8])>,
    ) -> TransportHeaders;
}
```

**Matching logic:**

1. Each inbound header is compared against `match_names` (case-insensitive)
2. If matched, the `store_as` name is used (or the first matched name,
   lowercased)
3. Binary detection: `-bin` suffix or explicit `value_kind` in rule
4. Limits enforced: `max_entries`, `max_name_bytes`, `max_value_bytes`
5. On limit violation: `drop` (skip header) or `reject` (stop capture)

### PropagationEngine

Applies a `HeaderPropagationPolicy` to filter captured headers for egress:

```rust
pub struct PropagationEngine {
    policy: HeaderPropagationPolicy,
}

impl PropagationEngine {
    pub fn new(policy: HeaderPropagationPolicy) -> Self;

    /// Filter captured headers according to policy.
    pub fn propagate(&self, captured: &TransportHeaders) -> TransportHeaders;
}
```

**Resolution order:**

1. Check `overrides` first—if a header matches an override, use that action
2. Otherwise, check if header passes the default `selector`:
   - `all_captured`: all headers selected
   - `none`: no headers selected (overrides can still include specific ones)
   - `named`: only headers in the explicit list
3. Apply `action`: `propagate` or `drop`
4. Apply `name` strategy: `preserve` (use wire_name) or `stored_name` (use
   normalized name)

## 3. Configuration

Policy types are defined in `crates/config/src/transport_headers_policy.rs`.

### TransportHeadersPolicy

Top-level policy containing both capture and propagation rules:

```rust
pub struct TransportHeadersPolicy {
    pub header_capture: HeaderCapturePolicy,
    pub header_propagation: HeaderPropagationPolicy,
}
```

### HeaderCapturePolicy

```yaml
header_capture:
  defaults:
    max_entries: 32        # max headers captured per message (default: 32)
    max_name_bytes: 128    # max header name length (default: 128)
    max_value_bytes: 4096  # max header value length (default: 4096)
    on_error: drop         # drop | reject (default: drop)

  headers:
    - match_names: ["x-tenant-id"]
      store_as: tenant_id        # rename to "tenant_id" (optional)

    - match_names: ["x-request-id"]
      # store_as defaults to first matched name, lowercased

    - match_names: ["authorization"]
      sensitive: true            # hint for logging/debug (default: false)

    - match_names: ["trace-ctx-bin"]
      value_kind: binary         # force binary (default: auto-detect from -bin suffix)
```

### HeaderPropagationPolicy

```yaml
header_propagation:
  default:
    selector: all_captured  # all_captured | none | [named list] (default: all_captured)
    action: propagate       # propagate | drop (default: propagate)
    name: preserve          # preserve | stored_name (default: preserve)
    on_error: drop          # drop | reject (default: drop)

  overrides:
    - match:
        stored_names: ["authorization"]
      action: drop          # drop auth header on egress
```

### Full YAML Example

This example matches the design spec's scenario:

```yaml
version: otel_dataflow/v1

groups:
  default:
    policies:
      transport_headers:
        header_capture:
          defaults:
            max_entries: 32
            max_name_bytes: 128
            max_value_bytes: 4096
            on_error: drop
          headers:
            - match_names: ["x-tenant-id"]
              store_as: tenant_id
            - match_names: ["x-request-id"]
            - match_names: ["authorization"]
              sensitive: true

        header_propagation:
          default:
            selector: all_captured
            action: propagate
            name: preserve
            on_error: drop
          overrides:
            - match:
                stored_names: ["authorization"]
              action: drop

    pipelines:
      ingest:
        nodes:
          otlp_ingest:
            type: receiver:otlp
            config:
              protocols:
                grpc:
                  listening_addr: "0.0.0.0:4317"
              header_capture:    # Node-level policy reference
                headers:
                  - match_names: ["x-tenant-id"]
                    store_as: tenant_id
                  - match_names: ["x-request-id"]
                  - match_names: ["authorization"]

          batch:
            type: processor:batch
            config: {}

          otap_export:
            type: exporter:otap
            config:
              grpc_endpoint: "http://127.0.0.1:50051"
              header_propagation:
                overrides:
                  - match:
                      stored_names: ["authorization"]
                    action: drop

        connections:
          - from: otlp_ingest
            to: batch
          - from: batch
            to: otap_export
```

**Behavior:**

- `otlp_ingest` captures `tenant_id`, `x-request-id`, and `authorization`
- `batch` preserves captured headers unchanged (processor transparency)
- `otap_export` propagates all except `authorization` (dropped by override)

### Integration with Policies Hierarchy

Transport headers policy integrates with the existing `Policies` system in
`crates/config/src/policy.rs`:

```rust
pub struct Policies {
    pub channel_capacity: Option<ChannelCapacityPolicy>,
    pub health: Option<HealthPolicy>,
    pub telemetry: Option<TelemetryPolicy>,
    pub resources: Option<ResourcesPolicy>,
    pub transport_headers: Option<TransportHeadersPolicy>,  // NEW
}

pub struct ResolvedPolicies {
    pub channel_capacity: ChannelCapacityPolicy,
    pub health: HealthPolicy,
    pub telemetry: TelemetryPolicy,
    pub resources: ResourcesPolicy,
    pub transport_headers: Option<TransportHeadersPolicy>,  // None = feature disabled
}
```

The engine observability pipeline (internal telemetry) explicitly sets
`transport_headers: None` since it doesn't need header propagation
(`crates/config/src/engine.rs`).

## 4. Context Integration

Transport headers are stored in `OtapPdata.Context`
(`crates/otap/src/pdata.rs`):

```rust
pub struct Context {
    /// Ack/Nack routing stack (reset at transport boundaries)
    stack: Vec<Frame>,
    /// Transport headers (PRESERVED at transport boundaries)
    transport_headers: Option<TransportHeaders>,
}
```

**Why `Option`?**

- When no capture policy is configured (the common case), no allocation occurs
- Zero overhead for pipelines that don't use this feature

### Access Methods

On `Context`:

```rust
impl Context {
    pub fn transport_headers(&self) -> Option<&TransportHeaders>;
    pub fn set_transport_headers(&mut self, headers: TransportHeaders);
}
```

On `OtapPdata` (convenience wrappers):

```rust
impl OtapPdata {
    pub fn transport_headers(&self) -> Option<&TransportHeaders>;
    pub fn set_transport_headers(&mut self, headers: TransportHeaders);
    pub fn with_transport_headers(self, headers: TransportHeaders) -> Self;
}
```

## 6. Tests

All tests are in `crates/otap/src/transport_headers.rs` and
`crates/config/src/transport_headers_policy.rs`.

| Category | Tests | Coverage |
| -------- | ----- | -------- |
| **Core types** | 3 | duplicates, find_by_name, value_as_str |
| **CaptureEngine** | 6 | Empty policy, matching, case-insensitive, max_entries, oversized values, binary detection |
| **PropagationEngine** | 5 | All captured, override drops auth, selector none, stored_name strategy, named selector |
| **End-to-end** | 3 | Full lifecycle, duplicate preservation, binary preservation |
| **Policy serde** | 5 | Defaults, capture roundtrip, propagation roundtrip, full policy, named selector |

**Key end-to-end test:** `end_to_end_capture_preserve_propagate`

This test simulates the complete pipeline flow:

1. Capture headers from mock gRPC metadata (including `authorization`)
2. Attach to `OtapPdata` via `with_transport_headers()`
3. Simulate processor pass-through via `clone_without_context()`
4. Apply propagation policy that drops `authorization`
5. Verify propagated headers exclude sensitive data

## 7. File Reference

### New Files

| File | Purpose |
| ---- | ------- |
| `config/src/transport_headers_policy.rs` | Policy types: `TransportHeadersPolicy`, `HeaderCapturePolicy`, `HeaderPropagationPolicy` |
| `otap/src/transport_headers.rs` | Core types (`TransportHeader`, `TransportHeaders`) + engines (`CaptureEngine`, `PropagationEngine`) |

### Modified Files

| File | Purpose |
| ---- | ------- |
| `config/src/lib.rs` | Export `transport_headers_policy` module |
| `config/src/policy.rs` | Add `transport_headers` to `Policies` and `ResolvedPolicies` |
| `config/src/engine.rs` | Set `transport_headers: None` for observability pipelines |
| `otap/src/lib.rs` | Export `transport_headers` module |
| `otap/src/pdata.rs` | Add `transport_headers` field to `Context`, accessor methods, preserve in `clone_without_context()` |
