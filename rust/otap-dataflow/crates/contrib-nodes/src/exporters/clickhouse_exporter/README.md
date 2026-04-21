# Clickhouse Exporter

> This is a POC / Experimental implementation not suitable for integration with the dataflow engine in its current state.
> It's being made available to accelerate an actual implementation, but isn't wired in
> (via feature flags or otherwise) to the engine.

This crate implements a **ClickHouse exporter** for an OpenTelemetry gateway built on **OTAP** (the Arrow-based OpenTelemetry representation).
It receives OTAP pdata messages, transforms OTAP Arrow payloads into ClickHouse-compatible Arrow `RecordBatch`es, and writes them to ClickHouse using an Arrow-capable client.

It is designed to support multiple schema layouts, including:

- **Inline attributes** (denormalized on the signal table), and
- **Lookup/normalized attributes** (stored in dedicated attribute tables with optional deduplication), with **views** to present a query-friendly logical schema.

## Current State

- Logs are mostly implemented and (probably) mostly correct (see testing / todo below)
  - Top level `ServiceName` column still needs to be surfaced from resource attributes.
- Traces are mostly implemented and maybe correct.
  - Top level `ServiceName` column still needs to be surfaced from resource attributes.
  - Either fake data generator creates invalid trace/span IDs or there's something wrong with processing.
  - Haven't been able to test with SpanLinks, Events (ListArray code may or may not work)
  - Duration field proessing is broken, or fake data is broken (see TODO in transform/transform_plan.rs)
  - Supporting alternative storage mechanisms (e.g. lookup table) for spanLink, events is probably a good idea
- Attributes as Map(String,String) (arrow MapArray) may not be working completely.
- Table indexes creation need to be supported, and generally everything needs to properly align with https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/exporter/clickhouseexporter/internal/sqltemplates
  - table_snapshots/ has currently generated schemas for various combinations. table.rs contains the snapshot test code.
- Metrics are not supported yet
- Performance and correctness validation for everything above needs lots more attention.
- Logging, metrics, error handling aren't up to date with current oss practices

---

## High-level architecture

At runtime the exporter does the following:

1. **Start & initialize**
   - Reads config (`ConfigPatch` → `Config`)
   - Connects to ClickHouse
   - Ensures the configured database exists
   - Creates tables and views (if `create_schema` is enabled)

2. **Per message**
   - Converts OTAP pdata payload into `OtapArrowRecords`
   - Applies a **transformation plan** per `ArrowPayloadType` to produce ClickHouse-ready `RecordBatch`es
   - Writes the resulting batches to ClickHouse (attributes first, then signals)
   - Emits internal exporter metrics and pdata metrics

---

## Supported payloads

The exporter currently works on both “table-backed” payloads (payloads that correspond to physical tables or logical views in ClickHouse):

- `ResourceAttrs`
- `ScopeAttrs`
- `Logs`
- `LogAttrs`
- `Spans`
- `SpanAttrs`

And "always inlined" payloads (we should eventually support using lookup tables for these too):

- `SpanLinks`
- `SpanLinkAttrs`
- `SpanEvents`
- `SpanEventAttrs`

You’ll see TODO markers like `[support_new_signal]` where new signal types and schemas are intended to be added.

---

## Schema model

The schema is configurable along two primary axes:

### 1. Attribute storage

- **Inline**: attributes stored directly on the signal tables (logs/spans)
- **Lookup** / **LookupDeduped**: attributes stored in separate tables; signal tables store IDs

### 2. Attribute representation

- **StringMap**: `Map(LowCardinality(String), String)`
- **Json**: ClickHouse `JSON` type (with required client settings enabled)
- **OtapArray**: attribute rows stored in an OTAP/Arrow-native “typed rows” form; logical views can aggregate them into a map-like column for querying

When lookup tables are used, the exporter can create:

- a physical “raw” table (typically `{logical_name}_raw`) where inserts go, and
- a logical view (named `logical_name`) that joins signal rows to resource/scope/signal attribute tables.

---

## Configuration

Configuration is deserialized via `serde` into a patch model and then normalized into a runtime config:

- `ConfigPatch`: user-provided config (JSON/YAML), with defaults
- `Config`: final resolved config used throughout the exporter

Key configurable areas:

- ClickHouse endpoint and credentials
- database name
- async insert setting
- per-attribute-group storage/representation (resource/scope/log/metric/trace)
- global table defaults (engine, TTL interval, create_schema)
- per-table overrides (name, physical_name, TTL, engine override, create_schema)

Validation is performed during deserialization for invalid combinations, e.g.:

- `AttributeRepresentation::OtapArray` **cannot** be used with `AttributeStorage::Inline`.

---

## Writing flow

Writes are performed by `ClickHouseWriter`:

- Connects initially to the `default` database (so the target database can be created if missing)
- Initializes database/tables/views when constructed
- Writes batches in two phases:
  1. **attribute payloads** first
  2. **signal payloads** second

This ordering ensures normalized schemas work correctly (signals may reference attribute IDs that must already exist).

Each write produces `WriteStat` entries (payload type + rows written) which feed internal exporter telemetry.

---

## Transformation pipeline

Transforming OTAP Arrow batches into ClickHouse-ready batches is handled by `BatchTransformer` using per-payload `TransformationPlan`s.

Conceptually, transformations are executed in two stages:

1. **Multi-column stage**
   - Operations that need to inspect or restructure multiple columns at once
   - Includes attribute grouping/deduplication and building remap tables for IDs

2. **Single-column stage**
   - Per-column operations such as rename/cast/drop, flattening struct fields, coercions, synthetic columns, ID reindexing, and inlining child payload data into parent payloads

Cross-payload coordination is supported via `MultiColumnOpResult` remap maps:

- attribute-table transformations can produce `old_id -> new_id`
- signal-table transformations can then rewrite foreign keys accordingly

---

## ClickHouse initialization

Schema and view creation is driven by configuration and implemented by `tables.rs`:

- `CHTableBuilder` renders `CREATE TABLE` SQL with engine params, partitioning, ordering, primary keys, and TTL
- Attribute tables may be created in one of several shapes:
  - flattened map/json table
  - OTAP array attribute row table
- Views are created when needed:
  - attribute aggregation views for OTAP array attribute rows
  - signal views joining to resource/scope/signal attribute tables when lookup storage is enabled

---

## Table DDL snapshotting (`table_snapshots/`)

This exporter generates ClickHouse DDL (tables + optional views) dynamically from configuration.
To keep those schemas stable and make changes reviewable, we use **snapshot tests** (via `insta`)
to capture the exact SQL emitted by `tables.rs` into `table_snapshots/`.

What gets snapshotted:

- **Attribute tables** across a matrix of configurations:
  - per attribute group: `resource`, `scope`, `log`, `trace`
  - storage/representation variants:
    - `lookup_json` (lookup storage, JSON representation)
    - `lookup_otap` (lookup storage, OTAP array representation)
    - `inline` (inline storage; expected to **not** create an attribute table)

- **Signal tables** (`logs`, `traces`) under two “end-to-end” layouts:
  - `all_lookup`: resource/scope/signal attributes all stored in lookup tables
    - expects: a physical raw table plus a logical view that joins attributes
  - `all_inline`: all attributes inlined on the signal table
    - expects: table only, and usually no view

The snapshot tests assert both:

- the `CREATE TABLE` SQL (when a table should exist), and
- the `CREATE VIEW` SQL (when a view should exist, e.g. OTAP-array attribute views or signal views in lookup mode).

### Updating snapshots

When you intentionally change schema generation (columns, engines, ORDER BY/PARTITION BY, view joins, etc.), regenerate snapshots with:

```bash
cargo test -p <crate-name> -F clickhouse-exporter
INSTA_UPDATE=always cargo test -p <crate-name> -F clickhouse-exporter
```

(Use the INSTA_UPDATE=always run to accept the new SQL output; review the diff in table_snapshots/ as part of the change.)

This workflow makes schema changes explicit in code review and prevents accidental drift in emitted DDL.

---

## Directory structure

```tree
.
├── arrays.rs
├── config.rs
├── consts.rs
├── error.rs
├── idgen.rs
├── metrics.rs
├── mod.rs
├── README.md
├── schema.rs
├── table_snapshots
│   ├── log_attributes_lookup_json.snap
│   └── ...
├── tables.rs
├── transform
│   ├── mod.rs
│   ├── transform_attributes.rs
│   ├── transform_batch.rs
│   ├── transform_column.rs
│   └── transform_plan.rs
└── writer.rs
```

### What lives where

- `mod.rs`
  - Exporter entrypoint and OTAP exporter factory registration (`CLICKHOUSE_EXPORTER_URN`)
  - Main async loop: receive pdata, transform, write, report metrics, handle shutdown

- `config.rs`
  - Configuration types and defaulting/merging logic
  - Attribute and table configuration, engine/TTL defaults, validation rules

- `schema.rs`
  - ClickHouse column/type model and reusable column constants for signal and attribute tables

- `tables.rs`
  - DDL generation (tables and views)
  - “logical vs physical” routing for normalized schemas
  - payload → destination table mapping utilities

- `writer.rs`
  - ClickHouse client setup (including JSON settings)
  - Database initialization orchestration (tables then views)
  - Insert logic for Arrow `RecordBatch`es and write ordering

- `transform/`
  - `transform_plan.rs`: describes transformation operations and per-payload plans
  - `transform_batch.rs`: runs plans for a whole payload/batch; manages multi vs single-column stages
  - `transform_column.rs`: applies per-column operations and implements inlining/coercion logic
  - `transform_attributes.rs`: attribute grouping/serialization helpers (map/json)

- `idgen.rs`
  - Partition-scoped ID generation for payloads that need stable IDs to join between tables/views

- `arrays.rs`
  - Arrow accessors/helpers for safe typed access (nullable handling, struct access patterns)

- `metrics.rs`
  - Internal exporter telemetry: per-payload rows written, pdata consumed/exported/failed, etc.

- `error.rs`
  - Exporter error types used across init/transform/write phases

- `table_snapshots/`
  - Snapshot tests / expected DDL outputs for specific schema configurations

---

## Operational notes

- If any table uses ClickHouse `JSON` columns, the client enables:
  - `input_format_binary_read_json_as_string=1`
  - `allow_experimental_json_type=1`

- `async_insert` can be enabled/disabled via config and is applied as a client setting.

- Current error handling on write failures is intentionally conservative (fail-fast) with TODOs for
  retryable vs non-retryable errors and integration with retry processors.

---

## Extending the exporter

Most feature additions follow a predictable path:

1. Add new `ArrowPayloadType` handling:
   - Update the supported/table-backed payload lists
   - Add schema definitions in `schema.rs`
   - Add DDL/view rules in `tables.rs`
   - Add transform plan rules in `transform_plan.rs` (+ implementation helpers as needed)

2. Add a new signal type (metrics, etc.):
   - Implement table + view DDL generation
   - Define how attributes are stored/represented for that signal
   - Implement transformation plans and ensure correct write ordering/dependencies

Search for `[support_new_signal]` TODO markers for known extension points.
