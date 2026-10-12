# SQL Server Configuration and Query Preparation

This is the internal foundation for a future SQL Server query-to-logs receiver,
not a runnable receiver. Enable `sql-server` explicitly to expose validated
configuration and T-SQL plans. The feature is excluded from all aggregate
features, and the top-level feature forwards only to contrib nodes.

**No `urn:otel:receiver:sql_server` factory is registered.** This layer has no
Tiberius dependency, adapter, credential acquisition, network connection,
filesystem access during validation, telemetry registration, or live tests.
It does not enable `flat-file-user-pass-auth`. Do not use it in runtime YAML yet.

Part 1 is tracked in [#4359][part-1] under [RFC #3918][rfc]. Adapter integration
is deferred to [#4360][part-2]; runtime activation and deployment documentation
are deferred to [#4361][part-3].

## Preparation API

Deserialize `SqlServerReceiverConfig` with Serde. Successful preparation exposes
the logical `source_id`, validated connection settings, a shared `CompiledQuery`,
checkpoint policy, semantic fingerprint, and a `SqlServerStatement`.
The statement retains canonical operator SQL separately from generated native
SQL and ordered `SqlServerParameter` roles. Getters expose these values for the
future adapter; they do not execute the query.

Configuration errors and `Debug` output redact SQL, endpoints, and cursor
values. Explicit SQL and connection getters are not redacted and must not be
logged as diagnostics.

## Configuration Contract

Unknown fields, invalid types, and unsupported enum values are rejected rather
than replaced with defaults. The required top-level objects are `connection`,
`query`, `watermark`, and `checkpoint`, together with `source_id`.

- `source_id`: 1..256 ASCII letters, digits, underscores, hyphens, or dots.
- `connection`: explicit `host` (DNS name or IP) and `database`; optional `port`
  (default 1433), `connect_timeout` (default 10s), and `tls.ca_file`. The CA path
  is validated syntactically, not opened. Credentials, connection strings,
  instance discovery, and insecure TLS overrides are rejected.
- `query`: `statement`, `max_rows_per_poll`, and `max_batch_bytes` are required.
  `interval` defaults to 60s, `timeout` to 30s, and `fetch_size_rows` to 300.
  Shared `catch_up` defaults are 32 pages and 10s.
- `watermark`: an explicit supported mode and initial cursor where applicable.
- `checkpoint`: the shared directory, retry, and feedback policy. Preparation
  does not create directories, acquire leases, or read/write checkpoints.

The interval must be 60s..24h; query and connect timeouts must be 1s..300s.
These durations require whole seconds. Shared bounds require 1..10,000 rows
per poll, a nonzero fetch size no larger than the row limit, and batch capacity
from one byte through 256 MiB. Catch-up permits 1..1024 pages and a duration
from 1ms through 5 minutes. Existing shared scraper and Oracle
configuration and checkpoint representations are unchanged.

### Cursor Modes

| Mode | Required source contract | Prepared parameters |
| --- | --- | --- |
| Snapshot | Complete bounded result, no cursor | Page limit plus one overflow probe |
| Scalar `int64` | Unique, non-null signed integer | Cursor, page limit |
| Scalar timestamp | Unique, non-null UTC `datetime2` | Timestamp, page limit |
| Composite | Non-null UTC `datetime2` plus signed-int tie-breaker; unique pair | Timestamp, tie-breaker, page limit |

Scalar cursors use `column`, `bind` (without `@`), and typed `initial`.
Composite cursors use `timestamp` (`column`, `bind`, `initial`, `timezone: UTC`)
and `tie_breaker` (`column`, `bind`, signed-integer `initial`).

Initial timestamps normalize to UTC with exactly seven fractional digits.
Offset-qualified RFC 3339 and timezone-less UTC date/time input are accepted
only when exactly representable as `datetime2`: years 0001..9999, no leap
seconds, and no sub-100ns precision loss. Input beyond nanosecond precision is
rejected rather than truncated.

**Preparation cannot prove commit-visible source ordering.** Incremental
sources must ensure no row becomes visible later at or below an acknowledged
cursor, and selected cursor values must not change after publication. A source
ID, increasing timestamp, or identity/sequence allocation order alone does not
prove this: transactions can commit out of order. A composite key distinguishes
equal timestamps but does not solve late visibility. Unsupported sources need a
publication-order guarantee, a suitable source view, or a different ingestion
strategy. Database permissions, live types, nullability, uniqueness, and source
semantics still require validation outside this layer.

## Restricted T-SQL

Only one `SELECT` from one schema-qualified table or view is accepted. Projections
must have 1..256 explicit, case-insensitively unique names. Cursor columns must
be projected directly, without aliases, using their configured spelling.

Scalar predicates must be exactly `column > @bind`, with ascending ordering by
that column. Composite predicates must have this strict keyset shape:

```sql
SELECT EVENT_TS, EVENT_ID, PAYLOAD FROM dbo.EVENTS
WHERE EVENT_TS > @last_timestamp
   OR (EVENT_TS = @last_timestamp AND EVENT_ID > @last_id)
ORDER BY EVENT_TS ASC, EVENT_ID ASC
```

Incremental queries cannot add filters or aggregation. Use a source view for
additional source-side filtering while preserving the source-ordering contract.
Snapshots support restricted expressions, filtering, grouping, and aggregates.
Allowed scalar functions are `ABS`, `COALESCE`, `ISNULL`, `LOWER`, `UPPER`, `LEN`,
and `DATALENGTH`; snapshot aggregates are `COUNT`, `COUNT_BIG`, `MIN`, `MAX`,
`SUM`, and `AVG`. Decimal casts require explicit valid precision and scale.

Writes, extra statements, comments, wildcard projections, operator-provided
`TOP`, offsets, hints, joins, subqueries, CTEs, table aliases, external sources,
user-defined functions, and undeclared parameters are rejected. Identifiers may
be regular or bracketed, not double-quoted. One trailing semicolon is allowed.
Input and canonical SQL are limited to 16 KiB; parsing is limited to 2048 tokens
and recursion depth 32.

Native SQL injects a bound `TOP` and maps configured cursor binds to positional
`@P1`, `@P2`, etc., without interpolating values. Snapshot limits include one
extra row to detect overflow. Token-aware rendering preserves quoted literals,
national strings, escaped bracket identifiers, and bind-like text inside them.
The future adapter must supply every value through typed parameter binding.

## Compatibility Fingerprint

The versioned BLAKE3 fingerprint includes database system, source ID, endpoint,
database, canonical SQL, and normalized cursor configuration. SQL whitespace and
DNS-name case are normalized. Changing source/query/cursor semantics changes
the fingerprint.

Polling budgets, timeouts, CA paths, checkpoint location, and retry tuning are
excluded. Credentials are not accepted by this configuration and cannot enter
the fingerprint. No shared checkpoint format is changed.

## Focused Development Checks

Run from `rust/otap-dataflow`:

```console
cargo check -p otel-arrow-dfe-contrib-nodes --no-default-features --features sql-server
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features sql-server --lib sql_server_receiver
cargo clippy -p otel-arrow-dfe-contrib-nodes --no-default-features --features sql-server --lib --tests -- -D warnings
```

These are deterministic configuration/compiler checks and require no SQL Server.
Tests also ensure this preparation layer registers no runtime factory. Linux
all-feature CI and the synchronized Windows CI/cache feature sets include it.

[part-1]: https://github.com/open-telemetry/otel-arrow/issues/4359
[part-2]: https://github.com/open-telemetry/otel-arrow/issues/4360
[part-3]: https://github.com/open-telemetry/otel-arrow/issues/4361
[rfc]: https://github.com/open-telemetry/otel-arrow/issues/3918
