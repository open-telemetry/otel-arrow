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
| `expiry_buffer` | duration | `5m` | Duration ahead of token expiry at which background refresh runs. Must be greater than `30s`. |
| `default_token_lifetime` | duration | `1h` | Assumed lifetime when the metadata response omits `expires_in`. Must be greater than `expiry_buffer`. |
| `timeout` | duration | `10s` | Per-request timeout on the metadata HTTP client. Must be non-zero. |
| `connect_timeout` | duration | `5s` | Connection timeout on the metadata HTTP client. Must be non-zero. |
| `startup_timeout` | duration | `30s` | Maximum time pipeline startup waits for the first token publish. Must be non-zero. |
