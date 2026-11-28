<!-- markdownlint-disable MD013 -->

# GCP Auth Extension

## Metadata

- URN: `urn:otel:extension:gcp_auth`
- Feature gate: `gcp-auth` (or the aggregate `contrib-extensions`)
- Capability provided: `bearer_token_provider`
- Execution model: Active + Shared
- Stability: Experimental

## Overview

Acquires and refreshes Google Cloud OAuth 2.0 access tokens from the Google
Compute Engine (GCE) / Google Kubernetes Engine (GKE) metadata server
(`/computeMetadata/v1/instance/service-accounts/{service_account}/token` with
`Metadata-Flavor: Google`) and exposes them to data-path nodes through the
`bearer_token_provider` capability.

Tokens are cached and refreshed ahead of expiry in a background task, concurrent
cache misses are coalesced onto a single credential call, and pipeline startup is
held until the first token is published.

## Getting started

Declare the extension in the pipeline's `extensions:` section and bind it on a
consumer node (such as `urn:otel:exporter:otlp_grpc` or
`urn:otel:exporter:otlp_http`) via the node's `capabilities:` map:

```yaml
extensions:
  gcp_auth:
    type: "urn:otel:extension:gcp_auth"
    config: {}

nodes:
  gcm_otlp:
    type: "urn:otel:exporter:otlp_grpc"
    capabilities:
      bearer_token_provider: gcp_auth
    config:
      grpc_endpoint: "https://telemetry.googleapis.com:443"
      tls:
        include_system_ca_certs_pool: true
```

## Building

Enable the extension's feature gate together with the nodes that consume it:

```bash
cargo build --release --features gcp-auth,otlp
```

The default build includes `crypto-ring`. If you build with
`--no-default-features`, enable one of the workspace `crypto-*` features
(`crypto-ring`, `crypto-aws-lc`, `crypto-openssl`, or `crypto-symcrypt`) so the
metadata HTTP client has a TLS crypto provider installed.

## Configuration

All fields are optional; an empty `config: {}` queries the default service
account on `http://metadata.google.internal` (or `http://$GCE_METADATA_HOST` when
the `GCE_METADATA_HOST` environment variable is set) with the
`https://www.googleapis.com/auth/cloud-platform` scope.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `endpoint` | string | `http://metadata.google.internal` (or `http://$GCE_METADATA_HOST`) | Base URL of the GCE/GKE metadata server. Must start with `http://` or `https://`. |
| `service_account` | string | `"default"` | Service account email or alias under `/computeMetadata/v1/instance/service-accounts/{service_account}/token`. |
| `scopes` | list of strings | `["https://www.googleapis.com/auth/cloud-platform"]` | OAuth 2.0 scopes passed in the `?scopes=` query parameter when non-empty. |
| `expiry_buffer` | duration | `4m` | Duration ahead of token expiry at which background refresh runs. Must be greater than `30s`. |
| `default_token_lifetime` | duration | `1h` | Assumed lifetime when the metadata response omits `expires_in`. Must be greater than `expiry_buffer`. |
| `timeout` | duration | `10s` | Per-request timeout on the metadata HTTP client. Must be non-zero. |
| `connect_timeout` | duration | `5s` | Connection timeout on the metadata HTTP client. Must be non-zero. |
| `startup_timeout` | duration | `30s` | Maximum time pipeline startup waits for the first token publish. Must be non-zero. |

## Telemetry

Metric set: `extension.auth` (shared).

| Metric | Metric set | Type | Unit | Description |
| --- | --- | --- | --- | --- |
| `acquisitions` | `extension.auth` | Counter | `{acquisition}` | Credential acquisitions partitioned by `outcome` (`success`, `failure`). |
| `publishes` | `extension.auth` | Counter | `{publication}` | Credentials or tokens published to consumers via the watch channel partitioned by `outcome`. |
| `duration` | `extension.auth` | Mmsc | `s` | Acquisition duration in seconds partitioned by `outcome` (min/max/sum/count). |

Events:

| Event | Severity | Description |
| --- | --- | --- |
| `gcp_auth.token_refresh_failed` | `warn` | A credential acquisition failed; the loop retries with backoff. |
