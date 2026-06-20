# Geneva Exporter

## Metadata

- Type: `urn:microsoft:exporter:geneva`
- Feature gate: `geneva-exporter`
- Stability: Alpha; supports logs and traces

## Overview

The Geneva Exporter is designed for Microsoft products to send telemetry data
to Microsoft's Geneva monitoring backend. It is not meant to be used outside
of Microsoft products and is open sourced to demonstrate best practices and to
be transparent about what is being collected.

## Getting Started

Configure the Geneva endpoint, identity, and upload concurrency:

```yaml
type: urn:microsoft:exporter:geneva
config:
  endpoint: "https://geneva.example.com"
  environment: production
  account: "my-account"
  namespace: "my-namespace"
  region: westus2
  config_major_version: 1
  tenant: "my-tenant"
  role_name: "df-engine"
  role_instance: "instance-001"
  auth:
    type: systemmanagedidentity
    msi_resource: "https://monitor.azure.com/"
  max_concurrent_uploads: 4
```

## Build df_engine with Geneva Exporter

From the `otap-dataflow` directory:

```bash
cargo build --release --features geneva-exporter
```

## Verify the exporter is registered

```bash
./target/release/df_engine --help
```

You should see `urn:microsoft:exporter:geneva` in the Exporters list.

## Usage

### Running

```bash
./target/release/df_engine --config config.yaml --num-cores 4
```

### Notes on throughput knobs

- `max_concurrent_uploads` limits how many batches the exporter will upload concurrently.
- `max_buffer_size` is currently reserved for a future buffering/flush implementation.
  It is accepted by config parsing but does not change runtime behavior yet.

## Configuration

```yaml
type: urn:microsoft:exporter:geneva
config:
  # Geneva endpoint and routing identity (all required).
  endpoint: "https://geneva.example.com"
  environment: production
  account: "my-account"
  namespace: "my-namespace"
  region: westus2
  config_major_version: 1
  tenant: "my-tenant"
  role_name: "df-engine"
  role_instance: "instance-001"

  # Authentication method. Other supported values are "certificate",
  # "usermanagedidentity", "usermanagedidentitybyarmresourceid", and
  # "workloadidentity".
  auth:
    type: systemmanagedidentity
    msi_resource: "https://monitor.azure.com/"

  # Reserved for future buffering/flush behavior (default: 1000).
  max_buffer_size: 1000

  # Maximum concurrent uploads (default: 4).
  max_concurrent_uploads: 4
```

## Test Configuration

To test using the configuration file `otlp-geneva.yaml` provided
in this directory:

```bash
# Start the collector
./target/release/df_engine \
  --config crates/otap/src/experimental/geneva_exporter/\
otlp-geneva.yaml \
  --num-cores 1

# In another terminal, send test data:

# Option A: Using telemetrygen (easiest)
telemetrygen logs --otlp-endpoint localhost:4317 --otlp-insecure --logs 10
telemetrygen traces --otlp-endpoint localhost:4317 --otlp-insecure --traces 10

# Option B: Using grpcurl (for manual testing)
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
          "body": {"stringValue": "test"}
        }]
      }]
    }]
  }' \
  localhost:4317 \
  opentelemetry.proto.collector.logs.v1.LogsService/Export

# Option C: Configure your instrumented app to send to localhost:4317
```

## License

Apache 2.0
