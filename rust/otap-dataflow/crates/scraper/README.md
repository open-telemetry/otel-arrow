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
- Documentation scope: Common database contracts, checkpoint storage, and source ownership (this PR); planned polling, mapping, and delivery integration (part 3).

## Overview

The shared scraper is the database-neutral foundation for query-polling
receivers in OTAP Dataflow. It defines common configuration, validated query
plans, cursor and row types, and the interface that database-specific adapters
implement. `CheckpointStore` and `SourceLease` provide concrete persistence
and checkpoint-identity ownership. Part 3 will add the polling component
`DatabaseReceiver` to integrate these primitives with OTLP mapping and
downstream feedback. A later concrete receiver will supply the adapter and
register the node. This PR does not schedule polls or emit records.

| Component | Responsibility |
| --- | --- |
| Database contracts | Configuration validation, query plans, adapter interfaces, row/cursor/page types, and size-accounting helpers |
| Checkpointing | Atomic file storage with platform-dependent durability and exclusive ownership of a checkpoint identity |
| Polling (part 3) | Scheduling, OTLP mapping, backpressure, ACK/NACK handling, checkpoint integration, lifecycle, and telemetry |
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
  |           `-- Polling, mapping, and delivery (planned for part 3)
  |
  `-- Existing processors and exporters
```

| Responsibility | Shared scraper | Vendor receiver / host |
| --- | --- | --- |
| Query policy | Common limits and validated plan | Operator-authored SQL and dialect-specific validation |
| Cursor parameters | Logical bind names and a composite cursor | Binding through the driver's parameter API |
| Row representation | Database-neutral values, metadata, and page contract | Native type inspection and precision-preserving conversion |
| Timing and lifecycle | Common scheduling, control-message handling, and worker cleanup coordination | Native timeout, cancellation, and connection cleanup |
| Progress | Common checkpoint policy, concrete file store, and source lease | Stable source identity and vendor-specific compatibility inputs |
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

All five fields above are required by the shared deserialization type. Omitting
`catch_up` uses bounded multi-page cycles: up to 32 page fetches or 10 seconds
of elapsed admission time. The byte limit
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
| `interval` | duration string | **required** | Between `1ms` and `24h`, inclusive. Delay after a cycle ends, not between its pages; unresolved downstream feedback blocks the next page. |
| `timeout` | duration string | **required** | Must be greater than zero. The contract exposes a native-call timeout, not a guaranteed whole-poll deadline. |
| `max_rows_per_poll` | integer | **required** | Between `1` and `10000`. Hard row ceiling the adapter must enforce while building its returned page. |
| `fetch_size` | integer | **required** | Between `1` and `10000`, and no larger than `max_rows_per_poll`. Target native fetch size. |
| `max_batch_bytes` | integer bytes | **required** | Between `1` and `268435456` (256 MiB). Applied separately to accounted normalized-row storage and the exact serialized OTLP payload; not a combined memory ceiling. |
| `catch_up` | object | Default budgets below | Optional budget overrides for normal bounded paging. Omitted fields use defaults; null and unknown fields are rejected. |
| `catch_up.max_pages` | integer | `32` | Between `1` and `1024`, inclusive. Maximum page fetches per cycle, including empty probes. Set to `1` for single-page cycles. |
| `catch_up.max_duration` | duration string | `10s` | Between `1ms` and `5min`, inclusive. Elapsed cycle budget for admitting the next fetch; not a whole-poll hard deadline. |

For example, this explicitly supplies the default cycle budgets without
introducing another receiver URN or changing the per-page limits:

```yaml
# A shared polling block, not a complete receiver configuration.
interval: 5m
timeout: 2m
max_rows_per_poll: 10000
fetch_size: 1000
max_batch_bytes: 10485760
catch_up:
  max_pages: 32
  max_duration: 10s
```

The 32-page and 10-second defaults are implementation choices, not values
prescribed by the RFC or universal production sizing recommendations. Tune them
for database capacity and downstream latency. A partial override such as
`catch_up: { max_pages: 1 }` keeps the default duration but fetches at most one
page per cycle; it does not restore an interval timer measured from page send.
`max_rows_per_poll` and `max_batch_bytes` continue to bound each fetched/emitted
page, not the whole cycle. The cycle's maximum row work is
`catch_up.max_pages * max_rows_per_poll`; its aggregate normalized-row and
serialized-payload budgets are each at most
`catch_up.max_pages * max_batch_bytes`, separately. These are work bounds, not
an RSS ceiling or a promise to retain all those pages concurrently.

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
ordered safely as text. Part 3's polling controller will compare validated UTC
instants.

### Checkpoint Configuration

This block validates a persistence and replay policy. Constructing configuration
alone does not perform I/O. Future receiver construction will create a
`CheckpointStore` and acquire a `SourceLease`; part 3's polling controller
will manage when they are used.

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

The planned OTLP mapper will use a structured key-value body for selected columns.
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
separately, so their combined footprint can exceed this value. Byte accounting
does not provide an RSS ceiling. Separately, the controller gates new fetches
on the engine's local memory-pressure admission state, as described below;
it does not implement full global `MemoryAdmission` accounting.

The polling controller keeps one OTLP encoder per receiver. It caches validated
column metadata, the event-time column index, and constant resource/scope
metadata and wire sizes. Each page's full metadata is compared with the cached
schema; changed names, order, types, or nullability trigger revalidation before
encoding, including for empty result sets. The cache stores only the current schema, not query results or a
history of schemas.

The encoder moves owned cell strings and bytes into protobuf values, then hands
the serialized OTLP buffer to downstream ownership. It clears all row payloads
after every call, including failures, and reuses only the empty record vector
up to a 4 MiB capacity bound. This retention bound is not a page or process
memory limit. Protobuf still requires owned per-record keys and metadata; this
is not zero-copy encoding. The exact serialized-byte ceiling and the cursor of
the last emitted row are unchanged.

An optional cache-reuse microbenchmark compares a warmed encoder with the
optimized one-shot API. Run it in isolation from the other tests:

```powershell
cargo test -p otel-arrow-dfe-scraper compare_warmed_encoder_with_optimized_one_shot -- --ignored --nocapture --test-threads=1
```

It checks equivalent output and reports timings without a speed assertion.
Debug-build timings are not production throughput or a comparison with an
earlier revision.

For large-page memory qualification, run the ignored profile alone in a fresh
test process:

```powershell
cargo test -p otel-arrow-dfe-scraper profile_large_owned_page_memory -- --ignored --nocapture --test-threads=1
```

It constructs 10,000 owned rows with a 24 KiB string each, under a 256 MiB
`max_batch_bytes` limit, and reports normalized-row and serialized-payload sizes.
The phase markers allow an external process-memory sampler to capture the
baseline, completed input, and retained output. Measure the test process, not
Cargo or a concurrently running test suite.

Peak resident memory is a measured property, not the sum of the reported
logical byte counters: row storage, protobuf metadata, the output buffer,
allocator overhead, and the runtime can coexist. This synthetic profile does
not include an Oracle client, native fetch/prefetch buffers, transport
compression, gRPC/TLS buffers, or other pipeline nodes. Deployment sizing must
include those separately; the profile is not a container-memory guarantee.

One encoder-only Windows x64 debug measurement (Rust 1.98.1, three fresh
processes, OS peak working set sampled every 10 ms) produced:

| Quantity | Result |
| --- | --- |
| Rows and string size | 10,000 rows, 24 KiB per row |
| Accounted normalized-row storage | 234.91 MiB |
| Serialized OTLP payload | 236.07 MiB |
| Peak process working set | 482.43-482.44 MiB |

Both logical representations fit the 256 MiB setting, while peak resident
memory was much larger. The measured peak includes the test process and
encoding allocations, not native database or downstream buffers. It is a
workload-specific sizing example, not a portable peak-memory bound.

`CellValue` and `CompositeCursor` debug output redact their values; nested
cursor rows/pages therefore do not reveal the cursor through their debug
representation. `CompiledQuery` also redacts SQL and its initial cursor.
Timestamp and tie-breaker configuration debug output redacts `initial`,
including when nested inside `WatermarkConfig`; the actual configured values
remain available for query binding.
`EncodedPage` also redacts its serialized payload and cursor in direct, pretty,
and nested `Debug` output; only counts and sizes remain visible. The payload
itself still contains the original customer data for delivery. Formatting
extracted raw payloads directly bypasses this wrapper's protection.
This is not blanket redaction of every configuration type or error: callers
must not log raw watermark configuration, native driver errors, endpoints,
or other sensitive inputs.

## Polling and Delivery Semantics

This section describes the planned part 3 polling-controller integration,
not behavior shipped in this PR. The controller will permit one pending page
per source and be a reusable receiver core, not a vendor-registered node by
itself. Checkpoint storage and leases alone do not schedule queries, send
data, or process acknowledgements.

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
| Matching ACK | Advance progress only after checkpoint installation succeeds, subject to the filesystem guarantees below. |
| NACK, failed delivery, or uncertain outcome | Do not skip unacknowledged source positions. Apply an explicit replay or failure policy. |
| Crash after destination acceptance but before checkpoint commit | Allow replay; do not claim exactly-once delivery. |
| Invalid or incompatible checkpoint | Fail explicitly rather than silently assume a fresh position. |
| Shutdown/cancellation | Stop admitting work and coordinate native cleanup before permitting a competing source owner. |

The planned initial runtime will keep one page pending per source. Multiple
in-flight batches would additionally require a contiguous acknowledgement
frontier; a later ACK must never skip an earlier unresolved batch.

### Bounded Catch-Up and Admission

Bounded catch-up is the normal polling behavior. A successful page may be followed
immediately by the next page only after its matching ACK and durable checkpoint commit. Each fetch
still uses the existing per-page row and byte bounds. The elapsed cycle budget
includes fetch, encoding, downstream ACK, and checkpoint waits. It only gates
admission of the next fetch: already-admitted work may finish, receive its ACK,
and commit after the budget expires. It is not a whole-poll hard deadline or a
normal-operation ACK timeout.

An empty query result ends the cycle. A short nonempty page is not proof of
end-of-data: normalized or serialized byte limits can truncate it before the
row limit. Another fetch, including an extra empty probe, is allowed while
both budgets permit it. When a cycle ends, the normal `interval` starts then.
There is no separate opt-in or legacy scheduling mode; unresolved feedback still
blocks another fetch.

A NACK ends the burst and uses the existing `nack_backoff` without advancing
the committed cursor. Native query errors still fail the receiver rather
than gaining a new retry policy. A checkpoint write failure ends immediate
catch-up, but the existing checkpoint retry policy must first finish committing
the ACKed page (or reach its terminal failure limit).

Downstream backpressure and Hard memory pressure in Enforce mode end the burst
and pause new-fetch admission. Already-admitted work may still finish, ACK, and
commit; pressure is not a request to discard acknowledged progress. Observe-only
memory pressure does not block admission. Control messages remain handled
during fetch, encoding, send, feedback, and checkpoint phases.

Receiver factories must bootstrap admission from the process state, not assume
startup is unpressured:

```rust,ignore
let admission =
    LocalReceiverAdmissionState::from_process_state(&pipeline_ctx.memory_pressure_state());
// Supply admission immediately before metrics in DatabaseReceiver::new(...).
```

`DatabaseReceiver::new` requires this `LocalReceiverAdmissionState` argument
before its metrics argument. This honors an initial Hard state and the configured
Enforce versus observe-only mode. Vendor factory wiring is a downstream
integration responsibility; this library does not register a new receiver.

### Filesystem Guarantees

Checkpoint writes fsync the temporary file, atomically rename it into place,
then fsync that file's parent directory on Unix (the same pattern as the
journald receiver). mkdir of a new state tree is not fsynced. A power loss
before those ancestor directory entries are durable can look like a first
start and replay from the initial cursor. On Windows there is no portable
directory-fsync step: atomic visibility after a process crash does not
guarantee that a rename survives a machine crash or power loss. Source
retention must cover that recovery window; power-loss behavior has not been
experimentally qualified.

Checkpoint filenames live beneath `directory/@v1/`, with one component for
each of the pipeline group, pipeline, receiver, and source IDs. IDs up to 64
UTF-8 bytes use `id-` followed by the lowercase hex encoding of their exact
bytes; longer IDs use `hash-` followed by their BLAKE3 digest. Source components
end in `.checkpoint`. This keeps `Orders` and `orders` separate even on
case-insensitive filesystems. The checkpoint payload still verifies the exact
source ID and configuration fingerprint.

If no versioned checkpoint exists, startup also reads the earlier unversioned
names, including both the readable and digest-based names for long source IDs.
If only a case-different old name or directory exists, recovery rejects the
ambiguous path rather than treating it as missing or adopting another source.
The next successful write uses the versioned path; older files are not removed
by that write. Stop old writers before upgrading: they do not share this
namespace or its lease and must not write concurrently with the new version.

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

`CheckpointStore::lease_key()` returns the native filesystem path derived from
the state directory, pipeline group, pipeline, receiver name, and `source_id`.
Pass it directly to `SourceLease::acquire()` without converting it to a string:
lossy text conversion can move the lock away from the checkpoint if the state
directory contains non-UTF-8 components. `SourceLease` prevents competing
owners of that same storage identity using a process-local registry and an
advisory filesystem lock. Cross-process exclusion requires access to the same
lock on a filesystem that honors those locking semantics.

On-disk lock, generation, and temporary-file names use the checkpoint filename,
not its absolute mount path. Processes mounting the same backing directory at
different paths therefore share the same storage lock and recovery namespace.
The in-process registry still uses the canonical full path.

This pre-release lock namespace differs from older path-derived builds. Stop all
old writers before upgrading; mixed old/new writers do not coordinate, and
generation continuity across those layouts is not guaranteed. Checkpoint
revision filenames now use the versioned layout; their JSON payload format is
unchanged.

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

Each receiver owns one dedicated thread for encoding and checkpoint I/O, with
a capacity-one request queue and nonblocking submission. This keeps large-page
encoding and filesystem calls off the pipeline's async thread without creating
per-page threads or using Tokio's runtime-owned blocking pool.

The controller continues processing control messages while work runs.
Checkpoint retries inherit an already-active drain deadline rather than waiting
for a new stop message. Active-operation cancellation and checkpoint stop waits
use the earlier supplied deadline and a five-second cap. Final adapter and
scraper-worker cleanup are attempted concurrently within the remaining stop
budget; ordinary error exits receive a five-second cleanup budget. A drain may
still wait for downstream feedback until its supplied deadline; this is not a
universal five-second bound on the entire drain.

Worker completion requires an explicit exit acknowledgement, not merely a
dropped result handle. If both adapter and scraper cleanup subsequently succeed,
an earlier abandonment flag is cleared: the original error is preserved and the
lease can be reacquired without a process restart.
If native work or a scraper job cannot be confirmed
stopped, the receiver reports an error and retains its source lease until
process exit. The dedicated scraper thread does not make Tokio runtime
destruction wait for a stalled filesystem/encoding job. The thread is not
forcibly killed: a supervisor must terminate/restart the process to clear an
unconfirmed worker and its quarantined ownership. Starting a replacement
receiver in the same process must not bypass that quarantine.

Adapter cancellation must stop the whole operation. In particular, native fetch
and conversion loops must check cancellation between calls rather than assume
interrupting one call stops later calls. Potentially uninterruptible native
operations, including the cancellation operation itself, must use adapter-owned
workers rather than the pipeline runtime's blocking pool. Timing out a future
does not cancel a running native call.

Stop the existing pipeline before starting it with changed configuration,
including interval-only changes. A replacement started first can conflict with
the old lease. Coordinated replacement/readiness is separate work tracked in
[the readiness issue][readiness].

## Telemetry

### Metric Sets

Part 3 will define `DatabaseReceiverMetrics`, the polling controller's
`receiver.database` metric set. Concrete receiver construction will register
the set and supply its handle to the controller. Configuration validation,
checkpoint storage, and leases do not emit these runtime metrics by themselves.

| Counter fields | Purpose |
| --- | --- |
| `starts`, `polls`, `query_failures` | Receiver starts, attempted page polls, and failed query executions. |
| `batches_sent`, `rows_sent`, `encoded_bytes_sent` | Admitted pages, records, and encoded bytes. |
| `event_time_fallbacks` | Records whose event time cannot fit the OTLP timestamp range. |
| `acks`, `nacks`, `replays`, `stale_feedback` | Matched downstream outcomes, replay, and rejected stale feedback. |
| `checkpoint_commits`, `checkpoint_failures`, `checkpoint_cleanup_failures` | Durable progress and persistence/cleanup failures. |
| `cancellations`, `drains`, `shutdowns` | Cancellation attempts and received drain/shutdown requests, including during checkpoint writes and retries; not counts of successful cleanup. |

Measurement attributes are intentionally omitted to keep cardinality bounded.
The RFC's duration histograms, lag gauges, and broader health signals are not
implemented. SQL, endpoints, table names, row values, cursor values, and raw
error messages must not become metric dimensions.

## Limits

- This is not a runnable generic receiver, SQL Agent binary, installer, or exporter.
- Only composite cursor configuration and `on_nack: rewind` are accepted.
- File checkpoints, leases, scheduling, mapping, and feedback are shared library functionality; database I/O and node registration remain vendor responsibilities.
- Multiple named queries, jitter, snapshot/scalar polling, richer output mapping, collection of database metrics as an output signal, and CDC are not implemented. Internal runtime counters are implemented.
- Byte-limit validation does not bound RSS or native allocations. Local memory-pressure state gates new fetches, but full global `MemoryAdmission` accounting is not implemented.
- Authentication capabilities, credential rotation, TLS configuration, distributed ownership, and automatic source partitioning require separate work.
- Bounded immediate catch-up is the default, with configurable cycle budgets and memory-pressure new-fetch gating. Whole-poll and normal-operation ACK deadlines are not; elapsed catch-up budgets only gate the next fetch.
- No exactly-once guarantee, live database qualification, or production performance guarantee is provided by the unit tests.

## Related Issue

- [Database receiver RFC and discussion][database-rfc]

[database-rfc]: https://github.com/open-telemetry/otel-arrow/issues/3918
[auth-review]: https://github.com/open-telemetry/otel-arrow/pull/3969#discussion_r4018012217
[readiness]: https://github.com/open-telemetry/otel-arrow/issues/4049
