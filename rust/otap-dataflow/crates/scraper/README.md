# Shared Database Scraper

<!-- markdownlint-disable MD013 -->

## Metadata

- Crate: `otel-arrow-dfe-scraper`
- Kind: Shared Rust library for database receiver implementations.
- Receiver type: None. This crate does not register a receiver URN.
- Feature gate: No crate-local vendor feature. Database drivers belong to independently gated vendor receivers, not this crate.
- Status: In development. This change provides the shared crate and database-neutral contracts; the polling runtime and concrete receivers are separate changes.
- Intended signal: Logs for the initial query-polling use case. Database metrics and CDC are separate design work.

## Overview

The shared scraper is the database-neutral foundation for query-polling
receivers in OTAP Dataflow. It defines common configuration, validated query
plans, cursor and row types, and the interface that database-specific adapters
implement.

The goal is to share polling, delivery, checkpointing, and resource-management
behavior without putting every database driver into one receiver. A vendor
receiver owns its connection configuration, native driver, SQL dialect checks,
and component registration. The shared crate must not depend on those vendor
implementations.

This is scheduled, read-only query polling, not Change Data Capture (CDC).
Transaction-log ingestion requires database-specific source and recovery
semantics and is outside this crate's initial scope.

### Current Implementation and Planned Runtime

The [database receiver RFC][database-rfc] describes a broader design than this
initial change. Configuration types are not evidence that their corresponding
runtime behavior is already implemented.

| Area | Available in this change | Separate or future work |
| --- | --- | --- |
| Crate structure | Shared crate, workspace dependency, publication-policy registration | Vendor receiver registration and executable composition |
| Configuration | Polling limits, composite watermark, checkpoint policy, and output-column validation | Vendor connection, credential, and TLS schemas |
| Query planning | `CompiledQuery` and basic shared SQL checks | Vendor SQL validation, live preparation, and execution |
| Adapter boundary | `DriverAdapter` and `DriverCancellation` | Concrete database drivers and their cancellation/cleanup implementations |
| Rows and pages | `CellValue`, metadata, normalized size accounting, and cursor-bearing pages | Native value conversion and OTLP encoding |
| Checkpointing | Configuration and source-position types | Durable storage and source ownership |
| Polling and delivery | Contracts consumed by the future controller | Scheduling, downstream sends, backpressure, ACK/NACK handling, retries, and shutdown |
| Observability | No scraper metrics or events emitted here | Shared runtime telemetry |

### Architecture and Responsibilities

The intended composition is:

```text
Existing Dataflow host
  |
  +-- Vendor receiver in contrib-nodes
  |     +-- Vendor configuration and node registration
  |     +-- Database driver / native client
  |     +-- DriverAdapter implementation
  |     `-- Shared scraper
  |           +-- Database-neutral contracts        [this change]
  |           +-- Checkpoint storage and ownership  [separate change]
  |           `-- Polling, mapping, and delivery    [separate change]
  |
  `-- Existing processors and exporters
```

| Responsibility | Shared scraper | Vendor receiver / host |
| --- | --- | --- |
| Query policy | Common limits and validated plan | Operator-authored SQL and dialect-specific validation |
| Cursor parameters | Logical bind names and a composite cursor | Binding through the driver's parameter API |
| Row representation | Database-neutral values, metadata, and page contract | Native type inspection and precision-preserving conversion |
| Timing and lifecycle | Common policy; controller implementation follows separately | Native timeout, cancellation, and connection cleanup |
| Progress | Common checkpoint policy; durable implementation follows separately | Stable source identity and vendor-specific compatibility inputs |
| Authentication and TLS | No credential storage or connection implementation | Vendor/capability integration and driver configuration |
| Deployment | No installer, image, or Helm resources | Host executable and deployment tooling |

Dependencies point from vendor receiver code toward the scraper, never from
the scraper toward a vendor driver. The shared crate may reuse engine, OTAP,
pdata, and telemetry APIs as the runtime is introduced; it must not duplicate
the controller or exporters.

The initial adapter contract uses `#[async_trait(?Send)]` to preserve the
engine's local, thread-per-core execution model. A blocking driver must arrange
its own bounded off-core execution rather than run native calls on the local
pipeline thread.

## Getting Started

This is a library, not a receiver that can be started with a `type:` block.
There is no `receiver:scraper` or generic `receiver:database` registration in
this change. Selecting a receiver URN cannot load a driver that was not compiled
into the host.

From the repository root, the crate can be built and its contract tests run
without installing a database or native client:

```powershell
cd rust\otap-dataflow
cargo check -p otel-arrow-dfe-scraper
cargo test -p otel-arrow-dfe-scraper
```

A receiver implementation uses the library by constructing and validating
`PollingConfig`, `WatermarkConfig`, `CheckpointConfig`, and `OutputConfig`,
then passing them with its SQL to `CompiledQuery::compile`. Its adapter
implements `DriverAdapter` to execute that plan.

The minimal shared polling block is:

```yaml
# A PollingConfig value, not a complete receiver configuration.
interval: 5m
timeout: 2m
max_rows_per_poll: 10000
fetch_size: 1000
max_batch_bytes: 10485760
max_normalized_bytes: 5242880
```

All six fields are required by the shared deserialization type. A vendor
configuration may supply its own defaults or convenience syntax, but those
are not defaults of this library.

## Configuration

### Shared Contract Blocks

There is no single top-level database receiver configuration in this crate.
Concrete receivers own the outer schema and translate it into these types.

| Input | Rust type | How it is supplied |
| --- | --- | --- |
| SQL statement | `String` | Separate argument to `CompiledQuery::compile`; not a field of `PollingConfig` |
| Poll timing and bounds | `PollingConfig` | Deserializable common configuration |
| Source position | `WatermarkConfig` | Deserializable, tagged by `mode` |
| Persistence/replay policy | `CheckpointConfig` | Deserializable common configuration; does not create a store |
| Output-column requirements | `OutputConfig` | Constructed by Rust code; no `Deserialize` implementation here |

The deserializable configuration structs reject unknown fields. Successful
deserialization alone does not run all semantic checks; call `validate()` or
use `CompiledQuery::compile`, which validates all four configuration inputs.

### Polling Configuration

| Field | Type | Default | Validation / meaning |
| --- | --- | --- | --- |
| `interval` | duration string | **required** | Between `1ms` and `24h`, inclusive. Describes the polling interval for the future controller. |
| `timeout` | duration string | **required** | Must be greater than zero. The contract exposes a native-call timeout, not a guaranteed whole-poll deadline. |
| `max_rows_per_poll` | integer | **required** | Between `1` and `10000`. Hard row ceiling to be enforced during execution. |
| `fetch_size` | integer | **required** | Between `1` and `10000`, and no larger than `max_rows_per_poll`. Target native fetch size. |
| `max_batch_bytes` | integer bytes | **required** | Between `1` and `268435456` (256 MiB). Intended exact serialized OTLP ceiling. |
| `max_normalized_bytes` | integer bytes | **required** | Between `1` and `268435456` (256 MiB). Independent retained normalized-row storage ceiling. |

The byte fields are `u64` values in the shared schema: use integer byte counts,
not strings such as `10 MiB`. Duration strings are parsed by
`humantime-serde`. The two byte ceilings may differ; neither is a process-RSS
limit, and validating them does not itself limit driver allocations.

### Watermark Configuration

Only `mode: composite` is represented by the current enum. `scalar`,
`snapshot`, and the RFC's conceptual `composite_watermark` spelling are not
accepted values for this schema.

```yaml
# A WatermarkConfig value.
mode: composite
timestamp:
  column: EVENT_TS
  bind: last_timestamp
  initial: "1970-01-01 00:00:00"
  timezone: UTC
tie_breaker:
  column: EVENT_ID
  bind: last_tie_breaker
  initial: 0
```

| Field | Type | Default | Validation / meaning |
| --- | --- | --- | --- |
| `mode` | string | **required** | Only `composite`. |
| `timestamp.column` | string | **required** | Non-empty result-column name. Must differ from the tie-breaker column, ignoring ASCII case. |
| `timestamp.bind` | string | **required** | Logical bind name without a leading colon. Must differ from the tie-breaker bind, ignoring ASCII case. |
| `timestamp.initial` | string | **required** | Non-empty initial timestamp text. Parsing and normalization are adapter responsibilities. |
| `timestamp.timezone` | string | **required** | Only `UTC`, ignoring ASCII case. |
| `tie_breaker.column` | string | **required** | Non-empty result-column name for a non-null signed 64-bit tie-breaker. Live type/nullability checks belong to the adapter. |
| `tie_breaker.bind` | string | **required** | Logical bind name without a leading colon. |
| `tie_breaker.initial` | signed 64-bit integer | **required** | Initial tie-breaker value; zero is valid. |

Bind names start with an ASCII letter or `_`; subsequent characters are ASCII
letters, digits, or `_`. Column names are only checked for emptiness and
distinctness here. Each adapter must apply its identifier and SQL rules.

A composite cursor orders rows by timestamp and then by a tie-breaker unique
within that timestamp group. The adapter must return a consistent timestamp
representation and deterministic ordering; retaining timestamp text avoids
forcing a lossy conversion at the shared boundary.

### Checkpoint Configuration

This block validates a persistence and replay policy. It does not perform
filesystem I/O or acquire source ownership in the current change.

| Field | Type | Default | Validation / meaning |
| --- | --- | --- | --- |
| `directory` | string | **required** | Non-empty path without `..` path components, as interpreted by the host platform. |
| `on_nack` | string | **required** | Only `rewind`; other policies are rejected by deserialization. |
| `nack_backoff` | duration string | **required** | Between `1ms` and `5m`, inclusive. Fixed delay before replay. |
| `max_consecutive_failures` | integer | **required** | Between `1` and `1000`. Intended consecutive checkpoint-write failure limit, not a limit on all query or NACK retries. |

```yaml
# A CheckpointConfig value.
directory: ./state/database
on_nack: rewind
nack_backoff: 1s
max_consecutive_failures: 5
```

Durable storage and ownership come before the polling controller in the
implementation sequence. They are deliberately not abstracted behind an extra
storage/ownership trait just to support that sequence.

### Output Configuration

`OutputConfig` is an internal Rust contract, not a currently supported YAML
`output:` block.

| Field | Rust type | Default | Meaning |
| --- | --- | --- | --- |
| `timestamp_column` | `Option<String>` | `None` | Optional event-time column for the future mapper. A supplied name must be non-empty. |
| `validation_columns` | `Vec<String>` | Empty | Additional columns that must be present during future live metadata validation. Names must be non-empty. |

The intended initial mapping is one row per log record with all selected
columns in a structured body. Rich body/attribute selection, renaming, and
metric mapping require additional implementation. Do not assume that result
columns named after OTLP fields are automatically promoted to those fields.

### SQL Validation

`CompiledQuery::compile` currently performs only these shared SQL checks:

1. The statement is at most 16 KiB, measured in UTF-8 bytes.
2. Its first whitespace-delimited word is `SELECT`, ignoring ASCII case.
3. It does not contain `FOR UPDATE`, ignoring ASCII case.

This is an early filter, **not a SQL parser or a proof of read-only execution**.
It does not verify bind occurrences, the keyset predicate, result ordering,
column aliases, or statement count. Vendor validation and a least-privileged
read-only database account are still required.

The library does not rewrite arbitrary SQL, invent identifiers, or concatenate
cursor values into a query. The adapter contract requires cursor values to be
bound as database parameters.

### Authentication and TLS

There are no connection, password, TLS, or secret-provider fields in the shared
configuration. These belong to the concrete receiver and capability integration.
The shared crate must not introduce a dependency on a vendor driver or secret
store to resolve credentials.

Use dedicated read-only credentials and verified transport according to the
concrete receiver's supported configuration. Credential rotation and shared
database authentication capabilities are not implemented by this scaffold.

## Adapter and Data Contracts

### Driver Lifecycle

| Method / type | Responsibility |
| --- | --- |
| `DriverAdapter::system` | Return a stable database-system identity. `DatabaseSystem` currently contains only `Oracle`; that enum value does not include an Oracle driver. |
| `begin_operation` | Reset operation cancellation state and return a cancellation handle before native work begins. |
| `validate_query` | Inspect live metadata and reject cursor types/nullability that cannot produce a deterministic composite cursor. |
| `execute` | Execute strictly after the supplied committed cursor and return a bounded `QueryPage`. |
| `shutdown` | Stop workers and destroy native resources off the pipeline thread. The default is a no-op; an adapter owning such resources must override it. |
| `classify_error` | Translate an adapter error into an engine `ReceiverErrorKind`; the default is `Other`. |
| `DriverCancellation::cancel` | Request interruption of one active operation. The handle is cloneable and its future is local (`?Send`). |

`execute` returns a page, not an unbounded row stream. A `Vec` is not inherently
bounded: implementations must account for rows and bytes while fetching and
converting, before constructing an arbitrarily large page.

### Values, Metadata, and Pages

| Type | Purpose |
| --- | --- |
| `CellValue` | Closed representation of supported database scalar values. |
| `ColumnMetadata` | Column name, adapter-reported type name, and nullability, shared by the page. |
| `Row` | Ordered values corresponding to the result metadata. |
| `CompositeCursor` | Timestamp text plus a signed 64-bit tie-breaker. |
| `CursorRow` | A row paired with its own source position. |
| `QueryPage` | Shared column metadata and an ordered vector of cursor-bearing rows. |

Current `CellValue` variants are:

| Variants | Representation / adapter obligation |
| --- | --- |
| `Null`, `Bool` | Preserve SQL null and Boolean values distinctly. |
| `Int64`, `UInt64` | Preserve signed and unsigned 64-bit values without narrowing. |
| `Decimal` | Preserve exact decimal text rather than rounding through floating point. |
| `Float64` | Intended for finite floating-point values; the enum itself does not reject NaN or infinity. |
| `String`, `Bytes` | Owned UTF-8 text and binary bytes. |
| `Timestamp`, `TimestampTz`, `Interval` | Adapter-normalized text that preserves source precision and temporal meaning. |

The RFC's additional `Date`, `Json`, and `Uuid` variants are not present here.
This change also contains no OTLP encoder, so these representations do not yet
establish an emitted OTLP wire format.

`Row::normalized_size` includes row/value storage and retained string/vector
capacities, using saturating arithmetic. This supports conservative accounting;
it does not include every native-client, metadata, cursor, page-vector,
serialization, or allocator overhead. The adapter and future controller must
account for their own additional allocations.

### Sensitive Data

`CellValue` debug output redacts scalar contents, and `CompiledQuery` debug
output redacts its SQL field. This is not blanket redaction of all shared types:
cursor and watermark types still contain sensitive timestamp/position text.
Do not log whole query/configuration/cursor structures, raw driver errors,
connection strings, or rows. Concrete adapters and runtime diagnostics must
enforce their own safe error reporting.

## Polling and Delivery Semantics

**The following is the runtime design target, not executable behavior provided
by this scaffold.**

```text
Acquire source ownership and load committed position
  -> Execute a bounded query page through DriverAdapter
  -> Validate metadata and map rows to bounded log batches
  -> Send downstream, honoring backpressure
  -> Receive matching ACK/NACK
  -> ACK: durably commit only acknowledged progress
  -> NACK: retain committed progress and replay after backoff
```

| Condition | Required runtime behavior |
| --- | --- |
| Downstream backpressure | Stop admitting more work rather than accumulating unbounded pages. |
| Matching ACK | Advance progress only after the corresponding checkpoint is durably installed. |
| NACK, failed delivery, or uncertain outcome | Do not skip unacknowledged source positions. Apply an explicit replay or failure policy. |
| Crash after destination acceptance but before checkpoint commit | Allow replay; do not claim exactly-once delivery. |
| Invalid or incompatible checkpoint | Fail explicitly rather than silently assume a fresh position. |
| Shutdown/cancellation | Stop admitting work and coordinate native cleanup before permitting a competing source owner. |

The initial runtime is intended to keep one page pending per source. Multiple
in-flight batches would additionally require a contiguous acknowledgement
frontier; a later ACK must never skip an earlier unresolved batch.

### Source Correctness and Ownership

At-least-once delivery requires more than an increasing ID or timestamp. Rows
must become visible in an order compatible with the cursor, cursor values must
remain stable, and source data must survive long enough for outage/retry replay.
An earlier transaction may commit after a newer cursor is saved, even when IDs
come from a sequence. The receiver cannot recover that row without an explicit
late-arrival strategy.

A NACK-driven query replay also cannot reproduce a row that has since changed
or been deleted. Sources without suitable ordering, immutability, and retention
need an explicitly designed overlap/deduplication policy or a CDC receiver.

The initial unpartitioned design requires one active poller per query/source
range. One process does not necessarily mean one poller under per-core pipeline
placement. Concrete receivers must enforce appropriate placement/ownership.
Neither this scaffold nor its `source_id` concept automatically partitions data
or coordinates replicas. A checkpoint revision check alone does not prevent
duplicate database work.

## Examples

### Composite Keyset Query

This illustrates the logical cursor relationship; it is not a complete runnable
receiver configuration. Bind syntax and additional validation are vendor-owned.

```sql
SELECT EVENT_ID, EVENT_TS, PAYLOAD
FROM EVENTS
WHERE EVENT_TS > :last_timestamp
   OR (EVENT_TS = :last_timestamp AND EVENT_ID > :last_tie_breaker)
ORDER BY EVENT_TS ASC, EVENT_ID ASC
```

For a committed cursor `(2026-01-01 12:00:00, 42)`, the next page must contain
rows strictly after that tuple. Returning an empty page means there is no data
to emit for that execution; it must not fabricate checkpoint advancement.

### Constructing Output Requirements

Adapter code can select an event-time column and require cursor columns during
live validation:

```rust
use otel_arrow_dfe_scraper::database::OutputConfig;

let output = OutputConfig {
    timestamp_column: Some("EVENT_TS".to_owned()),
    validation_columns: vec!["EVENT_ID".to_owned(), "EVENT_TS".to_owned()],
};
output.validate().expect("static column names are valid");
```

This validates names only. It does not connect to a database, confirm those
columns exist, or emit logs.

## Telemetry

### Metric Sets

This change registers **no scraper metric set**. The RFC calls for shared,
low-cardinality runtime telemetry in a later implementation.

| Planned signal group | Purpose |
| --- | --- |
| Poll outcomes | Identify started, completed, and failed query work. |
| Row and byte accounting | Observe admitted volume and bounded resource usage. |
| Delivery feedback | Track pending work, ACKs, NACKs, and replay. |
| Checkpoint outcomes | Distinguish committed progress from persistence failures. |
| Timing and health | Observe query duration, backpressure, and time since successful work. |

These are design categories, not metric names available for dashboards today.
SQL, endpoints, table names, row values, cursor values, and raw error messages
must not become metric dimensions.

### Events

No receiver lifecycle events are emitted by this scaffold. Shared validation
returns `ConfigError` or `QueryError`; the host/receiver decides how to report
them. Driver errors must be classified and sanitized before entering engine
diagnostics. Metrics and events must not be copied from Kafka merely because
the receiver documentation follows the same layout.

## Limits

- This is not a runnable generic receiver, SQL Agent binary, installer, or exporter.
- Only composite cursor configuration and `on_nack: rewind` are accepted; their runtime implementation is outside this change.
- `DatabaseSystem::Oracle` is the only identity currently represented. PostgreSQL, SQL Server, and MySQL are RFC targets, not working adapters here.
- No scheduler, database connection, checkpoint store, source lease, OTLP mapper, or ACK/NACK controller is included.
- Multiple named queries, jitter, snapshot/scalar polling, richer output mapping, metrics collection, and CDC are not implemented.
- Byte-limit validation does not enforce process-wide memory pressure, native allocations, or end-to-end execution deadlines.
- Shared SQL checks do not establish safe dialect semantics or database permissions.
- Authentication capabilities, credential rotation, TLS configuration, distributed ownership, and automatic source partitioning require separate work.
- No exactly-once guarantee, live database qualification, or production performance guarantee is provided by the contract tests.

## Related Docs

- [Database receiver RFC and discussion][database-rfc]
- [Crate-boundary and single-owner discussion][crate-boundary]
- [Commit-order and immutable-cursor discussion][cursor-correctness]
- [Shared configuration types](src/database/config.rs)
- [Driver adapter contract](src/database/driver.rs)
- [Compiled query plan](src/database/query.rs)
- [Row and value model](src/database/row.rs)
- [Cursor and page model](src/database/page.rs)
- [Contract tests](src/database/tests.rs)
- [Pipeline configuration and core allocation](../../docs/configuration.md)
- [OTAP Dataflow contribution guidelines](../../CONTRIBUTING.md)

[database-rfc]: https://github.com/open-telemetry/otel-arrow/issues/3918
[crate-boundary]: https://github.com/open-telemetry/otel-arrow/issues/3918#issuecomment-5486569534
[cursor-correctness]: https://github.com/open-telemetry/otel-arrow/issues/3918#issuecomment-5471215404
