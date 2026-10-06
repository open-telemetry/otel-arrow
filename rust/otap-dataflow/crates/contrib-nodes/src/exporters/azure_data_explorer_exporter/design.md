# Azure Data Explorer Exporter Design

<!-- markdownlint-disable MD013 -->

Status: alpha implementation

Owner: contrib-nodes

Module: `azure_data_explorer_exporter`

URN: `urn:microsoft:exporter:azure_data_explorer`

Feature: `azure-data-explorer`

## Implemented scope

The Azure Data Explorer exporter sends OpenTelemetry logs, metrics, and traces
to Azure Data Explorer through the streaming ingestion REST API.

The implementation:

- accepts OTAP Arrow records and OTLP protobuf request bytes;
- transforms logs, metrics, and traces into ADX JSON rows;
- gzip-compresses newline-delimited JSON requests;
- sends requests directly to the ADX cluster endpoint;
- obtains bearer tokens from a bound `bearer_token_provider`;
- supports bounded request concurrency and optional cross-message coalescing;
- retries transient HTTP and transport failures with bounded backoff;
- retains source messages until ADX accepts or rejects their request;
- reports shared exporter boundary metrics and ADX-specific diagnostics.

The exporter does not currently implement ADX queued ingestion or the Go Kusto
SDK's managed streaming client. The removed `ingestion_type` field is rejected
as an unknown field. See [Future work](#future-work).

## Component integration

The exporter is registered in `exporters/mod.rs` behind the
`azure-data-explorer` feature. The workspace feature also enables the
`azure_identity_auth` extension.

The implementation is split across:

- `mod.rs`: factory registration, URN, and capability resolution;
- `config.rs`: typed user configuration and validation;
- `exporter.rs`: pipeline loop, transformation, coalescing, backpressure,
  authentication integration, acknowledgements, and shutdown;
- `client.rs`: HTTP request construction, retries, timeout, and response
  classification;
- `transformer.rs`: log, metric, and trace row transformation;
- `encoding.rs`: JSON string and hexadecimal encoding helpers;
- `error.rs`: error types and retry/refusal classification;
- `metrics.rs`: shared exporter metrics and ADX-specific metric sets;
- `telemetry.md`: emitted metric and event inventory.

The factory requires a local `BearerTokenProvider` capability. Configuration
resolution fails before startup when that capability is not bound.

## Authentication

Credential acquisition and refresh belong to the shared
`azure_identity_auth` extension. The exporter does not accept client IDs,
client secrets, tenant IDs, or credential-selection fields.

A pipeline declares and binds the extension:

```yaml
extensions:
  azure-data-explorer-auth:
    type: "urn:microsoft:extension:azure_identity_auth"
    config:
      method: managed_identity
      scope: "https://<cluster>.<region>.kusto.windows.net/.default"

nodes:
  azure-data-explorer-exporter:
    type: "urn:microsoft:exporter:azure_data_explorer"
    capabilities:
      bearer_token_provider: azure-data-explorer-auth
    config:
      cluster_uri: "https://<cluster>.<region>.kusto.windows.net"
```

The extension supports managed identity, development credentials, and workload
identity. One extension instance represents one identity and OAuth scope and
may serve multiple exporters in the same pipeline.

The exporter uses `BearerAuth` to:

- wait for the first usable provider token before accepting data;
- cache the current authorization header and token generation;
- stop accepting new data near token expiry;
- invalidate only the token generation rejected by ADX;
- resume when the provider publishes a replacement token.

An HTTP 401 terminates the current physical HTTP operation. When request
coalescing is enabled, the exporter may retain the compressed request and its
source messages until a replacement token is available. Retained
reauthentication retries are bounded by `max_retries`. Without coalescing, a
401 produces a terminal NACK for that input.

## Streaming ingestion protocol

For each logical request, the exporter constructs:

```text
POST {cluster_uri}/v1/rest/ingest/{database}/{table}?streamFormat=JSON[&mappingName={mapping}]
```

Database, table, and optional mapping names are percent-encoded. The exporter
omits `mappingName` when no non-empty mapping is configured.

Requests include:

- `Authorization: Bearer <token>`;
- `Content-Type: application/json`;
- `Content-Encoding: gzip`;
- an `x-ms-client-request-id` containing the table, physical attempt number,
  and a stable UUID for the HTTP operation.

The body is gzip-compressed JSON Lines. Rows are separated by newline bytes
without an additional trailing newline.

The configured `cluster_uri` must be the query/cluster URI. An
`ingest-`-prefixed Data Ingestion URI is rejected because this implementation
uses the cluster's streaming endpoint.

## Configuration

Unknown fields are rejected.

| Field | Default | Behavior |
| --- | --- | --- |
| `cluster_uri` | required | ADX cluster URI used as the streaming REST base |
| `db_name` | `oteldb` | Target database |
| `logs_table_name` | `OTELLogs` | Target logs table |
| `metrics_table_name` | `OTELMetrics` | Target metrics table |
| `traces_table_name` | `OTELTraces` | Target traces table |
| `logs_table_json_mapping` | omitted | Optional pre-created logs JSON mapping name |
| `metrics_table_json_mapping` | omitted | Optional pre-created metrics JSON mapping name |
| `traces_table_json_mapping` | omitted | Optional pre-created traces JSON mapping name |
| `legacy_logs_body_string` | `false` | Encode log bodies as strings for legacy tables |
| `export_event_name` | `true` | Emit the top-level log `EventName` property |
| `add_event_name_to_log_attributes` | `true` | Add `event.name` when it is not already present |
| `timeout` | `30s` | Total deadline for one HTTP operation, including retries and backoff |
| `gzip_compression_level` | `6` | Gzip level from 0 through 9 |
| `max_row_bytes` | `921600` | Maximum serialized row size including JSON Lines framing |
| `max_in_flight` | `16` | Maximum concurrent HTTP requests per exporter instance |
| `max_retries` | `5` | Retries after the initial attempt |
| `network_requests` | see below | Optional cross-message request coalescing |
| `log_failed_payload` | `false` | Log failed payload and sample-row content at debug level |
| `log_response_body` | `false` | Log ADX error response content at debug level |

Empty mapping names are treated as omitted. Configured names must refer to
pre-created mappings on their target tables. Standard ADX JSON streaming
ingestion requires a mapping; omission remains available for compatible
endpoints or service configurations that accept an unmapped request.

`cluster_uri` must be an HTTPS origin without user information, a path, query,
or fragment. HTTP is accepted only for loopback development endpoints. The
Azure Identity scope for direct REST authentication is the cluster URI
followed by `/.default`.

`max_row_bytes` must be between 1 and 921600 bytes. The limit is checked before
compression and before admission to a coalesced request. One oversized row
permanently refuses its complete source message.

Both logging options are intended only for temporarily debugging export or
connection issues. They are disabled by default, emit sensitive content only
at debug level, and are not recommended in production unless strictly
necessary. `log_failed_payload` controls outbound request payloads and sample
rows. `log_response_body` independently controls inbound ADX error response
chunks. Payloads and responses may contain PII, schema values, telemetry
attributes, or other customer-sensitive data. Operators must disable these
settings immediately after diagnosis and protect collected debug logs
appropriately. These debug events are emitted only for failed or rejected
exports; successful exports do not log payload or response content. Body-free
failure warnings continue to report status, destination, attempt, and response
size.

### Network request coalescing

The nested request settings are:

```yaml
network_requests:
  coalesce: false
  max_pending_messages: 16
  max_rows: 1000
  max_bytes: 4194304
  flush_interval: 1s
```

When `coalesce` is false, each non-empty input message becomes one logical ADX
request. When it is true, complete source messages of the same signal may be
combined until a row, byte, message, or time threshold triggers a flush.
`flush_interval` is ignored when coalescing is disabled.

Source messages accumulated for coalescing remain unresolved: the exporter
does not ACK or NACK them until the request is flushed, sent, and ADX returns a
terminal response. A larger `flush_interval` directly increases acknowledgement
latency for partial requests. If set too high, it can retain receiver responses,
consume `max_pending_messages`, and apply backpressure that holds up the
upstream pipeline. The 1-second default limits that latency while still
allowing low-volume messages to be coalesced.

Rows from one source message are never split across requests. The exporter
permanently refuses the complete source message if its transformed output
exceeds `max_rows` or `max_bytes`. `max_bytes` cannot exceed the ADX streaming
ingestion limit of 4 MiB uncompressed.

`max_rows` is limited to 1 through 100000, `max_bytes` to 1 through 4194304,
and `max_in_flight` to 1 through 1024.

JSON writers enforce the per-row and remaining request byte budgets during
serialization. Recursive OTLP arrays and key-value lists are limited to 64
levels and 65536 visited values per source message.

`max_pending_messages` bounds retained source messages across accumulators,
in-flight requests, and retained authentication retries for one exporter
instance. Once the limit is reached, the exporter stops receiving PData until
capacity becomes available. `max_in_flight` independently bounds concurrent
HTTP requests.

All bounds are per exporter instance. Process-wide capacity scales with the
number of pipeline instances.

## Retry and timeout contract

`max_retries` follows the repository's retry naming convention and counts
retries after the initial attempt:

- `0` performs one initial attempt and no retry;
- `1` permits one retry, for at most two physical attempts;
- the default `5` permits at most six physical attempts.

Transport failures, HTTP 429, HTTP 5xx, and qualifying response-body failures
are retryable. Other HTTP 4xx responses are terminal refusals. Backoff starts
at approximately three seconds, uses jitter, and is capped at thirty seconds.
When ADX supplies `Retry-After`, the exporter waits at least that long within
the operation deadline.

One `timeout` deadline covers request transmission, response-body handling,
transient retries, and retry backoff for an HTTP operation. Each retained
reauthentication dispatch starts a new HTTP operation and therefore a new
deadline.

The operation UUID and compressed bytes remain stable across transient HTTP
retries. Every physical attempt records a separate shared exporter attempt.

## Acknowledgement and failure contract

The exporter retains ownership of every non-empty source PData message until
its logical request reaches a terminal result.

- Transformation or view-construction failure produces a permanent NACK.
- A row larger than `max_row_bytes` produces a permanent refused NACK.
- A valid input that contains no signal records is ACKed immediately.
- Successful ADX HTTP acceptance ACKs every source message in the request.
- Terminal client refusals permanently NACK every source message with cause
  `refused`. Timeouts, throttling, server, transport, and authentication
  failures remain retryable.
- Coalesced requests rejected with HTTP 401 may remain pending while waiting
  for a replacement token.
- Shutdown flushes and drains only until the engine deadline. At the deadline,
  remaining requests are canceled. The exporter routes `node_shutdown` NACKs
  while the deadline permits, then records and abandons work that cannot be
  routed without exceeding the deadline.

The exporter never ACKs a message merely because it entered a local
accumulator. This preserves upstream retry behavior when ADX does not accept
the request.

The exporter does not claim exactly-once delivery. A network failure after ADX
accepts a request but before the response reaches the exporter can lead to
duplicates when upstream replay occurs.

## Signal transformation

All three signal transformers live in `transformer.rs`. The implementation
targets the documented ADX table shapes through a lossy projection rather than
providing complete semantic parity with every OTLP field. Primary query fields
are retained, but omitted, flattened, merged, dropped, or substituted values
mean that the original OTLP message cannot be reconstructed exactly from ADX
rows. The sections below define the currently known fidelity limitations.

### Common encoding

- Resource and signal attributes are encoded as JSON objects.
- Attribute arrays and key-value lists are recursively encoded.
- Attribute byte arrays are encoded as lowercase hexadecimal strings.
- Non-finite attribute floating-point values are encoded as JSON `null`.
- Trace and span IDs are lowercase hexadecimal strings.
- Missing IDs are empty strings.
- Scope name, version, and scope attributes are merged into signal attributes.
- Record, data-point, or span attributes take precedence when duplicate keys
  are interpreted using last-key-wins JSON handling.

### Logs

One row is emitted per log record. The row includes timestamp, observed
timestamp, trace and span IDs, severity, body, optional event name, resource
attributes, and merged log attributes.

With `legacy_logs_body_string: false`, scalar bodies retain their JSON scalar
type, arrays are recursively encoded as JSON arrays, and key-value lists are
recursively encoded as JSON objects. Empty bodies encode as `null`. With the
legacy option enabled, non-string bodies are serialized as JSON and stored as
strings.

Go exporter column compatibility requires all three settings:

```yaml
legacy_logs_body_string: true
export_event_name: false
add_event_name_to_log_attributes: false
```

This preserves the Go logs column layout but does not claim byte-for-byte
transformation parity.

Missing log timestamps use `0001-01-01T00:00:00Z`. When enabled, event-name
fallback adds `event.name` only if neither scope nor log attributes already
contains that key.

### Metrics

Gauge and sum points produce one flattened row per data point. Histogram
points expand into sum, count, and bucket rows. Summary points expand into
sum, count, and quantile rows. Exponential histograms expand into optional
sum, count, cumulative bucket, and positive-infinity bucket rows.
Explicit histograms without a bucket distribution still emit the known sum
and count. Generated `le` and `qt` dimensions are Go-compatible strings,
including `+Inf` and `-Inf`.

Histogram cardinality, computed bounds, zero threshold, cumulative totals, and
declared count are validated before any rows for the point are emitted.
Malformed histogram points permanently refuse their complete source message.

Rows use the Go-compatible `MetricValue` shape rather than preserving complete
OTLP aggregate structures, exemplars, flags, temporality, or monotonicity.
Missing or non-finite numeric values permanently refuse their complete source
message. A zero metric timestamp uses the exporter's current wall-clock time.

### Traces

One row is emitted per span. Rows include IDs, name, status, kind, start and
end times, resource and span attributes, events, and links.

Events preserve name, timestamp, and attributes. Links preserve trace ID,
span ID, trace state, and attributes. The current row does not preserve every
OTLP field because the Go-compatible ADX table shape has no corresponding
columns. Specifically, it omits span-level trace state; resource and scope
schema URLs; span dropped-attribute, dropped-event, and dropped-link counts;
and dropped-attribute counts for events and links. Link-level trace state is
preserved. Missing span timestamps use `0001-01-01T00:00:00Z`.

These values are not inserted into `ResourceAttributes` or `TraceAttributes`
under synthetic keys because doing so would mix OTLP structural metadata with
user attributes and could collide with customer-provided keys. Preserving them
requires an explicit versioned table or mapping evolution with documented
columns.

## Telemetry

The complete inventory is maintained in [telemetry.md](telemetry.md).

The exporter adopts the shared exporter boundary metrics:

- `exporter.attempted.messages`;
- `exporter.attempted.duration`;
- `exporter.attempted.payload.size`;
- `exporter.attempted.items`.

One observation is recorded for every physical HTTP attempt, including each
retry. Preparation-only failures and refusals are recorded even when no HTTP
request is sent. Payload size is the compressed request size, and item count is
the number of source OTLP signal items represented by the request. Initial
attempt duration includes gzip compression and HTTP work; retry duration uses
the already encoded payload and excludes backoff. Coalescing wait and
transformation before batch formation are excluded.

Inputs that successfully produce no ADX rows record a successful no-op attempt
before ACK.

ADX-specific metrics are:

- `exporter.azure_data_explorer.batches`, partitioned by signal and terminal
  outcome;
- `exporter.azure_data_explorer.http.responses`, partitioned by bounded HTTP
  response category.

Logical batch totals are recorded once and are not duplicated by retries.
Authentication acquisition and refresh telemetry belongs to the bound
authentication extension.

ADX response bodies, failed request payloads, and sample rows are omitted by
default. When explicitly enabled for debugging, sensitive content is emitted
only at debug level; normal warning events remain body-free.

## Runtime and resource model

The exporter uses local `Rc<RefCell<_>>` metric state and local boxed futures.
It does not introduce cross-core shared mutable state. Each pipeline instance
owns its client pool, accumulators, retained authentication requests, and
in-flight futures.

Transformation and gzip compression execute synchronously on the pipeline's
local runtime thread. Transformation stops at the configured request row or
byte budget. Set coalescing thresholds and upstream batching with this bounded
local-runtime work in mind.

The request body and transformed rows are held in memory. There is no
temporary-file spool. Per-row size, transformed row count, and uncompressed
JSON Lines bytes are bounded before gzip compression. Error response content
is retained only up to 64 KiB.

## Internal telemetry pipelines

Engine observability pipelines support pipeline-scoped extensions, so the same
authentication binding can be used for internal logs and metrics. Use the
canonical component URNs:

```yaml
engine:
  observability:
    pipeline:
      extensions:
        adx-auth:
          type: "urn:microsoft:extension:azure_identity_auth"
          config:
            method: managed_identity
            scope: "https://<cluster>.<region>.kusto.windows.net/.default"
      nodes:
        internal-telemetry:
          type: "urn:otel:receiver:internal_telemetry"
          config:
            signals: [logs, metrics]
        adx-internal:
          type: "urn:microsoft:exporter:azure_data_explorer"
          capabilities:
            bearer_token_provider: adx-auth
          config:
            cluster_uri: "https://<cluster>.<region>.kusto.windows.net"
            db_name: "oteldb"
            logs_table_name: "OTELInternalLogs"
            metrics_table_name: "OTELEngineMetrics"
            logs_table_json_mapping: "OTELInternalLogsMapping"
            metrics_table_json_mapping: "OTELEngineMetricsMapping"
            max_in_flight: 1
            network_requests:
              coalesce: true
              max_pending_messages: 16
              max_rows: 1000
              max_bytes: 4194304
              flush_interval: 1s
      connections:
        - from: internal-telemetry
          to: adx-internal
```

The internal log provider must avoid routing exporter diagnostics back into the
same observability pipeline, or a feedback loop can result.

## Testing strategy

Tests cover:

- typed configuration defaults and validation;
- endpoint encoding and JSON mapping behavior;
- request headers and JSON Lines framing;
- transient retries, `Retry-After`, operation timeout, and per-attempt metrics;
- bounded error responses and response classification;
- malformed raw OTLP framing for logs, metrics, and traces;
- missing bearer-token capability rejection;
- HTTPS-only endpoint validation and request-size refusal;
- request coalescing thresholds and message boundaries;
- recoverable in-flight ownership for deadline-bound shutdown;
- backpressured completion routing at the shutdown deadline;
- successful no-op attempt accounting;
- shutdown while request capacity is full;
- 401 retention followed by replacement-token retry;
- log schema options and event-name precedence;
- explicit and exponential histogram transformation;
- OTAP Arrow transformation for logs, metrics, and traces;
- exporter-loop ACK/NACK coverage for metrics and traces;
- bounded metric expansion and timestamp/sum semantics;
- bounded JSON escaping, recursive nesting, and malformed metric refusal;
- shared and ADX-specific metric partitioning.

Validate configuration examples with:

```bash
cargo run --features azure-data-explorer -- \
  --validate-and-exit \
  --config <example.yaml>
```

The feature-specific test command is:

```bash
cargo test -p otel-arrow-dfe-contrib-nodes \
  --no-default-features \
  --features azure-data-explorer \
  --lib
```

## Known limitations

- Only direct streaming ingestion is implemented.
- ADX streaming ingestion must be enabled for the target tables.
- Queued ingestion and post-submission ingestion-status monitoring are absent.
- Client-secret service-principal fields are intentionally not accepted.
- One source message is never split to satisfy coalescing thresholds.
- [Signal transformation](#signal-transformation) is lossy; review the
  documented omissions and substitutions before relying on complete OTLP
  semantic fidelity.
- Exponential histograms are projected to cumulative bucket rows.
- Compression and transformation are synchronous local-runtime work.

## Future work

Queued-ingestion parity with the Go exporter requires a separate design and
implementation for:

- ingestion-resource discovery and refresh;
- temporary storage upload;
- service queue selection and submission;
- ingestion-property and mapping-reference encoding;
- stable source identifiers and deduplication properties;
- managed streaming eligibility and queued fallback;
- post-submission status telemetry;
- wire-level conformance fixtures against a pinned Go Kusto SDK version.

Future semantic-fidelity work may preserve:

- metric flags, exemplars, temporality, and monotonicity;
- span-level trace state, resource and scope schema URLs, and span, event, and
  link dropped counts through an explicit ADX schema evolution;
- explicit transformation diagnostics for dropped or substituted values.

Add configuration for these features only with their protocol and delivery
implementations and tests.
