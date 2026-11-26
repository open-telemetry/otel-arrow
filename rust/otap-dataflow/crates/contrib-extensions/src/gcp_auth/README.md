<!-- markdownlint-disable MD013 -->

# GCP Auth Extension

## Metadata

- URN: `urn:otel:extension:gcp_auth`
- Feature gate: `gcp-auth` (or the aggregate `contrib-extensions`)
- Capability provided: `bearer_token_provider`
- Execution model: Active + Shared
- Stability: Experimental

## Overview

Acquires and refreshes Google Cloud OAuth 2.0 access tokens via the official
[`google-cloud-auth`](https://crates.io/crates/google-cloud-auth) SDK and
exposes them to data-path nodes through the `bearer_token_provider` capability.

By default (when `endpoint` is omitted), credentials are resolved using standard
Google Cloud Application Default Credentials (ADC) via
`google_cloud_auth::credentials::Builder`:

1. `GOOGLE_APPLICATION_CREDENTIALS` environment variable (service account key,
   authorized user credentials, impersonated service account, or external
   account / Workload Identity Federation).
2. Well-known `gcloud` ADC credentials file
   (`$HOME/.config/gcloud/application_default_credentials.json` on Linux/macOS,
   `%APPDATA%\gcloud\application_default_credentials.json` on Windows) created
   by `gcloud auth application-default login`.
3. Google Compute Engine (GCE) / Google Kubernetes Engine (GKE) / Cloud Run
   compute metadata server fallback (`http://metadata.google.internal`, or
   `http://$GCE_METADATA_HOST` when `GCE_METADATA_HOST` is set).

When `endpoint` is explicitly configured, `google_cloud_auth::credentials::mds::Builder`
is used directly against that metadata server base URL.

Tokens are cached and refreshed ahead of expiry (4 minutes before expiration) in
`google-cloud-auth`'s internal `TokenCache`, concurrent cache misses are
coalesced onto a single credential call, and pipeline startup is held until the
first token is published. Because `google-cloud-auth` manages token expiry
internally without exposing the raw `expires_in` timestamp through
`Credentials::headers`, published `BearerToken` values carry `expires_on: None`
and are evicted from the extension watch channel whenever the SDK reports an
expired or failed token refresh.

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
`google-cloud-auth` HTTP client has a TLS crypto provider installed.

## Configuration

All fields are optional; an empty `config: {}` uses Application Default
Credentials (ADC) with the `https://www.googleapis.com/auth/cloud-platform`
scope, falling back to the default service account on
`http://metadata.google.internal` when no ADC file is present. When the
`GCE_METADATA_HOST` environment variable is set, `google-cloud-auth` gives
`http://$GCE_METADATA_HOST` precedence over `endpoint` for metadata server
requests.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `endpoint` | string | omitted (uses ADC, falling back to `http://metadata.google.internal` or `http://$GCE_METADATA_HOST`) | Optional GCE/GKE metadata server base URL override. When set, bypasses local ADC file discovery and queries this metadata server directly. Must start with `http://` or `https://`. |
| `scopes` | list of strings | `["https://www.googleapis.com/auth/cloud-platform"]` | OAuth 2.0 scopes requested for the access token when non-empty. |
| `timeout` | duration | `10s` | Per-attempt timeout for acquiring a token via `google-cloud-auth`. Must be non-zero. |
| `startup_timeout` | duration | `30s` | Maximum time pipeline startup waits for the first token publish. Must be non-zero. |

## Telemetry

Each extension instance registers the `extension.gcp_auth` metric set:

| Metric | Instrument | Unit | Description |
| --- | --- | --- | --- |
| `auth_successes` | `Counter<u64>` | `{acquisition}` | Number of successful token acquisitions. |
| `auth_failures` | `Counter<u64>` | `{acquisition}` | Number of failed token acquisitions. |
| `auth_token_publish` | `Counter<u64>` | `{token}` | Number of tokens published to consumers via the watch channel. |
| `auth_success_latency` | `Mmsc` | `ms` | Latency of successful token acquisitions (min/max/sum/count). |

Failed background token refresh attempts also emit the
`gcp_auth.token_refresh_failed` warning event with the acquisition error.
