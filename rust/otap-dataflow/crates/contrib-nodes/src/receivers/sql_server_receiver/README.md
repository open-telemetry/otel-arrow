# SQL Server Preparation and Driver Adapter

<!-- markdownlint-configure-file
{ "MD013": { "tables": false, "code_blocks": false } }
-->

This is the internal preparation and driver-integration layer for a SQL Server
query-to-logs receiver, not a runnable receiver. Enable `sql-server` explicitly
to expose validated configuration, T-SQL plans, and `SqlServerAdapter<C>` for
the shared scraper. The feature is excluded from all aggregate features, and
the top-level feature forwards only to contrib nodes.

**No `urn:otel:receiver:sql_server` factory is registered.** This layer has no
production TCP/TLS connector, node capability wiring, component inventory entry,
or live fixture. The adapter uses Tiberius through an injected
`SqlServerConnector` and obtains credentials from an injected `BasicAuthProvider`.
It does not enable `flat-file-user-pass-auth`. Do not use it in runtime YAML yet.

Part 1 [#4359][part-1] supplied preparation under [RFC #3918][rfc].
This adapter integration is Part 2 [#4360][part-2]; the production connector,
runtime activation, live fixtures, and deployment documentation remain
Part 3 [#4361][part-3].

## Preparation API

Deserialize `SqlServerReceiverConfig` with Serde. Successful preparation exposes
the logical `source_id`, validated connection settings, a shared `CompiledQuery`,
checkpoint policy, semantic fingerprint, and a `SqlServerStatement`.
The statement retains canonical operator SQL separately from generated native
SQL and ordered `SqlServerParameter` roles. Getters expose these values for the
adapter; deserialization and getters do not execute the query or access files.

Configuration errors and `Debug` output redact SQL, endpoints, and cursor
values. Explicit SQL and connection getters are not redacted and must not be
logged as diagnostics.

## Configuration Contract

Unknown fields, invalid types, and unsupported enum values are rejected rather
than replaced with defaults. The required top-level objects are `connection`,
`query`, `watermark`, and `checkpoint`, together with `source_id`.

- `source_id`: 1..256 ASCII letters, digits, underscores, hyphens, or dots.
- `connection`: explicit `host` (DNS name or IP) and `database`; optional `port`
  (default 1433), `connect_timeout` (default 10s), and the SQL Server-only `tls`
  settings below. Credentials, connection strings, and instance discovery are
  rejected.
- `query`: `statement`, `max_rows_per_poll`, and `max_batch_bytes` are required.
  `interval` defaults to 60s, `timeout` to 30s, and `fetch_size_rows` to
  `min(300, max_rows_per_poll)`. An explicit fetch size above the row limit is
  rejected, not reduced. Shared `catch_up` defaults are 32 pages and 10s.
- `watermark`: an explicit supported mode and initial cursor where applicable.
- `checkpoint`: the shared directory, retry, and feedback policy. Preparation
  does not create directories, acquire leases, or read/write checkpoints.

The interval must be 60s..24h; query and connect timeouts must be 1s..300s.
These durations require whole seconds. Shared bounds require 1..10,000 rows
per poll, a nonzero fetch size no larger than the row limit, and batch capacity
from one byte through 256 MiB. Catch-up permits 1..1024 pages and a duration
from 1ms through 5 minutes. Existing shared scraper and Oracle
configuration and checkpoint representations are unchanged.

### SQL Server TLS Settings

These `connection.tls` settings define the transport contract passed to the
injected connector. **The production TLS connector is not included here.**
They are SQL Server-specific; shared database/TLS configuration is unchanged.

| Field | Default | Prepared meaning |
| --- | --- | --- |
| `enabled` | `true` | Require TLS with no silent non-TLS fallback. |
| `trust_server_certificate` | `false` | When true, bypass both certificate-chain and hostname validation; controlled testing only. |
| `ca_file` | Omitted | Optional PEM CA path; validated syntactically, never opened during preparation. |
| `server_name_override` | Omitted | Optional DNS name or unbracketed IP for TLS identity/SNI, instead of `connection.host`; not a new connection address. |

Omitting `tls` or supplying an empty object preserves TLS and certificate/name
verification. Explicit null or nonboolean flags are rejected. Optional strings
must not be null; CA paths must be nonempty, and identity overrides follow the
same DNS/IP rules as `host`, rejecting whitespace, wildcards, URLs, ports,
userinfo, bracketed IPs, and control characters. DNS names normalize to lowercase.

`enabled: false` requests a non-TLS connection; SQL Server policy may refuse it
or require encryption. The Tiberius mapping is `NotSupported`, not `Off`
(which uses login-only TLS). A disabled TLS contract rejects `ca_file`,
`server_name_override`, and `trust_server_certificate: true` as inapplicable;
an omitted or explicit false trust flag is allowed.

With TLS enabled, both `ca_file` and `server_name_override` conflict with
`trust_server_certificate: true`. Preparation rejects these combinations
instead of silently ignoring CA trust or a requested identity. Tiberius's
native-TLS bypass disables SNI as well as **both certificate-chain and hostname
validation**. Use an override only with verified TLS; bypass alone remains an
explicit controlled-testing option. Configurations that previously combined
bypass and an override must remove the override or restore verification.

Connection accessors `tls_enabled()`, `trust_server_certificate()`, and
`server_name_override()` expose this contract. The override getter returns
`None` when omitted, meaning use `host()` for TLS identity.

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
`SUM`, and `AVG`. `COALESCE` requires at least two arguments, `ISNULL` exactly
two, and every other allowed function exactly one. `COUNT(*)` and `COUNT_BIG(*)`
are supported. Decimal casts require explicit valid precision and scale;
`ARRAY` modifiers are unsupported.

Snapshot projections, `HAVING`, and `ORDER BY` may wrap aggregates in scalar
functions, for example `COALESCE(SUM(AMOUNT), 0)`. Aggregates cannot contain
other aggregates, even through scalar wrappers. Wrapping an aggregate does not
allow it in incremental queries or `GROUP BY`.

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
The adapter supplies every value through typed parameter binding.

### Validation Boundary

Preparation enforces the receiver's structural rules: one restricted read-only
query, direct cursor projections, strict keyset predicates and ordering,
declared parameter bindings, and bounded paging. Passing these checks is **not
proof of general SQL validity**. For example, snapshot expressions such as
`WHERE COUNT(*) > 0` or `WHERE EVENT_ID` can pass preparation but SQL Server
will reject them.

`DriverAdapter::validate_query` validates native SQL and live metadata through
parameterized `sys.dm_exec_describe_first_result_set`, before polling and again
after the shared controller reconnects. SQL and parameter declarations are
bound values, not concatenated into the metadata query. SQL Server checks
general expression validity, column types, object existence, and permissions;
the adapter also checks the receiver's live cursor and metadata requirements.
Every fetch re-describes and compares the complete plan before reading rows.

Invalid SQL, incompatible metadata, and authorization failures are permanent,
non-retryable errors, not reconnect-loop triggers. An unsuccessful validation
leaves no executable plan. Deterministic TDS fixtures cover locally accepted
expressions such as the examples above, missing objects, denied permissions,
metadata-reported errors, and nullable cursors at startup and after reconnect.
Shared-controller tests assert no emitted page, unchanged checkpoints, and no
extra fetch or reconnect after permanent rejection. These protocol fixtures
exercise the real driver; real-server qualification remains in Part 3.

Database acceptance does not replace preparation's polling and checkpoint
rules, or prove the source ordering, uniqueness, and immutability requirements
described above.

## Compatibility Fingerprint

The versioned BLAKE3 fingerprint includes database system, source ID, endpoint,
database, canonical SQL, and normalized cursor configuration. SQL whitespace and
DNS-name case are normalized. Changing source/query/cursor semantics changes
the fingerprint.

Polling budgets, timeouts, all TLS transport settings (including encryption,
trust bypass, CA paths, and identity override), checkpoint location, and retry
tuning are excluded. Credentials are not accepted by this configuration and
cannot enter the fingerprint. No shared checkpoint format is changed.

## Adapter and Shared Delivery

`SqlServerAdapter<C>` implements the existing local `DriverAdapter` contract.
Its futures, session, metadata plan, and cancellation state stay on the pipeline
executor. The transport is `Send` only because Tiberius requires it; this does
not introduce a session pool or multithreaded polling runtime.

An embedding harness supplies the connector and local `BasicAuthProvider`,
then constructs the shared `DatabaseReceiver`. The shared scraper owns
scheduling, catch-up, memory-pressure admission, OTLP encoding, one pending
page, ACK/NACK replay, retry backoff, checkpoint commits, ownership, and
telemetry. It contains no Tiberius dependency. SQL Server adds only its
`microsoft.sql_server` database-system identity to the shared contract.

The adapter acquires current credentials on every connection/reconnection and
rejects credentials expiring within the shared usability margin. Connection
and credential acquisition share one absolute connect timeout. Refreshing
credentials does not reauthenticate an already-open session or reset progress.
Metadata validation and each metadata-plus-fetch operation have an absolute
query timeout. Driver timeout defaults are disabled in favor of these budgets.
Deterministic tests cover credential expiry, stalled acquisition, the shared
connect budget, and rotated credentials in the actual in-memory LOGIN7 exchange.

Cancellation handles are generation-specific: an old handle cannot cancel a
later operation. Response-completion state is retained by the adapter from
submission through exhaustion, including early metadata and conversion failures.
For an unfinished response, the adapter attempts ATTENTION under a one-second
cap. Unconfirmed cleanup discards the session, blocks subsequent operations,
and causes shutdown to fail so the shared runtime retains ownership, including
when an outer deadline drops submission, streaming, or cancellation. Errors
before submission or after confirmed completion do not cancel an idle session.
A connector must cancel on future drop and confirm any owned connection work
has stopped in `shutdown`; the default no-op shutdown is only appropriate for
connectors with no outstanding off-core work.

The published 0.13.0 driver's [token stream][tiberius-token-stream] defers server
errors until response EOF, so a returned server error also confirms completion.
Metadata-reported errors and conversion failures do not provide this guarantee.
Recheck this assumption when updating Tiberius.

Tiberius 0.13.0 has an unresolved [cross-message ATTENTION defect][attention-bug]:
a `DONE_ERROR` message followed by a separate `DONE_ATTN` message can fail
cancellation despite the server acknowledgement. Switching TLS backends does
not fix it. Deterministic tests exercise this exact two-message response,
missing/late acknowledgements, interrupted cleanup, and transport reset during
streaming using the published driver and real shared controller. Unconfirmed
cleanup discards the session, returns no partial page from the failed operation,
leaves its checkpoint unchanged, blocks reconnect/reuse, and keeps shutdown
unsuccessful so source ownership is quarantined until process exit.

This is fail-closed containment with an availability cost, not proof that
server-side work stopped. Operators must establish that prior work is stopped
and preserve the last durable checkpoint before a safe single-owner restart;
socket closure or process restart alone does not establish server termination.
The [upstream cancellation fix][attention-fix] remains a follow-up through a
published release. Successful acknowledged cleanup remains distinct: it permits
a fresh validated connection after a failed query. Acknowledged byte-limited
prefixes retain their synchronized connection instead, with fresh metadata
validation before the next fetch and a new cancellation generation.

### Recovery Boundaries

The shared controller retries a metadata/fetch timeout only after the server
acknowledges cancellation. The adapter returns a distinct `QueryTimeout` in
this case; the generic `Deadline` remains terminal for credential/connection
timeouts and other unconfirmed deadlines. Retry uses shared bounded backoff,
reconnects, revalidates metadata, and resumes at the last durable cursor.
Neither the failed operation nor its retry advances the checkpoint before a
downstream ACK.

Transport errors are retryable only when there is no outstanding submitted
response, or when ATTENTION has explicitly confirmed completion. An established
query is marked pending before submission, so even an error during its write
is treated conservatively: the adapter cannot infer from the error that no
bytes reached the server. A reset before the connector returns a session uses
the connector's cancellation/owned-work contract. A reset after submission or
during streaming requires cleanup confirmation; socket closure alone never
permits recovery.

If confirmed, the original allowlisted transport error is preserved for retry
on a fresh connection. If not confirmed, `UnconfirmedTransport` preserves the
driver's safe I/O error kind in both the operation and shutdown diagnostic,
but is terminal and keeps ownership quarantined. Tiberius 0.13.0 maps framed
read errors to `BrokenPipe`; the adapter retains the kind actually returned by
the driver, not sensitive raw error strings. Interrupted cleanup preserves this
cause as well as the pending-response state.

For a successful byte-limited prefix, Tiberius's acknowledged-ATTENTION contract
permits reuse. The next driver query drains any response tail before sending
new SQL, and the adapter re-describes metadata before fetching. A failed or
interrupted acknowledgement never reinserts the client. Failed queries still
discard their connection even after confirmation. Deterministic tests exercise
subsequent responses on the same logged-in session, stale cancellation handles,
and shared normalized/encoded prefix replay from the acknowledged cursor.

SQL, metadata, authorization, conversion, TLS, and unconfirmed failures are not
automatically retried. Raw driver error text is
discarded, keeping safe categories and numeric server codes. Tiberius can log
server error text and row contents, so its futures run with a suppressed tracing
subscriber; shared redacted diagnostics remain enabled.

## Values and Resource Limits

Result metadata is limited to 256 columns and 1 MiB of retained name/type/plan
data. Names must be unambiguous; cursor columns must be non-null and match the
configured native signed-integer range or exact `datetime2(p)` precision.
Supported body values include integers, bits/nulls, finite floats, strings,
bytes, GUID text, exact decimals through precision 38, date/time, datetime2, and
datetimeoffset. Raw money, legacy datetime types, XML, UDTs, variants, and other
unsupported types fail rather than being silently coerced. UTF-16 decoding is
strict. Decimal scale and temporal precision/offsets are preserved.

The driver stream is consumed incrementally. A snapshot must fit completely;
the limit-plus-one probe detects row overflow and no truncated prefix is
returned. An incremental normalized-byte limit may return a nonempty prefix
only after cancellation of the unread request. The next page uses a fresh
metadata description on the acknowledged reusable session. An oversized first
row fails explicitly.
`fetch_size_rows` is a cooperative decode-chunk size, not a TDS packet size.

**These are normalized/encoded page budgets, not driver-internal or RSS
limits.** Tiberius has already allocated native row/value buffers before the
adapter accounts for them. Driver-internal value/protocol bounds and broader
cancellation, fault, load, and deployment qualification remain readiness work.
TOP bounds returned rows, not necessarily database scan or sort work.

## Published Driver and Native TLS

Tiberius is optional and resolves from crates.io at 0.13.0 in the reviewed
lockfile, with default features disabled and `tds73,native-tls` enabled.
Both Tiberius and `tiberius-macros` are registry dependencies; no Git pin or
producer-local patch is required. The provider-neutral Rustls proposal is no
longer a prerequisite for this receiver.

SQL Server explicitly uses platform-native TLS: Schannel on Windows, OpenSSL
on Linux, and Security Framework on macOS. This scoped choice is independent
of the engine's Rustls `crypto-*` selection; other components' provider features
and dependencies are unchanged. The native driver does not require a
process-installed Rustls provider. Do not enable another Tiberius TLS backend
through feature unification.

Linux builds require suitable OpenSSL development libraries and discovery
tooling. Deployments require the corresponding native libraries and trusted
root store; linking strategy, library/root updates, and complete production
artifact dependency/governance qualification belong to Part 3. Windows uses
its system TLS libraries and certificate stores. Selecting native TLS is not
by itself a compliance certification or proof that the complete engine has no
other crypto backends.

The injected production connector must honor CA/name verification and the
explicit SQL Server test modes. Native connector setup and certificate loading
can be synchronous, so Part 3 must keep bounded, owned connection setup off the
local executor and confirm cleanup before releasing source ownership.
The deterministic connector sets `NotSupported` against in-memory protocol
bytes: these tests do **not** qualify native TLS handshakes, CA rotation, or
server identity verification. Those live checks remain in Part 3.

This adapter does not require customer-installed ODBC/JDBC drivers, a JVM, or
native database client. No server, container, or certificate-generation tools
are required for its deterministic tests.

## Focused Development Checks

Run from `rust/otap-dataflow`:

```console
cargo check -p otel-arrow-dfe-contrib-nodes --no-default-features --features sql-server
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features --features sql-server --lib sql_server_receiver
cargo clippy -p otel-arrow-dfe-contrib-nodes --no-default-features --features sql-server --lib --tests -- -D warnings
cargo test -p otel-arrow-dfe-contrib-nodes -p otel-arrow-dfe-scraper --no-default-features --features otel-arrow-dfe-contrib-nodes/sql-server,otel-arrow-dfe-contrib-nodes/oracle --lib
cargo tree -p otel-arrow-dfe-contrib-nodes --no-default-features --features sql-server --edges normal
```

These deterministic preparation, conversion, protocol, and shared-delivery tests
require no SQL Server. They also preserve the absence of a runtime factory.
Linux all-feature CI and the synchronized Windows CI/cache feature sets include
the opt-in adapter. Live TLS/SQL Server and runnable-engine evidence belong to
Part 3, not these tests.

[part-1]: https://github.com/open-telemetry/otel-arrow/issues/4359
[part-2]: https://github.com/open-telemetry/otel-arrow/issues/4360
[part-3]: https://github.com/open-telemetry/otel-arrow/issues/4361
[rfc]: https://github.com/open-telemetry/otel-arrow/issues/3918
[tiberius-token-stream]: https://docs.rs/crate/tiberius/0.13.0/source/src/tds/stream/token.rs
[attention-bug]: https://github.com/tiberius-rs/tiberius/issues/459
[attention-fix]: https://github.com/tiberius-rs/tiberius/pull/460
