# Azure Monitor Exporter

## Metadata

- Type: `urn:microsoft:exporter:azure_monitor`
- Feature gate: `azure-monitor-exporter`
- Stability: Alpha; supports logs only

## Overview

The Azure Monitor Exporter sends OpenTelemetry logs to Azure using the
[Azure Logs Ingestion API][logs-api]. It transforms OTLP log data into
the format expected by Azure Log Analytics and provides configurable schema
mapping for custom log tables.

The exporter does not acquire credentials itself. It authenticates using OAuth
bearer tokens supplied by the
[`azure_identity_auth`](../../../../contrib-extensions/src/azure_identity_auth/README.md)
extension, which it consumes through the `bearer_token_provider` capability. The
extension owns all credential acquisition and token refresh; the exporter simply
reacts to each token it publishes. See [Authentication](#authentication) for how
to wire the two together.

Telemetry reference: [telemetry.md](telemetry.md)

## Getting Started

Declare an `azure_identity_auth` extension and bind it to the exporter node via
the `bearer_token_provider` capability, then point the exporter at your Azure
Logs Ingestion target:

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          azure_identity:
            type: "urn:microsoft:extension:azure_identity_auth"
            config:
              method: managed_identity   # or "development", "workload_identity"
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

## Build df_engine with Azure Monitor Exporter

The exporter requires a bound `bearer_token_provider`. Build it together with the
`azure_identity_auth` extension that supplies one. From the `otap-dataflow`
directory:

```bash
cargo build --release \
  --features azure-monitor-exporter,azure-identity-auth-extension
```

The `azure_identity_auth` extension talks to Azure over TLS and needs a
process-wide `rustls` crypto provider. The workspace binary's default build
already enables `crypto-ring`; if you build a custom binary, enable exactly one
`crypto-*` feature (`crypto-ring`, `crypto-aws-lc`, `crypto-openssl`, or
`crypto-symcrypt`), otherwise token acquisition panics at runtime with "No
provider set". See the
[extension README](../../../../contrib-extensions/src/azure_identity_auth/README.md)
for details.

## Verify the exporter is registered

```bash
./target/release/df_engine --help
```

You should see `urn:microsoft:exporter:azure_monitor` in the Exporters list.

## Configuration

The exporter's own `config` covers the Data Collection Rule target, schema
mapping, and heartbeat. Authentication is configured on the bound
`azure_identity_auth` extension, not here (see [Authentication](#authentication)).

```yaml
type: urn:microsoft:exporter:azure_monitor
# Bind the bearer token provider to an azure_identity_auth extension instance
# declared in the pipeline's `extensions:` section.
capabilities:
  bearer_token_provider: azure_identity
config:
  # Azure Monitor API configuration (required).
  api:
    dcr_endpoint: "https://my-workspace.eastus-1.ingest.monitor.azure.com"
    stream_name: "Custom-MyLogTable_CL"
    dcr: "dcr-abc123def456"
    # Schema mapping is optional; only the sections you configure are emitted
    # (unmapped resource/scope attributes and log-record fields are dropped, not
    # passed through).
    schema:
      # Map OTLP resource attributes to Azure fields.
      resource_mapping:
        "service.name": "ServiceName"
        "service.version": "ServiceVersion"
        "host.name": "HostName"
        "deployment.environment": "Environment"
      # Map OTLP scope attributes to Azure fields.
      scope_mapping:
        "otel.library.name": "InstrumentationLibrary"
        "otel.library.version": "InstrumentationVersion"
      # Map OTLP log record fields to Azure fields.
      log_record_mapping:
        "body": "Message"
        "severity_text": "SeverityText"
        "time_unix_nano": "TimeGenerated"
        "trace_id": "TraceId"
        "span_id": "SpanId"
        # `attributes` is either an explicit per-attribute mapping (below) or the
        # string `passthrough` to emit every log attribute as-is as its own
        # top-level "key": value column. See "Attribute Passthrough Mode" below.
        "attributes":
          "message": "ParsedMessage"

  # Optional heartbeat rows.
  heartbeat:
    enabled: false
    frequency: 60s
```

### Authentication

The exporter obtains OAuth bearer tokens from the
[`azure_identity_auth`](../../../../contrib-extensions/src/azure_identity_auth/README.md)
extension through the `bearer_token_provider` capability. Wiring has two parts:

1. Declare the extension instance in the pipeline's `extensions:` section.
2. Bind it on the exporter node via the node's `capabilities:` map, e.g.
   `bearer_token_provider: <instance-name>`.

The authentication flow is chosen by the extension's `method` field:

- `managed_identity` (aliases: `msi`, `managedidentity`) - Azure Managed
  Identity. Set `client_id` for a user-assigned identity; omit it for the
  system-assigned identity.
- `development` (aliases: `dev`, `developer`, `cli`) - local Azure developer
  credentials (Azure CLI / Azure Developer CLI).
- `workload_identity` (aliases: `wif`, `workloadidentity`) - Workload Identity
  Federation. Reads a projected federated ServiceAccount token and exchanges it
  with Entra ID for an access token. Useful for Kubernetes workloads without a
  managed identity (e.g. self-hosted or non-AKS clusters). Uses `client_id`,
  `tenant_id`, and `token_file_path`, each falling back to the standard
  `AZURE_CLIENT_ID` / `AZURE_TENANT_ID` / `AZURE_FEDERATED_TOKEN_FILE`
  environment variables injected by the Azure Workload Identity webhook.

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          azure_identity:
            type: "urn:microsoft:extension:azure_identity_auth"
            config:
              method: workload_identity
              scope: "https://monitor.azure.com/.default"
              # All fields below are optional; fall back to the standard AZURE_* env vars.
              client_id: "00000000-0000-0000-0000-000000000000"
              tenant_id: "11111111-1111-1111-1111-111111111111"
              token_file_path: "/var/run/secrets/azure/tokens/azure-identity-token"

        nodes:
          azure-monitor-exporter:
            type: "urn:microsoft:exporter:azure_monitor"
            capabilities:
              bearer_token_provider: azure_identity
            config:
              api:
                dcr_endpoint: "https://my-workspace.eastus-1.ingest.monitor.azure.com"
                stream_name: "Custom-MyLogTable_CL"
                dcr: "dcr-abc123def456"
```

For the full extension configuration reference (including `scope` and
`startup_timeout`), see the
[`azure_identity_auth` extension README](../../../../contrib-extensions/src/azure_identity_auth/README.md)
and its [design doc](../../../../../docs/azure-identity-auth-extension.md).

## Usage

### Running

```bash
./target/release/df_engine --config config.yaml --num-cores 4
```

### Testing with OTLP Receiver

To test using the provided configuration file:

```bash
# Start the collector
./target/release/df_engine \
  --config crates/otap/src/experimental/azure_monitor_exporter/otlp-ame.yaml \
  --num-cores 1

# In another terminal, send test data:

# Option A: Using telemetrygen (recommended)
telemetrygen logs --otlp-endpoint localhost:4317 --otlp-insecure --logs 10

# Option B: Using grpcurl for manual testing
grpcurl -plaintext \
  -import-path ../otel-arrow/proto/opentelemetry-proto \
  -proto opentelemetry/proto/logs/v1/logs.proto \
  -proto opentelemetry/proto/collector/logs/v1/logs_service.proto \
  -proto opentelemetry/proto/common/v1/common.proto \
  -proto opentelemetry/proto/resource/v1/resource.proto \
  -d '{
    "resourceLogs": [{
      "scopeLogs": [{
        "logRecords": [{
          "body": {"stringValue": "Hello from Azure Monitor Exporter!"},
          "severityText": "INFO"
        }]
      }]
    }]
  }' \
  localhost:4317 \
  opentelemetry.proto.collector.logs.v1.LogsService/Export

# Option C: Configure your instrumented app to send OTLP Logs to localhost:4317
```

## Schema Mapping

The Azure Monitor Exporter provides flexible schema mapping to transform OTLP data
structures into Azure Log Analytics table format:

### Resource Mapping

Maps OpenTelemetry resource attributes to Azure fields:

```yaml
resource_mapping:
  "service.name": "ServiceName"      # Maps service.name to ServiceName
  "host.name": "MachineName"         # Maps host.name to MachineName
```

### Scope Mapping

Maps OpenTelemetry instrumentation scope to Azure fields:

```yaml
scope_mapping:
  "otel.library.name": "LibraryName"
  "otel.library.version": "LibraryVersion"
```

### Log Record Mapping

Maps OTLP log record fields to Azure columns:

```yaml
log_record_mapping:
  "body": "Message"                  # Log body to Message column
  "time_unix_nano": "TimeGenerated"  # Timestamp to TimeGenerated
  "severity_text": "Level"           # Severity to Level column
  "trace_id": "TraceId"              # Trace ID to TraceId column
  "attributes":                      # Nested attribute mapping
    "user.id": "UserId"              # Specific attribute mapping
```

### Attribute Passthrough Mode

By default only the attributes you list under `log_record_mapping.attributes`
are emitted (each to its own column). To emit **all** log record attributes
without enumerating them, set `attributes` to the string `passthrough`:

```yaml
schema:
  log_record_mapping:
    attributes: passthrough
```

In this mode every log record **attribute** is written as-is as a top-level
`"<key>": <value>` pair, using the attribute key as the column name. Passthrough
applies to log-record attributes only: resource attributes, scope attributes,
and top-level log-record fields are **not** passed through automatically. They
are emitted only when you configure them via `resource_mapping`, `scope_mapping`,
or the other `log_record_mapping` fields, which compose with passthrough and
continue to emit their own columns. If a runtime attribute key collides with a
mapped column name, the attribute value wins (the innermost/most-specific value),
and the column is emitted only once.

This override is **intentional, not accidental**: a runtime attribute can
deliberately override a configured `resource_mapping`, `scope_mapping`, or
`log_record_mapping` column. It lets a producer that has already projected the
final value into a log attribute (for example, the last projection of a KQL
transformation that writes the value into an attribute) take precedence over a
static mapping, without having to remove that mapping from the exporter
configuration. The innermost-wins rule is the general, consistent behavior:
runtime attributes are the innermost, most-specific source and therefore win,
while a non-conflicting configuration still lets explicit mappings apply.

Because passthrough column names come from runtime attribute keys, they are not
validated for duplicates at config load time the way explicit mappings are;
collisions are resolved at emit time by the innermost-wins rule above.

**Passthrough expects each attribute key to already be a valid Azure Log
Analytics column name, not an arbitrary OpenTelemetry attribute key.** The mode
is designed for producers that have already projected their data into
DCR-shaped attributes (`"<ColumnName>": <value>`), so the attribute key *is* the
destination column. The exporter does not translate, sanitize, or namespace the
key; a raw OTel key such as `service.name` or `http.request.method` (with dots)
is not a valid column name and will be dropped at ingestion (see the warning
below). If your attributes use OTel-style keys, map them to valid columns
explicitly with the `attributes` object form instead of passthrough.

> **Warning -- silent data loss:** Azure Log Analytics only ingests columns that
> exist in the DCR stream schema and whose names satisfy the column-naming rules
> (letters, digits, and underscores; must start with a letter or underscore).
> Attribute keys are emitted verbatim and are **not** sanitized, so a key such as
> `service.name` (containing a `.`) or any key without a matching, validly-named
> DCR column is **silently dropped at ingestion** -- the exporter does not rewrite
> or reject it. Map such attributes to valid column names explicitly (via the
> `attributes` object form) or ensure your DCR defines matching columns.
>
> The drop is **per column, not per record**: a record carrying an unmatched
> column is still ingested with its remaining, matched columns -- ingestion does
> not reject the record or the batch over an unknown column (only malformed JSON
> or DCR/schema errors fail a batch). The loss is **silent to the exporter and
> the sender** (no error is returned), but it is **observable to the workspace
> operator**: ingestion emits a dropped-column count metric and records the
> unmatched column names, so data loss can be detected on the Azure side even
> though this exporter cannot surface it.

Note on the mandatory `TimeGenerated` column: a log record that does not carry a
`TimeGenerated` value does **not** get rejected -- Azure Log Analytics ingestion
auto-populates `TimeGenerated` with the ingestion time when it is absent. To
control it explicitly, map a log-record field to it (e.g.
`log_record_mapping: { time_unix_nano: TimeGenerated }`).

## Azure Setup

Before using the Azure Monitor Exporter, you need to set up Azure Log Analytics:

1. **Create a Log Analytics Workspace**

2. **Create a custom table as needed with the expected columns**

3. **Create a Data Collection Rule (DCR)** with a custom log table

4. **Configure authentication** on the bound `azure_identity_auth` extension
   (managed identity, workload identity, or developer credentials) and grant
   the identity permission to publish to the DCR

5. **Get the DCR endpoint URL** from the Azure portal

Example DCR endpoint:

```text
https://my-workspace.eastus-1.ingest.monitor.azure.com/dataCollectionRules/dcr-abc123def456/streams/Custom-MyLogTable_CL
```

## Features

- [x] **Logs only** - Specifically designed for log analytics

- [x] **Schema mapping** - Flexible OTLP to Azure field mapping

- [x] **Gzip compression** - Automatic request compression

- [x] **Azure authentication** - Bearer tokens supplied by the
  `azure_identity_auth` extension via the `bearer_token_provider` capability

- [x] **Error handling** - Detailed error messages and retry logic

- [ ] **Metrics** - Not supported (logs only)

- [ ] **Traces** - Not supported (logs only)

## Troubleshooting

### Authentication Issues

- Ensure Azure credentials are properly configured

- Check that the service principal has Log Analytics Contributor role

- Verify the scope URL is correct for your Azure environment

### Data Collection Issues

- Confirm the DCR endpoint URL is correct

- Verify the stream name matches your custom log table

- Check that schema mappings align with your table structure

## License

Apache 2.0

[logs-api]: https://learn.microsoft.com/en-us/azure/azure-monitor/logs/logs-ingestion-api-overview
