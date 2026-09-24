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
| `username` | string | *required* | Basic authentication username. Must be non-empty. Cannot contain `:` or control characters. |
| `password_secret` | string | *none* | Password supplied inline. Required unless `password_secret_file` is set; prefer the file form for secrets. Cannot contain control characters. |
| `password_secret_file` | path | *none* | File holding the password. Re-read on each acquisition; takes precedence over `password_secret`. File contents must be valid `UTF-8`. Trailing `\r\n` chacters are automatically stripped. |
| `password_secret_file_refresh` | duration | `1h` | How often to refresh the password file. Must be `5m` or greater if specified. |

Duration fields accept human-readable values such as `5m`, `1h`, or `1d`.
