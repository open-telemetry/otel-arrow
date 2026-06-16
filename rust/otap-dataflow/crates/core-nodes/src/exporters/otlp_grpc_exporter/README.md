# OTLP gRPC Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:otlp_grpc` (`urn:otel:exporter:otlp_grpc`)
- Feature gate: Default
- Stability: Experimental

## Overview

The OTLP gRPC exporter sends logs, metrics, and traces as unary OTLP export
requests. It converts OTAP records to OTLP protobuf bytes when needed and
propagates request success or failure back into the dataflow ACK/NACK path.

## Getting Started

Point the exporter at an OTLP/gRPC endpoint:

```yaml
type: exporter:otlp_grpc
config:
  grpc_endpoint: "http://127.0.0.1:4317"
  max_in_flight: 8
  num_connections: 1
```

## Configuration

The config embeds shared gRPC client settings and adds exporter concurrency
settings.

```yaml
type: exporter:otlp_grpc
config:
  # gRPC endpoint to connect to (required).
  grpc_endpoint: "http://127.0.0.1:4317"

  # Optional outbound request compression.
  compression: gzip

  # Maximum concurrent export RPCs (default: 5).
  max_in_flight: 8

  # Number of gRPC channels to open (default: 1).
  num_connections: 1

  # Static metadata (headers) added to every outbound OTLP/gRPC request
  # (optional). Useful for authentication or tenant routing. Keys and values
  # must be valid ASCII gRPC metadata and are validated at config load.
  headers:
    authorization: "Basic <base64(user:password)>"
    x-scope-orgid: "tenant-1"
```

Shared gRPC client fields include connect timeout, request timeout, TCP
keepalive, HTTP/2 settings, TLS, proxy, and transport buffer settings.

### Static request headers

`headers` is a map of metadata name to value added to every outbound
request (auth tokens, multi-tenant routing, tracing-vendor metadata). Values are
sent verbatim, so treat secrets in the rendered config as sensitive.

Validation at config load rejects:

- invalid metadata names (must be a valid ASCII gRPC metadata key: an HTTP/2
  token that is sent lowercased and must not end in `-bin`, which is reserved
  for binary metadata), and
- invalid metadata values (must be visible ASCII).

When [header propagation](../../../../../docs/transport-headers.md) is also
enabled, statically configured headers take precedence: a propagated header
whose key matches a configured one is dropped, so a configured backend
credential (e.g. `authorization`) is never overridden or duplicated.

## Examples

With request compression:

```yaml
type: exporter:otlp_grpc
config:
  grpc_endpoint: "http://127.0.0.1:4317"
  compression: gzip
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

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otlp.exporter.grpc.start` | `info` | Exporter startup with the configured gRPC endpoint. |
| `otlp.exporter.grpc.channels` | `info` | gRPC channel pool creation with connection count and endpoint. |
| `otlp.exporter.grpc.receive` | `debug` | A pdata batch was received by the exporter loop. |
| `otlp.exporter.grpc.shutdown` | `info` | Exporter shutdown. |
| `otlp.exporter.http.export_error` | `warn` | A gRPC export request did not complete successfully. |
| `otlp.exporter.grpc.header_skip` | `debug` | A propagated transport header was skipped while building gRPC metadata. |

## Limits

- `max_in_flight` bounds concurrent export RPCs inside the node.
- `num_connections` only improves distribution when the downstream endpoint can
  balance separate connections.
- OTLP partial success responses are treated as export failures by the current
  implementation.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Proxy support](../../../../../docs/proxy-support.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Core node catalog](../../../README.md)
