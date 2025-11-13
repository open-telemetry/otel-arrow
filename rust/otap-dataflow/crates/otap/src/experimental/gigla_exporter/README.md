# GigLA Exporter

**Status:** ALPHA (Functional - supports logs only)

The GigLA (Geneva Infrastructure General-purpose Logging Analytics) Exporter sends OpenTelemetry logs to Azure Log Analytics using the Data Collection Rules (DCR) API. It transforms OTLP log data into the format expected by Azure Log Analytics and provides configurable schema mapping for custom log tables.

## Build df_engine with GigLA Exporter

From the `otap-dataflow` directory:

```bash
cargo build --release --features gigla-exporter
```

## Verify the exporter is registered

```bash
./target/release/df_engine --help
```

You should see `urn:otel:gigla:exporter` in the Exporters list.

## Configuration

The GigLA exporter requires Azure authentication and Data Collection Rule configuration:

### Basic Configuration

```yaml
nodes:
  gigla-exporter:
    kind: exporter
    plugin_urn: "urn:otel:gigla:exporter"
    config:
      # API configuration (REQUIRED)
      api:
        dcr_endpoint: "https://my-workspace.eastus-1.ingest.monitor.azure.com"
        stream_name: "Custom-MyLogTable_CL"
        dcr: "dcr-abc123def456"
        schema:
          # Map OTLP resource attributes to Azure fields
          resource_mapping:
            "service.name": "ServiceName"
            "service.version": "ServiceVersion"
            "host.name": "HostName"
            "deployment.environment": "Environment"
          # Map OTLP scope attributes to Azure fields
          scope_mapping:
            "otel.library.name": "InstrumentationLibrary"
            "otel.library.version": "InstrumentationVersion"
          # Map OTLP log record fields to Azure fields
          log_record_mapping:
            "body": "Message"
            "severity_text": "SeverityText"
            "time_unix_nano": "TimeGenerated"
            "trace_id": "TraceId"
            "span_id": "SpanId"
            "attributes":
              "message": "ParsedMessage"
          disable_schema_mapping: false
      
      # Authentication configuration (uses Azure SDK defaults)
      auth:
        scope: "https://monitor.azure.com/.default"
      
      # Optional: Disable export for testing
      disable_gig_export: false
```

### Authentication

The exporter uses Azure SDK authentication with the following precedence:
1. Environment variables (`AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`)
2. Managed Identity (when running in Azure)
3. Azure CLI credentials
4. Visual Studio Code credentials

## Usage

### Running

```bash
./target/release/df_engine --pipeline config.yaml --num-cores 4
```

### Testing with OTLP Receiver

To test using the provided configuration file:

```bash
# Start the collector
./target/release/df_engine \
  --pipeline crates/otap/src/experimental/gigla_exporter/otlp-gigla.yaml \
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
          "body": {"stringValue": "Hello from GigLA exporter!"},
          "severityText": "INFO"
        }]
      }]
    }]
  }' \
  localhost:4317 \
  opentelemetry.proto.collector.logs.v1.LogsService/Export

# Option C: Configure your instrumented app to send to localhost:4317
```

## Schema Mapping

The GigLA exporter provides flexible schema mapping to transform OTLP data structures into Azure Log Analytics table format:

### Resource Mapping
Maps OpenTelemetry resource attributes to Azure fields:
```yaml
resource_mapping:
  "service.name": "ServiceName"      # Maps service.name → ServiceName
  "host.name": "MachineName"         # Maps host.name → MachineName
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
  "body": "Message"                  # Log body → Message column
  "time_unix_nano": "TimeGenerated"  # Timestamp → TimeGenerated
  "severity_text": "Level"          # Severity → Level column
  "trace_id": "TraceId"              # Trace ID → TraceId column
  "attributes":                      # Nested attribute mapping
    "user.id": "UserId"              # Specific attribute mapping
```

## Azure Setup

Before using the GigLA exporter, you need to set up Azure Log Analytics:

1. **Create a Log Analytics Workspace**
2. **Create a Data Collection Rule (DCR)** with a custom log table
3. **Configure authentication** (service principal or managed identity)
4. **Get the DCR endpoint URL** from the Azure portal

Example DCR endpoint:
```
https://my-workspace.eastus-1.ingest.monitor.azure.com/dataCollectionRules/dcr-abc123def456/streams/Custom-MyLogTable_CL
```

## Features

- ✅ **Logs only** - Specifically designed for log analytics
- ✅ **Schema mapping** - Flexible OTLP to Azure field mapping
- ✅ **Gzip compression** - Automatic request compression
- ✅ **Azure authentication** - Uses Azure SDK credential chain
- ✅ **Error handling** - Detailed error messages and retry logic
- ❌ **Metrics** - Not supported (logs only)
- ❌ **Traces** - Not supported (logs only)

## Troubleshooting

### Authentication Issues
- Ensure Azure credentials are properly configured
- Check that the service principal has Log Analytics Contributor role
- Verify the scope URL is correct for your Azure environment

### Data Collection Issues
- Confirm the DCR endpoint URL is correct
- Verify the stream name matches your custom log table
- Check that schema mappings align with your table structure

### Performance
- Use multiple cores (`--num-cores`) for higher throughput
- Consider adjusting compression settings for large payloads
- Monitor Azure Log Analytics ingestion limits

## License

Apache 2.0