# OTLP HTTP Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:otlp_http` (`urn:otel:exporter:otlp_http`)
- Feature gate: Default
- Stability: Experimental

## Overview

The OTLP HTTP exporter sends logs, metrics, and traces to OTLP/HTTP endpoints.
It uses `/v1/logs`, `/v1/metrics`, and `/v1/traces` paths derived from
`endpoint` unless a signal-specific endpoint override is provided.

## Getting Started

Point the exporter at an OTLP/HTTP base endpoint:

```yaml
type: exporter:otlp_http
config:
  endpoint: "http://127.0.0.1:4318"
  client_pool_size: 1
  http:
    compression: gzip
  max_in_flight: 8
```

## Configuration

```yaml
type: exporter:otlp_http
config:
  # Base OTLP/HTTP endpoint without the signal path (required).
  endpoint: "http://127.0.0.1:4318"

  # Full signal-specific endpoint overrides (optional).
  traces_endpoint: "http://traces.example.test:4318/v1/traces"
  metrics_endpoint: "http://metrics.example.test:4318/v1/metrics"
  logs_endpoint: "http://logs.example.test:4318/v1/logs"

  # Maximum response body size in bytes (default: 10485760).
  max_response_body_length: 10485760

  # Number of HTTP clients in the pool (required, must be non-zero).
  client_pool_size: 1

  # Maximum concurrent export requests (default: 5).
  max_in_flight: 8

  # Shared HTTP client settings (required).
  http:
    compression: gzip

    # Static headers added to every outbound OTLP/HTTP request (optional).
    # Useful for authentication or backend routing. Protocol headers
    # (Content-Type / Content-Encoding / Content-Length / Host) cannot be set
    # here and are rejected at config load.
    headers:
      authorization: "Basic <base64(user:password)>"
      x-scope-orgid: "tenant-1"
```

Shared HTTP client fields include concurrency limit, connect timeout, request
timeout, TCP keepalive, TLS, request-body compression, and static request
`headers`.

### Static request headers

`http.headers` is an ordered map of header name to value applied to every
outbound request (auth tokens, multi-tenant routing, tracing-vendor headers).
Values are sent verbatim, so treat secrets in the rendered config as sensitive.

Validation at config load rejects:

- invalid header names (must be valid HTTP token characters), and
- invalid header values (must be visible ASCII), and
- protocol-reserved names: `content-type`, `content-encoding`,
  `content-length`, and `host`.

Protocol headers always take precedence over configured headers.

## Examples

With one signal-specific URL:

```yaml
type: exporter:otlp_http
config:
  endpoint: "http://127.0.0.1:4318"
  logs_endpoint: "http://logs.example.test:4318/v1/logs"
  client_pool_size: 2
  http: {}
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
| `otlp.exporter.http.validate_insecure_flag` | `warn` | The HTTP exporter ignored a TLS-only insecure flag for an HTTP endpoint. |
| `otlp.exporter.http.start` | `info` | Exporter startup with the configured HTTP endpoint. |
| `otlp.exporter.http.receive` | `debug` | A pdata batch was received by the exporter loop. |
| `otlp.exporter.http.shutdown` | `info` | Exporter shutdown and terminal reason. |
| `otlp.exporter.http.zero_partial_rejected` | `debug` | A zero-length partial-success response was rejected. |
| `otlp.exporter.http.export_error` | `warn` | An HTTP export request did not complete successfully. |

## Limits

- `client_pool_size` must be non-zero.
- Signal-specific endpoint fields must include the full OTLP path.
- Response bodies larger than `max_response_body_length` fail the export.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Proxy support](../../../../../docs/proxy-support.md)
- [Core node catalog](../../../README.md)
