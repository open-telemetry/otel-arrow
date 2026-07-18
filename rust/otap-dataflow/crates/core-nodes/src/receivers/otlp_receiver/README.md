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

#### `receiver.otlp`

| Metric | Unit | Description |
| --- | --- | --- |
| `receiver.otlp.acks_received` | `{acks}` | Number of acks received from downstream (routed back to the caller). |
| `receiver.otlp.nacks_received` | `{nacks}` | Number of nacks received from downstream (routed back to the caller). |
| `receiver.otlp.acks_nacks_invalid_or_expired` | `{ack_or_nack}` | Number of invalid/expired acks/nacks. |
| `receiver.otlp.requests_started` | `{requests}` | Number of OTLP RPCs started. |
| `receiver.otlp.requests_completed` | `{requests}` | Number of OTLP RPCs completed (success + nack). |
| `receiver.otlp.rejected_requests` | `{requests}` | Number of OTLP RPCs rejected before entering the pipeline (e.g. slot exhaustion). |
| `receiver.otlp.refused_memory_pressure` | `{requests}` | Number of OTLP RPCs rejected specifically because process-wide memory pressure was active. |
| `receiver.otlp.refused_rate_limit` | `{requests}` | Number of OTLP RPCs rejected specifically because pressure-aware rate limiting refused the request. |
| `receiver.otlp.would_refuse_rate_limit` | `{requests}` | Number of OTLP RPCs that would have been rejected by pressure-aware rate limiting in enforce mode, but were admitted in observe-only mode. |
| `receiver.otlp.transport_errors` | `{errors}` | Number of transport-level errors surfaced by tonic/server. |
| `receiver.otlp.request_bytes` | `By` | Total decompressed payload bytes for successfully received OTLP requests. |

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
