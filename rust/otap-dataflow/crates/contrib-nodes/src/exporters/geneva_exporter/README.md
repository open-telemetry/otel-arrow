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

### Agent-fed credentials

When the embedding host supplies the Geneva token and routing metadata, bind
the combined credential-provider capability and use `agentfed` authentication:

```yaml
type: urn:microsoft:exporter:geneva
capabilities:
  agent_fed_credential_provider: agent-auth
config:
  environment: production
  account: "my-account"
  namespace: "my-namespace"
  config_major_version: 1
  tenant: "my-tenant"
  role_name: "df-engine"
  role_instance: "instance-001"
  auth:
    type: agentfed
  max_concurrent_uploads: 4
```

The `attributes` object in each credential snapshot must use this shape:

```json
{
  "endpoint": "https://ingest.example.com",
  "moniker_map": {
    "my-account": "my-moniker",
    "default": "fallback-moniker"
  }
}
```

`endpoint` and `region` are not required in this mode because the host supplies
the ingestion endpoint through the credential snapshot's `endpoint` attribute.
That attribute must be a non-empty absolute HTTPS URL with a host and cannot
contain embedded credentials, a query string, or a fragment. The exporter
selects a non-empty string from `moniker_map` by the configured `account`,
falling back only to an explicit `default`. A map containing neither key is
rejected, even if it has a single entry. Empty or malformed routing is also
rejected. If the configured account or `default` key exists with an invalid
value, the snapshot is rejected instead of falling back to another entry. The
selected moniker must be safe to use as one URL query value without additional
encoding. Surrounding whitespace is trimmed; the remaining value may contain
only ASCII letters, digits, hyphen, dot, underscore, and tilde. Embedded
whitespace, non-ASCII text, and reserved delimiters are rejected.

The provider must load the token and routing attributes from one atomically
published host snapshot. Each upload consumes one immutable snapshot, so a host
rotation cannot produce a mixed-generation token/routing pair. Subsequent
uploads observe the new snapshot without reconstructing the exporter. Tokens
with a known expiry must remain usable for more than 30 seconds; expired or
near-expiry snapshots fail closed while the provider refreshes them.

Capability bindings are checked when the exporter is created because the
factory's earlier config-validation hook receives only the `config` object.
Capability factories create a clone for each consumer, so the host extension's
clones must share the same atomically swapped snapshot state. The host extension
must register `agent_fed_credential_provider` for the shared execution model;
a local-only registration cannot satisfy the uploader's thread-safe credential
source.

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
  # Geneva config-service endpoint and region are required for every
  # authentication method except "agentfed".
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
  # "usermanagedidentity", "usermanagedidentitybyarmresourceid",
  # "workloadidentity", and "agentfed".
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
