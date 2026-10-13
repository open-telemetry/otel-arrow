# PostgreSQL Receiver

<!-- markdownlint-disable MD013 -->

Experimental PostgreSQL 15 receiver: `urn:otel:receiver:postgresql`. The opt-in `postgresql` feature is excluded from `contrib-receivers`. Each selected row becomes one OTLP log with column values in its structured body, not a Log Analytics custom-table mapping.

**Database transport is unencrypted TCP without server-identity verification. Use only isolated, trusted development environments and disposable credentials.** TLS settings are rejected; `crypto-ring` can secure exporters but does not encrypt this database connection. Local database qualification used PostgreSQL 15.19, not every PostgreSQL version.

## Getting started

Use the [console example](../../../../../configs/postgresql-console.yaml). Provision a least-privilege account with SELECT access and adjust the schema-qualified source, connection, and initial cursor. Set `PG_HOST`, `PG_USERNAME_FILE`, and `PG_PASSWORD_FILE`; the latter two name readable credential files. Provide persistent, writable checkpoint storage.

From `rust/otap-dataflow`:

```sh
cargo run --no-default-features --features postgresql,crypto-ring --bin df_engine -- \
  --config configs/postgresql-console.yaml
```

No OTLP endpoint is needed for the example. Console output is not durable downstream storage.

## Configuration

One receiver owns one query/collecting connection and requires a one-core pipeline. Unknown fields and null configuration values are rejected. Result metadata is discovered automatically; customer-authored result schemas are not supported.

| Field | Contract |
| --- | --- |
| `source_id` | Required stable name, 1-128 UTF-8 bytes without control characters. |
| `connection.host` | Required single DNS name or IP address; no URI or host list. |
| `connection.port` | Defaults to `5432`; nonzero. |
| `connection.database` | Required name, 1-63 UTF-8 bytes. |
| `query.statement` | Required read-only SELECT, at most 16 KiB. |
| `query.interval` | Default `1m`; `1m`-`24h` in whole seconds (`1.5m` is valid). |
| `query.max_rows_per_poll` | Required page ceiling, `1`-`10000`; no default or whole-cycle row cap. |
| `query.max_batch_bytes` | Required separate normalized-page and encoded-OTLP ceilings, `1` byte-`256 MiB`; integer bytes or strings such as `10 MiB`. |
| `query.timeout` | Default `30s`; `1s`-`5m` in whole seconds. Bounds credential/worker operations and native statement/idle-transaction timeouts. |
| `query.fetch_size_rows` | Default `300`; `1`-`10000`, no greater than the page ceiling. Byte estimates may reduce a fetch. |
| `query.catch_up.max_pages` | Default `32`; `1`-`1024`, including empty probes. |
| `query.catch_up.max_duration` | Default `10s`; `1ms`-`5min`. Gates the next page, not an active page. |
| `watermark` | Required composite timestamp plus signed-integer tie-breaker; see the example. |
| `checkpoint.directory` | Required persistent storage directory. |

Checkpoint defaults: `on_nack: rewind`, `on_permanent_nack: pause`, `nack_backoff: 1s`, `max_consecutive_failures: 5`. The last counts checkpoint-write failures, not connection retries. Interval, timeout, page/fetch budgets, and catch-up changes preserve checkpoint compatibility; connection, authored SQL, and watermark changes invalidate it. Incompatible checkpoints fail, never silently reset.

The composite watermark's timestamp column automatically sets the log event time, as in Oracle; there is no separate `query.output` setting. Checkpoints from earlier configurable-output builds are incompatible: use a new checkpoint directory and an explicitly reviewed initial cursor.

### Authentication and source identity

Bind `basic_auth_provider` to the [flat-file provider](../../../../contrib-extensions/src/flat_file_user_pass_auth/README.md), included by the root `postgresql` feature. The capability supplies credentials for PostgreSQL's native protocol, not HTTP Basic headers.

Files refresh at `password_secret_file_refresh`; a reconnect before refresh may use the old password and fail. Healthy sessions are not replaced. Username changes during a receiver's lifetime are rejected: stop, review source identity/grants, and restart. After restart the principal must still read the same source.

### SQL and cursor contract

Use schema-qualified persistent ordinary tables, aliases, and explicit qualified projections. Up to three inner joins are allowed when equality predicates cover a proven unique key on each joined side. Cursor columns must be direct, non-null columns of the driving table; their pair must uniquely identify each result position.

Use either keyset below, optionally ANDed with supported static comparisons, and order by timestamp then tie-breaker ascending:

```sql
(e.ts > :last_timestamp OR
 (e.ts = :last_timestamp AND e.id > :last_tie_breaker))

(e.ts, e.id) > (:last_timestamp, :last_tie_breaker)
```

Native `$1`/`$2` binds are also accepted; do not mix styles. Execution rewrites the validated keyset to equivalent tuple comparison and adds the configured page `LIMIT`; cursor values stay bound. Ask the database owner for an index beginning with timestamp and tie-breaker. No write privileges are needed.

Timestamp cursors require `timestamp`/`timestamptz`, `timezone: UTC`, and an ISO 8601 initial value with `T`/`Z`, at most six fractional digits, exactly representable at native precision. Numeric offsets, spaces, and precision loss are rejected. Tie-breakers use range-checked `int2`, `int4`, or `int8`.

Views, RLS, partitioned/foreign/temporary tables, system schemas, functions, computed projections/casts, stars, outer joins, subqueries, grouping, unrestricted OR, user LIMIT/OFFSET/FETCH, and multiple statements are unsupported. All referenced columns, including filter/join columns, are validated.

### Metadata discovery

Local validation derives output names and cursor positions without contacting PostgreSQL. Bounded catalog queries discover native types, modifiers, and nullability before preparing the user query; unsupported types, invalid cursor/event-time columns, and unrepresentable initial values fail closed. Prepared output names, native types/modifiers, and table/column origins must match.

Catalog reads share the operation timeout and receive limits: at most two relation matches, requested attributes plus one, and 129 index descriptions (more than 128 indexes fails). No source-data scan is needed. Every page/reconnect compares fresh metadata with cached signatures; drift fails closed during the receiver's lifetime.

The schema baseline is **not persisted across process restarts**. Keep schema, grants, column semantics, and unique cursor ordering stable across restarts. Checkpoint identity hashes the authored connection/query/watermark and timestamp-mapping contract, not discovered types.

## Value mapping

| Native type | OTLP body value |
| --- | --- |
| `bool`; `int2`, `int4`, `int8` | Boolean; signed 64-bit integer. |
| `numeric` | Exact decimal string preserving scale. |
| `float4`, `float8` | Finite double; NaN/infinity rejected. |
| `text`, `varchar`, `bpchar`; `bytea` | Valid UTF-8 string preserving padding; bytes. |
| `timestamp`, `timestamptz` | `YYYY-MM-DDTHH:MM:SS.ffffffZ`; timestamp without timezone interpreted as UTC. |
| `date`; `uuid` | `YYYY-MM-DD`; hyphenated UUID string. |
| `json`, `jsonb` | Validated JSON text preserving numeric precision, not expanded OTLP fields. |
| `interval`; nullable `NULL` | `months=N;days=N;microseconds=N`; empty OTLP value retaining the column name. |

Invalid/unsupported values, UTF-8 errors, precision loss, and oversized records fail explicitly without skipping, truncation, or automatic type-to-string fallback. Timestamps outside the OTLP time range use observation time while retaining the original body value; watermark timestamps remain strict and non-null.

## Resource and delivery boundaries

The example chooses `10000` rows/`10 MiB`; neither is a default. Other bounds: 10-second connect budget (also limited by operation timeout), 128 output columns, and 16,384 numeric-output characters. Fetching probes one row then adapts to the largest observed row; abrupt size increases may cause re-reading part of a fetch. Geometric page allocation charges metadata, cursors, and spare capacity; hard budgets do not depend on the estimate.

DataRow frames are checked before driver body allocation against `max_batch_bytes + 519` wire-overhead bytes. Only non-DataRow messages get a 64 KiB minimum allowance; catalog DataRows do not. Frames are forwarded individually with fixed 8 KiB read-ahead. Normalized allocations and encoded OTLP are checked separately against the configured byte budget; wire admission does not authorize expansion. There is no independent one-MiB text/bytea cap or total-process RSS guarantee.

The [shared scraper](../../../../scraper/README.md#polling-and-delivery-semantics) owns scheduling, admission, encoding, ACKs, checkpointing, and shutdown. Default catch-up stops after 32 page fetches or ten seconds, then waits the interval; byte budgets/backpressure can reduce progress. Each page waits for ACK and durable checkpointing. Confirmed timeouts/transient failures use shared capped backoff; authentication, permissions, invalid data, and unconfirmed cleanup remain terminal.

Driver work stays on one dedicated thread behind a capacity-one channel. Transactions end before waiting for ACK. Unconfirmed cleanup retains source ownership until process exit. At-least-once delivery requires stable unique positions, commit-visible ordering, retained immutable rows, and meaningful downstream ACKs. Late-visible rows behind the cursor, scalar/snapshot modes, automatic tail discovery, TLS, and full production qualification remain unsupported.

## Development

```sh
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver
cargo clippy -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib --tests -- -D warnings
```
