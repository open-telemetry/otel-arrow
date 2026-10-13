# OTAP Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:otap` (`urn:otel:receiver:otap`)
- Feature gate: `otap`
- Stability: Experimental

## Overview

The OTAP receiver accepts OTAP Arrow streams over gRPC and forwards received
payloads into the pipeline as `OtapPdata`. It can wait for immediate downstream
ACK/NACK outcomes before responding to the client.

## Getting Started

Listen for OTAP Arrow stream clients on a gRPC address:

```yaml
type: receiver:otap
config:
  listening_addr: "127.0.0.1:4317"
  response_stream_channel_size: 128
```

## Configuration

The OTAP receiver shares the common gRPC server settings
(`GrpcServerSettings`) with the OTLP receiver. Those fields are configured
inline alongside the OTAP-specific fields below.

```yaml
type: receiver:otap
config:
  # Address for the OTAP gRPC server (required).
  listening_addr: "127.0.0.1:4317"

  # Compression methods accepted for requests (optional). A single value, a
  # list, or "none". When omitted, zstd, gzip, and deflate are all accepted.
  request_compression: zstd

  # Compression methods offered for responses (optional). A single value, a
  # list, or "none". When omitted, responses are sent uncompressed.
  response_compression: zstd

  # Response stream channel capacity (required). Bounds the per-stream buffer of
  # outgoing batch-status responses; a slow client applies backpressure here.
  response_stream_channel_size: 128

  # Global in-flight request limit (optional). Defaults to 0, which adopts the
  # downstream pdata channel capacity; larger values are clamped to it.
  max_concurrent_requests: 0

  # Per-stream in-flight request limit (default: 16).
  max_concurrent_requests_per_stream: 16

  # Wait for immediate downstream outcome (default: false).
  wait_for_result: false

  # Upper bound on concurrently active HTTP/2 streams per connection (optional).
  # Unset (or 0) means no limit. See the warning in "Limits" before setting this
  # to an explicit value
  max_concurrent_streams: 0

  # Optional RPC timeout.
  timeout: 30s

  # Optional server TLS or mTLS settings.
  # tls:
  #   cert_file: /path/to/server.crt
  #   key_file: /path/to/server.key
```

Other shared gRPC server fields (for example `max_decoding_message_size` and the
TCP/HTTP2 tuning fields) are also accepted; see the
[OTLP receiver](../otlp_receiver/README.md) for the full list.

### Deprecated fields

- `compression_method`: deprecated alias that applies a single method (or
  `"none"`) to both `request_compression` and `response_compression`. It cannot
  be combined with either new field. Prefer `request_compression` and
  `response_compression`.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.otap.batches`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otap.batches.started` | `{batch}` | `signal` | Number of batches admitted to the pipeline send path. |
| `receiver.otap.batches.completed` | `{batch}` | `signal` | Number of admitted batches whose receiver work terminated. |
| `receiver.otap.batches.payload_size` | `By` | `signal` | Protobuf-encoded batch bytes after gRPC transport decompression. |

#### `receiver.otap.acknowledgements`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otap.acknowledgements.responses` | `{response}` | `signal`, `outcome` | Number of routed or invalid acknowledgement responses. |

`signal` is one of `traces`, `metrics`, or `logs`. `outcome` is `success` for
an ACK sent to the client, `refused` for a NACK sent to the client, and
`failure` when the acknowledgement route was invalid or expired.

#### `receiver.otap.rejections`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otap.rejections.streams` | `{stream}` | `error.type` | Number of OTAP streaming RPCs rejected before pipeline admission. |
| `receiver.otap.rejections.batches` | `{batch}` | `error.type` | Number of OTAP batches rejected within admitted streams. |

The OTAP receiver emits the bounded `error.type` values `memory_pressure`,
`concurrency_limit`, and `invalid_request`.

Fatal gRPC serving failures are surfaced through component failure diagnostics,
not a terminal-only metric that cannot be handed off on an error return.

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otap_receiver.start` | `info` | Receiver startup with listening address and enabled signal services. |
| `otap_receiver.drain_ingress` | `info` | Receiver ingress drain started. |
| `otap_receiver.shutdown` | `info` | Receiver shutdown completed. |

## Limits

- `max_concurrent_requests_per_stream` must be greater than zero and is clamped
  to `max_concurrent_requests` at runtime.
- `max_concurrent_requests` defaults to the downstream pdata channel capacity
  (configured value `0`); any larger configured value is clamped to it.
- `max_concurrent_streams` is the HTTP/2 per-connection stream limit. OTAP clients
  hold one long-lived stream per signal (3 total), and `3 * streams_per_signal`
  when an OTAP exporter sets `streams_per_signal > 1`. A limit below that count can
  silently stall exporters, whose extra streams wait forever to open. The OTAP
  receiver therefore leaves it **uncapped by default** (unset or `0`), unlike the
  OTLP receiver. If you set it, use at least `3 * streams_per_signal`; the receiver
  logs a warning when it is set below `3`.
- `wait_for_result` defaults to `false`; `wait_for_result: false` suppresses
  downstream failures from client responses.
- The server accepts OTAP Arrow stream clients, not standard OTLP clients.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Core node catalog](../../../README.md)
