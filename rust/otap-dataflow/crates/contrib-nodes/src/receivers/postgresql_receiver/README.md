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

## Quick walkthrough

The [console example](../../../../../configs/postgresql-console.yaml) works with
the included Docker Compose fixture on Windows and Linux. It creates a
TLS-enabled PostgreSQL 15.19 database, private CA, reader, table, and sample
rows. PostgreSQL binds only to `127.0.0.1`. Run from `rust/otap-dataflow`.

```sh
docker compose --file crates/contrib-nodes/src/receivers/postgresql_receiver/fixtures/tls/compose.yaml up --build --detach --wait
```

The fixture writes the username, password, and CA to its `out` directory.

Run the receiver:

```sh
cargo run --no-default-features --features postgresql,crypto-ring --bin df_engine -- --config configs/postgresql-console.yaml
```

No OTLP endpoint is needed. The console emits three OTLP logs shaped like:

```text
RESOURCE v1.Resource [db.system.name=postgresql, receiver.database.source_id=orders-audit]
INFO database.query.row: {event_ts=2026-10-05T00:00:01.000000Z, event_id=1, actor=alice}
INFO database.query.row: {event_ts=2026-10-05T00:00:01.000000Z, event_id=2, actor=bob}
INFO database.query.row: {event_ts=2026-10-05T00:00:02.123456Z, event_id=3, actor=carol}
```

Console output is not durable downstream storage. In another terminal, verify
the live connection is encrypted:

```sh
docker compose --file crates/contrib-nodes/src/receivers/postgresql_receiver/fixtures/tls/compose.yaml exec postgres psql -U postgres -d receiver_fixture -c "SELECT usename, ssl, version, cipher FROM pg_stat_ssl JOIN pg_stat_activity USING (pid) WHERE usename = 'otel_reader';"
```

The result must show `ssl` as `t`. Stop the receiver with Ctrl+C, then clean up:

```sh
docker compose --file crates/contrib-nodes/src/receivers/postgresql_receiver/fixtures/tls/compose.yaml down --volumes --remove-orphans
rm -rf crates/contrib-nodes/src/receivers/postgresql_receiver/fixtures/tls/out .otap-state/postgresql
```

In PowerShell, replace the final `rm -rf` with:

```powershell
Remove-Item -Recurse -Force crates\contrib-nodes\src\receivers\postgresql_receiver\fixtures\tls\out -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force .otap-state\postgresql -ErrorAction SilentlyContinue
```

## Configuration

| Field | Required? | Default / contract |
| --- | --- | --- |
| `source_id` | Yes | Stable name, 1-128 UTF-8 bytes without control characters. |
| `connection.host` | Yes | One DNS name or IP address; no URI or host list. |
| `connection.port` | No | `5432`; must be nonzero. |
| `connection.database` | Yes | Database name, 1-63 UTF-8 bytes. |
| `connection.tls.enabled` | No | `true`; set `false` only to request plaintext explicitly. |
| `connection.tls.ca_file` | No | PEM CA bundle that augments platform trust. Valid only when TLS is enabled. |
| `query.statement` | Yes | One read-only `SELECT`, at most 16 KiB. |
| `query.interval` | No | `1m`; `1m`-`24h` in whole seconds (`1.5m` is valid). |
| `query.timeout` | N/A | Fixed at default `30s` and not configurable for v1. |
| `query.max_rows_per_poll` | Yes | `1`-`10000`, no default. Composite/scalar: per-page ceiling, not a whole-cycle cap. Snapshot: complete-result ceiling; overflow rejects the entire observation. |
| `query.max_batch_bytes` | Yes | Separate normalized-page and encoded-OTLP ceilings, `1` byte-`256 MiB`; integer bytes or strings such as `10 MiB`. A snapshot must fit both limits completely. |
| `query.fetch_size_rows` | No | `300`; `1`-`10000`, no greater than the page ceiling. The largest of the last 32 rows may reduce a fetch; sustained smaller rows let it grow again. |
| `query.catch_up.max_pages` | No | `32`; `1`-`1024`, including empty probes. Composite/scalar only; snapshot never immediately repeats a successful observation for catch-up. |
| `query.catch_up.max_duration` | No | `10s`; `1ms`-`5min`. Composite/scalar cycle budget; gates the next page, not an active page. |
| `watermark` | Yes | Select `composite`, `scalar`, or `snapshot` using the mode-specific fields below. |
| `checkpoint.directory` | Yes | Persistent writable state directory. |

Checkpoint defaults: `on_nack: rewind`, `on_permanent_nack: pause`, `nack_backoff: 1s`, `max_consecutive_failures: 5`.

Explicit `null` values are rejected for PostgreSQL receiver fields. Omit optional
fields to use defaults.

### Watermark modes

All modes retain the existing PostgreSQL query schema: use
`fetch_size_rows`, not `fetch_size`, and omit `query.timeout` because operations
retain the fixed `30s` deadline. No new connection or authentication fields are
needed. Fetch size is a target; the existing byte-aware fetch sizing can reduce it.

| Mode | Required watermark fields | Collection behavior | OTLP event timestamp |
| --- | --- | --- | --- |
| `composite` | `timestamp` and `tie_breaker` | Incremental pages ordered by timestamp and integer tie-breaker; resume from acknowledged progress. | Cursor timestamp, with the existing observation-time fallback for values outside the OTLP time range. |
| `scalar` | `column`, `bind`, and typed `initial` | Incremental pages ordered by one unique, non-null key; resume from acknowledged progress. | Timestamp key for `initial.type: timestamp`; observation time for `initial.type: int64`. |
| `snapshot` | No fields beyond `mode` | Complete result each interval, including unchanged rows; no resumable row position. | Observation time. Selected timestamp columns remain body fields. |

The following are alternative `query` and `watermark` fragments under the
receiver's `config`, not complete pipeline configurations. Keep the existing
`source_id`, `connection`, `checkpoint`, and `basic_auth_provider` wiring.
Use a new source/checkpoint identity when changing modes; do not reuse an
incompatible checkpoint.

#### Composite: timestamp and tie-breaker

The existing configuration remains valid:

```yaml
query:
  statement: >
    SELECT a.audit_id, a.last_updated, a.user_name, a.action
    FROM public.audit_logs AS a
    WHERE (
      a.last_updated > :last_timestamp
      OR (a.last_updated = :last_timestamp AND a.audit_id > :last_tie_breaker)
    )
    ORDER BY a.last_updated ASC, a.audit_id ASC
  interval: 5m
  fetch_size_rows: 1000
  max_rows_per_poll: 10000
  max_batch_bytes: 10 MiB
watermark:
  mode: composite
  timestamp:
    column: last_updated
    bind: last_timestamp
    initial: "2026-01-01T00:00:00.000000Z"
    timezone: UTC
  tie_breaker:
    column: audit_id
    bind: last_tie_breaker
    initial: 0
```

#### Scalar: one typed key

Use a single non-null, unique column with strictly increasing values. Scalar
configuration follows the shared Oracle and SQL Server watermark shape:

```yaml
query:
  statement: >
    SELECT a.audit_id, a.last_updated, a.user_name, a.action
    FROM public.audit_logs AS a
    WHERE a.audit_id > :last_value
    ORDER BY a.audit_id ASC
  interval: 5m
  fetch_size_rows: 1000
  max_rows_per_poll: 10000
  max_batch_bytes: 10 MiB
watermark:
  mode: scalar
  column: audit_id
  bind: last_value
  initial:
    type: int64
    value: 0
```

Supported scalar types are `int64` for PostgreSQL `int2`/`int4`/`int8`
and `timestamp` for `timestamp`/`timestamptz`. Integer values must fit the native
column type. Timestamp initial values use the existing UTC normalization and
precision rules; for example, `initial.type: timestamp` with
`initial.value: "2026-01-01T00:00:00.000000Z"`. The selected column, predicate,
and ordering must all refer to that timestamp key. String and unsigned scalar
keys are not supported.

Scalar uniqueness must be proven by a valid, immediate, nonpartial unique index
whose sole key column is the cursor column; included non-key columns are allowed.
Revalidate that guarantee on every page. A timestamp shared by multiple rows
requires composite mode instead. Merely selecting `last_updated` in the integer
example does not make it the event timestamp.

#### Snapshot: complete result

No cursor predicate, parameter, initial value, or unique key is required.
Ordering is optional; this example retains it for predictable presentation:

```yaml
query:
  statement: >
    SELECT a.audit_id, a.last_updated, a.user_name, a.action
    FROM public.audit_logs AS a
    ORDER BY a.audit_id ASC
  interval: 5m
  fetch_size_rows: 1000
  max_rows_per_poll: 10000
  max_batch_bytes: 10 MiB
watermark:
  mode: snapshot
```

Omitting `interval` uses the existing `1m` default; snapshot does not mean
one-shot execution. Duplicate rows and nullable values are allowed. Each result
must fit the row, normalized-memory, and encoded-OTLP limits; exceeding a limit
must not emit a partial snapshot. Deterministic complete-result row/byte overflow
pauses the source until repaired and restarted.

Checkpoint storage remains required for ownership and acknowledged revisions,
but snapshot checkpoints do not retain a row offset. A retry or restart queries
the current source again, not a saved copy of an earlier observation. Empty
results emit no logs. Transactions must finish before waiting for downstream ACK.

### Transport security

TLS is required by default and the connection does not fall back to plaintext. The configured
host is used for certificate verification and SNI. Platform CA certificates
are trusted; `connection.tls.ca_file` can add one or more PEM-encoded private
CA certificates. The CA file is read during each connection attempt, so a
replacement applies on reconnect without changing the source fingerprint.
The bundle must be nonempty, valid PEM and no larger than 4 MiB. Platform
roots are cached for the process lifetime. Trust changes do not replace a
healthy session.

Build with one of the workspace `crypto-*` features (for example
`postgresql,crypto-ring`). Connection setup, including trust loading, DNS,
TCP, TLS, and login, has a fixed 10-second deadline. Cancellation requests
use the same TLS policy and server identity as the active session.

For an isolated development database that does not support TLS, opt out
explicitly:

```yaml
connection:
  host: localhost
  port: 5432
  database: receiver_fixture
  tls:
    enabled: false
```

`ca_file` cannot be combined with `enabled: false`. The receiver does not
support skipping certificate or hostname verification.

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

Use schema-qualified persistent ordinary tables, aliases, and explicit qualified projections in every mode. In composite/scalar mode, cursor columns must be direct, non-null columns of the driving table. Snapshot has no cursor columns.

The timestamp/tie-breaker pair must uniquely identify each result position.
The receiver checks column presence, types, nullability, predicate, ordering,
and joined-side uniqueness, but cannot prove composite driving-source uniqueness,
commit ordering, or immutability. Scalar mode additionally checks
the driving key's unique index; commit ordering and immutability remain source
requirements.

For composite/scalar modes, the at-least-once guarantee also requires cursor-ordered commit visibility: no row may become visible later at or before a committed cursor. The initial cursor excludes rows at or before that position. Snapshot instead repeats complete observations of current data.

For composite mode, use either keyset below, optionally ANDed with supported static comparisons, and order by timestamp then tie-breaker ascending:

```sql
(e.ts > :last_timestamp OR
 (e.ts = :last_timestamp AND e.id > :last_tie_breaker))

(e.ts, e.id) > (:last_timestamp, :last_tie_breaker)
```

Native `$1`/`$2` binds are also accepted in composite mode; do not mix styles.
Scalar mode accepts `a.audit_id > :last_value` or
`a.audit_id > $1`, optionally ANDed with supported static comparisons, with
ascending cursor ordering. Snapshot rejects all placeholders and can omit
`WHERE` entirely or use the same supported static filters.

Timestamp cursors require `timestamp`/`timestamptz`; composite additionally
requires `timezone: UTC` in its timestamp block.
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
| Cursor predicate | Composite: exactly one supported composite keyset using both configured binds. Scalar: one strict `>` keyset using the configured bind. Snapshot: no cursor predicate or binds. |
| Static filters | Qualified column compared to a numeric, Boolean, or single-quoted literal with `=`, `!=`, `>`, `<`, `>=`, or `<=`; `IS NULL` and `IS NOT NULL` are also accepted. Combine filters with `AND`. For `date`/`timestamp`/`timestamptz` columns, single-quoted literals must be absolute values (for example `2026-10-05`, `2026-10-05 00:00:00`, or RFC 3339); relative values such as `today`, `now`, and `epoch` are rejected. |
| Joins | Up to three `INNER JOIN ... ON` clauses. Join predicates are `AND` combinations of direct column equalities. The columns on each newly joined side must cover a catalog-proven unique key and have matching native types/collations. |
| Ordering | Composite: cursor timestamp followed by tie-breaker, both ascending. Scalar: cursor column ascending. Snapshot: optional ordering by direct, qualified columns. |
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
| TLS configuration | Missing crypto provider, unavailable trust store, persistently missing/unreadable or invalid CA bundle, untrusted/expired/mismatched server certificate, TLS refusal | Terminal errors identify the failing TLS stage without exposing certificate contents, paths, or native server diagnostics. | Check the indicated provider, CA file, server certificate, hostname, or TLS protocol settings, then restart the receiver. |
| Authentication | Missing, expired, unavailable, or rejected username/password | Credential details are redacted. The latest provider credential is fetched for each operation, but an authentication failure is terminal rather than retried indefinitely. | Repair or rotate the provider credential, wait for provider refresh if configured, then restart the receiver. |
| Transient transport | Connection reset/refused, EOF, broken pipe (including during TLS handshakes), temporary CA-file storage I/O, failover, server restart, overload, idle-session or temporary lock timeout, serialization failure, deadlock | The partial page is discarded. The receiver reconnects and revalidates after shared backoff (`1s`, `2s`, `4s`, `8s`, `16s`, then `30s`); the checkpoint remains unchanged. | Usually none. Investigate the database, network, or CA-file storage if retries continue. |
| Timeout or cancellation | Native operation exceeds the fixed 30-second deadline, pipeline stop interrupts a query | PostgreSQL cancellation is requested and the local connection task is stopped and joined. Retryable timeouts reconnect after backoff; possible old server work is fenced for the statement timeout. Unconfirmed cleanup fails closed. | Usually none for a recovered timeout. Perform a full process restart after terminal `Cleanup`. |
| Query or metadata | Syntax error, missing table, permission denied, changed result columns/types, lost joined-side uniqueness | The query is non-retryable and collection terminates without emitting or checkpointing the failed page. | Restore the expected query, schema, indexes, and grants. For an intentional semantic change, use a reviewed initial cursor and new source/checkpoint identity. |
| Conversion or bounds | Unsupported type/value, overflow, malformed timestamp or UTF-8, precision loss, non-advancing cursor, oversized frame/row/page | The entire page fails without skipping, truncating, coercing to text, or advancing the checkpoint. | Repair the source value/query or increase `max_batch_bytes`, then restart the receiver. |
| Downstream delivery | Closed output channel, retryable NACK, permanent NACK/export rejection | A matching ACK and successful checkpoint write are both required to commit. Retryable NACK re-queries from the durable cursor after `nack_backoff`; permanent NACK pauses the source by default or uses explicitly configured capped retry. | Repair downstream delivery. Restart a source that remains paused under the default permanent-NACK policy. |
| Checkpoint or ownership | Corrupt/oversized state, checksum/version/source/fingerprint/revision mismatch, lease conflict, unavailable storage | Startup and reads fail closed. Write failures retry up to `max_consecutive_failures`, then terminate. The receiver never assumes a cursor position. | Repair or restore the exact checkpoint storage and ownership issue; do not delete or edit state without reviewing the replay position. |

## Development

### Live integration CI

The component's `ci/integration-test.json` registers an affected-path-selected,
advisory (`required: false`) PostgreSQL smoke job on Ubuntu 24.04 in the shared
contrib integration workflow, following the Oracle receiver pattern. It reuses
the pinned TLS Docker fixture and explicitly runs the otherwise ignored
`live_scalar_and_snapshot_native_execution` test. This covers real PostgreSQL
TLS/login, catalog validation, prepare/bind/portal execution, scalar pagination,
and both exact-fit and overflowing snapshots. The harness fails if the named
test is missing or no live test executes. Before provisioning, it runs
`ci/test_harness.py` contract checks for discovery, failure propagation,
cleanup, and rejection of missing or no-op tests using fake external commands.
Those checks are not a substitute for the subsequent live test.

From the repository root, with Docker Compose v2, OpenSSL, GNU `timeout`, Python,
and the Rust build prerequisites installed, run the same entry point as CI:

```sh
python3 .github/scripts/run-contrib-integration-test.py --manifest rust/otap-dataflow/crates/contrib-nodes/src/receivers/postgresql_receiver/ci/integration-test.json
```

Each run owns a unique Compose project, database, network, private temporary
export directory, and OS-assigned loopback port. It neither uses the walkthrough
database on port 5432 nor changes host trust. Compose waits for initialization
and fixture-query readiness, with bounded provisioning and test commands.
Cleanup runs on exit and through the shared runner, removes only that project's
containers/volumes and exported credentials, and surfaces cleanup failures.
Failure artifacts contain test output and container state, not credentials,
container logs, or Compose configuration.

The manual Compose walkthrough keeps its existing defaults. The live test also
accepts `OTEL_ARROW_POSTGRESQL_TEST_PORT` and
`OTEL_ARROW_POSTGRESQL_TEST_CA_FILE` for an isolated fixture; checkpoint state
always uses a per-test temporary directory.

### Unit and transport tests

Run commands from `rust/otap-dataflow`.

Run the complete PostgreSQL receiver unit and loopback transport suite:

```sh
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver
```

Run only the socket-level TLS transport scenarios:

```sh
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver::worker::tls_tests
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib stalled_tls_handshake_obeys_connection_deadline
```

Run focused PostgreSQL test groups:

```sh
# Configuration, SQL contract, cursor, and value conversion.
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver::tests

# Adapter error classification and retry policy.
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver::adapter::tests

# Worker session lifecycle, cancellation, cleanup, and reconnect fencing.
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver::worker::tests

# Bounded PostgreSQL backend frame handling.
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features postgresql --lib postgresql_receiver::worker::framing_tests
```

Check the production engine with PostgreSQL and the default Ring provider:

```sh
cargo check -p otel-arrow-dfe --no-default-features --features postgresql,crypto-ring
```
