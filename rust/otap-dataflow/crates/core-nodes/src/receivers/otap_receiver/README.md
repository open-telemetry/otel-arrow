# OTAP Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:receiver:otap`
- Type shortcut: `receiver:otap`
- Feature gate: Default
- Stability: Experimental

## Overview

The OTAP receiver accepts OTAP Arrow streams over gRPC and forwards received
payloads into the pipeline as `OtapPdata`. It can wait for immediate downstream
ACK/NACK outcomes before responding to the client.

## Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `listening_addr` | socket address | Required | Address for the OTAP gRPC server. |
| `compression_method` | enum | unset | Accepted or preferred OTAP compression setting. |
| `response_stream_channel_size` | integer | Required | Response stream channel capacity. |
| `max_concurrent_requests` | integer | `1000` | Global in-flight request limit. |
| `max_concurrent_requests_per_stream` | non-zero integer | `16` | Per-stream in-flight request limit. |
| `wait_for_result` | bool | `true` | Wait for immediate downstream outcome. |
| `timeout` | duration | unset | Optional RPC timeout. |
| `tls` | object | unset | Server TLS or mTLS settings. |

## Examples

```yaml
type: receiver:otap
config:
  listening_addr: "127.0.0.1:4317"
  response_stream_channel_size: 128
  max_concurrent_requests: 1000
  wait_for_result: true
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.otap`

| Metric | Unit | Description |
| --- | --- | --- |
| `receiver.otap.acks_sent` | `{acks}` | Number of acks sent. |
| `receiver.otap.nacks_sent` | `{nacks}` | Number of nacks sent. |
| `receiver.otap.acks_nacks_invalid_or_expired` | `{ack_or_nack}` | Number of invalid/expired acks/nacks. |
| `receiver.otap.rejected_requests` | `{requests}` | Number of OTAP RPCs rejected before entering the pipeline. |
| `receiver.otap.refused_memory_pressure` | `{requests}` | Number of OTAP RPCs rejected specifically because memory pressure was active. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otap_receiver.start` | `info` | Receiver startup with listening address and enabled signal services. |
| `otap_receiver.drain_ingress` | `info` | Receiver ingress drain started. |
| `otap_receiver.shutdown` | `info` | Receiver shutdown completed. |

## Limits

- `max_concurrent_requests_per_stream` must be greater than zero and is clamped
  to `max_concurrent_requests` at runtime.
- `wait_for_result: false` suppresses downstream failures from client
  responses.
- The server accepts OTAP Arrow stream clients, not standard OTLP clients.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Core node catalog](../../../README.md)
