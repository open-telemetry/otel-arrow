# Flat File User Pass Extension

## Metadata

- URN: `urn:otel:extension:flat_file_user_pass_auth`
- Feature gate: `flat-file-user-pass-auth` (or the aggregate `contrib-extensions`)
- Capability provided: `basic_auth_provider`
- Execution model: Active + Shared
- Stability: Draft

## Overview

Acquires basic auth credentials and exposes them to data-path nodes through the
`basic_auth_provider` capability, so nodes never construct credentials or manage
refresh themselves.

This README is the configuration reference for the extension. Nodes that consume
the capability -- for example the
[OTLP HTTP exporter](../../../core-nodes/src/exporters/otlp_http_exporter/README.md)
and the
[OTLP gRPC exporter](../../../core-nodes/src/exporters/otlp_grpc_exporter/README.md)
-- document only how they *use* basic auth, not how to configure a provider.

## Getting Started

Declare the extension in the pipeline's `extensions:` section and bind it on a
consumer node via the node's `capabilities:` map:

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          fileauth:
            type: "urn:otel:extension:flat_file_user_pass_auth"
            config:
              username: "<username>"
              password_secret_file: "/etc/secrets/fileauth_password"
              password_secret_file_refresh: 30m

        nodes:
          otlp-http-exporter:
            type: "urn:otel:exporter:otlp_http"
            # Bind the capability to the extension instance declared above.
            capabilities:
              basic_auth_provider: fileauth
            config:
              endpoint: "https://otlp.example.com:4318"
              client_pool_size: 1
              http: {}
```

One extension instance serves one client credential. Declare several
instances (under different names) when different consumers need different
credentials, and bind each consumer to the instance it needs.

## Building

Enable the extension's feature gate together with the nodes that consume it.
From the `otap-dataflow` directory:

```bash
cargo build --release --features flat-file-user-pass-auth
```

Verify registration with `./target/release/df_engine --help`;
`urn:otel:extension:flat_file_user_pass_auth` appears in the Extensions list.

## Configuration

Unknown fields are rejected, and the whole config is validated before the
pipeline starts, so a mistake fails at startup rather than on the first export.

### Common fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `username` | string | *none* | Inline username. Required unless `username_file` is set. Must be non-empty. Cannot contain `:` or control characters. |
| `username_file` | path | *none* | File holding the UTF-8 username. Takes precedence over `username` and is re-read on each acquisition. Trailing CR/LF is stripped. |
| `password_secret` | string | *none* | Password supplied inline. Required unless `password_secret_file` is set; prefer the file form for secrets. Cannot contain control characters. |
| `password_secret_file` | path | *none* | File holding the password. Re-read on each acquisition; takes precedence over `password_secret`. File contents must be valid `UTF-8`. Trailing `\r\n` chacters are automatically stripped. |
| `password_secret_file_refresh` | duration | `1h` | How often to refresh either credential file. Must be between `10s` and `365d`, inclusive. |

Duration fields accept human-readable values such as `5m`, `1h`, or `1d`.
Existing inline-username configurations remain supported. To load both values
from mounted secrets, configure:

```yaml
config:
  username_file: /run/secrets/auth/username
  password_secret_file: /run/secrets/auth/password
  password_secret_file_refresh: 30m
```

Paths must be non-empty. File values use the shared bounded UTF-8 reader and the
same Basic Auth validation as inline values. A failed preferred file read never
falls back to its inline alternative. Only trailing CR/LF is stripped; other
whitespace is preserved.

File polling runs at this interval independently of credential expiry. Both
values are acquired and validated before a complete pair is published. If either
read or validation fails, the last good pair remains available and the extension
logs the failure and retries with bounded backoff.

Separate file reads are not an atomic snapshot. Coordinate updates to avoid
mixed-version username/password pairs, even when individual files are replaced
atomically. Kubernetes Secret mounts are ordinary local files to the extension;
`subPath` mounts do not receive automatic Secret updates. Applying refreshed
credentials to an existing connection remains the consumer's responsibility.
