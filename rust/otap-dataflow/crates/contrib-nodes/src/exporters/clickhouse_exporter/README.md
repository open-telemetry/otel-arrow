# ClickHouse Exporter

This exporter accepts OTAP Arrow payloads, reshapes them into
ClickHouse-compatible Arrow `RecordBatch`es, and inserts them into ClickHouse
over HTTP using the official ClickHouse Rust client (`clickhouse` +
`clickhouse-ext-arrow`, `FORMAT ArrowStream`).

The current architecture is intentionally simple:

- one flat table for logs
- one flat table for traces
- attributes always inlined on the signal tables as `Map(LowCardinality(String), String)`
- no lookup attribute tables
- no views
- no ID generation layer

The schema and transform behavior are aligned with the Go OpenTelemetry Collector ClickHouse exporter where practical.

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
docker run -it -p 8123:8123 -p 9000:9000 -e CLICKHOUSE_PASSWORD=TODO-TEST \
  --name clickhouse-server --ulimit nofile=262144:262144 \
  clickhouse/clickhouse-server
```

### 2. Run the data plane with the ClickHouse exporter

```bash
cargo run --features clickhouse-exporter -- --config configs/fake-clickhouse.yaml
```

### 3. Query ClickHouse

```bash
docker exec -it clickhouse-server clickhouse-client --password TODO-TEST
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
2. Connects to ClickHouse and creates the target database and configured tables if enabled
3. Receives `OtapPdata` messages from the engine
4. Converts payloads into `OtapArrowRecords`
5. Runs the transform pipeline across supported payload types
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

Only `Logs` and `Spans` are written to ClickHouse tables. The attribute and child payloads are consumed during transformation.

## Configuration Model

Top-level config fields:

- `endpoint`
- `database`
- `username`
- `password` (supports `${env:VAR}` / `${env:VAR:-default}` substitution, e.g. `"${env:CLICKHOUSE_PASSWORD}"`)
- `async_insert`
- `table_defaults`
- `tables`

Inline attributes are always stored as `Map(LowCardinality(String), String)`; there is no
per-group representation configuration.

Table config supports:

- table name overrides
- per-table TTL
- engine override
- `create_schema`

## Schema Shape

The current implementation follows the clickhouse-exporter in opentelemetry-collector-contrib
defined [here](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/clickhouseexporter/internal/sqltemplates).

Snapshots of the current structure are generated in the [table_snapshots](./table_snapshots/) directory.
There is currently no automated testing to ensure schema drift relative to the go collector.

## Verified Arrow → ClickHouse Type Mapping

Inserts go out as `FORMAT ArrowStream` (Arrow IPC over HTTP); ClickHouse performs the
Arrow column coercion server-side. The mappings below were validated end-to-end against a live
ClickHouse by the `e2e_*` integration tests in `transform/transform_batch.rs` (inserting the
realistic fixtures and reading every column back). Columns bind **by name**, so column order is
irrelevant, missing columns are server-defaulted, and an unknown column name errors on `end()`.

| Emitted Arrow type | ClickHouse column type | Example columns |
|---|---|---|
| `Map<Utf8, Utf8>` | `Map(LowCardinality(String), String)` | ResourceAttributes, ScopeAttributes, LogAttributes, SpanAttributes |
| `Dictionary<_, Utf8>` | `LowCardinality(String)` | ServiceName, SpanName, SpanKind, StatusCode |
| `Timestamp(Nanosecond)` | `DateTime64(9)` | Timestamp, Events.Timestamp (as `Array(DateTime64(9))`) |
| `Int*` → `UInt8` | `UInt8` | SeverityNumber |
| `*` → `UInt64` | `UInt64` | Duration |
| hex `Utf8` | `String` | TraceId, SpanId, ParentSpanId (top-level) |
| `Utf8` | `String` | Body, EventName, StatusMessage, TraceState |
| `List<Utf8>` | `Array(LowCardinality(String))` / `Array(String)` | Events.Name, Links.TraceState |
| `List<Timestamp(ns)>` | `Array(DateTime64(9))` | Events.Timestamp |
| `List<hex Utf8>` | `Array(String)` | Links.TraceId, Links.SpanId (and event equivalents), hex-encoded like the top-level ids |
| `List<Map<Utf8,Utf8>>` | `Array(Map(LowCardinality(String), String))` | Events.Attributes, Links.Attributes (one map per event/link) |

No special `input_format_arrow_*` settings were required for a clean insert.

## Attribute Representation

Inline attributes are stored as:

```sql
Map(LowCardinality(String), String)
```

Nested attribute values (Map/Slice) are transcoded from binary/CBOR into a JSON string stored as the map value.

## Transform Pipeline

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

The transformer reconstructs batches only for `Logs` and `Spans`. Child payloads remain internal to the transform process.

## Writer Behavior

`ClickHouseWriter`:

- creates the target database if needed
- initializes configured tables
- writes only signal payloads
- maps `Logs -> logs table` and `Spans -> traces table`

There is no longer any special write ordering for attribute tables because attribute tables do not exist.

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
