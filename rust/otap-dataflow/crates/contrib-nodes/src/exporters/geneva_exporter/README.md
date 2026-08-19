# Geneva Exporter

## Metadata

- Type: `urn:microsoft:exporter:geneva`
- Feature gate: `geneva-exporter`
- Optional certificate authentication: `geneva-certificate-auth` (disabled by default)
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
  account_routing:
    default_group: "my-account-group"
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
  account_routing:
    default_group: "my-account-group"
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
    "my-account-group": "my-primary-moniker",
    "another-account-group": "another-primary-moniker"
  }
}
```

`endpoint` and `region` are not required in this mode because the host supplies
the ingestion endpoint through the credential snapshot's `endpoint` attribute.
That attribute must be a non-empty absolute HTTPS URL with a host and cannot
contain embedded credentials, a query string, or a fragment. The exporter
canonicalizes it before use. The uploader uses that canonical value as both the
upload base URL and the `endpoint=` query fallback when a token has no usable
Endpoint claim. `moniker_map` maps each logical account group to its current
primary physical moniker. The exporter validates and preserves the complete
map; the uploader selects the entry named by `account_routing` for each batch.
An empty map, blank group, or invalid moniker rejects the complete snapshot
instead of allowing partial routing. Each moniker must be safe to use as one
URL query value without additional encoding. Surrounding whitespace is
trimmed; the remaining value may contain only ASCII letters, digits, hyphen,
dot, underscore, and tilde. Embedded whitespace, non-ASCII text, and reserved
delimiters are rejected.

The provider must load the token and routing attributes from one atomically
published host snapshot. Each upload consumes one immutable snapshot, so a host
rotation cannot produce a mixed-generation token/routing pair. Subsequent
uploads observe the new snapshot without reconstructing the exporter. Tokens
with a known expiry must remain usable for more than 30 seconds; expired or
near-expiry snapshots fail closed while the provider refreshes them.
Credential lookup, including time waiting for another lookup to release the
provider, is limited to five seconds. Provider futures must be cancellation-safe
and should normally clone an already-published snapshot instead of performing
network I/O.

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

Password-protected PKCS#12 certificate authentication is excluded by default.
Build with `--features geneva-certificate-auth` only when certificate
authentication is required. This opt-in feature adds PKCS#12 parsing and its
cryptographic dependencies.

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
  account_routing:
    default_group: "diagnostics"
  region: westus2
  config_major_version: 1
  tenant: "my-tenant"
  role_name: "df-engine"
  role_instance: "instance-001"

  # Authentication method. Other default-build values are
  # "usermanagedidentity", "usermanagedidentitybyarmresourceid",
  # "workloadidentity", and "agentfed". "certificate" requires the
  # opt-in "geneva-certificate-auth" build feature.
  auth:
    type: systemmanagedidentity
    msi_resource: "https://monitor.azure.com/"

  # Reserved for future buffering/flush behavior (default: 1000).
  max_buffer_size: 1000

  # Maximum concurrent uploads (default: 4).
  max_concurrent_uploads: 4
```

## On-Behalf-Of (OBO) with table routing

OBO lets a single agent upload telemetry on behalf of multiple customer
identities. When a batch's event/table name has an OBO entry, the exporter
attaches the customer identity (`onbehalfid`) and an optional annotations recipe
(`onbehalfannotations`) as GIG query parameters on the upload.

OBO entries are keyed by the **destination** event/table name -- the name
*after* `event_name_mapping` resolves it, not the pre-mapping source value.

The following example both renames tables via `event_name_mapping` and enables
OBO on the resolved destinations:

```yaml
type: urn:microsoft:exporter:geneva
config:
  endpoint: "https://geneva.example.com"
  environment: production
  account: "my-account"
  namespace: "my-namespace"
  account_routing:
    default_group: "diagnostics"
    events:
      AuditLogs: "audit"
      raw: "raw"
  region: westus2
  config_major_version: 1
  tenant: "my-tenant"
  role_name: "df-engine"
  role_instance: "instance-001"
  auth:
    type: systemmanagedidentity
    msi_resource: "https://monitor.azure.com/"

  # Routing: source event name -> destination table.
  logs:
    default_event_name: "Log"        # fallback table for unmapped records
    event_name_mapping:
      routing_key: event_name        # route by the record's event name
      events:
        audit: AuditLogs             # source "audit" -> table "AuditLogs"
        raw:                         # null: source "raw" -> table "raw" (unchanged)

  # OBO: keyed by the DESTINATION table name (post-mapping).
  obo:
    events:
      AuditLogs:                     # the destination name, NOT "audit"
        identity: "Microsoft.AuditService"
        annotations: '<Config onBehalfFields="resourceId" />'
      raw:                           # destination == source here (passthrough)
        identity: "Microsoft.RawService"
```

How a record flows through the exporter and uploader:

<!-- markdownlint-disable MD013 -->

| Incoming event | Destination table | Account group | OBO query parameters |
| --- | --- | --- | --- |
| `audit` | `AuditLogs` | `audit` | `onbehalfid=Microsoft.AuditService`, `onbehalfannotations=<Config .../>` |
| `raw` | `raw` | `raw` | `onbehalfid=Microsoft.RawService` |
| `foo` | `Log` | `diagnostics` | none |

<!-- markdownlint-enable MD013 -->

The uploader resolves the destination table first, then looks up OBO by that
resolved name. A single flat `obo.events` map is shared across `logs` and
`spans`, keyed by event/table name.

`account_routing` uses the same destination event/table names. Its required
`default_group` handles events without an exact override, while `events` maps
selected destinations to logical GCS account groups. The uploader resolves the
chosen logical group to the primary physical moniker from the current GCS or
agent-fed credential snapshot; YAML config contains group names, not physical
monikers.

Gotcha: because OBO keys on the destination, keying an entry on the source value
silently disables OBO. If you wrote `obo.events.audit` instead of
`obo.events.AuditLogs`, the post-routing lookup (`AuditLogs`) would miss and the
`audit` records would upload without OBO -- no error, just silently omitted.

## Telemetry

Input PData message volume is reported by the engine through
`channel.receiver.messages` and is not duplicated by the exporter.

<!-- markdownlint-disable MD013 -->

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.exports.messages` | `{message}` | `signal`, `outcome` | Number of PData messages whose Geneva export reached a terminal outcome. |
| `exporter.exports.duration` | `s` | `signal`, `outcome` | Time from dequeuing PData through the terminal Geneva upload result, including conversion and upload preparation but excluding Ack/Nack notification. |

<!-- markdownlint-enable MD013 -->

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
