# Flat File API Key Auth Extension

## Metadata

- URN: `urn:otel:extension:flat_file_api_key_auth`
- Feature gate: `flat-file-api-key-auth` (or the aggregate `contrib-extensions`)
- Capability provided: `api_key_provider`
- Execution model: Active + Shared
- Stability: Draft

## Overview

Acquires an API key and exposes it to data-path nodes through the
`api_key_provider` capability. The key may be supplied inline or loaded from a
file that is polled for rotation. Optional attributes describe how consumers
should use the opaque key; `http.header_name` identifies its HTTP header and
`http.header_scheme` supplies an optional scheme prefix.

The OTLP HTTP and OTLP gRPC exporters require `http.header_name` because they
send the API key as request metadata. Other consumers may use the API key
without HTTP header metadata.

One extension instance serves one API key. Declare several instances under
different names when consumers need different keys.

## Configuration

Declare the extension in a pipeline's `extensions:` section:

```yaml
extensions:
  api-key:
    type: "urn:otel:extension:flat_file_api_key_auth"
    config:
      key_secret_file: "/etc/secrets/api_key"
      key_secret_file_refresh: 30m
      attributes:
        http.header_name: "x-api-key"
        http.header_scheme: "ApiKey"
```

A node that supports the capability binds this extension instance using
`api_key_provider: api-key` in its `capabilities:` map.

Unknown fields are rejected, and configuration is validated before the
pipeline starts.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `key_secret` | string | *none* | API key supplied inline. Required unless `key_secret_file` is set; prefer the file form for secrets. Must not be empty. |
| `key_secret_file` | path | *none* | File holding the API key. Re-read on each acquisition and takes precedence over `key_secret`. Contents must be valid UTF-8. Trailing `\r` and `\n` characters are stripped. |
| `key_secret_file_refresh` | duration | `1h` | How often to refresh the API key file. Must be between `10s` and `365d`, inclusive. |
| `attributes` | object | `{}` | Optional API key metadata. `http.header_name`, when set, must be a non-empty string and is required by the OTLP HTTP and OTLP gRPC exporters. `http.header_scheme` is optional and must be a string. |

Duration fields accept human-readable values such as `5m`, `1h`, or `1d`.
If a poll fails, the last successfully read key remains available and the
extension retries with bounded backoff.

## Building

From the `otap-dataflow` directory:

```bash
cargo build --release --features flat-file-api-key-auth
```