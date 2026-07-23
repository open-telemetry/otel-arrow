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
    # Useful for arbitrary headers such as backend routing or multi-tenant
    # tenant IDs. Not recommended for authorization; prefer a dedicated Auth
    # extension instead. Protocol headers (Content-Type / Content-Encoding /
    # Content-Length / Host) and response-negotiation headers (Accept /
    # Accept-Encoding) cannot be set here and are rejected at config load.
    headers:
      x-scope-orgid: "tenant-1"
      environment: "production-west"
```

Shared HTTP client fields include concurrency limit, connect timeout, request
timeout, TCP keepalive, TLS, request-body compression, and static request
`headers`.

### Static request headers

`http.headers` is a map of header name to value applied to every outbound
request (multi-tenant routing IDs, tracing-vendor headers, and similar). For
request authentication, prefer the `bearer_token_provider` capability (see
[Authentication](#authentication)) rather than hard-coding an `authorization`
header here. Values are sent verbatim, so treat any secret in the rendered
config as sensitive.

Validation at config load rejects:

- invalid header names (must be valid HTTP token characters), and
- invalid header values (must be visible ASCII), and
- protocol-reserved names managed by the exporter: `content-type`,
  `content-encoding`, `content-length`, and `host`, and
- response-negotiation names dictated by the client's decode capabilities:
  `accept` and `accept-encoding`.

Protocol headers always take precedence over configured headers.

## Authentication

The exporter can inject an OAuth `Authorization: Bearer <token>` on every
outbound request by consuming the `bearer_token_provider` capability. Binding is
optional and additive: without it the exporter sends no `authorization` header
(the default); with it, the bound extension acquires and refreshes the token in
the background so credentials rotate without restarting the exporter.

Declare a provider extension (e.g.
[`azure_identity_auth`](../../../../contrib-extensions/src/azure_identity_auth/README.md))
in the pipeline's `extensions:` section and bind it on the exporter node via the
node's `capabilities:` map:

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          azure_identity:
            type: "urn:microsoft:extension:azure_identity_auth"
            config:
              method: managed_identity
              scope: "https://monitor.azure.com/.default"

        nodes:
          otlp-http-exporter:
            type: "urn:otel:exporter:otlp_http"
            # Bind the bearer token provider to the extension declared above.
            capabilities:
              bearer_token_provider: azure_identity
            config:
              endpoint: "https://my-endpoint:4318"
              client_pool_size: 1
              http: {}
```

The bearer token is applied per request, so it takes precedence over any
statically configured `authorization` header. The exporter subscribes to the
provider's token stream and caches the built header, rebuilding it only when the
provider refreshes the token, so credential work stays off the per-request path.
When no usable token is cached yet -- before the provider's first publish, or in
a degraded window where a refresh is failing and the cached token is within a
small safety margin of expiring -- the exporter **stops accepting new batches**
(back-pressures upstream) rather than sending an unauthenticated or soon-to-lapse
request. It resumes as soon as a usable token arrives; nothing is dropped. (If
buffered batches are force-drained during shutdown while no token is available,
they are NACK'd as **retryable**.) A token is guaranteed to eventually arrive:
the `azure_identity_auth` extension holds data-path startup until its first token
publish, and its token stream stays live for the exporter's lifetime.

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

Input PData message volume is reported by the engine through
`channel.receiver.recv.count` on the PData input channel and is not duplicated
by the exporter.

#### `exporter.pdata.exports`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.pdata.exports.messages` | `{message}` | `signal`, `outcome` | Number of PData messages whose export reached a terminal outcome. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otlp.exporter.http.validate_insecure_flag` | `warn` | The HTTP exporter ignored a TLS-only insecure flag for an HTTP endpoint. |
| `otlp.exporter.http.start` | `info` | Exporter startup with the configured HTTP endpoint. |
| `otlp.exporter.http.receive` | `debug` | A pdata batch was received by the exporter loop. |
| `otlp.exporter.http.shutdown` | `info` | Exporter shutdown and terminal reason. |
| `otlp.exporter.http.zero_partial_rejected` | `debug` | A zero-length partial-success response was rejected. |
| `otlp.exporter.http.export_error` | `warn` | An HTTP export request did not complete successfully. |
| `otlp.exporter.http.invalid_bearer_token` | `warn` | A bearer token from the provider could not be turned into a valid `Authorization` header. |
| `otlp.exporter.http.token_stream_closed` | `warn` | The bearer token provider closed its refresh stream; the last token (if any) is reused and no longer refreshes. |

## Limits

- `client_pool_size` must be non-zero.
- Signal-specific endpoint fields must include the full OTLP path.
- Response bodies larger than `max_response_body_length` fail the export.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Proxy support](../../../../../docs/proxy-support.md)
- [Core node catalog](../../../README.md)
