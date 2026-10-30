# Azure Key Vault User Pass Auth

<!-- markdownlint-disable MD013 -->

Experimental, opt-in extension `urn:otel:extension:azure_key_vault_user_pass_auth`
providing the existing shared `basic_auth_provider` capability. Oracle,
PostgreSQL, and SQL Server bind it through their existing local capability
adapter; no Azure logic is added to receivers.

Both values come from Azure Key Vault (AKV). The extension constructs
`BasicAuthCredential` directly, **not SASL credentials**. It reuses the common
Azure identity, endpoint validation, SDK client, secret acquisition, and safe
error categories used by the AKV SASL extension.

## Configuration

```yaml
extensions:
  database-credentials:
    type: urn:otel:extension:azure_key_vault_user_pass_auth
    config:
      vault_url: https://example-vault.vault.azure.net
      username_secret:
        name: database-username
        # Optional independent, immutable version pins:
        version: "0123456789abcdef0123456789abcdef"
      password_secret:
        name: database-password
        version: "fedcba9876543210fedcba9876543210"
      identity:
        method: managed_identity
      startup_timeout: 30s
nodes:
  database:
    type: urn:otel:receiver:postgresql
    capabilities:
      basic_auth_provider: database-credentials
    # Receiver-specific connection/query/checkpoint configuration goes here.
```

| Field | Contract |
| --- | --- |
| `vault_url` | Required HTTPS Azure vault root; public, US Government, or China vault suffix. No credentials, nonstandard port, path, query, or fragment. |
| `username_secret.name`, `password_secret.name` | Both required; 1-127 ASCII letters, digits, or hyphens. |
| Each reference's `version` | Optional 32-character hexadecimal version. Omit to select latest at startup. Quote version strings in YAML. |
| `identity` | Optional; defaults to system-assigned managed identity. |
| `startup_timeout` | Positive, representable duration; default `30s`. Bounds engine startup readiness and each acquisition. |

Unknown fields, inline values, file fallbacks, refresh/rotation options, and
custom token scopes are rejected. Prefer **one configured provider instance per
distinct credential pair**. Every database receiver requires one pipeline core.
A missing binding, unknown extension ID, or incompatible capability fails
wiring; enabling a feature alone does not select a provider.

### Identity and authorization

`managed_identity` (default) uses system-assigned identity; set `client_id` for a
user-assigned identity. Workload identity accepts:

```yaml
identity:
  method: workload_identity
  tenant_id: 00000000-0000-0000-0000-000000000000
  client_id: 11111111-1111-1111-1111-111111111111
  token_file_path: /var/run/secrets/azure/tokens/azure-identity-token
```

Omitted workload fields retain the SDK's `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`,
and `AZURE_FEDERATED_TOKEN_FILE` defaults. The projected token file authenticates
Azure access; it is not a database-password fallback. Configure the appropriate
SDK authority for sovereign-cloud workload identities.

For **local development only**, use `identity: {method: development}` with an
already authenticated Azure CLI / `azd`. Development accepts no identity
override fields. Managed identity accepts no tenant or token-file fields.

Grant least-privilege read access to both secrets, such as **Key Vault Secrets
User** RBAC at the appropriate scope or `secrets/get` in an access policy.
No list, write, delete, or management-plane permission is required. Provide
network access to the vault and identity endpoints, including private endpoint
DNS/firewall configuration where applicable.

AKV read authorization and database login/query permissions are **separate**.
AKV does not create database accounts, grant database permissions, or synchronize
database passwords. Provision a matching least-privileged database account.

### Value validation and version coordination

Neither value may be empty. Usernames cannot contain colons or control
characters; passwords cannot contain control characters (including trailing
newlines), but may contain colons. Values are never trimmed or normalized.
Missing, malformed, disabled, expired, or not-yet-valid values are rejected.
Incompatible values fail without exposing their contents.

The two reads are **not an atomic snapshot**. Provision matching values before
startup or pin coordinated versions independently. Do not update unpinned
values during startup. A failure of either read discards the entire attempt;
recovery reads both references again before publishing.

## Lifecycle and telemetry

Identity/client construction performs no secret reads. The engine's active
extension task acquires and validates a complete pair before publishing it and
signaling readiness. Concurrent cache misses coalesce using the existing shared
provider machinery. SDK retries are bounded; the background provider retries
failed initial acquisitions with bounded exponential backoff while the engine's
startup deadline remains open. Shutdown cancels in-flight acquisition, including
SDK retry delays. A readiness deadline failure prevents data-path startup and
shuts down the extension.

After the first successful publication, all calls and late subscriptions reuse
the **startup-only cached snapshot**. There are no further vault reads on
reconnect, database authentication failure, or elapsed time. Streams stay open
without refresh events. Restart the pipeline to adopt changed secrets.
Later vault updates, expiry, deletion, or disablement do **not** refresh or
revoke the cached pair, nor replace a healthy database session.

Readiness means a valid pair is available, **not** that a database accepted it.
The metric set `extension.azure_key_vault_user_pass_auth` reports acquisition
successes/failures, complete credential publications, and successful acquisition
latency in milliseconds, separately from database receiver telemetry. These
follow the shared provider's existing metrics contract.

Shared `azure_key_vault.acquisition_failed` events carry only a bounded stage
(`identity`, `client`, `username`, `password`, `acquisition`) and failure category.
Basic Auth validation failures identify `username` or `password` with the
`invalid_value` category, so operators can correct the offending secret.
Raw SDK errors, response bodies, tokens, and source chains are discarded.
Secret and credential Debug output is redacted; secret allocations are zeroized
on drop. Do not enable HTTP-body instrumentation or log exposed capability
values. Configuration parsing errors do not echo malformed input.

## Builds and complete examples

From `rust/otap-dataflow`, select the extension and the desired receiver:

```sh
cargo build --no-default-features --features azure-key-vault-user-pass-auth,postgresql,crypto-ring --bin df_engine
```

Replace `postgresql` with `oracle` or `sql-server`. Complete pipelines:

- [Oracle](../../../../configs/oracle-akv-console.yaml)
- [PostgreSQL](../../../../configs/postgresql-akv-console.yaml)
- [SQL Server](../../../../configs/sql-server-akv-console.yaml)

Replace example endpoints, secret references, connection parameters, and state
directories before running. Keep PostgreSQL and SQL Server TLS verification
enabled; add a trusted CA bundle when platform roots are insufficient. Oracle
transport protection remains the existing OCI/Oracle Net deployment
responsibility; configure it according to the
[Oracle receiver guidance](../../../contrib-nodes/src/receivers/oracle_receiver/README.md).
The extension does not alter TLS, ACK/NACK, checkpoint, or source ownership.
The console illustrates feedback but is not durable downstream storage.

The extension is independently selectable without SASL, Azure bearer-token, or
file-backed provider features. Existing root database feature bundles also
register the file provider for backward compatibility; this does not make AKV
use it or permit fallback.

Enable exactly one workspace crypto provider: `crypto-ring`, `crypto-aws-lc`,
`crypto-openssl` (Linux), or `crypto-symcrypt` (Windows). Azure dependencies keep
default crypto features disabled and reuse the selected provider. SQL Server's
existing native TLS transport is separate. The shared Azure SDK packages are
published preview versions, not a product/FIPS qualification claim.

## Validation and live qualification

Offline SDK transports test success, either-secret/identity failures, malformed
and inactive values, Basic Auth restrictions, concurrency, complete publication,
startup retry, cancellation, and redaction. Real receiver factory integration
tests use the complete examples to verify capability binding, exclusive source
ownership, and stop-before-start reconstruction without accessing Azure or a
database:

```sh
cargo test -p otel-arrow-dfe-contrib-extensions --no-default-features --features azure-key-vault-user-pass-auth,crypto-ring --lib
cargo test -p otel-arrow-dfe-validation --no-default-features --features akv-database-bindings,crypto-ring --test akv_database_bindings
```

Offline success is **not live qualification**. Live AKV/database qualification
must be recorded separately for **each** database; unavailable resources are
**blocked**, never a passing or silently skipped qualification.

Required authorized prerequisites: an existing vault, two readable secrets
containing a matching database login, a supported configured identity,
network access, an existing database/table with deterministic commit-visible
ordered rows and reader permissions, verified transport/trust configuration,
Oracle Instant Client where applicable, persistent writable checkpoint storage,
and a feedback-preserving downstream destination. Do not provision resources,
change permissions, or manipulate credentials as part of an offline test.

For each database, an authorized operator must:

1. Adapt its example and start the feature-enabled engine with `--config` and
   `--http-admin-bind 127.0.0.1:0`. Record the selected identity method and
   nonsecret version provenance, not secret contents or tokens.
2. Verify complete acquisition/publication, a successful database login,
   emitted expected rows, downstream ACKs, and durable checkpoint advancement.
   Capture trusted-transport evidence separately from provider readiness.
3. Stop gracefully, retain the same logical source and state directory, add
   authorized fixture rows, restart, and verify resume after acknowledged
   progress. Account for documented at-least-once replay windows.
4. With authorized fault injection, interrupt the database session and verify
   reconnect uses the unchanged snapshot without new vault acquisitions.
   Exercise rejected login and confirm no checkpoint advance or secret leakage.
   Changing credentials or vault permissions requires separate authorization.
5. Record engine revision/features, redacted configuration, identity/transport,
   row and checkpoint evidence, failure/reconnect evidence, and pass/fail/blocked
   status. Do not infer combined AKV qualification from file-provider database
   fixtures or standalone SDK tests.
