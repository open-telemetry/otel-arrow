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

Point the exporter at an OTAP-compatible gRPC receiver:

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

#### `exporter.pdata`

| Metric | Unit | Description |
| --- | --- | --- |
| `exporter.pdata.metrics_consumed` | `{msg}` | Number of pdata metrics consumed by this exporter. |
| `exporter.pdata.metrics_exported` | `{msg}` | Number of pdata metrics successfully exported. |
| `exporter.pdata.metrics_failed` | `{msg}` | Number of pdata metrics that failed to be exported. |
| `exporter.pdata.logs_consumed` | `{msg}` | Number of pdata logs consumed by this exporter. |
| `exporter.pdata.logs_exported` | `{msg}` | Number of pdata logs successfully exported. |
| `exporter.pdata.logs_failed` | `{msg}` | Number of pdata logs that failed to be exported. |
| `exporter.pdata.traces_consumed` | `{msg}` | Number of pdata traces consumed by this exporter. |
| `exporter.pdata.traces_exported` | `{msg}` | Number of pdata traces successfully exported. |
| `exporter.pdata.traces_failed` | `{msg}` | Number of pdata traces that failed to be exported. |

#### `otap.exporter.grpc.async`

| Metric | Unit | Description |
| --- | --- | --- |
| `otap.exporter.grpc.async.export.rpc.duration` | `ns` | End-to-end duration from yielding a batch to receiving the matching OTAP stream response. |
| `otap.exporter.grpc.async.stream.enqueue.duration` | `ns` | Time spent waiting to enqueue a batch into the per-signal stream task. |
| `otap.exporter.grpc.async.stream.enqueue.depth` | `{batch}` | Occupancy of the per-signal stream task queue before enqueueing a batch. |
| `otap.exporter.grpc.async.stream.encode.duration` | `ns` | Time spent encoding an OTAP batch into outbound Arrow batch records. |
| `otap.exporter.grpc.async.stream.correlation.enqueue.duration` | `ns` | Time spent enqueueing a yielded batch into the response correlation queue. |
| `otap.exporter.grpc.async.stream.correlation.depth` | `{batch}` | Occupancy of the response correlation queue before enqueueing a yielded batch. |
| `otap.exporter.grpc.async.stream.response.wait.duration` | `ns` | Time spent waiting for the next server response on an OTAP stream. |
| `otap.exporter.grpc.async.stream.response.inflight` | `{batch}` | Number of yielded batches awaiting a matching server response. |
| `otap.exporter.grpc.async.export.rpc.duration.p50` | `ns` | Median outbound gRPC export response duration for the latest telemetry interval. |
| `otap.exporter.grpc.async.export.rpc.duration.p90` | `ns` | 90th percentile outbound gRPC export response duration for the latest telemetry interval. |
| `otap.exporter.grpc.async.export.rpc.duration.p99` | `ns` | 99th percentile outbound gRPC export response duration for the latest telemetry interval. |

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
- Compression values are limited to the variants supported by the shared OTAP
  transport layer.
- End-to-end delivery depends on downstream OTAP stream responses.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Proxy support](../../../../../docs/proxy-support.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Core node catalog](../../../README.md)
