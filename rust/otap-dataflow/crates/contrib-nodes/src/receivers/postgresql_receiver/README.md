# PostgreSQL Receiver

This receiver is currently pre-1.0. Its public API may evolve between minor
releases.

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `urn:otel:receiver:postgresql`
- Crate: `otel-arrow-dfe-contrib-nodes`
- Feature gate: `postgresql` (opt-in; not enabled by `contrib-receivers`)
- Stability: Experimental
- Output: OTLP logs, one log record per selected database row
- Execution: One query per receiver, in a single-core pipeline

**Database transport is unencrypted TCP without server-identity verification. Use only isolated, trusted development environments and disposable credentials.**

The async PostgreSQL session and connection driver run as pipeline-local
`!Send` tasks on the existing engine runtime. The receiver does not create a
second Tokio runtime or a dedicated database thread.

## Getting started

The [console example](../../../../../configs/postgresql-console.yaml) reads
`public.receiver_events`. For a disposable local database, create the source and
a dedicated login:

```sql
-- Run as a PostgreSQL administrator.
CREATE DATABASE receiver_fixture;

-- Reconnect to receiver_fixture before continuing.
CREATE ROLE otel_reader LOGIN;
\password otel_reader

CREATE TABLE public.receiver_events (
  event_ts timestamptz(6) NOT NULL,
  event_id bigint NOT NULL,
  actor text NOT NULL,
  PRIMARY KEY (event_ts, event_id)
);
INSERT INTO public.receiver_events VALUES
  ('2026-10-05T00:00:01Z', 1, 'alice'),
  ('2026-10-05T00:00:01Z', 2, 'bob'),
  ('2026-10-05T00:00:02.123456Z', 3, 'carol');

GRANT CONNECT ON DATABASE receiver_fixture TO otel_reader;
GRANT USAGE ON SCHEMA public TO otel_reader;
GRANT SELECT ON public.receiver_events TO otel_reader;
```

Enter the same password into a protected file without putting it in YAML:

```sh
mkdir -p .secrets/postgresql
printf '%s\n' 'otel_reader' > .secrets/postgresql/username
read -r -s -p 'PostgreSQL password: ' PG_PASSWORD
printf '\n'
printf '%s\n' "$PG_PASSWORD" > .secrets/postgresql/password
unset PG_PASSWORD
chmod 600 .secrets/postgresql/username .secrets/postgresql/password

export PG_HOST=localhost
export PG_USERNAME_FILE="$PWD/.secrets/postgresql/username"
export PG_PASSWORD_FILE="$PWD/.secrets/postgresql/password"
```

Ensure the engine state directory used by `checkpoint.directory` is writable
and persisted across receiver or host restarts.

From `rust/otap-dataflow`:

```sh
cargo run --no-default-features --features postgresql,crypto-ring --bin df_engine -- \
  --config configs/postgresql-console.yaml
```

No OTLP endpoint is needed for the example. Console output is not durable downstream storage.
The three rows produce output shaped like:

```text
RESOURCE v1.Resource [db.system.name=postgresql, receiver.database.source_id=orders-audit]
INFO database.query.row: {event_ts=2026-10-05T00:00:01.000000Z, event_id=1, actor=alice}
INFO database.query.row: {event_ts=2026-10-05T00:00:01.000000Z, event_id=2, actor=bob}
INFO database.query.row: {event_ts=2026-10-05T00:00:02.123456Z, event_id=3, actor=carol}
```

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
| `query.fetch_size_rows` | Default `300`; `1`-`10000`, no greater than the page ceiling. The largest of the last 32 rows may reduce a fetch; sustained smaller rows let it grow again. |
| `query.catch_up.max_pages` | Default `32`; `1`-`1024`, including empty probes. |
| `query.catch_up.max_duration` | Default `10s`; `1ms`-`5min`. Gates the next page, not an active page. |
| `watermark` | Required composite timestamp plus signed-integer tie-breaker; see the example. |
| `checkpoint.directory` | Required persistent storage directory. |

Database, credential, and native statement operations use a fixed 30-second timeout. Connection attempts use a separate 10-second cap.

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

The initial cursor is exclusive: rows at or before its timestamp/tie-breaker
pair are not collected.

### Supported SQL forms

| Area | Accepted contract |
| --- | --- |
| Source relations | Schema-qualified, aliased, ordinary persistent tables outside `pg_*` and `information_schema`. |
| Projection | Between 1 and 128 direct, qualified columns, optionally renamed with unique aliases. |
| Cursor predicate | Exactly one supported composite keyset shown above, using both configured binds. |
| Static filters | Qualified column compared to a numeric, Boolean, or single-quoted literal with `=`, `!=`, `>`, `<`, `>=`, or `<=`; `IS NULL` and `IS NOT NULL` are also accepted. Combine filters with `AND`. |
| Joins | Up to three `INNER JOIN ... ON` clauses. Join predicates are `AND` combinations of direct column equalities. The columns on each newly joined side must cover a catalog-proven unique key and have matching native types/collations. |
| Ordering | Cursor timestamp followed by tie-breaker, both ascending. |
| Rejected | Views, materialized/foreign/partitioned/temporary tables, RLS-enabled tables, system schemas, stars, expressions, casts, functions, aggregates, `DISTINCT`, outer/cross joins, subqueries, CTEs, `OR` outside the keyset, grouping, set operations, user `LIMIT`/`OFFSET`/`FETCH`, row locks, and multiple statements. |

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

## Failure recovery

| Condition | Behavior | Operator action |
| --- | --- | --- |
| Connection loss, server restart, idle-session timeout, temporary lock timeout, serialization failure, deadlock, or fixed query timeout | The partial page is discarded, the transaction/session is cleaned up, and the receiver reconnects and revalidates after shared backoff (`1s`, `2s`, `4s`, `8s`, `16s`, then `30s`). The acknowledged checkpoint does not advance. | Usually none. Investigate if retries continue. |
| Rejected or unavailable credentials | Terminal configuration failure; collection does not retry with an unusable credential. | Repair the credential file/provider, wait for its configured refresh if applicable, then restart the receiver/pipeline. |
| SQL, permissions, unsupported type/value, or schema/signature change | Fails closed without emitting or checkpointing the failed page. | Restore the expected query/schema/grants and restart the receiver. For an intentional semantic change, use a reviewed initial cursor and a new source/checkpoint identity. |
| Oversized row, frame, normalized-page, or encoded-page byte limit | The row/page is not truncated and the failed page is not checkpointed. Composite collection stops because the error is non-retryable. | Increase `max_batch_bytes` or narrow the query, then restart the receiver. |
| Downstream NACK | With the default `on_nack: rewind`, the page is re-queried after `nack_backoff`. A permanent NACK pauses the source by default. | Repair downstream; restart a permanently paused receiver. |
| Cleanup cannot be confirmed | Terminal `Cleanup`; source ownership is quarantined to prevent overlapping collection. | Perform a full process restart. |

## Development

```sh
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver
cargo clippy -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib --tests -- -D warnings
```
