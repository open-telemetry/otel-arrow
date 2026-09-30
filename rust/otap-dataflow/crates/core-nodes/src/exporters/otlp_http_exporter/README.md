# OTLP HTTP Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:otlp_http` (`urn:otel:exporter:otlp_http`)
- Feature gate: `otlp`
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
request authentication, prefer one of the provider capabilities described in
[Authentication](#authentication) rather than hard-coding credentials here.
Values are sent verbatim, so treat any secret in the rendered config as
sensitive.

Validation at config load rejects:

- invalid header names (must be valid HTTP token characters), and
- invalid header values (must be visible ASCII), and
- protocol-reserved names managed by the exporter: `content-type`,
  `content-encoding`, `content-length`, and `host`, and
- response-negotiation names dictated by the client's decode capabilities:
  `accept` and `accept-encoding`.

Protocol headers always take precedence over configured headers.

## Authentication

By default the exporter sends requests without any authentication.
Authentication can be enabled by binding a [provider extension](../../../../contrib-extensions/README.md) to the exporter node
via its `capabilities` map. The following providers are supported:

- [`BearerTokenProvider`](#bearertokenprovider): Provides authentication via `Authorization: Bearer
  <token>` HTTP header.
- [`ApiKeyProvider`](#apikeyprovider): Provides authentication via a custom HTTP header in the form
  `<header_name>: <optional_scheme> <api_key>`.
- [`BasicAuthProvider`](#basicauthprovider): Provides authentication via `Authorization: Basic
  <base64-encoded(username:password)>` HTTP header.
- [`AgentFedCredentialProvider`](#agentfedcredentialprovider): Provides authentication using a credential
  snapshot published by the embedding host as `Authorization: Bearer <token>`
  HTTP header.

> [!IMPORTANT]
> Only one authentication provider can be bound to the exporter node at a time.
> If multiple providers are bound, the exporter will reject the configuration.

<!-- Separate consecutive admonitions. -->

> [!NOTE]
> Static authentication headers can be registered via the `http.headers`
> configuration (for example, `http.headers.authorization: "Bearer <token>"`)
> but this is NOT recommended because the value is not managed as a secret.

### BearerTokenProvider

The exporter can inject an OAuth `Authorization: Bearer <token>` on every
outbound request by consuming the `bearer_token_provider` capability. The bound
extension acquires and refreshes the token in the background so credentials
rotate without restarting the exporter.

Declare a provider extension in the pipeline's `extensions:` section and bind it
on the exporter node via the node's `capabilities:` map. Any provider works and
the exporter cannot tell them apart; today the available ones are
[`oauth2_client_auth`](../../../../contrib-extensions/src/oauth2_client_auth/README.md)
(any OAuth 2.0 token endpoint) and
[`azure_identity_auth`](../../../../contrib-extensions/src/azure_identity_auth/README.md)
(Azure identities). See the chosen extension's README for its configuration
reference; only the binding is documented here.

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          token_provider:
            type: "urn:otel:extension:oauth2_client_auth"
            config:
              token_url: "https://idp.example.com/oauth2/v1/token"
              client_id: "someclientid"
              client_secret_file: "/etc/secrets/oauth2_client_secret"

        nodes:
          otlp-http-exporter:
            type: "urn:otel:exporter:otlp_http"
            # Bind the bearer token provider to the extension declared above.
            capabilities:
              bearer_token_provider: token_provider
            config:
              endpoint: "https://my-endpoint:4318"
              client_pool_size: 1
              http: {}
```

The provider-generated Authorization header takes precedence over a statically
configured header of the same name. Ensure that the provider's configured
resource or scopes match the OTLP destination; a mismatch is reported by the
destination as an authentication failure rather than detected at startup.

### ApiKeyProvider

The `api_key_provider` capability supplies an API key together with the HTTP
header name and an optional authentication scheme. The exporter sends either
`<header_name>: <api_key>` or
`<header_name>: <header_scheme> <api_key>`, depending on whether the provider
sets `http.header_scheme`.

The [`flat_file_api_key_auth`](../../../../contrib-extensions/src/flat_file_api_key_auth/README.md)
extension can load the key from a file and poll for rotations. The
`http.header_name` attribute is required for OTLP/HTTP; `http.header_scheme` is
optional.

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          api_key:
            type: "urn:otel:extension:flat_file_api_key_auth"
            config:
              key_secret_file: "/etc/secrets/otlp_api_key"
              key_secret_file_refresh: 30m
              attributes:
                http.header_name: "x-api-key"
                http.header_scheme: "ApiKey"

        nodes:
          otlp-http-exporter:
            type: "urn:otel:exporter:otlp_http"
            capabilities:
              api_key_provider: api_key
            config:
              endpoint: "https://otlp.example.com:4318"
              client_pool_size: 1
              http: {}
```

Omit `http.header_scheme` when the destination expects the raw key, such as
`x-api-key: <api_key>`. See the extension README for inline-key configuration
and the complete field reference.

### BasicAuthProvider

The `basic_auth_provider` capability supplies a username and password. The
exporter constructs the HTTP Basic authentication header from the current
credential; the encoded header value is not configured directly.

The [`flat_file_user_pass_auth`](../../../../contrib-extensions/src/flat_file_user_pass_auth/README.md)
extension accepts a configured username and can load the password from a file
that is polled for rotations.

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          basic_auth:
            type: "urn:otel:extension:flat_file_user_pass_auth"
            config:
              username: "otlp-client"
              password_secret_file: "/etc/secrets/otlp_password"
              password_secret_file_refresh: 30m

        nodes:
          otlp-http-exporter:
            type: "urn:otel:exporter:otlp_http"
            capabilities:
              basic_auth_provider: basic_auth
            config:
              endpoint: "https://otlp.example.com:4318"
              client_pool_size: 1
              http: {}
```

See the extension README for inline-password configuration, credential
validation rules, and the complete field reference.

### AgentFedCredentialProvider

`agent_fed_credential_provider` is intended for deployments where the embedding
host supplies a bearer token and vendor attributes as one credential snapshot.
The exporter uses the snapshot's token for the HTTP Authorization header.
Vendor attributes are ignored because the configured OTLP endpoint and headers
remain authoritative.

```yaml
nodes:
  otlp-http-exporter:
    type: "urn:otel:exporter:otlp_http"
    capabilities:
      # "agent_auth" is an embedding-host extension instance that provides
      # agent_fed_credential_provider.
      agent_fed_credential_provider: agent_auth
    config:
      endpoint: "https://my-endpoint:4318"
      client_pool_size: 1
      http: {}
```

There is no agent-fed token field in the exporter configuration. The embedding
host must register an extension instance that provides the capability and
publish credential updates through that provider.

### Credential refresh and failures

For every provider type, the exporter subscribes to the provider's credential
stream and caches the prepared HTTP header. Credential acquisition and encoding
therefore stay off the per-request path.

The exporter stops accepting new batches when no usable credential is cached,
including before the first credential arrives, when a credential is malformed,
or when an expiring credential reaches its safety margin. This back-pressures
upstream instead of sending an unauthenticated request. It resumes when the
provider publishes a usable credential; buffered batches force-drained during
shutdown are NACK'd as retryable.

HTTP 401 responses invalidate the exact credential generation used by the
rejected request and are treated as retryable. The exporter does not reuse that
generation and resumes after the provider publishes a replacement. A delayed
401 for an older generation does not invalidate a newer credential.

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
`channel.receiver.messages` with its `signal` attribute on the PData input
channel and is not duplicated by the exporter.

#### `exporter.attempted`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.attempted.messages` | `{message}` | `signal`, `outcome` | Number of component-local HTTP delivery attempts, including preparation failures. |
| `exporter.attempted.duration` | `s` | `signal`, `outcome` | Attempt time through the terminal local or backend result, excluding Ack/Nack notification. Emitted when component duration is enabled. |
| `exporter.attempted.payload.size` | `By` | `signal`, `outcome` | Uncompressed OTLP protobuf payload bytes produced or submitted by the attempt. Emitted when size measurement is enabled and bytes are available. |
| `exporter.attempted.items` | `{item}` | `signal`, `outcome` | Signal items handled by the attempt. Emitted when item counting is enabled. |

#### `exporter.otlp_http.failures`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.otlp_http.failures.messages` | `{message}` | `signal`, `error.type` | Failed OTLP HTTP exports classified by actionable error type. |

`error.type` is one of `encoding`, `compression`, `authentication`,
`authorization`, `timeout`, `throttled`, `unavailable`, `rejected`,
`server_error`, `transport`, `response_too_large`, `response_decode`,
`partial_rejection`, or `other`. Successful exports, zero-rejection partial
successes, and Ack/Nack notification failures do not emit this metric.

#### `exporter.otlp_http.authentication`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.otlp_http.authentication.failures` | `{attempt}` | `source` | Auth credential polls that did not produce a usable credential, including failures before a signal batch is admitted. |

Authentication `source` is the name of the HTTP client auth implementation (ex:
`BearerAuth`) selected based on the auth capability configured.

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otlp.exporter.http.validate_insecure_flag` | `warn` | The HTTP exporter ignored a TLS-only insecure flag for an HTTP endpoint. |
| `otlp.exporter.http.start` | `info` | Exporter startup with the configured HTTP endpoint. |
| `otlp.exporter.http.receive` | `debug` | A pdata batch was received by the exporter loop. |
| `otlp.exporter.http.shutdown` | `info` | Exporter shutdown and terminal reason. |
| `otlp.exporter.http.zero_partial_rejected` | `debug` | A zero-length partial-success response was rejected. |
| `otlp.exporter.http.export_error` | `warn` | An HTTP export request failed; non-success responses include bounded backend error details when available. |
| `otlp.exporter.http.auth.invalid` | `warn` | A credential from the auth provider could not be turned into a valid header. |
| `otlp.exporter.http.auth.stream_closed` | `warn` | The auth provider closed its refresh stream; the last credential (if any) is reused and no longer refreshes. |

## Limits

- `client_pool_size` must be non-zero.
- Signal-specific endpoint fields must include the full OTLP path.
- Response bodies larger than `max_response_body_length` fail the export.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Proxy support](../../../../../docs/proxy-support.md)
- [Core node catalog](../../../README.md)
