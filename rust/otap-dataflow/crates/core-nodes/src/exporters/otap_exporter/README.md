# OTAP Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:otap` (`urn:otel:exporter:otap`)
- Feature gate: Default
- Stability: Experimental

## Overview

The OTAP exporter sends OTAP Arrow payloads over gRPC streams to an
OTAP-compatible receiver. It maintains independent per-signal stream tasks and
correlates stream responses back to upstream ACK/NACK handling.

## Getting Started

Point the exporter at an OTAP-compatible gRPC endpoint:

```yaml
type: exporter:otap
config:
  grpc_endpoint: "http://127.0.0.1:4317"
  compression_method: zstd
  stream_queue_capacity: 64
  streams_per_signal: 1
```

## Configuration

The config embeds shared gRPC client settings and adds OTAP stream options.

```yaml
type: exporter:otap
config:
  # gRPC endpoint to connect to (required).
  grpc_endpoint: "http://127.0.0.1:4317"

  # gRPC request compression from shared client settings (optional).
  compression: gzip

  # Legacy OTAP gRPC compression field (default: zstd; use "none" to disable).
  compression_method: zstd

  arrow:
    # Arrow IPC payload compression (default: zstd; use "none" to disable).
    payload_compression: zstd

  # Per-signal queue capacity feeding stream tasks (default: 64).
  stream_queue_capacity: 64

  # Number of streams opened for each signal type (default: 1).
  streams_per_signal: 1

  # Optional static gRPC request headers (metadata) attached once as the
  # initial metadata of each Arrow stream (e.g. auth or tenant routing).
  headers:
    authorization: "Basic dXNlcjpwYXNz"
    x-scope-orgid: "tenant-1"

  # Optional RPC timeout.
  timeout: 30s
```

Shared gRPC client fields include connection timeout, TCP keepalive, HTTP/2
window settings, TLS, proxy, and transport buffer settings.

## Examples

Disable OTAP and Arrow payload compression:

```yaml
type: exporter:otap
config:
  grpc_endpoint: "http://127.0.0.1:4317"
  compression_method: none
  arrow:
    payload_compression: none
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

Input PData message volume is reported by the engine through
`channel.receiver.recv.count` on the PData input channel and is not duplicated
by the exporter.

#### `exporter.pdata.exports`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.pdata.exports.messages` | `{message}` | `signal`, `outcome` | Number of PData messages whose export reached a terminal outcome. |

#### `exporter.otap.exports`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.otap.exports.duration` | `ns` | `signal`, `outcome` | End-to-end duration from yielding a batch to its terminal OTAP stream response. |
| `exporter.otap.exports.duration.p50` | `ns` | `signal`, `outcome` | Median export response duration for the latest telemetry interval. |
| `exporter.otap.exports.duration.p90` | `ns` | `signal`, `outcome` | 90th percentile export response duration for the latest telemetry interval. |
| `exporter.otap.exports.duration.p99` | `ns` | `signal`, `outcome` | 99th percentile export response duration for the latest telemetry interval. |

`signal` is one of `traces`, `metrics`, or `logs`. The exporter emits the
terminal `outcome` values `success` and `failure`.

#### `exporter.otap.streams`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.otap.streams.enqueue.duration` | `ns` | `signal` | Time spent waiting to enqueue a batch into a per-signal stream task. |
| `exporter.otap.streams.enqueue.depth` | `{batch}` | `signal` | Occupancy of the per-signal stream task queue before enqueueing a batch. |
| `exporter.otap.streams.encode.duration` | `ns` | `signal` | Time spent encoding an OTAP batch into outbound Arrow batch records. |
| `exporter.otap.streams.correlation.enqueue.duration` | `ns` | `signal` | Time spent enqueueing a yielded batch into the response correlation queue. |
| `exporter.otap.streams.correlation.depth` | `{batch}` | `signal` | Occupancy of the response correlation queue before enqueueing a yielded batch. |
| `exporter.otap.streams.response.wait.duration` | `ns` | `signal` | Time spent waiting for the next server response on an OTAP stream. |
| `exporter.otap.streams.response.inflight` | `{batch}` | `signal` | Number of yielded batches awaiting a matching server response. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otap_exporter.start` | `info` | Exporter startup with the configured OTAP endpoint. |
| `otap_exporter.shutdown` | `info` | Exporter shutdown and terminal reason. |
| `otap_exporter.request_failed` | `error` | An OTAP export request failed before a batch status could be handled. |
| `otap_exporter.batch_status_failed` | `warn` | A returned OTAP batch status indicated failure. |
| `otap_exporter.batch_status_unmatched` | `warn` | A returned OTAP batch status could not be matched to an in-flight batch. |
| `otap_exporter.response_stream_failed` | `warn` | The OTAP response stream failed after connection. |

## Limits

- `stream_queue_capacity` and `streams_per_signal` must be greater than zero.
- Static request `headers` are attached once, as the initial metadata of each
  Arrow stream when it is opened (or re-opened after a reconnect). They are not
  re-sent per `BatchArrowRecords`, so the per-message hot path stays
  allocation-free; the one-time cost is a single metadata clone per stream
  establishment. Header names/values are validated at config load via the shared
  `GrpcClientSettings` (ASCII-only, no gRPC-reserved metadata).
  - Because the metadata template is built once at exporter startup, a changed
    `headers` value (e.g. a rotated `authorization` or tenant header) only takes
    effect when the exporter is restarted/reconfigured and not on stream
    reconnects, which reuse the template captured at startup. This differs from
    the OTLP/gRPC exporter, which rebuilds metadata per unary request.
- Compression values are limited to the variants supported by the shared OTAP
  transport layer.
- End-to-end delivery depends on downstream OTAP stream responses.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Proxy support](../../../../../docs/proxy-support.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Core node catalog](../../../README.md)
