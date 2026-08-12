<!-- markdownlint-disable MD013 -->

# OAuth 2.0 Client Auth Extension

## Metadata

- URN: `urn:otel:extension:oauth2_client_auth`
- Feature gate: `oauth2-client-auth-extension` (or the aggregate `contrib-extensions`)
- Capability provided: `bearer_token_provider`
- Execution model: Active + Shared
- Stability: Draft

## Overview

Acquires and refreshes OAuth 2.0 access tokens and exposes them to data-path
nodes through the `bearer_token_provider` capability, so nodes never construct
credentials or manage token refresh themselves. Two grants are supported:

- **client credentials** (RFC 6749 section 4.4) -- authenticate with a client id
  and secret.
- **JWT bearer** (RFC 7523 section 2.1) -- sign a JWT and send it as the
  `assertion` parameter instead of a secret.

Tokens are cached and refreshed ahead of expiry in a background task, concurrent
cache misses are coalesced onto a single token request, and pipeline startup is
held until the first token is published. Credential material may be supplied
inline or via files that are re-read on each acquisition, so secrets can rotate
without a restart.

This README is the configuration reference for the extension. Nodes that consume
the capability -- for example the
[OTLP HTTP exporter](../../../core-nodes/src/exporters/otlp_http_exporter/README.md)
and the
[OTLP gRPC exporter](../../../core-nodes/src/exporters/otlp_grpc_exporter/README.md)
-- document only how they *use* a bearer token, not how to configure a provider.

For the design -- lifecycle, refresh and retry behavior, and the rationale behind
the defaults -- see [`design.md`](./design.md).

## Getting Started

Declare the extension in the pipeline's `extensions:` section and bind it on a
consumer node via the node's `capabilities:` map:

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          oauth2:
            type: "urn:otel:extension:oauth2_client_auth"
            config:
              token_url: "https://idp.example.com/oauth2/v1/token"
              client_id: "someclientid"
              client_secret_file: "/etc/secrets/oauth2_client_secret"
              scopes: ["telemetry.write"]

        nodes:
          otlp-http-exporter:
            type: "urn:otel:exporter:otlp_http"
            # Bind the capability to the extension instance declared above.
            capabilities:
              bearer_token_provider: oauth2
            config:
              endpoint: "https://otlp.example.com:4318"
              client_pool_size: 1
              http: {}
```

One extension instance serves one client identity and scope set. Declare several
instances (under different names) when different consumers need different
credentials, and bind each consumer to the instance it needs.

## Building

Enable the extension's feature gate together with the nodes that consume it.
From the `otap-dataflow` directory:

```bash
cargo build --release --features oauth2-client-auth-extension
```

The extension reaches the token endpoint over TLS through a `reqwest`/`rustls`
client, which requires a process-wide `rustls` crypto provider. The deployed
binary **must** enable exactly one `crypto-*` feature (`crypto-ring`,
`crypto-aws-lc`, `crypto-openssl`, or `crypto-symcrypt`, forwarded to
`otap-df-otap`); the workspace binary's default build includes `crypto-ring`. A
build that enables `oauth2-client-auth-extension` without any `crypto-*` feature
installs no provider, and token acquisition panics at runtime with "No provider
set".

Verify registration with `./target/release/df_engine --help`;
`urn:otel:extension:oauth2_client_auth` appears in the Extensions list.

## Configuration

Unknown fields are rejected, and the whole config is validated before the
pipeline starts, so a mistake fails at startup rather than on the first export.

### Common fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `grant_type` | enum | `client_credentials` | Grant used to acquire tokens: `client_credentials` or `jwt-bearer`. |
| `token_url` | string | *required* | Token endpoint URL (RFC 6749 section 3.2). Must be non-empty. Use `https://` in production; `http://` is accepted but logs the `oauth2_client_auth.insecure_token_url` warning. |
| `client_id` | string | *none* | Client identifier. Required unless `client_id_file` is set. |
| `client_id_file` | path | *none* | File holding the client identifier. Re-read on each acquisition; takes precedence over `client_id`. |
| `scopes` | list of string | `[]` | Scopes requested from the token endpoint, sent as the `scope` parameter. |
| `endpoint_params` | map of string to string | `{}` | Extra parameters sent to the token endpoint (for example `audience`). |
| `expiry_buffer` | duration | `5m` | Refresh this far ahead of the token's expiry. Must be greater than `30s`, the window before expiry in which a token is no longer used; a smaller buffer would schedule the refresh after consumers have already stopped sending. |
| `default_token_lifetime` | duration | `24h` | Lifetime assumed when the token response omits `expires_in`. Must be non-zero and greater than `expiry_buffer`. |
| `timeout` | duration | `30s` | Per-request timeout on the token client, covering the whole request. Must be non-zero. |
| `connect_timeout` | duration | `10s` | Connection-establishment timeout on the token client. Must be non-zero. |
| `tls` | object | *none* | Client TLS for the token endpoint. See [Token-endpoint TLS](#token-endpoint-tls). |
| `startup_timeout` | duration | `30s` | How long the engine holds data-path node startup waiting for the first token publish before aborting startup. Must be non-zero. |

Duration fields accept human-readable values such as `30s`, `5m`, or `1h`.

### Client credentials grant

`grant_type: client_credentials` (the default) authenticates with a client id
and secret. At least one of the secret fields must be set:

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `client_secret` | string | *none* | Client secret. Required unless `client_secret_file` is set. Redacted from logs, but the rendered pipeline config still holds it in cleartext -- prefer `client_secret_file`. |
| `client_secret_file` | path | *none* | File holding the client secret. Re-read on each acquisition; takes precedence over `client_secret`. |

```yaml
type: "urn:otel:extension:oauth2_client_auth"
config:
  grant_type: client_credentials
  token_url: "https://idp.example.com/oauth2/v1/token"
  client_id: "someclientid"
  client_secret_file: "/etc/secrets/oauth2_client_secret"
  scopes: ["telemetry.write"]
  endpoint_params:
    audience: "https://otlp.example.com"
  expiry_buffer: 5m
  timeout: 2s
```

The request sends `grant_type=client_credentials` with the client id and secret.

### JWT bearer grant

`grant_type: jwt-bearer` signs a JWT and sends it as the `assertion` parameter;
the signed JWT is itself the authorization grant, so no client secret is used.
The following fields apply only to this grant and are **rejected** for
`client_credentials` (and, conversely, `client_secret` / `client_secret_file` are
rejected here):

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `client_certificate_key` | string | *none* | PEM private key used to sign the assertion. Required unless `client_certificate_key_file` is set. Redacted from logs; prefer the file form. |
| `client_certificate_key_file` | path | *none* | File holding the signing key. Re-read on each acquisition; takes precedence over `client_certificate_key`. |
| `signature_algorithm` | enum | `RS256` | RSA algorithm used to sign the assertion: `RS256`, `RS384`, or `RS512`. |
| `client_certificate_key_id` | string | *none* | Optional `kid` header placed on the assertion. |
| `iss` | string | value of `client_id` | Assertion issuer (`iss` claim). |
| `audience` | string | value of `token_url` | Assertion audience (`aud` claim). |
| `claims` | map of string to string | `{}` | Extra claims added to the assertion. |

```yaml
type: "urn:otel:extension:oauth2_client_auth"
config:
  grant_type: jwt-bearer
  token_url: "https://idp.example.com/oauth2/v1/token"
  client_id: "someclientid"
  client_certificate_key_file: "/etc/secrets/oauth2_signing_key.pem"
  signature_algorithm: RS512
  client_certificate_key_id: "key-1"
  audience: "https://idp.example.com/oauth2/v1/token"
  claims:
    tenant: "acme"
  scopes: ["telemetry.write"]
```

The request sends `grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer` and
`assertion=<signed JWT>`. The assertion carries `iss` / `sub` (default
`client_id`), `aud` (default `token_url`), plus `exp`, `iat`, and `jti`.

### Credential rotation

Every credential field has a `_file` counterpart (`client_id_file`,
`client_secret_file`, `client_certificate_key_file`). The file is re-read on each
token acquisition, so rewriting it rotates the credential without restarting the
collector, and the secret never appears in the rendered pipeline config. When
both forms are set, the `_file` form wins. Prefer the file form in production.

### Token-endpoint TLS

The token endpoint carries the client secret (or signed assertion) and returns
bearer tokens, so it must be reached over TLS in production. Transport is decided
by the `token_url` scheme: `https://` uses TLS configured by the `tls` block,
while `http://` is plaintext and intended only for local development.

`tls` is the engine's shared `TlsClientConfig` -- the same type the OTLP/HTTP
exporter uses -- so the settings and their validation match the rest of the
collector. The most relevant fields:

| Field | Description |
| --- | --- |
| `ca_file` / `ca_pem` | Trust a private or enterprise CA. |
| `include_system_ca_certs_pool` | Also trust the system store (default `true`). |
| `cert_file` / `key_file` | Present a client certificate for mutual TLS (RFC 8705), in addition to or instead of a client secret. |
| `insecure_skip_verify` | Skip certificate verification. Development and testing only -- it disables verification on the connection that carries the client secret. |

```yaml
tls:
  ca_file: "/etc/ssl/idp-ca.pem"
  cert_file: "/etc/ssl/client.pem"
  key_file: "/etc/ssl/client-key.pem"
```

Two `TlsClientConfig` knobs are rejected at config validation rather than
silently ignored, so a config that relies on them fails at startup:

- **`server_name_override`** (SNI override) is not supported by the
  reqwest/rustls token client, matching the OTLP/HTTP exporter.
- **`insecure`** (disable TLS) is rejected alongside an `https://` `token_url`,
  because the scheme mandates a TLS handshake. Use an `http://` `token_url` for a
  plaintext endpoint; setting `insecure` alongside one is accepted as a no-op.

### Validation rules

Config validation rejects:

- an empty `token_url`;
- a zero `expiry_buffer`, `timeout`, `connect_timeout`, or `startup_timeout`;
- a `default_token_lifetime` at or below `expiry_buffer` (every refresh would be
  scheduled in the past);
- a missing client identifier (`client_id` and `client_id_file` both unset);
- a missing secret for `client_credentials`, or a missing signing key for
  `jwt-bearer`;
- fields belonging to the other grant;
- unknown fields.

## Telemetry

Metric set: `extension.oauth2_client_auth`.

| Metric | Type | Unit | Description |
| --- | --- | --- | --- |
| `auth_successes` | Counter | `{acquisition}` | Successful token acquisitions. |
| `auth_failures` | Counter | `{acquisition}` | Failed token acquisitions. |
| `auth_token_publish` | Counter | `{token}` | Tokens published to consumers. |
| `auth_success_latency` | Mmsc | `ms` | Latency of successful acquisitions (min/max/sum/count). |

Events:

| Event | Severity | Description |
| --- | --- | --- |
| `oauth2_client_auth.insecure_token_url` | `warn` | The configured `token_url` uses plaintext `http://`. |
| `oauth2_client_auth.token_refresh_failed` | `warn` | A token acquisition failed; the loop retries with backoff. |

## Troubleshooting

| Symptom | Likely cause |
| --- | --- |
| Panic at runtime with "No provider set" | The binary was built without a `crypto-*` feature. See [Building](#building). |
| Startup aborts with a readiness timeout | The first token was not acquired within `startup_timeout`. Check `token_url` reachability, credentials, and the `oauth2_client_auth.token_refresh_failed` event. |
| Config rejected at startup | See [Validation rules](#validation-rules); the error names the offending field. |
| Exporter stops accepting data mid-run | Refresh is failing and the cached token lapsed. Check `auth_failures` and the refresh-failure event. |
| Signing key errors on `jwt-bearer` | The build has no JWT signing backend, or the key is not a supported PEM RSA key. |

## Related Docs

- [Design](./design.md)
- [Contrib extension catalog](../../README.md)
- [Writing pipeline configuration](../../../../docs/configuration.md)
- [Configuration model](../../../../docs/configuration-model.md)
- [Extension system architecture](../../../../docs/extension-system-architecture.md)
