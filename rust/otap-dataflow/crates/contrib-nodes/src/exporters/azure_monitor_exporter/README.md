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

Telemetry reference: [telemetry.md](telemetry.md)

## Getting Started

Configure the Azure Logs Ingestion API target and authentication method:

```yaml
type: urn:microsoft:exporter:azure_monitor
config:
  api:
    dcr_endpoint: "https://my-workspace.eastus-1.ingest.monitor.azure.com"
    stream_name: "Custom-MyLogTable_CL"
    dcr: "dcr-abc123def456"
  auth:
    method: msi
```

## Build df_engine with Azure Monitor Exporter

From the `otap-dataflow` directory:

```bash
cargo build --release --features azure-monitor-exporter
```

## Verify the exporter is registered

```bash
./target/release/df_engine --help
```

You should see `urn:microsoft:exporter:azure_monitor` in the Exporters list.

## Configuration

The Azure Monitor Exporter requires Azure authentication and Data Collection Rule
configuration:

```yaml
type: urn:microsoft:exporter:azure_monitor
config:
  # Azure Monitor API configuration (required).
  api:
    dcr_endpoint: "https://my-workspace.eastus-1.ingest.monitor.azure.com"
    stream_name: "Custom-MyLogTable_CL"
    dcr: "dcr-abc123def456"
    # Schema mapping is optional. To emit all log record attributes as-is,
    # set `log_record_mapping.attributes` to the string `passthrough`; every log
    # attribute is then emitted as its own top-level "key": value column.
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
        "attributes":
          "message": "ParsedMessage"

  # Authentication configuration. Use "msi" for managed identity or "dev" for
  # local Azure developer credentials.
  auth:
    method: msi

  # Optional heartbeat rows.
  heartbeat:
    enabled: false
    frequency: 60s
```

### Authentication

The exporter uses Azure SDK authentication. The following `auth.method` values
are supported:

- `managedidentity` (aliases: `msi`, `managed_identity`) - managed identity.
  Set `client_id` to use a user-assigned identity; omit it to use the
  system-assigned identity.
- `development` (aliases: `dev`, `developer`, `cli`) - local Azure developer
  credentials (Azure CLI / Azure Developer CLI).
- `workloadidentity` (aliases: `wif`, `workload_identity`) - Workload Identity
  Federation. Reads a projected federated ServiceAccount token and exchanges it
  with Entra ID for an access token. Useful for Kubernetes workloads without a
  managed identity (e.g. self-hosted or non-AKS clusters).

For `workload_identity`, the following fields are used (each falls back to the
corresponding environment variable injected by the Azure Workload Identity
webhook when omitted):

- `client_id` - application (client) ID; defaults to `AZURE_CLIENT_ID`.
- `tenant_id` - Entra tenant ID; defaults to `AZURE_TENANT_ID`.
- `token_file_path` - path to the federated token file; defaults to
  `AZURE_FEDERATED_TOKEN_FILE`.

```yaml
auth:
  method: workload_identity
  # All fields optional; fall back to the standard AZURE_* environment variables.
  client_id: "00000000-0000-0000-0000-000000000000"
  tenant_id: "11111111-1111-1111-1111-111111111111"
  token_file_path: "/var/run/secrets/azure/tokens/azure-identity-token"
```

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

In this mode every log record attribute is written as-is as a top-level
`"<key>": <value>` pair, using the attribute key as the column name. Attributes
already mapped explicitly are skipped so they are not duplicated. Passthrough
composes with resource, scope, and top-level field mappings, which continue to
emit their own columns.

Note: Azure Log Analytics only ingests columns that exist in the DCR stream
schema and whose names satisfy the column-naming rules (letters, digits, and
underscores; must start with a letter or underscore). Attribute keys are emitted
verbatim, so a key such as `service.name` must have a matching column defined in
your DCR; attributes without a matching, valid column are dropped at ingestion.

## Azure Setup

Before using the Azure Monitor Exporter, you need to set up Azure Log Analytics:

1. **Create a Log Analytics Workspace**

2. **Create a custom table as needed with the expected columns**

3. **Create a Data Collection Rule (DCR)** with a custom log table

4. **Configure authentication** (service principal or managed identity)

5. **Get the DCR endpoint URL** from the Azure portal

Example DCR endpoint:

```text
https://my-workspace.eastus-1.ingest.monitor.azure.com/dataCollectionRules/dcr-abc123def456/streams/Custom-MyLogTable_CL
```

## Features

- [x] **Logs only** - Specifically designed for log analytics

- [x] **Schema mapping** - Flexible OTLP to Azure field mapping

- [x] **Gzip compression** - Automatic request compression

- [x] **Azure authentication** - Uses Azure SDK credential chain

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
