# ClickHouse Exporter

This exporter accepts OTAP Arrow payloads and serialized OTLP requests,
reshapes them into ClickHouse-compatible Arrow `RecordBatch`es, and inserts
them into ClickHouse over HTTP using the official ClickHouse Rust client
(`clickhouse` + `clickhouse-ext-arrow`, `FORMAT ArrowStream`).

The current architecture is intentionally simple:

- one flat table for logs
- one flat table for traces
- attributes always inlined on the signal tables as
  `Map(LowCardinality(String), String)`
- no lookup attribute tables
- no views
- no ID generation layer

The schema and transform behavior are aligned with the Go OpenTelemetry
Collector ClickHouse exporter where practical.

## Status

Implemented:

- logs table DDL aligned with the Go exporter structure
- traces table DDL aligned with the Go exporter structure
- `TraceFlags` passthrough for logs
- `Duration` decode/cast into traces
- `ServiceName` extraction from inlined resource attributes
- string-map attribute columns
- span events and links inlined into traces
- snapshot and unit coverage for the current schema/transform behavior

Not implemented yet:

- metrics export beyond stub table definitions
- broader integration/e2e validation against a live ClickHouse instance

## Quick Start

### 1. Start ClickHouse

```bash
docker run -it -p 8123:8123 -p 9000:9000 -e CLICKHOUSE_PASSWORD=test \
  --name clickhouse-server --ulimit nofile=262144:262144 \
  clickhouse/clickhouse-server
```

### 2. Run the data plane with the ClickHouse exporter

Run from the `rust/otap-dataflow` workspace directory.

```bash
cd rust/otap-dataflow
cargo run --features clickhouse-exporter -- --config configs/trafficgen-clickhouse.yaml
```

### 3. Query ClickHouse

```bash
docker exec -it clickhouse-server clickhouse-client --password test
```

Then:

```sql
USE otap;
SELECT * FROM otel_logs LIMIT 10;
SELECT * FROM otel_traces LIMIT 10;
```

## Runtime Flow

At runtime the exporter does the following:

1. Deserializes `ConfigPatch` and normalizes it into `Config`
2. Connects to ClickHouse and creates the target database and configured
   tables if enabled
3. Receives `OtapPdata` messages from the engine
4. Transforms serialized OTLP logs directly into ClickHouse columns; if that
   path cannot handle an input, converts it into `OtapArrowRecords`
5. Uses a second specialized transformer for canonical OTAP logs, with the
   generic transform pipeline as a fallback and for other inputs
6. Returns only signal batches (`Logs`, `Spans`) from the transformer
7. Inserts those batches into the destination tables

## Supported Payloads

The exporter currently understands these OTAP payload types:

- `ResourceAttrs`
- `ScopeAttrs`
- `Logs`
- `LogAttrs`
- `Spans`
- `SpanAttrs`
- `SpanEventAttrs`
- `SpanEvents`
- `SpanLinkAttrs`
- `SpanLinks`

Only `Logs` and `Spans` are written to ClickHouse tables. The attribute and
child payloads are consumed during transformation.

## Configuration Model

Top-level config fields:

- `endpoint`
- `database`
- `username`
- `password` (supports `${env:VAR}` / `${env:VAR:-default}` substitution, e.g. `"${env:CLICKHOUSE_PASSWORD}"`)
- `async_insert`
- `max_in_flight` (positive integer, defaults to `10`)
- `table_defaults`
- `tables`

`max_in_flight` bounds the number of ClickHouse HTTP insert requests that can
run concurrently. Values greater than one overlap synchronous inserts and may
complete them out of order. When the limit is reached, the exporter applies
backpressure until an insert completes. The default of `10` matches the insert
concurrency used by the benchmark Collector configuration. Set it to `1` to
retain serialized insert behavior.

Inline attributes are always stored as `Map(LowCardinality(String), String)`;
there is no per-group representation configuration.

Table config supports:

- table name overrides
- per-table TTL
- engine override
- `create_schema`

## Schema Shape

The exporter creates one row per log record in the configured logs table
(`otel_logs` by default) and one row per span in the configured traces table
(`otel_traces` by default). Both table names can be overridden under `tables`.
All columns are non-nullable; when a transformed Arrow batch omits an optional
column, ClickHouse supplies that column type's default value.

The schema follows the clickhouse-exporter in opentelemetry-collector-contrib
where practical, based on
[the Go exporter's SQL templates](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/clickhouseexporter/internal/sqltemplates).

### Logs table

<!-- markdownlint-disable MD013 -->

| Column | ClickHouse type | OTLP meaning |
| --- | --- | --- |
| `Timestamp` | `DateTime64(9)` | Log record `time_unix_nano` |
| `TraceId` | `String` | Lowercase hexadecimal trace ID |
| `SpanId` | `String` | Lowercase hexadecimal span ID |
| `TraceFlags` | `UInt8` | Trace-flags portion of the log flags |
| `SeverityText` | `LowCardinality(String)` | Original severity text |
| `SeverityNumber` | `UInt8` | OTLP severity number |
| `ServiceName` | `LowCardinality(String)` | First `service.name` resource attribute |
| `Body` | `String` | Stringified log body; complex values use JSON |
| `ResourceSchemaUrl` | `LowCardinality(String)` | Resource-logs schema URL |
| `ResourceAttributes` | `Map(LowCardinality(String), String)` | Resource attributes flattened onto each log row |
| `ScopeSchemaUrl` | `LowCardinality(String)` | Scope-logs schema URL |
| `ScopeName` | `String` | Instrumentation scope name |
| `ScopeVersion` | `LowCardinality(String)` | Instrumentation scope version |
| `ScopeAttributes` | `Map(LowCardinality(String), String)` | Scope attributes flattened onto each log row |
| `LogAttributes` | `Map(LowCardinality(String), String)` | Log-record attributes |
| `EventName` | `String` | Log-record event name |

<!-- markdownlint-enable MD013 -->

### Traces table

<!-- markdownlint-disable MD013 -->

| Column | ClickHouse type | OTLP meaning |
| --- | --- | --- |
| `Timestamp` | `DateTime64(9)` | Span start time |
| `TraceId` | `String` | Lowercase hexadecimal trace ID |
| `SpanId` | `String` | Lowercase hexadecimal span ID |
| `ParentSpanId` | `String` | Lowercase hexadecimal parent span ID |
| `TraceState` | `String` | W3C trace state |
| `SpanName` | `LowCardinality(String)` | Span name |
| `SpanKind` | `LowCardinality(String)` | String representation of the OTLP span kind |
| `ServiceName` | `LowCardinality(String)` | First `service.name` resource attribute |
| `ResourceAttributes` | `Map(LowCardinality(String), String)` | Resource attributes flattened onto each span row |
| `ScopeName` | `String` | Instrumentation scope name |
| `ScopeVersion` | `LowCardinality(String)` | Instrumentation scope version |
| `SpanAttributes` | `Map(LowCardinality(String), String)` | Span attributes |
| `Duration` | `UInt64` | Span duration in nanoseconds |
| `StatusCode` | `LowCardinality(String)` | String representation of the OTLP status code |
| `StatusMessage` | `String` | Span status message |
| `Events.Timestamp` | `Array(DateTime64(9))` | Event timestamps |
| `Events.Name` | `Array(LowCardinality(String))` | Event names |
| `Events.Attributes` | `Array(Map(LowCardinality(String), String))` | One attribute map per event |
| `Links.TraceId` | `Array(String)` | Lowercase hexadecimal linked trace IDs |
| `Links.SpanId` | `Array(String)` | Lowercase hexadecimal linked span IDs |
| `Links.TraceState` | `Array(String)` | Linked trace states |
| `Links.Attributes` | `Array(Map(LowCardinality(String), String))` | One attribute map per link |

<!-- markdownlint-enable MD013 -->

The three `Events.*` arrays have matching positions: element `i` in each array
describes the same event. The four `Links.*` arrays follow the same rule for a
link. Events and links therefore remain embedded in the span row rather than
being written to child tables.

### Storage layout and indexes

The default tables use `MergeTree`, partition by `toDate(Timestamp)`, and use
an index granularity of 8192. Logs are ordered by five-minute timestamp bucket,
service name, and timestamp. Traces are ordered by service name, span name, and
second-resolution timestamp. Both tables have a trace-ID bloom-filter index and
bloom-filter indexes for resource attribute keys and values. Logs add scope and
log attribute bloom filters plus a token bloom filter over lowercase body text;
traces add span attribute bloom filters and a min-max duration index.

String-like columns use ZSTD compression, and timestamps use Delta plus ZSTD.
Configuration can override the table engine and add a TTL. Exact generated DDL
is snapshot-tested in
[table_snapshots](./table_snapshots/); `schema.rs` and `tables.rs` are the source
of truth. There is currently no automated test for drift from the Go Collector
schema.

## Verified Arrow -> ClickHouse Type Mapping

Inserts go out as `FORMAT ArrowStream` (Arrow IPC over HTTP); ClickHouse
performs the Arrow column coercion server-side. The mappings below were
validated end-to-end against a live ClickHouse by the `e2e_*` integration tests
in `transform/transform_batch.rs` (inserting the realistic fixtures and reading
every column back). Columns bind **by name**, so column order is irrelevant,
missing columns are server-defaulted, and an unknown column name errors on
`end()`.

<!-- markdownlint-disable MD013 -->

| Emitted Arrow type | ClickHouse column type | Example columns |
| --- | --- | --- |
| `Map<Utf8, Utf8>` | `Map(LowCardinality(String), String)` | ResourceAttributes, ScopeAttributes, LogAttributes, SpanAttributes |
| `Dictionary<_, Utf8>` | `LowCardinality(String)` | ServiceName, SpanName, SpanKind, StatusCode |
| `Timestamp(Nanosecond)` | `DateTime64(9)` | Timestamp, Events.Timestamp (as `Array(DateTime64(9))`) |
| `Int*` -> `UInt8` | `UInt8` | SeverityNumber |
| `*` -> `UInt64` | `UInt64` | Duration |
| hex `Utf8` | `String` | TraceId, SpanId, ParentSpanId (top-level) |
| `Utf8` | `String` | Body, EventName, StatusMessage, TraceState |
| `List<Utf8>` | `Array(LowCardinality(String))` / `Array(String)` | Events.Name, Links.TraceState |
| `List<Timestamp(ns)>` | `Array(DateTime64(9))` | Events.Timestamp |
| `List<hex Utf8>` | `Array(String)` | Links.TraceId, Links.SpanId (and event equivalents), hex-encoded like the top-level ids |
| `List<Map<Utf8,Utf8>>` | `Array(Map(LowCardinality(String), String))` | Events.Attributes, Links.Attributes (one map per event/link) |

<!-- markdownlint-enable MD013 -->

No special `input_format_arrow_*` settings were required for a clean insert.

## Attribute Representation

Inline attributes are stored as:

```sql
Map(LowCardinality(String), String)
```

Nested attribute values (Map/Slice) are stored as a JSON string. The raw OTLP
path serializes them directly, while OTAP inputs transcode their CBOR values.

## Transform Pipeline

Serialized OTLP log requests build the final ClickHouse columns directly from
the protobuf byte view, avoiding an intermediate OTAP Arrow batch. Canonical
OTAP log batches use a separate specialized transformation path. Both paths
preserve the generic transformer's ClickHouse values. Raw OTLP transformation
errors use the legacy conversion path, while unsupported OTAP layouts use the
generic transformer. Traces and other supported payloads continue to use the
generic pipeline.

The transform pipeline has two stages per payload:

1. Multi-column stage
2. Single-column stage

Key operations:

- flattening OTAP structs such as `resource`, `scope`, and `status`
- grouping attribute rows by `parent_id` into compact map columns
- grouping span events and links into list columns
- inlining compact child payloads back into parent signal rows
- renaming OTAP columns to ClickHouse column names
- coercing log body values to strings
- extracting `service.name` from inlined resource attributes into `ServiceName`
- casting `duration_time_unix_nano` into `Duration`

The transformer reconstructs batches only for `Logs` and `Spans`. Child
payloads remain internal to the transform process.

## Writer Behavior

`ClickHouseWriter`:

- creates the target database if needed
- initializes configured tables
- writes only signal payloads
- maps `Logs -> logs table` and `Spans -> traces table`
- runs at most `max_in_flight` insert requests concurrently
- drains accepted insert requests until the shutdown deadline

If the shutdown deadline expires, the exporter stops waiting for active
inserts and drops queued inserts that have not started.

There is no longer any special write ordering for attribute tables because
attribute tables do not exist.

## Telemetry

Input PData message volume is reported by the engine through
`channel.receiver.messages` and is not duplicated by the exporter.

<!-- markdownlint-disable MD013 -->

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.exports.messages` | `{message}` | `signal`, `outcome` | Number of PData messages whose ClickHouse export reached a terminal outcome. |
| `exporter.exports.duration` | `s` | `signal`, `outcome` | Time from dequeuing PData through the terminal ClickHouse write result, including conversion, queueing, and transformation. |

<!-- markdownlint-enable MD013 -->

## Snapshots and Tests

DDL snapshot coverage currently lives in `table_snapshots/` and covers:

- `log_table_map_attrs.snap`
- `trace_table_map_attrs.snap`

The recommended validation loop for intentional DDL changes is:

```bash
cargo test -p otap-df-contrib-nodes --features clickhouse-exporter
INSTA_UPDATE=always cargo test -p otap-df-contrib-nodes --features clickhouse-exporter
```

## Important Files

- `mod.rs`: exporter entry point and message loop
- `config.rs`: configuration model and defaults
- `schema.rs`: reusable ClickHouse column and index model
- `tables.rs`: table SQL generation and schema initialization
- `writer.rs`: ClickHouse client bootstrap and inserts
- `transform/transform_plan.rs`: transform plan construction
- `transform/transform_batch.rs`: batch orchestration
- `transform/transform_column.rs`: column-level ops and inlining helpers
- `transform/transform_attributes.rs`: attribute grouping and serialization helpers
- `arrays.rs`: Arrow accessors used throughout the transform code

## Known Gaps

- metrics remain stubbed in DDL generation
- unit testing against realistic otap payloads is currently limited
