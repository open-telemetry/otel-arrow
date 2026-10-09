<!-- markdownlint-disable MD013 -->

# Azure Key Vault SASL Auth Extension

## Metadata

- URN: `urn:otel:extension:azure_key_vault_sasl_auth`
- Feature gate: `azure-key-vault-sasl-auth` (or `contrib-extensions`)
- Capability: `sasl_credential_provider`
- Execution model: Active + Shared
- Stability: Draft

## Overview

Retrieves **both** the initial SASL username and password from Azure Key Vault.
There is no inline or file fallback. The provider publishes one complete,
non-expiring `SaslCredential` only after both reads and validations succeed.
Username colons, whitespace, and other nonempty SASL values are preserved
exactly; consumers own mechanism-specific PLAIN/SCRAM validation.

Secret refresh and rotation are not supported. **Restart the pipeline to adopt
changed secrets**, even if their Key Vault metadata expires later. Azure SDK
access-token caching/acquisition is distinct from secret rotation.

The identity constructor, Key Vault configuration/client, secret validation, and
safe acquisition diagnostics are protocol-neutral shared code. A future Basic
Auth wrapper can reuse acquisition and separately construct `BasicAuthCredential`;
this extension does not convert through Basic Auth.

## Configuration

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          vault_sasl:
            type: urn:otel:extension:azure_key_vault_sasl_auth
            config:
              vault_url: https://my-vault.vault.azure.net
              username_secret:
                name: kafka-username
              password_secret:
                name: kafka-password
              identity:
                method: managed_identity
              startup_timeout: 30s
```

Declare the provider in the pipeline's `extensions` map. Only consumers that
support `sasl_credential_provider` can bind it in their `capabilities` map.
Kafka receiver binding and live broker reconnection are separate work; this
extension alone does not add either behavior.

| Option | Default | Meaning |
| --- | --- | --- |
| `vault_url` | Required | HTTPS vault root for Azure public, US Government, or China cloud. |
| `username_secret.name` | Required | 1-127 ASCII letters, digits, or hyphens. |
| `password_secret.name` | Required | Independent password secret reference. |
| `username_secret.version` | Latest at startup | Optional immutable 32-character hexadecimal version. |
| `password_secret.version` | Latest at startup | Optional independent version pin. |
| `identity.method` | `managed_identity` | Managed identity, workload identity, or local development tooling. |
| `identity.client_id` | None | User-assigned managed identity or workload identity client ID. |
| `identity.tenant_id` | None | Workload identity only; falls back to `AZURE_TENANT_ID`. |
| `identity.token_file_path` | None | Workload identity only; falls back to `AZURE_FEDERATED_TOKEN_FILE`. |
| `startup_timeout` | `30s` | Positive human-readable readiness/acquisition timeout, for example `1m`. |

Unknown fields are rejected at every level, including refresh options. Explicit
empty identity IDs or token paths, invalid secret references, and identity
options inapplicable to the selected method are rejected.

Vault URLs cannot include credentials, custom ports, paths beyond `/`, queries,
or fragments. Use the ordinary vault DNS name even with Private Link, and
configure private DNS/network reachability separately. Do not configure
`privatelink` aliases, IP addresses, proxies, or arbitrary hostnames as the vault
endpoint. SDK challenge-resource verification stays enabled and the SDK's default
transport disables redirects. Key Vault supplies the token resource; this
extension does not request the Azure Monitor OAuth scope.

### Identity and permissions

For production, use system-assigned managed identity by default, or select a
user-assigned identity with `identity.client_id`. Workload identity uses:

```yaml
identity:
  method: workload_identity
  tenant_id: 00000000-0000-0000-0000-000000000000
  client_id: 11111111-1111-1111-1111-111111111111
  token_file_path: /var/run/secrets/azure/tokens/azure-identity-token
```

Omit workload fields to retain the SDK's corresponding `AZURE_*` environment
fallbacks, including `AZURE_CLIENT_ID`. `development` uses Azure CLI / `azd`
credentials **for local development only**; it accepts none of the identity
override fields. Use the SDK's authority configuration for sovereign-cloud
workload identities.

Grant the identity least-privilege read access to both secrets: the **Key Vault
Secrets User** RBAC role at the appropriate scope, or `secrets/get` in an access
policy. No list, write, delete, or management-plane permission is required.
Ensure the vault's firewall/private endpoint and the identity's token endpoints
are reachable from the collector.

### Version selection and coordinated provisioning

Omitting a version selects the SDK's latest version **at startup**, with no
polling. Pin each version explicitly for reproducible provisioning:

```yaml
username_secret:
  name: kafka-username
  version: 0123456789abcdef0123456789abcdef
password_secret:
  name: kafka-password
  version: fedcba9876543210fedcba9876543210
```

The reads are separate requests, **not an atomic snapshot**. Provision matching
values before startup, avoid updating them during startup, or pin a coordinated
pair of versions. Neither successful read alone can signal readiness or publish
usable credentials.

## Startup and failure handling

Client/identity construction occurs during wiring, without secret network I/O.
The active extension retrieves secrets asynchronously. The engine holds
data-path startup until the first complete credential is published, bounded by
`startup_timeout`. A read also has this timeout, and shutdown cancels in-flight
acquisition.

The SDK's bounded transport/retry policy handles transient request failures.
The existing background provider can retry a failed initial acquisition with
backoff while the readiness deadline remains open. Authentication,
authorization, missing secrets/versions, malformed responses, and invalid values
surface as safe errors/events, never fallback credentials. A missing or empty
value or disabled, expired, or not-yet-valid secret is rejected at startup.
Once successful, acquisition stops: late independent subscriptions immediately
yield the cached snapshot and remain open, with no later secret reads.

If startup times out, correct the reported identity, permissions, references,
values, or network problem and restart. Increasing the timeout can accommodate
slow cold-start identity acquisition; it does not repair permanent failures.

## Telemetry

The component metric set `extension.azure_key_vault_sasl_auth` records complete
acquisition successes/failures, credential publications, and successful
acquisition latency in milliseconds. Shared acquisition events identify only
the stage (`identity`, `client`, `username`, `password`, or `acquisition`) and a
bounded error category, never secret values or raw SDK diagnostics.

Raw SDK errors, response bodies, tokens, and their source chains are discarded
before formatting errors or telemetry. Cached credential Debug output is
redacted and its secret allocations are zeroized on drop. Avoid application
instrumentation that explicitly exposes capability values or enables verbose
HTTP-body logging.

## Building

From `rust/otap-dataflow`:

```bash
cargo build --release --features azure-key-vault-sasl-auth
```

Enable exactly one workspace `crypto-*` backend in the deployed binary. The
workspace default enables `crypto-ring`; custom no-default-feature builds must
select a backend explicitly. The Azure SDK uses provider-neutral rustls/reqwest
TLS and reuses the workspace crypto-provider initialization.

Automated tests inject deterministic SDK transports/credentials and use no
real Azure resources, accounts, or secrets.
