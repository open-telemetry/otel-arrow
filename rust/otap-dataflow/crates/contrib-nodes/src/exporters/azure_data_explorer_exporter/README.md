# Azure Data Explorer (ADX) Exporter

## Metadata

- Type: `urn:microsoft:exporter:azure_data_explorer`
- Feature gate: `azure-data-explorer`
- Stability: Alpha; supports logs, metrics, and traces

## Overview

The Azure Data Explorer exporter sends OpenTelemetry logs, metrics, and
traces to [Azure Data Explorer][adx] (Kusto) via the streaming ingestion REST
API. Its table schemas are compatible with the Go `azuredataexplorerexporter`
in [opentelemetry-collector-contrib][go-contrib] where noted below. The log
schema has configurable extensions, and the transformation limitations are
documented explicitly.

Telemetry reference: [telemetry.md](telemetry.md)

[adx]: https://learn.microsoft.com/en-us/azure/data-explorer/
[go-contrib]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/azuredataexplorerexporter

## Build df_engine with ADX Exporter

From the `otap-dataflow` directory:

```bash
cargo build --release --features azure-data-explorer
```

The `contrib-exporters` and `contrib-nodes` umbrella features also enable the
shared `azure_identity_auth` extension required by authenticated Azure
exporters.

## Verify the exporter is registered

```bash
./target/release/df_engine --help
```

The Exporters list includes `urn:microsoft:exporter:azure_data_explorer`.

## Configuration

### Basic Configuration

```yaml
extensions:
  azure-data-explorer-auth:
    type: "urn:microsoft:extension:azure_identity_auth"
    config:
      method: managed_identity
      # Set client_id for a user-assigned managed identity. Omit it for the
      # system-assigned identity.
      # client_id: "00000000-0000-0000-0000-000000000000"
      # Direct REST authentication uses the cluster URI plus /.default.
      scope: "https://<cluster>.<region>.kusto.windows.net/.default"

nodes:
  azure-data-explorer-exporter:
    type: "urn:microsoft:exporter:azure_data_explorer"
    capabilities:
      bearer_token_provider: azure-data-explorer-auth
    config:
      # Required Kusto cluster URI. Do not use the Data Ingestion URI.
      cluster_uri: "https://<cluster>.<region>.kusto.windows.net"

      # Target database (default: "oteldb")
      db_name: "oteldb"

      # Target table names per signal
      logs_table_name: "OTELLogs"           # default
      metrics_table_name: "OTELMetrics"     # default
      traces_table_name: "OTELTraces"       # default

      # Optional pre-created ADX JSON ingestion mappings.
      logs_table_json_mapping: "${env:ADX_LOGS_JSON_MAPPING:-}"
      metrics_table_json_mapping: "${env:ADX_METRICS_JSON_MAPPING:-}"
      traces_table_json_mapping: "${env:ADX_TRACES_JSON_MAPPING:-}"

      # Debugging export or connection issues only. Logs the complete failed
      # JSON batch and a sample row at debug level. Do not enable in production
      # unless strictly necessary because telemetry may contain sensitive data.
      # log_failed_payload: true

      # Debugging export or connection issues only. Logs ADX error response
      # bodies at debug level. Do not enable in production unless strictly
      # necessary because responses may contain PII or customer-sensitive data.
      # log_response_body: true

      # Gzip compression level (0-9, default: 6)
      gzip_compression_level: 6

      # Optional log-schema compatibility controls.
      legacy_logs_body_string: false
      export_event_name: true
      add_event_name_to_log_attributes: true

      # Maximum serialized JSON row size, including its newline separator.
      # May be lowered but cannot exceed the 900 KiB safety ceiling.
      max_row_bytes: 921600

      # Total deadline for one ADX operation, including response handling,
      # transient retries, and retry backoff (default: 30s)
      timeout: 30s

      # Max concurrent in-flight HTTP requests (default: 16)
      max_in_flight: 16

      # Optional: combine multiple inbound PData messages into fewer ADX
      # requests. Disabled by default.
      network_requests:
        coalesce: false
        max_pending_messages: 16
        max_rows: 1000
        max_bytes: 4194304
        flush_interval: 1s

```

Authentication is supplied by a bound `bearer_token_provider` capability.
The `azure_identity_auth` extension supports system-assigned and user-assigned
managed identity, local development credentials, and workload identity. See
the [Azure Identity Auth extension reference](../../../../contrib-extensions/src/azure_identity_auth/README.md)
for all authentication fields and lifecycle behavior.

When configured, the exporter sends the signal's mapping name as the ADX
`mappingName` request parameter. Empty strings are treated as omitted. Each
referenced mapping must already exist on its target table. Standard ADX JSON
streaming ingestion requires a mapping, so omission is intended for compatible
endpoints or service configurations that accept an unmapped request.

`cluster_uri` must be an HTTPS origin without user information, a path, query,
or fragment. HTTP is accepted only for loopback development endpoints.

### Network request coalescing

`network_requests.coalesce` is an explicit opt-in that combines multiple
inbound PData messages into fewer, larger ADX HTTP requests. It reduces
outbound request rate, but each source acknowledgement remains pending until
ADX accepts the coalesced request.

Configure the group as follows:

| Field | Default | Behavior |
| --- | ---: | --- |
| `coalesce` | `false` | Enables cross-message request coalescing |
| `max_pending_messages` | `16` | When coalescing is enabled, bounds retained source PData globally and in each coalesced request |
| `max_rows` | `1000` | Maximum transformed rows per source message and request |
| `max_bytes` | `4194304` | Maximum uncompressed JSON Lines bytes per source message and request; cannot exceed ADX's 4 MiB streaming limit |
| `flush_interval` | `1s` | When coalescing is enabled, flushes a partial request after this delay; ignored otherwise |

An individual inbound PData message is never split to satisfy these limits. If
its transformed rows exceed either boundary, the exporter permanently refuses
the complete source message. When appending a valid message would cross a
request boundary, the existing partial request is flushed first.

`max_rows` must be between 1 and 100000, `max_bytes` between 1 and 4194304,
and `max_in_flight` between 1 and 1024.

The transformer applies row and request byte budgets while writing JSON, before
an oversized row is materialized. Composite OTLP values are limited to 64
levels of nesting and 65536 visited values per source message.

Keep `max_pending_messages` small for HTTP receivers, especially when clients
use HTTP/1.1 or many clients send concurrently. Coalescing delays their
responses until ADX completes the corresponding request. The bound applies per
exporter instance, so multiply it by the configured engine core count when
estimating process-wide pending requests. It is both a global admission bound
and a per-request message cap; in-flight requests count against the global
bound. Increase it deliberately if more than one full coalesced request must
run concurrently.

`flush_interval` applies only when `coalesce: true`. With coalescing disabled,
each non-empty input message is dispatched immediately. With coalescing
enabled, the exporter does not ACK or NACK a source message while it is waiting
in a partial request. Resolution occurs only after the request is flushed,
sent, and ADX returns a terminal response. Increasing `flush_interval`
therefore adds directly to acknowledgement latency during low-volume periods.
Setting it too high can retain receiver responses, consume the pending-message
bound, and apply backpressure that holds up the upstream pipeline.

When a 401 rejects a coalesced request, the exporter retains that request and
its acknowledgement contexts, waits for the bearer-token provider to publish
a replacement token, and retries with the replacement. Authentication retries
are bounded by `max_retries`; the value counts retries after the initial
submission. Retry exhaustion and transient failures return retryable NACKs.
Terminal client refusals return permanent NACKs. During shutdown, unresolved
messages are routed with the `node_shutdown` cause while the engine deadline
permits. At the deadline, remaining work is recorded as failed and abandoned
so shutdown cannot block indefinitely. Setting `max_retries: 0` disables both
transient HTTP retries and retained 401 retries.

`log_failed_payload` and `log_response_body` are disabled by default and are
intended only for temporarily debugging export or connection issues. They log
sensitive content at debug level and are not recommended in production unless
strictly necessary. `log_failed_payload` controls the outbound request payload
and sample row. `log_response_body` controls inbound ADX error response
chunks. Either source may contain PII, schema values, telemetry attributes, or
other customer-sensitive data. Disable the setting immediately after
diagnosis and ensure debug logs are handled according to applicable data
protection requirements.

For direct REST requests, the OAuth resource is the ADX cluster URI. Configure
the Azure Identity scope as `<cluster_uri>/.default`. National clouds can use a
different audience; query the cluster authentication metadata when the
resource is not known in advance.

`timeout` is a total deadline for one physical ADX export operation. It covers
request transmission, response-body handling, transient retries, and retry
backoff. A 401 ends the physical operation immediately so the exporter can
invalidate the rejected token generation and, when coalescing is enabled,
retain the request until the authentication provider publishes a replacement.

Database, table, and mapping names are percent-encoded when constructing the
streaming-ingestion endpoint. Each physical submission, including a transient
retry, records shared exporter-attempt telemetry with its signal, source item
count, compressed payload size, and success, refusal, or failure outcome.
Initial attempt duration includes gzip compression and HTTP work; retries reuse
the compressed payload and exclude backoff.

Each serialized JSON row is checked before batching or HTTP submission.
`max_row_bytes` defaults to 921600 bytes (900 KiB), leaving safety margin below
ADX's 1 MiB dynamic-value limit. An oversized input is permanently refused:
operators receive a `row_too_large` warning and counter increment, while OTLP
senders receive `INVALID_ARGUMENT` over gRPC or HTTP 400 with the actual and
allowed byte counts. The original input is not split because that could change
its acknowledgement semantics.

Raw OTLP protobuf framing is validated before transformation. Malformed logs,
metrics, or traces are permanently refused rather than treated as empty input.

Log schema compatibility can be controlled independently. Setting
`legacy_logs_body_string` converts non-string log bodies to their JSON string
representation. `export_event_name` controls the top-level `EventName` field,
while `add_event_name_to_log_attributes` adds `event.name` only when neither
the log record nor its instrumentation scope already provides that attribute.

Do not enable exporter coalescing merely because durable buffering is present.
If an upstream system already emits suitably sized requests, leave
`coalesce: false`. The in-process `processor:batch` reduces downstream message
count but keeps original client acknowledgements pending; configure its
`inbound_request_limit` to bound those clients. The `processor:durable_buffer`
can acknowledge after local persistence, but its path must be placed on
storage that survives the container, such as an Azure Files mount in Azure
Container Apps. Ephemeral container storage is not durable across replacement.

### ADX provisioning

The signal sections below contain complete KQL blocks for creating each table,
creating its JSON ingestion mapping, and enabling streaming ingestion. Run the
management commands one at a time for the signals used by the pipeline. The
mapping names match the names used by the examples in this directory.

The table commands use `.create-merge`, so rerunning them adds missing columns
without deleting existing data. The mapping commands use `.create-or-alter`.
Review schema changes before rerunning these commands against an existing
production table.

### Cluster URI vs Data Ingestion URI

Azure Data Explorer provides two endpoints per cluster:

| Endpoint | Example | Used for |
| --- | --- | --- |
| Cluster URI | `https://<cluster>.<region>.kusto.windows.net` | Queries and streaming ingestion |
| Data Ingestion URI | `https://ingest-<cluster>.<region>.kusto.windows.net` | Queued ingestion |

You can find both URIs in the Azure portal under your ADX cluster's
Overview page.

Set `cluster_uri` to the Cluster URI without the `ingest-` prefix. The
streaming ingestion REST API (`/v1/rest/ingest/...`) is served by the cluster,
not by the data management service.

If only the Data Ingestion URI is available, remove the `ingest-` prefix:

```text
https://ingest-<cluster>.<region>.kusto.windows.net  Data Ingestion URI
https://<cluster>.<region>.kusto.windows.net         Cluster URI
```

### Authentication

| Method | Alias(es) | Description |
| --- | --- | --- |
| `managedidentity` | `msi`, `managed_identity` | Azure Managed Identity. Use `client_id` for user-assigned. |
| `development` | `dev`, `developer`, `cli` | Azure CLI / Developer Tools. Best for local testing. |

### ADX Table Schema

The exporter writes log records extending the
[Go ADX exporter schema][go-schema] with `EventName`:

| Column | Type | Description |
| --- | --- | --- |
| `Timestamp` | `datetime` | Log record timestamp (RFC 3339 nanoseconds) |
| `ObservedTimestamp` | `datetime` | Observed timestamp |
| `TraceID` | `string` | Hex-encoded trace ID |
| `SpanID` | `string` | Hex-encoded span ID |
| `SeverityText` | `string` | e.g. `INFO`, `WARN` |
| `SeverityNumber` | `int` | OTel severity number (1-24) |
| `Body` | `dynamic` | Log body encoded as a JSON scalar, array, object, or null |
| `EventName` | `string` | OTel log event name |
| `ResourceAttributes` | `dynamic` | JSON object of OTel resource attributes |
| `LogsAttributes` | `dynamic` | JSON object of log + scope attributes |

With `export_event_name: false`, the `EventName` JSON property is omitted. ADX
named mappings that include `$.EventName` ingest a null value for that column.

#### Go exporter column compatibility

To write the Go exporter's original logs column layout, use:

```yaml
legacy_logs_body_string: true
export_event_name: false
add_event_name_to_log_attributes: false
```

These settings preserve the original string `Body` column and omit the Rust
exporter's optional event-name additions. They provide column compatibility,
not byte-for-byte transformation parity. For example, this exporter encodes
byte attributes as lowercase hexadecimal strings.

[go-schema]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/azuredataexplorerexporter/logsdata_to_adx.go

#### Default logs table, mapping, and streaming policy

```kql
.create-merge table OTELLogs (
    Timestamp: datetime,
    ObservedTimestamp: datetime,
    TraceID: string,
    SpanID: string,
    SeverityText: string,
    SeverityNumber: int,
    Body: dynamic,
    EventName: string,
    ResourceAttributes: dynamic,
    LogsAttributes: dynamic
)

.create-or-alter table OTELLogs ingestion json mapping "OTELLogsMapping"
'[{"column":"Timestamp","path":"$.Timestamp"},'
'{"column":"ObservedTimestamp","path":"$.ObservedTimestamp"},'
'{"column":"TraceID","path":"$.TraceID"},'
'{"column":"SpanID","path":"$.SpanID"},'
'{"column":"SeverityText","path":"$.SeverityText"},'
'{"column":"SeverityNumber","path":"$.SeverityNumber"},'
'{"column":"Body","path":"$.Body"},'
'{"column":"EventName","path":"$.EventName"},'
'{"column":"ResourceAttributes","path":"$.ResourceAttributes"},'
'{"column":"LogsAttributes","path":"$.LogsAttributes"}]'

.alter table OTELLogs policy streamingingestion enable

.show table OTELLogs ingestion json mappings
```

Configure the exporter to use the mapping:

```yaml
logs_table_name: "OTELLogs"
logs_table_json_mapping: "OTELLogsMapping"
legacy_logs_body_string: false
export_event_name: true
add_event_name_to_log_attributes: true
```

#### Legacy logs table, mapping, and streaming policy

Use this layout for a string-based log schema. It omits the top-level
`EventName` column. `Body`, `ResourceAttributes`, and `LogsAttributes` are
stored as strings. The exporter emits the two attribute values as JSON
objects; ADX serializes those mapped objects into the string columns.

```kql
.create-merge table OTELLogsLegacy (
    Timestamp: datetime,
    ObservedTimestamp: datetime,
    TraceID: string,
    SpanID: string,
    SeverityText: string,
    SeverityNumber: int,
    Body: string,
    ResourceAttributes: string,
    LogsAttributes: string
)

.create-or-alter table OTELLogsLegacy ingestion json mapping "OTELLogsLegacyMapping"
'[{"column":"Timestamp","path":"$.Timestamp"},'
'{"column":"ObservedTimestamp","path":"$.ObservedTimestamp"},'
'{"column":"TraceID","path":"$.TraceID"},'
'{"column":"SpanID","path":"$.SpanID"},'
'{"column":"SeverityText","path":"$.SeverityText"},'
'{"column":"SeverityNumber","path":"$.SeverityNumber"},'
'{"column":"Body","path":"$.Body"},'
'{"column":"ResourceAttributes","path":"$.ResourceAttributes"},'
'{"column":"LogsAttributes","path":"$.LogsAttributes"}]'

.alter table OTELLogsLegacy policy streamingingestion enable

.show table OTELLogsLegacy ingestion json mappings
```

Use all three compatibility settings with the legacy table:

```yaml
logs_table_name: "OTELLogsLegacy"
logs_table_json_mapping: "OTELLogsLegacyMapping"
legacy_logs_body_string: true
export_event_name: false
add_event_name_to_log_attributes: false
```

`legacy_logs_body_string` controls `Body` encoding. The string types for
`ResourceAttributes` and `LogsAttributes` are defined by the ADX table schema.
If an existing legacy table uses `dynamic` attribute columns, keep those
columns as `dynamic`; the mapping paths do not change.

#### Migrate `Body` from `string` to `dynamic`

To migrate an existing table from `string` to `dynamic`:

```kql
// 1. Add a new dynamic column
.alter-merge table OTELLogs (BodyDynamic: dynamic)

// 2. Backfill: parse existing string values into dynamic
//    (plain strings become dynamic strings, JSON objects get parsed)
.set-or-append OTELLogs with (extend_schema=false) <|
    OTELLogs
    | where isnotempty(Body)
    | extend BodyDynamic = parse_json(Body)

// 3. Drop the old string column
.alter table OTELLogs drop column Body

// 4. Rename the new column
.rename column OTELLogs.BodyDynamic to Body
```

> **Note:** If the table uses an ingestion JSON mapping, update it to
> target the new column name during the migration window, or pause
> ingestion during steps 3-4.

### Metrics table (`OTELMetrics`)

Uses the column layout defined by the Go exporter's
[`metricsdata_to_adx.go`][go-metrics-schema]. Histogram and summary data points
produce multiple rows (`_sum`, `_count`, and one row per bucket or quantile).
Exponential histograms are projected to `_sum`, `_count`, and cumulative
`_bucket` rows.
Histograms without a bucket distribution still emit their known `_sum` and
`_count` rows. Generated `le` and `qt` dimensions use Go-compatible string
values, including `"+Inf"` and `"-Inf"`.

| Column | Type | Description |
| --- | --- | --- |
| `Timestamp` | `datetime` | Data point timestamp (RFC 3339 nanoseconds) |
| `MetricName` | `string` | Metric name (suffixed with `_sum`/`_count`/`_bucket`/quantile for Histogram/Summary) |
| `MetricType` | `string` | `Gauge`, `Sum`, `Histogram`, or `Summary` |
| `MetricUnit` | `string` | The metric's unit |
| `MetricDescription` | `string` | The metric's description |
| `MetricValue` | `real` | The data point value |
| `MetricAttributes` | `dynamic` | Data point attributes (plus `le`/`qt` for buckets/quantiles) |
| `Host` | `string` | `host.name` resource attribute, or the exporter process's own hostname |
| `ResourceAttributes` | `dynamic` | JSON object of OTel resource attributes |

[go-metrics-schema]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/azuredataexplorerexporter/metricsdata_to_adx.go

```kql
.create-merge table OTELMetrics (
    Timestamp: datetime,
    MetricName: string,
    MetricType: string,
    MetricUnit: string,
    MetricDescription: string,
    MetricValue: real,
    MetricAttributes: dynamic,
    Host: string,
    ResourceAttributes: dynamic
)

.create-or-alter table OTELMetrics ingestion json mapping "OTELMetricsMapping"
'[{"column":"Timestamp","path":"$.Timestamp"},'
'{"column":"MetricName","path":"$.MetricName"},'
'{"column":"MetricType","path":"$.MetricType"},'
'{"column":"MetricUnit","path":"$.MetricUnit"},'
'{"column":"MetricDescription","path":"$.MetricDescription"},'
'{"column":"MetricValue","path":"$.MetricValue"},'
'{"column":"MetricAttributes","path":"$.MetricAttributes"},'
'{"column":"Host","path":"$.Host"},'
'{"column":"ResourceAttributes","path":"$.ResourceAttributes"}]'

.alter table OTELMetrics policy streamingingestion enable

.show table OTELMetrics ingestion json mappings
```

Configure `metrics_table_json_mapping: "OTELMetricsMapping"` when using this
mapping.

### Traces table (`OTELTraces`)

Uses the column layout defined by the Go exporter's
[`tracesdata_to_adx.go`][go-traces-schema].

| Column | Type | Description |
| --- | --- | --- |
| `TraceID` | `string` | Hex-encoded trace ID |
| `SpanID` | `string` | Hex-encoded span ID |
| `ParentID` | `string` | Hex-encoded parent span ID |
| `SpanName` | `string` | The span name |
| `SpanStatus` | `string` | `Unset`, `Ok`, or `Error` |
| `SpanStatusMessage` | `string` | The status message |
| `SpanKind` | `string` | `Unspecified`, `Internal`, `Server`, `Client`, `Producer`, or `Consumer` |
| `StartTime` | `datetime` | Span start timestamp |
| `EndTime` | `datetime` | Span end timestamp |
| `ResourceAttributes` | `dynamic` | JSON object of OTel resource attributes |
| `TraceAttributes` | `dynamic` | Span + scope attributes |
| `Events` | `dynamic` | JSON array of `{EventName, Timestamp, EventAttributes}` |
| `Links` | `dynamic` | JSON array of `{TraceID, SpanID, TraceState, SpanLinkAttributes}` |

[go-traces-schema]: https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/exporter/azuredataexplorerexporter/tracesdata_to_adx.go

```kql
.create-merge table OTELTraces (
    TraceID: string,
    SpanID: string,
    ParentID: string,
    SpanName: string,
    SpanStatus: string,
    SpanStatusMessage: string,
    SpanKind: string,
    StartTime: datetime,
    EndTime: datetime,
    ResourceAttributes: dynamic,
    TraceAttributes: dynamic,
    Events: dynamic,
    Links: dynamic
)

.create-or-alter table OTELTraces ingestion json mapping "OTELTracesMapping"
'[{"column":"TraceID","path":"$.TraceID"},'
'{"column":"SpanID","path":"$.SpanID"},'
'{"column":"ParentID","path":"$.ParentID"},'
'{"column":"SpanName","path":"$.SpanName"},'
'{"column":"SpanStatus","path":"$.SpanStatus"},'
'{"column":"SpanStatusMessage","path":"$.SpanStatusMessage"},'
'{"column":"SpanKind","path":"$.SpanKind"},'
'{"column":"StartTime","path":"$.StartTime"},'
'{"column":"EndTime","path":"$.EndTime"},'
'{"column":"ResourceAttributes","path":"$.ResourceAttributes"},'
'{"column":"TraceAttributes","path":"$.TraceAttributes"},'
'{"column":"Events","path":"$.Events"},'
'{"column":"Links","path":"$.Links"}]'

.alter table OTELTraces policy streamingingestion enable

.show table OTELTraces ingestion json mappings
```

Configure `traces_table_json_mapping: "OTELTracesMapping"` when using this
mapping.

### Transformation fidelity and OTLP limitations

The exporter performs a lossy projection from the complete OTLP data model
into the ADX table schemas above. The primary fields needed to query logs,
metrics, and traces are retained, but an original OTLP message cannot be
reconstructed exactly from the resulting ADX rows.

Known omissions and substitutions are:

- Scope name, version, and attributes are merged into signal attributes, so
  their original OTLP structure is not retained.
- Byte attributes are encoded as lowercase hexadecimal strings, and
  non-finite floating-point attributes are encoded as JSON `null`.
- Log body arrays and key-value lists are recursively encoded as native JSON
  arrays and objects. With legacy string-body encoding enabled, those JSON
  values are stored as strings instead. Empty bodies encode as `null`, and
  missing log timestamps use `0001-01-01T00:00:00Z`.
- Metrics are flattened into `MetricValue` rows. Exemplars, data-point flags,
  aggregation temporality, and sum monotonicity are not retained.
- Histogram, exponential histogram, and summary aggregates are expanded into
  separate rows. Missing or non-finite numeric metric values permanently
  refuse the source message. A zero metric timestamp is replaced once per data
  point with the current wall-clock time.
- Histogram bucket bounds and summary quantiles are stored as strings in
  `MetricAttributes` to match the Go exporter representation.
- Malformed explicit or exponential histogram points permanently refuse the
  source message; no partial sum, count, or bucket rows are submitted.
- The Go-compatible trace table has no columns for span-level trace state,
  resource or scope schema URLs, span dropped-attribute, dropped-event, or
  dropped-link counts, or event and link dropped-attribute counts. Link-level
  trace state is retained. Preserving the omitted structural metadata requires
  an explicit table or mapping schema evolution; it is not mixed into customer
  attribute objects under synthetic keys.
- Missing span timestamps use `0001-01-01T00:00:00Z`, and missing IDs are
  encoded as empty strings.

These limitations can affect queries that depend on complete OTLP semantic
fidelity. See [Signal transformation](design.md#signal-transformation) in the
design document for the maintained per-signal behavior.

#### Internal telemetry logs table

When using the engine's observability pipeline to export internal logs
(`otel_info!`, `otel_warn!`, etc.) to ADX, create a separate table to
keep engine telemetry distinct from customer data:

```kql
.create-merge table OTELInternalLogs (
    Timestamp: datetime,
    ObservedTimestamp: datetime,
    TraceID: string,
    SpanID: string,
    SeverityText: string,
    SeverityNumber: int,
    Body: string,
    EventName: string,
    ResourceAttributes: dynamic,
    LogsAttributes: dynamic
)

.create-or-alter table OTELInternalLogs ingestion json mapping "OTELInternalLogsMapping"
'[{"column":"Timestamp","path":"$.Timestamp"},'
'{"column":"ObservedTimestamp","path":"$.ObservedTimestamp"},'
'{"column":"TraceID","path":"$.TraceID"},'
'{"column":"SpanID","path":"$.SpanID"},'
'{"column":"SeverityText","path":"$.SeverityText"},'
'{"column":"SeverityNumber","path":"$.SeverityNumber"},'
'{"column":"Body","path":"$.Body"},'
'{"column":"EventName","path":"$.EventName"},'
'{"column":"ResourceAttributes","path":"$.ResourceAttributes"},'
'{"column":"LogsAttributes","path":"$.LogsAttributes"}]'

.alter table OTELInternalLogs policy streamingingestion enable

.show table OTELInternalLogs ingestion json mappings
```

#### Internal telemetry metrics table

Store engine metrics in a separate table with the `OTELMetrics` schema to keep
engine telemetry distinct from customer data:

```kql
.create-merge table OTELEngineMetrics (
    Timestamp: datetime,
    MetricName: string,
    MetricType: string,
    MetricUnit: string,
    MetricDescription: string,
    MetricValue: real,
    MetricAttributes: dynamic,
    Host: string,
    ResourceAttributes: dynamic
)

.create-or-alter table OTELEngineMetrics ingestion json mapping "OTELEngineMetricsMapping"
'[{"column":"Timestamp","path":"$.Timestamp"},'
'{"column":"MetricName","path":"$.MetricName"},'
'{"column":"MetricType","path":"$.MetricType"},'
'{"column":"MetricUnit","path":"$.MetricUnit"},'
'{"column":"MetricDescription","path":"$.MetricDescription"},'
'{"column":"MetricValue","path":"$.MetricValue"},'
'{"column":"MetricAttributes","path":"$.MetricAttributes"},'
'{"column":"Host","path":"$.Host"},'
'{"column":"ResourceAttributes","path":"$.ResourceAttributes"}]'

.alter table OTELEngineMetrics policy streamingingestion enable

.show table OTELEngineMetrics ingestion json mappings
```

Then configure the observability pipeline in your pipeline YAML. A single
`adx` exporter instance handles both signals. Route logs and metrics from
`internal_telemetry` to the same node:

```yaml
engine:
  telemetry:
    logs:
      providers:
        global: its
        engine: its
        internal: console_direct
        admin: console_direct
  observability:
    pipeline:
      extensions:
        azure-data-explorer-auth:
          type: "urn:microsoft:extension:azure_identity_auth"
          config:
            method: managed_identity
            scope: "https://<your-cluster>.kusto.windows.net/.default"
      nodes:
        itr:
          type: "receiver:internal_telemetry"
          config:
            signals: [logs, metrics]
        adx-internal:
          type: "urn:microsoft:exporter:azure_data_explorer"
          capabilities:
            bearer_token_provider: azure-data-explorer-auth
          config:
            cluster_uri: "https://<your-cluster>.kusto.windows.net"
            db_name: "<your-db>"
            logs_table_name: "OTELInternalLogs"
            metrics_table_name: "OTELEngineMetrics"
            logs_table_json_mapping: "OTELInternalLogsMapping"
            metrics_table_json_mapping: "OTELEngineMetricsMapping"
            network_requests:
              coalesce: true
              max_pending_messages: 16
              max_rows: 1000
              flush_interval: 1s
            max_in_flight: 1
      connections:
        - from: itr
          to: adx-internal
```

> **Note:** The `internal` provider must be `console_direct` or `noop` to
> prevent a feedback loop where logs from the observability pipeline feed
> back into itself.

## Usage

### Running

```bash
./target/release/df_engine --config config.yaml --num-cores 4
```

### Testing with OTLP Receiver

```bash
# Start the collector with the Azure Identity Auth extension enabled
cargo run --release --features azure-data-explorer -- \
  --config crates/contrib-nodes/src/exporters/azure_data_explorer_exporter/otlp-adx.yaml \
  --num-cores 1

# In another terminal, send test data:

# Option A: Using telemetrygen
telemetrygen logs --otlp-endpoint localhost:4317 --otlp-insecure --logs 10

# Option B: Using grpcurl
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
          "body": {"stringValue": "Hello from ADX Exporter!"},
          "severityText": "INFO"
        }]
      }]
    }]
  }' \
  localhost:4317 \
  opentelemetry.proto.collector.logs.v1.LogsService/Export

# Option C: Configure your instrumented app to send OTLP Logs to localhost:4317
```

### Testing with fake data generator

```bash
cargo run --release --features azure-data-explorer -- \
  --config crates/contrib-nodes/src/exporters/azure_data_explorer_exporter/fakegen-adx.yaml \
  --num-cores 1
```

### Traffic generator examples

Two traffic-generator examples are provided:

| File | Destination | Prerequisites |
| --- | --- | --- |
| [`traffic-sde.yaml`](traffic-sde.yaml) | Azure Data Explorer streaming ingestion | Replace the cluster URI, database, and table; run `az login` or sign in with `azd`; grant the identity Database Ingestor access |
| [`traffic-ade-local.yaml`](traffic-ade-local.yaml) | ADX-compatible endpoint at `http://localhost:8080` | Start a loopback endpoint that accepts gzip-compressed JSON Lines POST requests |

Run either example from the `rust/otap-dataflow` directory:

```bash
cargo run --release --features azure-data-explorer -- \
  --config crates/contrib-nodes/src/exporters/azure_data_explorer_exporter/traffic-sde.yaml \
  --num-cores 1

cargo run --release --features azure-data-explorer -- \
  --config crates/contrib-nodes/src/exporters/azure_data_explorer_exporter/traffic-ade-local.yaml \
  --num-cores 1
```

The local endpoint must accept `POST` requests at
`/v1/rest/ingest/oteldb/OTELLogs?streamFormat=JSON`. Request bodies use
`Content-Encoding: gzip` and contain JSON Lines after decompression. The
exporter still requires the shared `azure_identity_auth` extension, so the
example uses development credentials even when the local endpoint ignores the
`Authorization` header.

## Azure Setup

1. Create or select an ADX cluster.
2. Create the target database, for example `oteldb`.
3. Create the required tables and any JSON ingestion mappings referenced by
   the exporter configuration.
4. Enable streaming ingestion on the cluster and target tables. See the
   [streaming ingestion documentation][streaming-docs].

Grant the exporter identity the `Database Ingestor` role. Use the managed
identity principal ID and tenant ID. `.add` preserves existing role
assignments; `.set` replaces them.

```kql
.add database oteldb ingestors ('aadapp=00000000-0000-0000-0000-000000000000;11111111-1111-1111-1111-111111111111')
```

Verify the role assignment:

```kql
.show database oteldb principals
| where Role == "Database Ingestor"
```

Configure authentication with managed identity or development credentials.
For a user-assigned managed identity, set `client_id` in the
`azure_identity_auth` extension. Omit it to use the system-assigned identity.

Use the Cluster URI, not the Data Ingestion URI, for `cluster_uri`.

For an Azure Container Apps walkthrough, including identity creation and
environment configuration, see the One Collector
[deployment guide](../../../../../../../collector/examples/microsoft-json/DEPLOYMENT.md#configure-managed-identity-for-adx).

[streaming-docs]: https://learn.microsoft.com/en-us/azure/data-explorer/ingest-data-streaming

## License

Apache 2.0
