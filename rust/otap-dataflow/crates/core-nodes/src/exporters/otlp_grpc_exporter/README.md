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
  # (optional). Useful for arbitrary metadata such as tenant routing. Not
  # recommended for authorization; prefer a dedicated Auth extension instead.
  # Keys and values must be valid ASCII gRPC metadata and are validated at
  # config load.
  headers:
    x-scope-orgid: "tenant-1"
    environment: "production-west"
```

Shared gRPC client fields include connect timeout, request timeout, TCP
keepalive, HTTP/2 settings, TLS, proxy, and transport buffer settings.

### Static request headers

`headers` is a map of metadata name to value added to every outbound request
(multi-tenant routing IDs, tracing-vendor metadata, and similar). For request
authentication, prefer the `bearer_token_provider` capability (see
[Authentication](#authentication)) rather than hard-coding an `authorization`
entry here. Values are sent verbatim, so treat any secret in the rendered config
as sensitive.

Validation at config load rejects:

- invalid metadata names (must be a valid ASCII gRPC metadata key: an HTTP/2
  token that is sent lowercased and must not end in `-bin`, which is reserved
  for binary metadata), and
- invalid metadata values (must be visible ASCII), and
- protocol-reserved metadata managed by the gRPC transport: `content-type`,
  `te`, `user-agent`, and any name with the spec-reserved `grpc-` prefix
  (e.g. `grpc-timeout`, `grpc-encoding`).

When [header propagation](../../../../../docs/transport-headers.md) is also
enabled, statically configured headers take precedence: a propagated header
whose key matches a configured one is dropped, so a configured routing header
(e.g. `x-scope-orgid`) is never overridden or duplicated.

## Authentication

The exporter can inject an OAuth `authorization: Bearer <token>` on every
outbound request by consuming the `bearer_token_provider` capability. Binding is
optional and additive: without it the exporter sends no `authorization` metadata
(the default); with it, the bound extension acquires and refreshes the token in
the background so credentials rotate without restarting the exporter.

Declare a provider extension -- for example
[`oauth2_client_auth`](../../../../contrib-extensions/src/oauth2_client_auth/README.md)
(any OAuth 2.0 token endpoint), or any other extension exposing
`bearer_token_provider` whose tokens are accepted by a gRPC OTLP endpoint -- in
the pipeline's `extensions:` section and bind it on the exporter node via the
node's `capabilities:` map. See the chosen extension's README for its
configuration reference; only the binding is documented here.

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          oauth2:
            type: "urn:otel:extension:oauth2_client_auth"
            config:
              grant_type: client_credentials
              token_url: "https://idp.example.com/oauth2/v1/token"
              client_id: "someclientid"
              client_secret_file: "/etc/secrets/oauth2_client_secret"
              scopes: ["telemetry.write"]

        nodes:
          otlp-grpc-exporter:
            type: "urn:otel:exporter:otlp_grpc"
            # Bind the bearer token provider to the extension declared above.
            capabilities:
              bearer_token_provider: oauth2
            config:
              grpc_endpoint: "https://otlp.example.com:4317"
```

The bearer token is applied per request, so it takes precedence over both a
statically configured `authorization` entry and any propagated `authorization`
transport header; exactly one `authorization` value is sent. The exporter
subscribes to the provider's token stream and caches the built metadata value,
rebuilding it only when the provider refreshes the token, so credential work
stays off the per-request path. The value is marked sensitive, which keeps the
credential out of the HTTP/2 HPACK dynamic table.

When no usable token is cached yet -- before the provider's first publish, in a
degraded window where a refresh is failing and the cached token is within a small
safety margin of expiring, or after the server rejects the cached token -- the
exporter **stops accepting new batches** (back-pressures upstream) rather than
sending an unauthenticated or soon-to-lapse request. It resumes when the provider
publishes a usable token; nothing is dropped. Note that a rejection only drops
the exporter's own copy -- it does not make the provider refresh early -- so
recovery waits for that provider's next scheduled publication. (If
buffered batches are force-drained during shutdown while no token is available,
they are NACK'd as **retryable**.) A token is guaranteed to eventually arrive:
the bound extension holds data-path startup until its first token publish, and
its token stream stays live for the exporter's lifetime.

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

Input PData message volume is reported by the engine through
`channel.receiver.messages` with its `signal` attribute on the PData input
channel and is not duplicated by the exporter.

#### `exporter.exports`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.exports.messages` | `{message}` | `signal`, `outcome` | Number of PData messages whose export reached a terminal outcome. |
| `exporter.exports.duration` | `s` | `signal`, `outcome` | Time from dequeuing PData through the terminal gRPC export result, including encoding and in-flight queueing but excluding Ack/Nack notification. |

#### `exporter.otlp_grpc.failures`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.otlp_grpc.failures.messages` | `{message}` | `signal`, `error.type` | Failed OTLP gRPC exports classified by actionable error type. |

`error.type` is one of `encoding`, `authentication`, `authorization`,
`timeout`, `throttled`, `unavailable`, `rejected`, `server_error`, `transport`,
or `other`. Successful exports and Ack/Nack notification failures do not emit
this metric.

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otlp.exporter.grpc.start` | `info` | Exporter startup with the configured gRPC endpoint. |
| `otlp.exporter.grpc.channels` | `info` | gRPC channel pool creation with connection count and endpoint. |
| `otlp.exporter.grpc.receive` | `debug` | A pdata batch was received by the exporter loop. |
| `otlp.exporter.grpc.shutdown` | `info` | Exporter shutdown. |
| `otlp.exporter.grpc.export_error` | `warn` | A gRPC export request did not complete successfully. |
| `otlp.exporter.grpc.header_skip` | `debug` | A propagated transport header was skipped while building gRPC metadata. |
| `otlp.exporter.grpc.invalid_bearer_token` | `warn` | A bearer token from the provider could not be turned into a valid `authorization` header. |
| `otlp.exporter.grpc.token_stream_closed` | `warn` | The bearer token provider closed its refresh stream; the last token (if any) is reused and no longer refreshes. |

## Limits

- `max_in_flight` bounds concurrent export RPCs inside the node.
- `num_connections` only improves distribution when the downstream endpoint can
  balance separate connections.
- OTLP partial success responses are treated as export failures by the current
  implementation.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [OAuth2 client auth extension](../../../../contrib-extensions/src/oauth2_client_auth/README.md)
- [Proxy support](../../../../../docs/proxy-support.md)
- [Transport headers](../../../../../docs/transport-headers.md)
- [Core node catalog](../../../README.md)
