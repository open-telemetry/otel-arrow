# PostgreSQL Receiver

<!-- markdownlint-disable MD013 -->

PostgreSQL receiver: `urn:otel:receiver:postgresql`. The opt-in `postgresql` feature is excluded from `contrib-receivers`.

**Database transport is unencrypted TCP without server-identity verification. Use only isolated, trusted development environments and disposable credentials.**

## Getting started

Use the [console example](../../../../../configs/postgresql-console.yaml). Provision a least-privilege account with SELECT access and adjust the schema-qualified source, connection, and initial cursor. Set `PG_HOST`, `PG_USERNAME_FILE`, and `PG_PASSWORD_FILE`; the latter two name readable credential files. Provide persistent, writable checkpoint storage.

From `rust/otap-dataflow`:

```sh
cargo run --no-default-features --features postgresql,crypto-ring --bin df_engine -- \
  --config configs/postgresql-console.yaml
```

No OTLP endpoint is needed for the example. Console output is not durable downstream storage.

## Configuration

One receiver owns one query/collecting connection and requires a one-core pipeline. Unknown fields and null configuration values are rejected.

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

Checkpoint defaults: `on_nack: rewind`, `on_permanent_nack: pause`, `nack_backoff: 1s`, `max_consecutive_failures: 5`.

### Authentication and source identity

Bind `basic_auth_provider` to the [flat-file provider](../../../../contrib-extensions/src/flat_file_user_pass_auth/README.md), included by the root `postgresql` feature. The capability supplies credentials for PostgreSQL's native protocol, not HTTP Basic headers.

Files refresh at `password_secret_file_refresh`; a reconnect before refresh may use the old password and fail. Healthy sessions are not replaced. Username changes during a receiver's lifetime are rejected: stop, review source identity/grants, and restart. After restart the principal must still read the same source.

### SQL and cursor contract

Use schema-qualified persistent ordinary tables, aliases, and explicit qualified projections. Cursor columns must be direct, non-null columns of the driving table, and their pair must uniquely identify each result position.

The at-least-once guarantee also requires cursor-ordered commit visibility: no row may become visible later at or before a committed cursor. Keep cursor and selected values stable, and retain rows unchanged throughout the maximum outage, downstream retry, and checkpoint-recovery window. Backdated inserts, late commits, updates, or early deletion can otherwise be skipped or make a NACK replay differ from the original page.

Use either keyset below, optionally ANDed with supported static comparisons, and order by timestamp then tie-breaker ascending:

```sql
(e.ts > :last_timestamp OR
 (e.ts = :last_timestamp AND e.id > :last_tie_breaker))

(e.ts, e.id) > (:last_timestamp, :last_tie_breaker)
```

Native `$1`/`$2` binds are also accepted; do not mix styles.

Timestamp cursors require `timestamp`/`timestamptz`, `timezone: UTC`, and an ISO 8601 initial value with `T`/`Z`, at most six fractional digits, exactly representable at native precision. Numeric offsets, spaces, and precision loss are rejected. Tie-breakers use range-checked `int2`, `int4`, or `int8`.

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

Invalid/unsupported values, UTF-8 errors, precision loss, and oversized records fail explicitly without skipping, truncation, or automatic type-to-string fallback.

## Development

```sh
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver
cargo clippy -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib --tests -- -D warnings
```
