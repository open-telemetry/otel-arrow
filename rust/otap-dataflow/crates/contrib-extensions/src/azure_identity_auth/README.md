<!-- markdownlint-disable MD013 -->

# Azure Identity Auth Extension

## Metadata

- URN: `urn:microsoft:extension:azure_identity_auth`
- Feature gate: `azure-identity-auth-extension` (or the aggregate `contrib-extensions`)
- Capability provided: `bearer_token_provider`
- Execution model: Active + Shared
- Stability: Draft

## Overview

Acquires and refreshes Azure OAuth access tokens via the `azure_identity` SDK and
exposes them to data-path nodes through the `bearer_token_provider` capability,
so nodes never construct credentials or manage token refresh themselves. Three
authentication flows are supported: Managed Identity, local developer tooling,
and Workload Identity Federation.

Tokens are cached and refreshed ahead of expiry in a background task, concurrent
cache misses are coalesced onto a single credential call, and pipeline startup is
held until the first token is published.

This README is the configuration reference for the extension. Nodes that consume
the capability (for example the
[Azure Monitor exporter](../../../contrib-nodes/src/exporters/azure_monitor_exporter/README.md)
and the
[OTLP HTTP exporter](../../../core-nodes/src/exporters/otlp_http_exporter/README.md))
document only how they *use* a bearer token, not how to configure a provider.

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
          azure_identity:
            type: "urn:microsoft:extension:azure_identity_auth"
            config:
              method: managed_identity
              scope: "https://monitor.azure.com/.default"

        nodes:
          azure-monitor-exporter:
            type: "urn:microsoft:exporter:azure_monitor"
            # Bind the capability to the extension instance declared above.
            capabilities:
              bearer_token_provider: azure_identity
            config:
              api:
                dcr_endpoint: "https://my-workspace.eastus-1.ingest.monitor.azure.com"
                stream_name: "Custom-MyLogTable_CL"
                dcr: "dcr-abc123def456"
```

One extension instance serves one identity and one scope. Consumers that need a
different Azure resource need their own instance -- for example
`https://monitor.azure.com/.default` for Azure Monitor and
`https://storage.azure.com/.default` for Blob Storage. Declare each under its own
name and bind consumers accordingly.

## Building

Enable the extension's feature gate together with the nodes that consume it.
From the `otap-dataflow` directory:

```bash
cargo build --release \
  --features azure-identity-auth-extension,azure-monitor-exporter
```

The extension talks to Azure over TLS through the Azure SDK's `reqwest`/`rustls`
client, which requires a process-wide `rustls` crypto provider. The deployed
binary **must** enable exactly one `crypto-*` feature (`crypto-ring`,
`crypto-aws-lc`, `crypto-openssl`, or `crypto-symcrypt`, forwarded to
`otap-df-otap`); the workspace binary's default build includes `crypto-ring`. A
build that enables `azure-identity-auth-extension` without any `crypto-*` feature
installs no provider, and token acquisition panics at runtime with "No provider
set".

Verify registration with `./target/release/df_engine --help`;
`urn:microsoft:extension:azure_identity_auth` appears in the Extensions list.

## Configuration

Unknown fields are rejected, and the whole config is validated before the
pipeline starts, so a mistake fails at startup rather than on the first export.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `method` | enum | `managed_identity` | Authentication flow. See [Authentication methods](#authentication-methods). |
| `client_id` | string | *none* | Entra client ID. For `managed_identity`, the user-assigned identity's client ID (omit for the system-assigned identity). For `workload_identity`, the application client ID; falls back to `AZURE_CLIENT_ID`. Not valid for `development`. |
| `tenant_id` | string | *none* | Entra tenant ID. Only valid for `workload_identity`; falls back to `AZURE_TENANT_ID`. |
| `token_file_path` | path | *none* | Projected federated token file. Only valid for `workload_identity`; falls back to `AZURE_FEDERATED_TOKEN_FILE`. |
| `scope` | string | `https://monitor.azure.com/.default` | OAuth scope to request tokens for. Must be non-empty. |
| `startup_timeout` | duration | `30s` | How long the engine holds data-path node startup waiting for the first token publish before aborting startup. Must be non-zero. |

`startup_timeout` accepts human-readable durations such as `30s` or `1m`.

### Authentication methods

| `method` | Aliases | Credential | Notes |
| --- | --- | --- | --- |
| `managed_identity` | `msi`, `managedidentity` | `ManagedIdentityCredential` | System-assigned by default; set `client_id` for a user-assigned identity. Requires no secrets in the pipeline config. |
| `development` | `dev`, `developer`, `cli` | `DeveloperToolsCredential` | Uses the operator's local Azure CLI / `azd` session. Local development only. |
| `workload_identity` | `wif`, `workloadidentity` | `WorkloadIdentityCredential` | Exchanges a projected federated ServiceAccount token for an Entra ID access token. For Kubernetes workloads without a managed identity (self-hosted or non-AKS). |

Workload Identity Federation reads `client_id`, `tenant_id`, and
`token_file_path`, each falling back to the `AZURE_CLIENT_ID` /
`AZURE_TENANT_ID` / `AZURE_FEDERATED_TOKEN_FILE` environment variables injected
by the Azure Workload Identity webhook, so all three may be omitted from the
config:

```yaml
type: "urn:microsoft:extension:azure_identity_auth"
config:
  method: workload_identity
  scope: "https://monitor.azure.com/.default"
  # All three are optional; they fall back to the standard AZURE_* env vars.
  client_id: "00000000-0000-0000-0000-000000000000"
  tenant_id: "11111111-1111-1111-1111-111111111111"
  token_file_path: "/var/run/secrets/azure/tokens/azure-identity-token"
```

### Validation rules

Config validation rejects:

- an empty or whitespace-only `scope`;
- a zero `startup_timeout`;
- `tenant_id` or `token_file_path` with a method other than `workload_identity`;
- `client_id` with the `development` method;
- unknown fields.

## Security Notes

- Token secrets are held only in memory and are never logged; log and telemetry
  sites emit credential type, scope, and timing only.
- `managed_identity` and `workload_identity` require no secrets in the pipeline
  config, which is the recommended posture for production.
- `development` relies on the operator's local Azure CLI session and is intended
  for local development only.
- The extension requests exactly the configured `scope`; scope it to the least
  privilege the consumer needs.

## Telemetry

Metric set: `extension.azure_identity_auth`.

| Metric | Type | Unit | Description |
| --- | --- | --- | --- |
| `auth_successes` | Counter | `{acquisition}` | Successful credential acquisitions. |
| `auth_failures` | Counter | `{acquisition}` | Failed credential acquisitions. |
| `auth_token_publish` | Counter | `{token}` | Tokens published to consumers. |
| `auth_success_latency` | Mmsc | `ms` | Latency of successful acquisitions (min/max/sum/count). |

Events:

| Event | Severity | Description |
| --- | --- | --- |
| `azure_identity_auth.token_refresh_failed` | `warn` | A credential acquisition failed; the loop retries with backoff. |

## Troubleshooting

| Symptom | Likely cause |
| --- | --- |
| Panic at runtime with "No provider set" | The binary was built without a `crypto-*` feature. See [Building](#building). |
| Startup aborts with a readiness timeout | The first token was not acquired within `startup_timeout`. Check identity assignment, network reachability of IMDS or Entra ID, and the `azure_identity_auth.token_refresh_failed` event. |
| Config rejected at startup | See [Validation rules](#validation-rules); the error names the offending field. |
| `403` from the Azure service despite successful acquisition | The identity lacks a role assignment on the target resource, or `scope` targets the wrong resource. |
| Exporter stops accepting data mid-run | Refresh is failing and the cached token lapsed. Check `auth_failures` and the refresh-failure event. |

## Related Docs

- [Design](./design.md)
- [Contrib extension catalog](../../README.md)
- [Writing pipeline configuration](../../../../docs/configuration.md)
- [Configuration model](../../../../docs/configuration-model.md)
- [Extension system architecture](../../../../docs/extension-system-architecture.md)
