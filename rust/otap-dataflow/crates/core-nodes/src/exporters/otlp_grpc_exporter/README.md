# OTLP gRPC Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:otlp_grpc` (`urn:otel:exporter:otlp_grpc`)
- Feature gate: `otlp`
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
authentication, prefer one of the provider capabilities described in
[Authentication](#authentication) rather than hard-coding credentials here.
Values are sent verbatim, so treat any secret in the rendered config as
sensitive.

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

By default the exporter sends requests without any authentication.
Authentication can be enabled by binding a
[provider extension](../../../../contrib-extensions/README.md) to the exporter
node via its `capabilities` map. The following providers are supported:

- [`BearerTokenProvider`](#bearertokenprovider): Provides OAuth bearer
  authorization metadata.
- [`ApiKeyProvider`](#apikeyprovider): Provides an API key through custom
  metadata in the form `<header_name>: <optional_scheme> <api_key>`.
- [`BasicAuthProvider`](#basicauthprovider): Provides HTTP Basic authorization
  metadata from a username and password.
- [`AgentFedCredentialProvider`](#agentfedcredentialprovider): Provides bearer
  authorization metadata from a credential snapshot published by the embedding
  host.

> [!IMPORTANT]
> Only one authentication provider can be bound to the exporter node at a time.
> If multiple providers are bound, the exporter will reject the configuration.

<!-- Separate consecutive admonitions. -->

> [!NOTE]
> Static authentication metadata can be registered via `headers`, but this is
> NOT recommended because the credential remains embedded in the rendered
> configuration and cannot be refreshed by a provider.

### BearerTokenProvider

The exporter can inject an OAuth `authorization: Bearer <token>` on every
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

The provider-generated authorization metadata takes precedence over both a
statically configured entry and propagated metadata of the same name; exactly
one value is sent. Ensure that the provider's configured resource or scopes
match the OTLP destination; a mismatch is reported by the destination as an
authentication failure rather than detected at startup. The metadata value is
marked sensitive, which keeps the credential out of the HTTP/2 HPACK dynamic
table.

### ApiKeyProvider

The `api_key_provider` capability supplies an API key together with the metadata
name and an optional authentication scheme. The exporter sends either
`<header_name>: <api_key>` or
`<header_name>: <header_scheme> <api_key>`, depending on whether the provider
sets `http.header_scheme`.

The [`flat_file_api_key_auth`](../../../../contrib-extensions/src/flat_file_api_key_auth/README.md)
extension can load the key from a file and poll for rotations. The
`http.header_name` attribute is required for OTLP/gRPC; `http.header_scheme` is
optional. The header name must also satisfy the gRPC metadata restrictions
described under [Static request headers](#static-request-headers).

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
          otlp-grpc-exporter:
            type: "urn:otel:exporter:otlp_grpc"
            capabilities:
              api_key_provider: api_key
            config:
              grpc_endpoint: "https://otlp.example.com:4317"
```

Omit `http.header_scheme` when the destination expects the raw key, such as
`x-api-key: <api_key>`. See the extension README for inline-key configuration
and the complete field reference.

### BasicAuthProvider

The `basic_auth_provider` capability supplies a username and password. The
exporter constructs the HTTP Basic authorization metadata from the current
credential; the encoded value is not configured directly.

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
          otlp-grpc-exporter:
            type: "urn:otel:exporter:otlp_grpc"
            capabilities:
              basic_auth_provider: basic_auth
            config:
              grpc_endpoint: "https://otlp.example.com:4317"
```

See the extension README for inline-password configuration, credential
validation rules, and the complete field reference.

### AgentFedCredentialProvider

`agent_fed_credential_provider` is intended for deployments where the embedding
host supplies a bearer token and vendor attributes as one credential snapshot.
The exporter uses the snapshot's token for the authorization metadata. Vendor
attributes are ignored because the configured OTLP endpoint and metadata remain
authoritative.

```yaml
nodes:
  otlp-grpc-exporter:
    type: "urn:otel:exporter:otlp_grpc"
    capabilities:
      # "agent_auth" is an embedding-host extension instance that provides
      # agent_fed_credential_provider.
      agent_fed_credential_provider: agent_auth
    config:
      grpc_endpoint: "https://otlp.example.com:4317"
```

There is no agent-fed token field in the exporter configuration. The embedding
host must register an extension instance that provides the capability and
publish credential updates through that provider.

### Credential refresh and failures

For every provider type, the exporter subscribes to the provider's credential
stream and caches the prepared gRPC metadata. Credential acquisition and
encoding therefore stay off the per-request path. Credential metadata values
are marked sensitive so they are not indexed in the HTTP/2 HPACK dynamic table.

The exporter stops accepting new batches when no usable credential is cached,
including before the first credential arrives, when a credential is malformed,
or when an expiring credential reaches its safety margin. This back-pressures
upstream instead of sending an unauthenticated request. It resumes when the
provider publishes a usable credential; buffered batches force-drained during
shutdown are NACK'd as retryable.

gRPC `UNAUTHENTICATED` responses invalidate the exact credential generation
used by the rejected request and are treated as retryable. The exporter does not
reuse that generation and resumes after the provider publishes a replacement. A
delayed response for an older generation does not invalidate a newer
credential.

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

#### `exporter.attempted`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.attempted.messages` | `{message}` | `signal`, `outcome` | Number of component-local gRPC delivery attempts, including preparation failures. |
| `exporter.attempted.duration` | `s` | `signal`, `outcome` | Attempt time through the terminal local or backend result, excluding Ack/Nack notification. Emitted when component duration is enabled. |
| `exporter.attempted.payload.size` | `By` | `signal`, `outcome` | Uncompressed OTLP protobuf payload bytes submitted by the attempt before transport compression. Emitted when size measurement is enabled. |
| `exporter.attempted.items` | `{item}` | `signal`, `outcome` | Signal items handled by the attempt. Emitted when item counting is enabled. |

#### `exporter.otlp_grpc.failures`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.otlp_grpc.failures.messages` | `{message}` | `signal`, `error.type` | Failed OTLP gRPC exports classified by actionable error type. |

`error.type` is one of `encoding`, `authentication`, `authorization`,
`timeout`, `throttled`, `unavailable`, `rejected`, `server_error`, `transport`,
or `other`. Successful exports and Ack/Nack notification failures do not emit
this metric.

#### `exporter.otlp_grpc.authentication`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.otlp_grpc.authentication.failures` | `{attempt}` | `source` | Auth credential polls that did not produce a usable credential, including failures before a signal batch is admitted. |

Authentication `source` is the name of the HTTP client auth implementation (ex:
`BearerAuth`) selected based on the auth capability configured.

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `otlp.exporter.grpc.start` | `info` | Exporter startup with the configured gRPC endpoint. |
| `otlp.exporter.grpc.channels` | `info` | gRPC channel pool creation with connection count and endpoint. |
| `otlp.exporter.grpc.receive` | `debug` | A pdata batch was received by the exporter loop. |
| `otlp.exporter.grpc.shutdown` | `info` | Exporter shutdown. |
| `otlp.exporter.grpc.export_error` | `warn` | A gRPC export request did not complete successfully. |
| `otlp.exporter.grpc.header_skip` | `debug` | A propagated transport header was skipped while building gRPC metadata. |
| `otlp.exporter.grpc.auth.invalid` | `warn` | A credential from the auth provider could not be turned into a valid header. |
| `otlp.exporter.grpc.auth.stream_closed` | `warn` | The auth provider closed its refresh stream; the last credential (if any) is reused and no longer refreshes. |

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
