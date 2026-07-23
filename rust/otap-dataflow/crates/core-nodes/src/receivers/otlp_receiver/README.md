# OTLP Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:otlp` (`urn:otel:receiver:otlp`)
- Feature gate: Default
- Stability: Experimental

## Overview

The OTLP receiver accepts OTLP/gRPC, OTLP/HTTP, or both. It forwards received
logs, metrics, and traces into the pipeline as pdata and can wait for immediate
downstream ACK/NACK outcomes before responding to clients.

## Getting Started

Enable at least one OTLP protocol. For gRPC only:

```yaml
type: receiver:otlp
config:
  protocols:
    grpc:
      listening_addr: "127.0.0.1:4317"
```

## Configuration

```yaml
type: receiver:otlp
config:
  # At least one protocol must be configured.
  protocols:
    grpc:
      # Enables and configures OTLP/gRPC.
      listening_addr: "127.0.0.1:4317"
      wait_for_result: true
      timeout: 30s
    http:
      # Enables and configures OTLP/HTTP.
      listening_addr: "127.0.0.1:4318"
      wait_for_result: true
      timeout: 30s
```

Common gRPC protocol fields include:

- `listening_addr`
- `request_compression`
- `response_compression`
- `max_concurrent_requests`
- `max_concurrent_streams`
- TCP and HTTP/2 tuning fields
- `wait_for_result`
- `timeout`
- `tls`

Common HTTP protocol fields include:

- `listening_addr`
- `max_concurrent_requests`
- `max_request_body_size`
- `wait_for_result`
- `timeout`
- `accept_compressed_requests`
- `tls`

## Examples

gRPC and HTTP:

```yaml
type: receiver:otlp
config:
  protocols:
    grpc:
      listening_addr: "127.0.0.1:4317"
    http:
      listening_addr: "127.0.0.1:4318"
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.otlp.requests`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otlp.requests.started` | `{request}` | `signal`, `protocol` | Number of requests admitted to the pipeline send path. |
| `receiver.otlp.requests.completed` | `{request}` | `signal`, `protocol` | Number of admitted requests whose receiver work terminated. |
| `receiver.otlp.requests.payload_size` | `By` | `signal`, `protocol` | Decompressed payload bytes for requests admitted to the pipeline send path. |

#### `receiver.otlp.rejections`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otlp.rejections.requests` | `{request}` | `protocol`, `error.type` | Number of requests rejected before pipeline admission. |

#### `receiver.otlp.acknowledgements`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otlp.acknowledgements.responses` | `{response}` | `signal`, `outcome` | Number of routed or invalid acknowledgement responses. |

#### `receiver.otlp.transport`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.otlp.transport.errors` | `{error}` | `protocol` | Number of transport-level server errors. |

Attribute values are bounded: `signal` is `traces`, `metrics`, or `logs`;
`protocol` is `grpc` or `http`; `outcome` is `success`, `failure`, or
`refused`; and `error.type` is `memory_pressure`, `concurrency_limit`,
`rate_limit`, `payload_too_large`, `invalid_request`, or `internal`.

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otlp.receiver.grpc.start` | `info` | OTLP/gRPC listener startup. |
| `otlp.receiver.http.start` | `info` | OTLP/HTTP listener startup. |
| `otlp.receiver.drain_ingress` | `info` | Receiver ingress drain started. |
| `otlp.receiver.shutdown` | `info` | Receiver shutdown completed. |

## Limits

- At least one of `protocols.grpc` or `protocols.http` is required.
- HTTP request body limits apply to both compressed and decompressed payload
  size.
- `wait_for_result` reflects the immediate downstream node, not necessarily the
  final exporter.

## Related Docs

- [OTLP receiver design](../../../../../docs/otlp-receiver.md)
- [Configuration model](../../../../../docs/configuration-model.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Core node catalog](../../../README.md)
