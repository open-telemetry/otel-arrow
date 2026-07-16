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
    # Schema mapping is optional; only the sections you configure are emitted.
    # Unmapped resource/scope attributes and log-record fields are dropped, not
    # passed through.
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
        # `attributes` is either an explicit per-attribute mapping (below) or
        # `passthrough` to emit every log attribute as its own column. See
        # [Attribute Passthrough Mode](#attribute-passthrough-mode).
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

By default only the attributes listed under `log_record_mapping.attributes` are
emitted. To emit **all** log record attributes without enumerating them, set
`attributes` to `passthrough`:

```yaml
schema:
  log_record_mapping:
    attributes: passthrough
```

Each log record attribute is then written as a top-level `"<key>": <value>`
column, using the attribute key as the column name. Passthrough covers
log-record attributes only; `resource_mapping`, `scope_mapping`, and the other
`log_record_mapping` fields still emit their own columns and compose with it.

If a passthrough attribute key collides with a mapped column, the attribute
value wins and the column is emitted once. This is intentional: a producer that
has already projected the final value into an attribute overrides the static
mapping without editing the exporter config. Passthrough keys come from runtime
data, so unlike explicit mappings they are not checked for duplicates at config
load; collisions are resolved at emit time by this rule.

Passthrough keys are emitted verbatim, so each must already be a valid DCR
column name. The exporter does not sanitize or namespace keys, so a raw OTel key
such as `service.name` (with a `.`) is not a valid column. Map such attributes
to valid columns explicitly instead of using passthrough.

> **Warning -- data loss:** an attribute whose key does not match a DCR column
> is dropped by that column at ingestion; the rest of the record is still
> ingested (only malformed JSON or DCR/schema errors fail a batch). The drop is
> silent to this exporter and the sender (no error returned) but visible to the
> workspace operator via ingestion's dropped-column metrics. This reflects
> current Log Analytics ingestion behavior, which Azure may change.

The exporter auto-injects the mandatory `TimeGenerated` column from the record's
event time (`time_unix_nano`, falling back to `observed_time_unix_nano`, then
the current time) unless it is already mapped or passed through, so every record
carries a timestamp.

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
