# Shared Database Scraper

<!-- markdownlint-disable MD013 -->

## Metadata

- Crate: `otel-arrow-dfe-scraper`
- Kind: Shared Rust library for database receiver implementations.
- Receiver type: None. This crate does not register a receiver URN.
- Feature gate: No crate-local vendor feature. Database drivers belong to independently gated vendor receivers, not this crate.
- Status: In development. The shared runtime implements single-query composite polling, mapping, checkpoints, and ownership; it is not the complete database receiver RFC.
 

## Overview

The shared scraper is the database-neutral foundation for query-polling
receivers in OTAP Dataflow. It defines common configuration, validated query
plans, cursor and row types, and the interface that database-specific adapters
implement. `DatabaseReceiver` combines these with a concrete checkpoint store,
filesystem-backed source lease, bounded OTLP mapping, and downstream feedback.
The concrete receiver must still supply an adapter and register the node.

The goal is to share polling, delivery, checkpointing, and resource-management
behavior without putting every database driver into one receiver. A vendor
receiver owns its connection configuration, native driver, SQL dialect checks,
and component registration. The shared crate must not depend on those vendor
implementations.

This is scheduled, read-only query polling, not Change Data Capture (CDC). The [database receiver RFC][database-rfc] remains
broader than this initial single-query, composite-watermark runtime.


### I## Architecture and Responsibilities

The composition is:

```text
Existing Dataflow host
  |
  +-- Vendor receiver in contrib-nodes
  |     +-- Vendor configuration and node registration
  |     +-- Database driver / native client
  |     +-- DriverAdapter implementation
  |     `-- Shared scraper
  |           +-- Database-neutral contracts
  |           +-- Checkpoint storage and ownership
  |           `-- Polling, mapping, and delivery
  |
  `-- Existing processors and exporters
```

| Responsibility | Shared scraper | Vendor receiver / host |
| --- | --- | --- |
| Query policy | Common limits and validated plan | Operator-authored SQL and dialect-specific validation |
| Cursor parameters | Logical bind names and a composite cursor | Binding through the driver's parameter API |
| Row representation | Database-neutral values, metadata, and page contract | Native type inspection and precision-preserving conversion |
| Timing and lifecycle | Common scheduling, control-message handling, and worker cleanup coordination | Native timeout, cancellation, and connection cleanup |
| Progress | Common checkpoint policy, concrete durable file store, and source lease | Stable source identity and vendor-specific compatibility inputs |
| Authentication and TLS | No credential storage or connection implementation | Vendor/capability integration and driver configuration |
| Deployment | No installer, image, or Helm resources | Host executable and deployment tooling |

Dependencies point from vendor receiver code toward the scraper, never from
the scraper toward a vendor driver. The shared runtime reuses engine, OTAP,
pdata, and telemetry APIs; it does not replace the Dataflow host/controller or
implement another exporter.

The initial adapter contract uses `#[async_trait(?Send)]` to preserve the
engine's local, thread-per-core execution model. A blocking driver must arrange
its own bounded off-core execution rather than run native calls on the local
pipeline thread.

## Getting Started

This is a library, not a receiver that can be started with a `type:` block.
There is no `receiver:scraper` or generic `receiver:database` registration in
this change. Selecting a receiver URN cannot load a driver that was not compiled
into the host.

From the repository root, the crate can be built and its tests run
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

All six fields are required by the shared deserialization type. 

###
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
| `interval` | duration string | **required** | Between `1ms` and `24h`, inclusive. Configured interval between eligible polls; unresolved downstream feedback blocks the next page. |
| `timeout` | duration string | **required** | Must be greater than zero. The contract exposes a native-call timeout, not a guaranteed whole-poll deadline. |
| `max_rows_per_poll` | integer | **required** | Between `1` and `10000`. Hard row ceiling the adapter must enforce while building its returned page. |
| `fetch_size` | integer | **required** | Between `1` and `10000`, and no larger than `max_rows_per_poll`. Target native fetch size. |
| `max_batch_bytes` | integer bytes | **required** | Between `1` and `268435456` (256 MiB). Exact serialized OTLP ceiling. |
| `max_normalized_bytes` | integer bytes | **required** | Between `1` and `268435456` (256 MiB). Independent retained normalized-row storage ceiling. |

### Watermark Configuration

Only `mode: composite` is represented by the current enum. `scalar`,
`snapshot` mode support are planned work..

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


A composite cursor orders rows by timestamp and then by a tie-breaker unique
within that timestamp group. The adapter must return a consistent timestamp
representation and deterministic ordering.
### Checkpoint Configuration

This block validates a persistence and replay policy. Constructing configuration
alone does not perform I/O; the receiver must construct `CheckpointStore` and
acquire `SourceLease` separately.

| Field | Type | Default | Validation / meaning |
| --- | --- | --- | --- |
| `directory` | string | **required** | Non-empty path without `..` path components, as interpreted by the host platform. |
| `on_nack` | string | **required** | Only `rewind`; other policies are rejected by deserialization. |
| `nack_backoff` | duration string | **required** | Between `1ms` and `5m`, inclusive. Fixed delay before replay. |
| `max_consecutive_failures` | integer | **required** | Between `1` and `1000`. Consecutive checkpoint-write failure limit, not a limit on all query or NACK retries. |

```yaml
# A CheckpointConfig value.
directory: ./state/database
on_nack: rewind
nack_backoff: 1s
max_consecutive_failures: 5
```


###
##

### SQL Validation

`CompiledQuery::compile` currently performs only these shared SQL checks:

1. The statement is at most 16 KiB, measured in UTF-8 bytes.
2. Its first whitespace-delimited word is `SELECT`, ignoring ASCII case.
3. It does not contain `FOR UPDATE`, ignoring ASCII case.

This is an early filter.
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

The RFC's additional `Date`, `Json`, and `Uuid` variants are planned work.
The mapper consumes owned values, preserving bytes as OTLP `BytesValue` and
decimal precision as text rather than coercing it to floating point.



## Polling and Delivery Semantics

The initial controller implements this flow for one pending page per source.
It is a reusable receiver core, not a vendor-registered node by itself.

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

The initial runtime keeps one page pending per source. Multiple
in-flight batches would additionally require a contiguous acknowledgement
frontier; a later ACK must never skip an earlier unresolved batch.


## Telemetry

### Metric Sets

`DatabaseReceiverMetrics` defines the `receiver.database` metric set. Concrete
receiver construction registers the set and supplies its handle to the shared
controller.

| Counter fields | Purpose |
| --- | --- |
| `starts`, `polls`, `query_failures` | Receiver starts, attempted page polls, and failed query executions. |
| `batches_sent`, `rows_sent`, `encoded_bytes_sent` | Admitted pages, records, and encoded bytes. |
| `event_time_fallbacks` | Records whose event time cannot fit the OTLP timestamp range. |
| `acks`, `nacks`, `replays`, `stale_feedback` | Matched downstream outcomes, replay, and rejected stale feedback. |
| `checkpoint_commits`, `checkpoint_failures`, `checkpoint_cleanup_failures` | Durable progress and persistence/cleanup failures. |
| `cancellations`, `drains`, `shutdowns` | Receiver lifecycle operations. |

Measurement attributes are intentionally omitted to keep cardinality bounded.
The RFC's duration histograms, lag gauges, and broader health signals are not
implemented. SQL, endpoints, table names, row values, cursor values, and raw
error messages must not become metric dimensions.



## Limits

- This is not a runnable generic receiver, SQL Agent binary, installer, or exporter.
- Only composite cursors and `on_nack: rewind` are supported; other modes/policies require separate implementation.

- Scheduling, mapping, feedback, checkpoints, and leases are shared library behavior; no database connection implementation or vendor node registration is included here.
- Multiple named queries, jitter, snapshot/scalar polling, richer output mapping, collection of database metrics as an output signal, and CDC are not implemented. Internal runtime counters are implemented.
- Byte-limit validation does not enforce process-wide memory pressure, native allocations, or end-to-end execution deadlines.

- Authentication capabilities, credential rotation, TLS configuration, distributed ownership, and automatic source partitioning require separate work.
- Whole-poll and normal-operation ACK deadlines and immediate backlog catch-up are not implemented.
- No exactly-once guarantee, live database qualification, or production performance guarantee is provided by the unit tests.

## Related Issue

- [Database receiver RFC and discussion][database-rfc]

