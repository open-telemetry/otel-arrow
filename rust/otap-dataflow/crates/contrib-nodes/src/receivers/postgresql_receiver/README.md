# PostgreSQL Receiver

<!-- markdownlint-disable MD013 -->

- Type: `urn:otel:receiver:postgresql`
- Feature: `postgresql` (explicit opt-in, excluded from `contrib-receivers`)
- Status: Experimental, development-only
- Output: One OTLP log per selected row, with columns in the structured body

**Database transport is unencrypted TCP without server-identity verification.
Use only isolated, trusted development environments and disposable credentials.**
TLS settings are rejected. The engine's `crypto-ring` feature can secure
exporters, but does not encrypt the PostgreSQL connection.

## Getting started

Use the [console example](../../../../../configs/postgresql-console.yaml).
Provision a source table and a least-privilege account with SELECT access, then
adjust the connection, query, and initial cursor. Output metadata is discovered
from PostgreSQL; customer-authored result schemas are not supported. Set `PG_HOST`,
`PG_USERNAME_FILE`, and `PG_PASSWORD_FILE`; the latter two identify readable
credential files. Provide persistent, writable checkpoint storage.

From `rust/otap-dataflow`:

```sh
cargo run --no-default-features --features postgresql,crypto-ring --bin df_engine -- \
  --config configs/postgresql-console.yaml
```

The example needs no OTLP endpoint and prints selected values locally. Use
appropriate development data; console output is not durable downstream storage.

## Configuration

One receiver owns one query/collecting connection and requires a one-core
pipeline. Unknown fields and null configuration values are rejected.

| Field | Contract |
| --- | --- |
| `source_id` | Required stable name, 1-128 UTF-8 bytes without control characters. |
| `connection.host` | Required single DNS name or IP address; no URI or host list. |
| `connection.port` | Defaults to `5432`; must be nonzero. |
| `connection.database` | Required name, 1-63 UTF-8 bytes. |
| `query.statement` | Required read-only SELECT, at most 16 KiB. |
| `query.interval` | Defaults to `1m`; accepts `1m` through `24h` in whole seconds. `1.5m` is valid. |
| `query.max_rows_per_poll` | Required per-page ceiling, `1` through `10000`; no default. Not a whole catch-up-cycle limit. |
| `query.max_batch_bytes` | Required separate normalized-page and encoded-OTLP ceilings, `1` byte through `256 MiB`; no default. Accepts integer bytes or byte-unit strings such as `10 MiB`. |
| `query.timeout` | Defaults to `30s`; accepts `1s` through `5m` in whole seconds. Bounds credential resolution and worker operations; also sets native statement/idle-transaction timeouts. |
| `query.fetch_size_rows` | Defaults to `300`; accepts `1` through `10000`, no greater than `max_rows_per_poll`. Byte estimates may reduce a fetch further. |
| `query.catch_up.max_pages` | Defaults to `32`; accepts `1` through `1024`, including empty probes. |
| `query.catch_up.max_duration` | Defaults to `10s`; accepts `1ms` through `5min`. Gates the next page, not an active page. |
| `query.output.timestamp_column` | Optional `timestamp`/`timestamptz` event-time column; omission uses observation time. |
| `watermark` | Required `composite`, typed `scalar`, or `snapshot` mode; see below. |
| `checkpoint.directory` | Required persistent storage directory. |

Checkpoint defaults are `on_nack: rewind`, `on_permanent_nack: pause`,
`nack_backoff: 1s`, and `max_consecutive_failures: 5`. The last setting counts
checkpoint-write failures, not connection retries. Interval, timeout, page-byte,
page-row, fetch-size, and catch-up changes preserve checkpoint compatibility;
connection, query, watermark, and output changes invalidate it.

### Metadata discovery and checkpoint compatibility

Configuration validation derives output names and cursor positions from explicit
SQL projections, including aliases and reordered columns, without contacting the
database or guessing types. At startup, bounded catalog queries discover native
type OIDs, type modifiers, and nullability for the referenced columns. No table
data scan is used for discovery. Unsupported types, nullable/invalid cursor
columns, invalid event-time types, and unrepresentable initial cursor values fail
before the user query is prepared.

Discovery uses the existing per-relation catalog bounds: at most two relation
matches, requested attributes plus one, and 129 index descriptions (more than
128 indexes fails). It shares the operation timeout and receive limits, not a
separate unbounded metadata path. Prepared output names, table/column origins,
native type OIDs, and modifiers must also agree with the proven catalog metadata.

Each page, including after reconnect, reads fresh catalog metadata before using
cached decoders. A changed signature fails closed during that receiver's lifetime.
This baseline is in memory, not persisted in checkpoints: discovery does **not**
detect schema drift across process restarts. Keep the source schema, grants,
column semantics, and unique cursor ordering stable across restarts, just as
source identity and retained rows must remain stable.

Checkpoint identity hashes the connection, authored SQL, watermark, and output
contract, without guessed or discovered types. Incompatible checkpoints fail;
they are never silently reset.

### Watermark modes

| Mode / scalar type | Native source contract |
| --- | --- |
| `composite` | Non-null `timestamp`/`timestamptz` plus `int2`/`int4`/`int8`; unique pair in ascending order. Existing bind defaults remain `last_timestamp` and `last_tie_breaker`. |
| `scalar` / `int64` | Non-null `int2`, `int4`, or `int8`, with checked native range. |
| `scalar` / `uint64` | Non-null `numeric(p,0)`, declared precision `1..20`; each value must fit both the declaration and `u64`. Native binary numeric binding preserves values above `i64::MAX`. |
| `scalar` / `string` | Non-null `text`/`varchar`, at most 1024 UTF-8 bytes, no NUL; UTF-8 database with catalog-proven deterministic `pg_catalog.C` or `pg_catalog.POSIX` column collation. |
| `scalar` / `timestamp` | Non-null `timestamp`/`timestamptz`; UTC `T`/`Z` text exactly representable at native precision, at most six fractional digits. |
| `snapshot` | No cursor, initial value, bind, ordering, or uniqueness requirement. Each interval reads the complete bounded result again. |

All scalar keys require a catalog-proven single-column unique index on the
driving table and ascending `key > :bind` ordering. Supported inner joins may
not multiply a scalar/composite position; each joined side must cover a unique
key. Composite pair uniqueness is the operator's contract, additionally guarded
by strict runtime ordering. No UUID, decimal, inferred type, cast, `bpchar`,
locale, unknown/default, or nondeterministic collation is accepted as a scalar
string key. Body columns retain their normal mappings.
String-key connections must report native `server_encoding: UTF8` after every
connect; catalog discovery also verifies UTF-8 database encoding. `C.UTF8` is
not inferred to be equivalent to the explicitly proven C/POSIX collations.

Replace the example's query statement and watermark together for scalar mode:

```yaml
query:
  statement: >
    SELECT e.event_id, e.actor FROM public.receiver_events AS e
    WHERE e.event_id > :last_value ORDER BY e.event_id ASC
  max_rows_per_poll: 10000
  max_batch_bytes: 10 MiB
watermark:
  mode: scalar
  column: event_id
  bind: last_value
  initial: {type: int64, value: 0}
```

Select the declared native key and one explicit initial type:
`{type: uint64, value: 9223372036854775808}`, `{type: string, value: ""}`,
or `{type: timestamp, value: "2026-01-01T00:00:00.000000Z"}`.
No scalar type is guessed from the YAML value.

For repeated complete snapshots, omit cursor predicates and ordering:

```yaml
query:
  statement: SELECT e.event_id, e.actor FROM public.receiver_events AS e
  max_rows_per_poll: 10000
  max_batch_bytes: 10 MiB
watermark:
  mode: snapshot
```

Snapshot mode is not CDC or incremental collection: unchanged rows are emitted
again each interval and deletions do not produce tombstones. Nullable/non-unique
projections and row-multiplying supported inner joins are allowed. An internal
`max_rows_per_poll + 1` probe detects oversized results. A row or normalized/
encoded-byte overflow pauses the source without emitting a partial prefix.
Increase the bound or narrow the query before restarting. Catalog/control
receive-limit errors and unconfirmed cleanup remain terminal, not successful
snapshots.

### Authentication

Bind `basic_auth_provider` to the existing
[flat-file provider](../../../../contrib-extensions/src/flat_file_user_pass_auth/README.md),
as shown in the console example. The root `postgresql` feature includes it.
The receiver consumes credentials through the capability, not a custom file
reader or HTTP Basic headers; PostgreSQL authentication uses its native protocol.

The provider refreshes configured files at `password_secret_file_refresh` and
can retain an old password until refresh. Healthy sessions are not replaced;
new connections use the current snapshot. A reconnect before refresh can fail
authentication. Changing the username during a receiver's lifetime is rejected:
stop, review source identity/grants, and restart. After restart, the principal
must still read the same schema-qualified source.

### SQL and cursor contract

Use schema-qualified persistent ordinary tables with aliases and explicit,
qualified column projections. Up to three inner joins are supported when their
equality predicates cover a proven unique key on each joined side for incremental modes. Cursor
columns must be direct, non-null columns of the driving table. The timestamp
and tie-breaker pair must uniquely identify each result position.

For composite mode, use either keyset below, optionally ANDed with supported static comparisons,
and order by timestamp then tie-breaker ascending:

```sql
(e.ts > :last_timestamp OR
 (e.ts = :last_timestamp AND e.id > :last_tie_breaker))

(e.ts, e.id) > (:last_timestamp, :last_tie_breaker)
```

Native `$1`/`$2` binds are also accepted; do not mix styles. After validating
the original query, execution uses the equivalent tuple keyset and internal
`LIMIT` set to `query.max_rows_per_poll`. Values remain bound parameters, and the original SQL defines
checkpoint compatibility. Ask the database owner to provision an index starting
with the timestamp and tie-breaker; the receiver requires no write privileges.

Timestamp cursors use `timestamp` or `timestamptz`, `timezone: UTC`, and an ISO
8601 initial value with `T` and `Z`, exactly representable at the column's
precision, without losing microseconds, with at most six fractional digits.
UTC syntax and PostgreSQL's general microsecond limit are checked locally; narrower native precision is
checked after discovery. Numeric offsets and space-separated values are rejected. Tie-breakers
use `int2`, `int4`, or `int8`, with checked ranges.

Views, RLS, partitioned/foreign/temporary tables, system schemas, functions,
computed projections/casts, stars, outer joins, subqueries, grouping,
unrestricted OR, user-supplied LIMIT/OFFSET/FETCH, and multiple statements are
unsupported. Referenced columns, including filter/join columns, are validated.
Catalog checks run every page; cached statements/decoders do not bypass them.
Metadata changes within a receiver lifetime fail closed.

## Value mapping

These are OTLP body mappings, not a Log Analytics custom-table mapping.

| Native type | Output |
| --- | --- |
| `bool`; `int2`, `int4`, `int8` | Boolean; signed 64-bit integer. |
| `numeric` | Exact decimal string, preserving scale. |
| `float4`, `float8` | Finite double; NaN/infinity are rejected. |
| `text`, `varchar`, `bpchar` | Valid UTF-8 string, preserving padding. |
| `bytea` | Bytes. |
| `timestamp`, `timestamptz` | ISO 8601 UTC text: `YYYY-MM-DDTHH:MM:SS.ffffffZ`. Timestamp without timezone is interpreted as UTC. |
| `date`; `uuid` | `YYYY-MM-DD`; hyphenated UUID string. |
| `json`, `jsonb` | Validated JSON text, preserving numeric precision rather than expanding into OTLP fields. |
| `interval` | `months=N;days=N;microseconds=N`. |
| Nullable `NULL` | Empty OTLP value, retaining the column name. |

Unsupported types, invalid values/UTF-8, precision loss, and oversized records
fail explicitly; rows are not silently skipped or truncated. No automatic
unsupported-type-to-string/dynamic fallback is provided. A NULL or out-of-range
optional event timestamp uses observation time while retaining its original
body value. Watermark timestamps remain strictly validated.

## Resource and delivery boundaries

Page rows and normalized/encoded bytes are required; the configurable operation
timeout defaults to `30s`. The example chooses `10000` rows and `10 MiB`; these are not
defaults. Fixed limits remain a 10-second connection budget (also bounded by the
operation timeout), 128 output columns, and 16,384 characters of numeric output.

Fetches probe one row, then adapt to the configured count and largest observed
row size. Abrupt size increases can require discarding/re-reading part of one
fetch; hard row/page bounds are independent of this estimate. Page allocation
grows with admitted rows and accounts for cursor storage, metadata, and spare
capacity, rather than reserving the full row ceiling for tiny byte budgets.

Wire size is not normalized size. DataRow frames are bounded before driver body
allocation by `max_batch_bytes + 519` bytes: one tag, four length bytes, two
field-count bytes, and four length bytes for each of at most 128 columns.
Only non-DataRow messages may use a 64 KiB minimum allowance for authentication,
startup metadata, and control messages. Larger configured budgets raise that
non-row bound too. Catalog DataRows retain the DataRow bound. Decoded page
allocations and encoded OTLP are checked separately against `max_batch_bytes`;
wire admission alone cannot authorize normalization expansion. Checked frames
are forwarded individually with fixed 8 KiB socket read-ahead. Text and bytea
have no independent one-MiB ceiling. These are bounded buffers, not a process-RSS
guarantee.

The Oracle-style catch-up defaults allow at most 32 page fetches or ten seconds
per cycle, then wait `query.interval`; byte limits and backpressure can reduce
progress further. Each page waits for ACK and durable checkpointing.
Confirmed timeouts and transient connection/query failures use shared capped
backoff. Authentication, permissions, invalid data, and unconfirmed cleanup
remain terminal.

The [shared scraper](../../../../scraper/README.md#polling-and-delivery-semantics)
owns scheduling, admission, encoding, feedback, checkpointing, and shutdown.
The adapter isolates driver work on one dedicated thread with a capacity-one
command channel, and ends transactions before waiting for ACK. Unconfirmed
cleanup retains source ownership until process exit.

At-least-once delivery requires stable unique positions, commit-visible ordering,
retained immutable rows, and a meaningful downstream ACK boundary. Late-visible
rows behind the committed incremental cursor cannot be recovered automatically.
Scalar timestamp keys must be unique; repeated timestamps require composite
mode. Keep incremental rows immutable and retained through ACK/checkpoint and
replay, and ensure new positions become commit-visible in source order.
Automatic tail discovery, TLS, and full production qualification remain unsupported.

## Development checks

From `rust/otap-dataflow`:

```sh
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver
cargo clippy -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib --tests -- -D warnings
```
