# Shared Database Scraper

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

<!-- markdownlint-disable MD013 -->

## Metadata

- Crate: `otel-arrow-dfe-scraper`
- Kind: Shared Rust library for database receiver implementations.
- Receiver type: None. This crate does not register a receiver URN.
- Feature gate: No crate-local vendor feature. Database drivers belong to independently gated vendor receivers, not this crate.
- Status: In development; not the complete database receiver RFC or a production-readiness claim.
- Documentation scope: Common database contracts, checkpoint storage, source ownership, and their integration with polling, mapping, and delivery.

## Overview

The shared scraper is the database-neutral foundation for query-polling
receivers in OTAP Dataflow. It defines common configuration, validated query
plans, cursor and row types, and the interface that database-specific adapters
implement. `CheckpointStore` and `SourceLease` provide concrete persistence
and checkpoint-identity ownership. The polling component, `DatabaseReceiver`,
integrates these primitives with OTLP mapping and downstream feedback.
A concrete receiver supplies the adapter and registers the node.

| Component | Responsibility |
| --- | --- |
| Database contracts | Configuration validation, query plans, adapter interfaces, row/cursor/page types, and size-accounting helpers |
| Checkpointing | Durable file storage and exclusive ownership of a checkpoint identity |
| Polling | Scheduling, OTLP mapping, backpressure, ACK/NACK handling, checkpoint integration, lifecycle, and telemetry |
| Vendor receiver | Native driver, connection/authentication settings, SQL validation, type conversion, and node registration |

The goal is to share polling, delivery, checkpointing, and resource-management
behavior without putting every database driver into one receiver. A vendor
receiver owns its connection configuration, native driver, SQL dialect checks,
and component registration. The shared crate must not depend on those vendor
implementations.

The design is scheduled, read-only query polling, not Change Data Capture
(CDC). The [database receiver RFC][database-rfc] remains broader than the initial
single-query, composite-watermark runtime.

## Architecture and Responsibilities

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
this library. Selecting a receiver URN cannot load a driver that was not compiled
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
```

All five fields are required by the shared deserialization type. The byte limit
is a numeric byte count in these common types; a vendor schema may provide
different defaults or convenience units.

## Configuration

The complete native receiver configuration belongs in the vendor README under
`crates/contrib-nodes/src/receivers`: its registered `type`, connection,
authentication, driver setup, SQL rules, and complete pipeline examples.
This shared reference owns the common contracts and semantics, not a runnable
generic `receiver:database` schema.

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
| `max_batch_bytes` | integer bytes | **required** | Between `1` and `268435456` (256 MiB). Applied separately to accounted normalized-row storage and the exact serialized OTLP payload; not a combined memory ceiling. |

The byte limit is a `u64` value in the shared schema: use an integer byte count,
not strings such as `10 MiB`. Duration strings are parsed by
`humantime-serde`. Normalized-row storage and serialized OTLP each use this same
limit, checked separately. Their combined footprint and native-driver buffers
can exceed it; this is not a process-RSS cap.

### Watermark Configuration

Only `mode: composite` is represented by the current enum. `scalar` and
`snapshot` are RFC proposals, not supported modes. The RFC's conceptual
`composite_watermark` spelling is not an accepted value for this schema.

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
`CompositeCursor` deliberately has no `Ord` or `PartialOrd` implementation:
timestamp strings with different offsets or fractional precision cannot be
ordered safely as text. The polling controller compares validated UTC instants.

### Checkpoint Configuration

This block validates a persistence and replay policy. Constructing configuration
alone does not perform I/O. Receiver construction creates a `CheckpointStore`
and acquires a `SourceLease`; the polling controller manages when they are used.

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

### Output Configuration

`OutputConfig` is constructed in Rust; it is not a deserializable native
`output:` block in the shared configuration schema.

| Field | Default | Meaning |
| --- | --- | --- |
| `timestamp_column` | `None` | Optional non-empty result-column name for event-time mapping |
| `validation_columns` | Empty list | Non-empty column names that must exist during live metadata validation |

The OTLP mapper uses a structured key-value body for selected columns.
It does not automatically promote every column to a LogRecord attribute, or
infer OTLP fields from matching column names. Richer mapping is future work.

### SQL Validation

`CompiledQuery::compile` currently performs only these shared SQL checks:

1. The statement is at most 16 KiB, measured in UTF-8 bytes.
2. Its first whitespace-delimited word is `SELECT`, ignoring ASCII case.

These checks do not prove SQL is safe. Before executing SQL or starting polling,
each adapter's `validate_query` must apply its dialect's rules to:

- Require one read-only SELECT; reject extra statements and row-locking forms
  such as `SELECT ... FOR UPDATE`.
- Verify both cursor binds are real parameters used by a complete keyset
  predicate that selects only rows strictly after the supplied cursor.
- Require deterministic ascending timestamp/tie-breaker ordering consistent
  with the predicate and selected cursor columns.
- Require non-null cursor columns with supported UTC timestamp and signed
  `int64` tie-breaker types.

Reject unsupported or ambiguous forms. Bind cursor values as parameters, never
SQL string concatenation. A least-privileged read-only account is required, but
does not replace adapter validation.

### Authentication and TLS

Operators must provision a dedicated, least-privileged account with only the
permissions needed for collection. The receiver does not audit account grants
or roles. The Oracle adapter additionally uses read-only transactions and
validates supported SQL forms.

There are no connection, password, TLS, or secret-provider fields in the shared
configuration. These belong to the concrete receiver and capability integration.
The shared crate must not introduce a dependency on a vendor driver or secret
store to resolve credentials. Driver dependencies must be optional in the
consuming vendor crate and activated by its feature; a workspace dependency
entry only specifies the reusable version.

Oracle's node-local mounted credential files were accepted for the first
iteration in [the authentication discussion][auth-review]. Shared database
authentication extensions remain follow-up work; exporter authentication
support does not by itself implement database login.

## Adapter and Data Contracts

### Driver Lifecycle

| Method / type | Responsibility |
| --- | --- |
| `DriverAdapter::system` | Return a stable database-system identity. `DatabaseSystem` currently contains only `Oracle`; that enum value does not include an Oracle driver. |
| `begin_operation` | Reset operation cancellation state and return a cancellation handle before native work begins. |
| `validate_query` | Validate single-statement/read-only SQL, cursor binds, keyset predicate, ordering, and non-null cursor metadata before polling. |
| `execute` | Execute the validated query with bound cursor parameters and return a bounded `QueryPage` strictly after the committed cursor. |
| `shutdown` | Stop workers and destroy native resources off the pipeline thread. The default is a no-op; an adapter owning such resources must override it. |
| `classify_error` | Translate an adapter error into an engine `ReceiverErrorKind`; the default is `Other`. |
| `DriverCancellation::cancel` | Request interruption of one active operation. The handle is cloneable and its future is local (`?Send`). |

The page's vectors are not intrinsically bounded. The adapter must enforce
fetch, row, and normalized-byte limits while reading/converting, not after
materializing an arbitrary result set.

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

The RFC's additional `Date`, `Json`, and `Uuid` variants are not implemented.
The OTLP mapper consumes owned values, preserving bytes as OTLP
`BytesValue` and decimal precision as text rather than coercing it to floating
point.

`Row::normalized_size` includes structural storage and retained value
capacities, not the entire process working set. OTLP records, serialized output,
metadata, and native fetch buffers may coexist. The adapter's normalized-row
budget and the encoder's serialized-payload budget both use `max_batch_bytes`;
there is no additional receiver setting. Each representation is checked
separately, so their combined footprint can exceed this value. This does not
provide process-wide memory-pressure admission or an RSS ceiling.

`CellValue` and `CompositeCursor` debug output redact their values; nested
cursor rows/pages therefore do not reveal the cursor through their debug
representation. `CompiledQuery` also redacts SQL and its initial cursor.
This is not blanket redaction of every configuration type or error: callers
must not log raw watermark configuration, native driver errors, endpoints,
or other sensitive inputs.

## Polling and Delivery Semantics

This section describes polling-controller integration. The controller permits
one pending page per source and is a reusable receiver core, not a
vendor-registered node by itself. Checkpoint storage and leases alone do not
schedule queries, send data, or process acknowledgements.

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

### Source Correctness and Ownership

At-least-once behavior requires commit-visible cursor ordering, stable cursor
and row values, and source retention longer than the expected outage and replay
window. An increasing timestamp or sequence ID alone is insufficient: an older
transaction can become visible after the checkpoint has advanced. NACK replay
re-executes SQL and cannot reproduce rows that have changed or been deleted.

The lease protects a checkpoint identity, not the underlying database query.
Different pipeline/receiver names or state directories can still cause duplicate
polling. Deployments must enforce one active poller per unpartitioned source
range. Automatic distributed partitioning and source discovery are not provided.

#### Checkpoint Ownership Is Not Database-Source Ownership

`CheckpointStore::lease_key()` derives its key from the state directory,
pipeline group, pipeline, receiver name, and `source_id`. `SourceLease` prevents
competing owners of that same storage identity using a process-local registry
and an advisory filesystem lock. Cross-process exclusion requires access to
the same lock on a filesystem that honors those locking semantics.

For example, two one-core pipelines named `audit-a` and `audit-b` can query
the same database rows with the same `source_id`. Their different pipeline
names give them different checkpoint locations and lease keys, so both can
acquire a lease and emit duplicate data. Separate state directories have the
same limitation. One-core placement prevents per-core duplication within a
pipeline; it does not detect equivalent sources across pipelines or replicas.

Operators must enforce a single active poller for each logical source range,
including during restarts and configuration replacement. Reusing `source_id`
alone does not enforce this rule across different checkpoint identities.
The configuration fingerprint checks whether saved progress is compatible;
it is not a database-source ownership key.

### Shutdown and Live Configuration Changes

The polling controller offloads encoding and checkpoint I/O while handling control
messages. Checkpoint retries honor an already-active drain deadline. Worker-stop
waits respect the earlier supplied deadline and the five-second stop cap.
Unconfirmed cleanup retains ownership until process exit rather than allowing
overlapping source work; uninterruptible native work can require a supervisor
to terminate the process.

Stop the existing pipeline before starting it with changed configuration,
including interval-only changes. A replacement started first can conflict with
the old lease. Coordinated replacement/readiness is separate work tracked in
[the readiness issue][readiness].

## Telemetry

### Metric Sets

`DatabaseReceiverMetrics` defines the polling controller's `receiver.database`
metric set. Concrete receiver construction registers the set and supplies its
handle to the controller. Configuration validation, checkpoint storage, and
leases do not emit these runtime metrics by themselves.

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
- Only composite cursor configuration and `on_nack: rewind` are accepted; the polling controller implements the corresponding delivery behavior.
- File checkpoints and leases are library primitives. Scheduling, mapping, and feedback require polling-controller integration; database I/O and node registration remain vendor responsibilities.
- Multiple named queries, jitter, snapshot/scalar polling, richer output mapping, collection of database metrics as an output signal, and CDC are not implemented. Internal runtime counters are implemented.
- Byte-limit validation does not enforce process-wide memory pressure, native allocations, or end-to-end execution deadlines.
- Authentication capabilities, credential rotation, TLS configuration, distributed ownership, and automatic source partitioning require separate work.
- Whole-poll and normal-operation ACK deadlines and immediate backlog catch-up are not implemented.
- No exactly-once guarantee, live database qualification, or production performance guarantee is provided by the unit tests.

## Related Issue

- [Database receiver RFC and discussion][database-rfc]

[database-rfc]: https://github.com/open-telemetry/otel-arrow/issues/3918
[auth-review]: https://github.com/open-telemetry/otel-arrow/pull/3969#discussion_r4018012217
[readiness]: https://github.com/open-telemetry/otel-arrow/issues/4049
