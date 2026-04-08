# Admin API SDK

`otap-df-admin-api` is the public Rust integration crate for the OTAP Dataflow
Engine admin surface. External applications should depend on this crate rather
than the server implementation crate.

It provides:

- a re-export of the public engine configuration model via
  `otap_df_admin_api::config`;
- shared admin request and response models;
- an async `AdminClient` facade for the admin endpoints currently exposed by
  the public SDK;
- explicit endpoint and transport settings for direct HTTP, HTTPS with custom
  CA trust, and mTLS.

The current admin surface is HTTP-based, but this crate is designed to keep
client code stable as the engine grows additional authentication modes or other
admin transports in the future. In v1, `AdminAuth::None` is the only supported
authentication mode.

## Deployment model

Today the engine exposes its admin API as HTTP. In practice, integrators
usually use one of these topologies:

- direct loopback or private-network access to the engine over plaintext HTTP;
- remote access through an HTTPS reverse proxy or gateway that terminates TLS
  and forwards to the engine's HTTP admin bind;
- a path-prefixed gateway in front of the engine, using `AdminEndpoint` base
  path support.

The same `AdminClient` API works in all of these cases. Callers choose the
endpoint scheme and TLS settings without changing the domain methods they use.

## Cargo features

Default SDK usage is usually enough:

```toml
[dependencies]
otap-df-admin-api = "0.1.0"
```

This enables:

- `http-client`
- `crypto-ring`

If you need a different rustls crypto backend, disable default features and
prefer enabling a single provider feature explicitly:

```toml
[dependencies]
otap-df-admin-api = { version = "0.1.0", default-features = false, features = ["http-client", "crypto-aws-lc"] }
```

Available provider features:

- `crypto-ring`: default general-purpose backend.
- `crypto-aws-lc`: alternative rustls backend.
- `crypto-openssl`: recommended starting point for regulated or FIPS-oriented
  deployments that need an OpenSSL-based cryptographic stack.
- If feature unification enables more than one provider, the SDK chooses one
  deterministically with this precedence:
  `crypto-openssl` > `crypto-aws-lc` > `crypto-ring`.

For FIPS-oriented deployments, start with:

```toml
[dependencies]
otap-df-admin-api = { version = "0.1.0", default-features = false, features = ["http-client", "crypto-openssl"] }
```

Important note:

- `crypto-openssl` is the recommended feature choice when your environment
  requires an OpenSSL-based cryptographic stack.
- This feature choice alone does not constitute a FIPS compliance or
  certification claim. Your OpenSSL build, runtime configuration, operating
  environment, and deployment validation remain your responsibility.

## Building a client

### Plain HTTP

Use this when the client connects directly to the engine on loopback or inside
an internal trusted network.

```rust
use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AdminClient::builder()
    .http(HttpAdminClientSettings::new(AdminEndpoint::http("127.0.0.1", 8080)))
    .build()?;

let status = client.engine().status().await?;
println!("pipelines={}", status.pipelines.len());
# Ok(())
# }
```

### HTTPS with a custom CA

Use this when the client connects through a reverse proxy, ingress, or gateway
that presents a TLS certificate signed by an internal CA.

```rust
use otap_df_admin_api::{
    config::tls::TlsClientConfig, AdminClient, AdminEndpoint,
    HttpAdminClientSettings,
};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AdminClient::builder()
    .http(
        HttpAdminClientSettings::new(AdminEndpoint::https("admin.example.com", 8443)).with_tls(
            TlsClientConfig {
                ca_file: Some("/etc/otap/admin-ca.pem".into()),
                include_system_ca_certs_pool: Some(true),
                ..TlsClientConfig::default()
            },
        ),
    )
    .build()?;

let ready = client.engine().readyz().await?;
println!("ready={:?}", ready.status);
# Ok(())
# }
```

### HTTPS with mTLS

Use this when the proxy or gateway requires client certificate authentication.

```rust
use otap_df_admin_api::{
    config::tls::{TlsClientConfig, TlsConfig},
    AdminClient, AdminEndpoint, HttpAdminClientSettings,
};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AdminClient::builder()
    .http(
        HttpAdminClientSettings::new(AdminEndpoint::https("admin.example.com", 8443)).with_tls(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: Some("/etc/otap/admin-client.crt".into()),
                    key_file: Some("/etc/otap/admin-client.key".into()),
                    ..TlsConfig::default()
                },
                ca_file: Some("/etc/otap/admin-ca.pem".into()),
                include_system_ca_certs_pool: Some(true),
                ..TlsClientConfig::default()
            },
        ),
    )
    .build()?;

let livez = client.engine().livez().await?;
println!("livez={:?}", livez.status);
# Ok(())
# }
```

### HTTPS with temporary verification bypass

Use this only for controlled lab or bring-up scenarios where you knowingly
accept certificate validation risk.

```rust
use otap_df_admin_api::{
    config::tls::TlsClientConfig, AdminClient, AdminEndpoint,
    HttpAdminClientSettings,
};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AdminClient::builder()
    .http(
        HttpAdminClientSettings::new(AdminEndpoint::https(
            "staging-admin.example.com",
            8443,
        ))
        .with_tls(TlsClientConfig {
            insecure_skip_verify: Some(true),
            ..TlsClientConfig::default()
        }),
    )
    .build()?;

let status = client.engine().status().await?;
println!("generated_at={}", status.generated_at);
# Ok(())
# }
```

### HTTPS behind a path-prefixed gateway

Use this when the proxy exposes the admin API under a URL prefix instead of at
the origin root.

```rust
use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let endpoint = AdminEndpoint::from_url("https://admin.example.com/engine-a")?;
let client = AdminClient::builder()
    .http(HttpAdminClientSettings::new(endpoint))
    .build()?;

let status = client.engine().status().await?;
println!("pipelines={}", status.pipelines.len());
# Ok(())
# }
```

## Transport notes

- Protocol selection is explicit: use `AdminEndpoint::http(...)` or
  `AdminEndpoint::https(...)`.
- `AdminEndpoint::from_socket_addr(...)` is available for direct address-based
  integrations.
- `AdminEndpoint::from_url(...)` or `with_base_path(...)` can target
  path-prefixed reverse proxies and gateways.
- `HttpAdminClientSettings` also exposes explicit connect timeout, request
  timeout, `TCP_NODELAY`, and keepalive settings.
- `AdminAuth::None` is the only supported auth mode today.
- `server_name_override` is not currently supported by the SDK transport.
- `tls.insecure` is not used to switch protocols. Use an `http` endpoint for
  plaintext connections instead.
- `insecure_skip_verify` is supported only when a caller explicitly enables it.

## Public admin endpoints

The SDK currently exposes the following public `/api/v1` admin endpoints. The
engine may expose additional admin routes that are not part of the public SDK
surface yet. The table below maps each public SDK route to the Rust client
method and its operational purpose.

| Route | SDK method | Purpose |
| --- | --- | --- |
| `GET /api/v1/status` | `engine().status()` | Full engine status snapshot across pipelines and cores. |
| `GET /api/v1/livez` | `engine().livez()` | Engine liveness probe with structured failure details. |
| `GET /api/v1/readyz` | `engine().readyz()` | Readiness probe for orchestration or traffic gating. |
| `GET /api/v1/pipeline-groups/status` | `pipeline_groups().status()` | Fleet-style pipeline status view. |
| `POST /api/v1/pipeline-groups/shutdown` | `pipeline_groups().shutdown(...)` | Coordinated shutdown request across running pipelines. |
| `GET /api/v1/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/status` | `pipelines().status(...)` | Detailed status for a single pipeline. |
| `GET /api/v1/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/livez` | `pipelines().livez(...)` | Semantic liveness probe result for a single pipeline. |
| `GET /api/v1/pipeline-groups/{pipeline_group_id}/pipelines/{pipeline_id}/readyz` | `pipelines().readyz(...)` | Semantic readiness probe result for a single pipeline. |
| `GET /api/v1/telemetry/logs` | `telemetry().logs(...)` | Retained admin logs when log retention is enabled. |
| `GET /api/v1/telemetry/metrics` | `telemetry().metrics(...)`, `telemetry().metrics_compact(...)` | Current engine metrics as structured JSON, using either the full or compact response shape. |

`GET /api/v1/metrics` remains a server-side alias for
`GET /api/v1/telemetry/metrics`. The SDK intentionally exposes only the
canonical `telemetry().metrics(...)` and `telemetry().metrics_compact(...)`
methods.

## Future evolution: live reconfiguration

Future live reconfiguration work is expected to extend the admin SDK from a
status-and-observability client into a richer control-plane client for
long-lived engine instances. The details are not stabilized yet, but the work
in progress already helps frame the direction for advanced integrators building
external controllers.

Main capabilities expected from this area of the admin API:

- read the live committed configuration for a single logical pipeline;
- create, replace, resize, or accept a `noop` update for one logical pipeline;
- track rollout progress through a dedicated rollout resource;
- track per-pipeline shutdown progress through a dedicated shutdown resource;
- expose generation-aware pipeline status during overlapping cutover.

The current SDK is intentionally narrower, and the main future extensions for
live reconfiguration are expected to center on:

- resource model: adding live pipeline details, rollout status, and shutdown
  status as first-class SDK resources instead of exposing only snapshots and
  probes;
- status shape: extending pipeline status with generation-aware fields such as
  `activeGeneration`, `servingGenerations`, rollout summaries, and
  per-generation instance views;
- operation semantics: treating create, replace, resize, and shutdown as
  long-running admin operations with both immediate-return and wait-or-poll
  interaction patterns;
- error and outcome modeling: representing rollout conflicts, validation
  failures, and timeout outcomes as typed SDK results rather than leaving them
  as transport-level concerns.

The intended integration direction is to keep `AdminClient` as the stable
entrypoint and absorb those changes behind typed client methods rather than
exposing raw route strings as the public contract. In practice, that likely
means:

- keeping transport and route-version differences behind backend adapters;
- adding job-oriented client methods for live pipeline read, update, rollout
  status, and per-pipeline shutdown tracking;
- supporting both immediate-return and wait-or-poll interaction patterns for
  long-running admin operations;
- continuing to treat experimental endpoints as opt-in additions only after
  their semantics and wire format settle.

## Client API cookbook

### Global status, liveness, and readiness

Use these methods for orchestration, startup checks, and fleet snapshots.

```rust
use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AdminClient::builder()
    .http(HttpAdminClientSettings::new(AdminEndpoint::http("127.0.0.1", 8080)))
    .build()?;

let status = client.engine().status().await?;
let livez = client.engine().livez().await?;
let readyz = client.engine().readyz().await?;

println!("status generated_at={}", status.generated_at);
println!("livez={:?}", livez.status);
println!("readyz={:?}", readyz.status);
# Ok(())
# }
```

### Pipeline group status and coordinated shutdown

Use this when an operator or control plane needs a fleet view and a single
engine-wide shutdown entrypoint.

```rust
use otap_df_admin_api::{
    operations::OperationOptions, AdminClient, AdminEndpoint,
    HttpAdminClientSettings,
};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AdminClient::builder()
    .http(HttpAdminClientSettings::new(AdminEndpoint::http("127.0.0.1", 8080)))
    .build()?;

let groups = client.pipeline_groups().status().await?;
println!("pipelines={}", groups.pipelines.len());

let shutdown = client
    .pipeline_groups()
    .shutdown(&OperationOptions {
        wait: true,
        timeout_secs: 30,
    })
    .await?;
println!("shutdown={:?}", shutdown.status);
# Ok(())
# }
```

### Single-pipeline status and probes

Use this when an external controller tracks one pipeline at a time. The public
SDK returns transport-agnostic probe results here; the current HTTP backend
maps the server's plain-text probe endpoints into this semantic shape.

```rust
use otap_df_admin_api::{AdminClient, AdminEndpoint, HttpAdminClientSettings};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AdminClient::builder()
    .http(HttpAdminClientSettings::new(AdminEndpoint::http("127.0.0.1", 8080)))
    .build()?;

let status = client.pipelines().status("default", "main").await?;
let livez = client.pipelines().livez("default", "main").await?;
let readyz = client.pipelines().readyz("default", "main").await?;

println!("pipeline_status_present={}", status.is_some());
println!("pipeline_livez={:?} {:?}", livez.status, livez.message);
println!("pipeline_readyz={:?} {:?}", readyz.status, readyz.message);
# Ok(())
# }
```

### Retained logs

Use retained logs for operational debugging when the server is configured to
keep them.

```rust
use otap_df_admin_api::{
    telemetry::LogsQuery, AdminClient, AdminEndpoint, HttpAdminClientSettings,
};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AdminClient::builder()
    .http(HttpAdminClientSettings::new(AdminEndpoint::http("127.0.0.1", 8080)))
    .build()?;

let logs = client
    .telemetry()
    .logs(&LogsQuery {
        after: None,
        limit: Some(200),
    })
    .await?;

match logs {
    Some(logs) => {
        println!("retained_logs={}", logs.logs.len());
        println!("next_cursor={}", logs.next_seq);
    }
    None => println!("retained logs endpoint disabled"),
}
# Ok(())
# }
```

### Metrics

Use `telemetry().metrics(...)` for the full structured JSON view, or
`telemetry().metrics_compact(...)` for the compact structured JSON view.

```rust
use otap_df_admin_api::{
    telemetry::MetricsOptions,
    AdminClient, AdminEndpoint, HttpAdminClientSettings,
};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = AdminClient::builder()
    .http(HttpAdminClientSettings::new(AdminEndpoint::http("127.0.0.1", 8080)))
    .build()?;

let metrics = client
    .telemetry()
    .metrics_compact(&MetricsOptions {
        reset: false,
        keep_all_zeroes: false,
    })
    .await?;

println!("compact_metric_sets={}", metrics.metric_sets.len());
# Ok(())
# }
```
