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

```bash
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

| Field | Required? | Default / contract |
| --- | --- | --- |
| `source_id` | Yes | Stable name, 1-128 UTF-8 bytes without control characters. |
| `connection.host` | Yes | One DNS name or IP address; no URI or host list. |
| `connection.port` | No | `5432`; must be nonzero. |
| `connection.database` | Yes | Database name, 1-63 UTF-8 bytes. |
| `query.statement` | Yes | One read-only `SELECT`, at most 16 KiB. |
| `query.interval` | No | `1m`; `1m`-`24h` in whole seconds (`1.5m` is valid). |
| `query.timeout` | N/A | Fixed at default `30s` and not configurable for v1. |
| `query.max_rows_per_poll` | Yes | Page ceiling, `1`-`10000`; no default or whole-cycle row cap. |
| `query.max_batch_bytes` | Yes | Separate normalized-page and encoded-OTLP ceilings, `1` byte-`256 MiB`; integer bytes or strings such as `10 MiB`. |
| `query.fetch_size_rows` | No | `300`; `1`-`10000`, no greater than the page ceiling. The largest of the last 32 rows may reduce a fetch; sustained smaller rows let it grow again. |
| `query.catch_up.max_pages` | No | `32`; `1`-`1024`, including empty probes. |
| `query.catch_up.max_duration` | No | `10s`; `1ms`-`5min`. Gates the next page, not an active page. |
| `watermark` | Yes | Composite timestamp plus signed-integer tie-breaker; see the example. |
| `checkpoint.directory` | Yes | Persistent writable state directory. |

Checkpoint defaults: `on_nack: rewind`, `on_permanent_nack: pause`, `nack_backoff: 1s`, `max_consecutive_failures: 5`.

Explicit `null` values are rejected for PostgreSQL receiver fields. Omit optional
fields to use defaults.

### Authentication and source identity

Bind `basic_auth_provider` to the [flat-file provider](../../../../contrib-extensions/src/flat_file_user_pass_auth/README.md), included by the root `postgresql` feature. The capability supplies credentials for PostgreSQL's native protocol, not HTTP Basic headers.

The PostgreSQL receiver config accepts no direct `username`, `password`,
`password_secret`, `username_file`, or `password_secret_file` fields. Its
credential reference is the required `capabilities.basic_auth_provider`
binding. Configure the referenced provider with both `username_file` and
`password_secret_file`; do not put either credential value in YAML.

```yaml
extensions:
  pg-credentials:
    type: urn:otel:extension:flat_file_user_pass_auth
    config:
      username_file: /run/secrets/postgresql/username
      password_secret_file: /run/secrets/postgresql/password
      password_secret_file_refresh: 1m
nodes:
  pg:
    type: urn:otel:receiver:postgresql
    capabilities:
      basic_auth_provider: pg-credentials
    config:
      # PostgreSQL receiver fields begin here; no credential fields are accepted.
```

| Credential field | Required for PostgreSQL? | Contract |
| --- | --- | --- |
| `username_file` | Yes | Readable UTF-8 file containing a valid PostgreSQL username. |
| `password_secret_file` | Yes | Readable UTF-8 file containing the password. |
| `password_secret_file_refresh` | No | Defaults to `1h`; accepted range `10s`-`365d`. Both files are reread on this interval. |

Files refresh at `password_secret_file_refresh`; a reconnect before refresh may use the old password and fail. Healthy sessions are not replaced. Username changes during a receiver's lifetime are rejected: stop, review source identity/grants, and restart. After restart the principal must still read the same source.

### SQL and cursor contract

Use schema-qualified persistent ordinary tables, aliases, and explicit qualified projections. Cursor columns must be direct, non-null columns of the driving table.

The timestamp/tie-breaker pair must uniquely identify each result position.
The receiver checks column presence, types, nullability, predicate, ordering,
and joined-side uniqueness, but cannot prove driving-source uniqueness, commit
ordering, or immutability.

The at-least-once guarantee also requires cursor-ordered commit visibility: no row may become visible later at or before a committed cursor. The initial cursor excludes rows at or before that position.

Use either keyset below, optionally ANDed with supported static comparisons, and order by timestamp then tie-breaker ascending:

```sql
(e.ts > :last_timestamp OR
 (e.ts = :last_timestamp AND e.id > :last_tie_breaker))

(e.ts, e.id) > (:last_timestamp, :last_tie_breaker)
```

Native `$1`/`$2` binds are also accepted; do not mix styles.

Timestamp cursors require `timestamp`/`timestamptz` and `timezone: UTC`.
Initial values accept UTC RFC 3339, offset-qualified RFC 3339, or
timezone-less `YYYY-MM-DD HH:MM:SS[.fraction]` interpreted as UTC. They
normalize to `YYYY-MM-DDTHH:MM:SS.ffffffZ`. Input may contain up to nine
fractional digits but must be exactly representable at PostgreSQL's
microsecond precision; rounding, leap seconds, and years outside `0001`-`9999`
are rejected. Tie-breakers use range-checked `int2`, `int4`, or `int8`.

### Supported SQL forms

| Area | Accepted contract |
| --- | --- |
| Source relations | Schema-qualified, aliased, ordinary persistent tables outside `pg_*` and `information_schema`. |
| Projection | Between 1 and 128 direct, qualified columns, optionally renamed with unique aliases. |
| Cursor predicate | Exactly one supported composite keyset shown above, using both configured binds. |
| Static filters | Qualified column compared to a numeric, Boolean, or single-quoted literal with `=`, `!=`, `>`, `<`, `>=`, or `<=`; `IS NULL` and `IS NOT NULL` are also accepted. Combine filters with `AND`. For `date`/`timestamp`/`timestamptz` columns, single-quoted literals must be absolute values (for example `2026-10-05`, `2026-10-05 00:00:00`, or RFC 3339); relative values such as `today`, `now`, and `epoch` are rejected. |
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

The receiver fails closed: a failed page is never emitted partially and never
advances the acknowledged checkpoint.

| Class | Examples | Behavior | Operator action |
| --- | --- | --- | --- |
| Configuration | Unknown or null fields, invalid host/database/cursor, unsupported SQL mapping, zero or out-of-range limits | Validation fails before collection starts. PostgreSQL has no configurable driver field. | Correct the configuration and restart the receiver. |
| Authentication | Missing, expired, unavailable, or rejected username/password | Credential details are redacted. The latest provider credential is fetched for each operation, but an authentication failure is terminal rather than retried indefinitely. | Repair or rotate the provider credential, wait for provider refresh if configured, then restart the receiver. |
| Transient transport | Connection reset/refused, EOF, broken pipe, failover, server restart, overload, idle-session or temporary lock timeout, serialization failure, deadlock | The partial page is discarded. The receiver reconnects and revalidates after shared backoff (`1s`, `2s`, `4s`, `8s`, `16s`, then `30s`); the checkpoint remains unchanged. | Usually none. Investigate the database or network if retries continue. |
| Timeout or cancellation | Native operation exceeds the fixed 30-second deadline, pipeline stop interrupts a query | PostgreSQL cancellation is requested and the local connection task is stopped and joined. Retryable timeouts reconnect after backoff; possible old server work is fenced for the statement timeout. Unconfirmed cleanup fails closed. | Usually none for a recovered timeout. Perform a full process restart after terminal `Cleanup`. |
| Query or metadata | Syntax error, missing table, permission denied, changed result columns/types, lost joined-side uniqueness | The query is non-retryable and collection terminates without emitting or checkpointing the failed page. | Restore the expected query, schema, indexes, and grants. For an intentional semantic change, use a reviewed initial cursor and new source/checkpoint identity. |
| Conversion or bounds | Unsupported type/value, overflow, malformed timestamp or UTF-8, precision loss, non-advancing cursor, oversized frame/row/page | The entire page fails without skipping, truncating, coercing to text, or advancing the checkpoint. | Repair the source value/query or increase `max_batch_bytes`, then restart the receiver. |
| Downstream delivery | Closed output channel, retryable NACK, permanent NACK/export rejection | A matching ACK and successful checkpoint write are both required to commit. Retryable NACK re-queries from the durable cursor after `nack_backoff`; permanent NACK pauses the source by default or uses explicitly configured capped retry. | Repair downstream delivery. Restart a source that remains paused under the default permanent-NACK policy. |
| Checkpoint or ownership | Corrupt/oversized state, checksum/version/source/fingerprint/revision mismatch, lease conflict, unavailable storage | Startup and reads fail closed. Write failures retry up to `max_consecutive_failures`, then terminate. The receiver never assumes a cursor position. | Repair or restore the exact checkpoint storage and ownership issue; do not delete or edit state without reviewing the replay position. |

## Development

```sh
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver
cargo clippy -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib --tests -- -D warnings
```
