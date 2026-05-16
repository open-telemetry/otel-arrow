# ClickHouse Exporter

This exporter accepts OTAP Arrow payloads, reshapes them into ClickHouse-compatible Arrow `RecordBatch`es, and inserts them into ClickHouse using `clickhouse-arrow`.

The current architecture is intentionally simple:

- one flat table for logs
- one flat table for traces
- attributes always inlined on the signal tables
- two attribute representations: `string_map` and `json`
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
- map and JSON attribute representations
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
cargo run -p data-plane --features clickhouse-exporter -- --config configs/fake-clickhouse.yaml
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
- `password`
- `async_insert`
- `attributes`
- `table_defaults`
- `tables`

Attribute config is per group and only controls representation:

- `resource.representation`
- `scope.representation`
- `log.representation`
- `metric.representation`
- `trace.representation`

Supported values:

- `string_map` (default)
- `json`

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

## Attribute Representations

### `string_map`

Inline attributes are stored as:

```sql
Map(LowCardinality(String), String)
```

This is the default mode.

### `json`

Inline attributes are stored as ClickHouse `JSON` columns, with companion `*Keys` columns added to support indexing.

When any attribute group uses `json`, the client enables the ClickHouse settings required for JSON binary input.

## Transform Pipeline

The transform pipeline has two stages per payload:

1. Multi-column stage
2. Single-column stage

Key operations:

- flattening OTAP structs such as `resource`, `scope`, and `status`
- grouping attribute rows by `parent_id` into compact map or JSON columns
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
- `log_table_json_attrs.snap`
- `trace_table_map_attrs.snap`
- `trace_table_json_attrs.snap`

The recommended validation loop for intentional DDL changes is:

```bash
cargo test -p gateway-exporters --features clickhouse-exporter
INSTA_UPDATE=always cargo test -p gateway-exporters --features clickhouse-exporter
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
- end-to-end live ClickHouse validation is still limited compared to unit/snapshot coverage
- unit testing against realistic otap payloads is currently limited
