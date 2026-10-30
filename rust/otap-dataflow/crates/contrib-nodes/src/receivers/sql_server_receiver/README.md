# SQL Server Receiver

<!-- markdownlint-configure-file
{
  "MD013": { "tables": false, "code_blocks": false },
  "MD033": { "allowed_elements": ["details", "summary"] }
}
-->

## Metadata

| Property | Value |
| --- | --- |
| Type | `urn:otel:receiver:sql_server` |
| Crate | `otel-arrow-dfe-contrib-nodes` |
| Feature gate | `sql-server` (opt-in; excluded from all aggregate features) |
| Output | OTLP logs, one log record per selected database row |
| Execution | One query per receiver, in a single-core pipeline |
| Authentication | SQL login through `basic_auth_provider`; no Integrated or Entra authentication |
| Transport | Verified native TLS by default; no separately installed database driver |

## Overview

The SQL Server receiver polls a read-only T-SQL query, converts its rows into
structured OpenTelemetry logs, and saves acknowledged progress in filesystem
checkpoints. Choose a **snapshot** for periodic complete results, or a
**scalar/composite watermark** for incremental collection.

This is **scheduled query polling**, not Change Data Capture (CDC) or
database-monitoring metrics. Delivery is conditionally at least once, subject
to the [source and downstream requirements](#delivery-guarantees); it is not
exactly once.

SQL Server connectivity, query validation, and native type conversion live in
this receiver. Scheduling, backpressure, log encoding, downstream feedback,
and checkpoint storage use the shared database scraper described in
[RFC #3918][rfc].

## In This Guide

- [Getting started](#getting-started): prerequisites, build, and run.
- [Configuration](#configuration): example, authentication, field reference,
  and TLS.
- [Watermark modes](#cursor-modes): composite, scalar, and snapshot examples.
- [Delivery guarantees](#delivery-guarantees) and
  [values and resource limits](#values-and-resource-limits).
- [Recovery](#recovery-boundaries), including
  [quarantined sources](#recovering-a-quarantined-source).
- [Development checks](#focused-development-checks) and
  [live fixture qualification](#live-fixture-and-qualification).

## Getting Started

For a disposable database with generated credentials and certificates, follow
the [PowerShell demo](DEMO.md). It covers fixture setup, collection, checkpoint
resume, and cleanup.

For an existing database:

1. Choose a [watermark mode](#cursor-modes) and confirm the
   [source-ordering requirements](#delivery-guarantees).
2. Provision a dedicated SQL login with CONNECT access to the configured
   database and SELECT access only to the intended table or view. Metadata
   discovery requires permission to execute the same query, not blanket
   administrator access.
3. Provide a credential extension, trusted server certificate, and persistent,
   writable checkpoint directory.
4. Adapt the [complete console pipeline][example] and run one active poller
   for the source range.

### Build

An existing feature-enabled binary needs no Rust toolchain. To build your own,
run from `rust\otap-dataflow`:

```powershell
cargo build --locked --no-default-features `
    --features sql-server,core-nodes,crypto-ring --bin df_engine
```

The root `sql-server` feature also enables `flat-file-user-pass-auth`.
SQL Server uses OS-native TLS independently of the engine's `crypto-*`
selection; the example selects `crypto-ring` for other engine components.

No ODBC/JDBC driver, JVM, SQL tools, or native database client is needed.
Windows uses system Schannel. Linux builds need OpenSSL development libraries;
Linux deployments need matching OpenSSL runtime libraries and an up-to-date
platform root store. Docker, Python, and the OpenSSL command-line tool are
fixture-only prerequisites.

### Run

Set these environment variables for the [console pipeline][example].
The defaults below belong to that example, not the receiver schema.

| Variable | Example default | What to provide |
| --- | --- | --- |
| `SQL_SERVER_HOST` | `localhost` | Server DNS name or IP address. |
| `SQL_SERVER_PORT` | `1433` | Explicit TCP port; no instance discovery. |
| `SQL_SERVER_DATABASE` | `otap_fixture` | Database containing the selected table or view. |
| `SQL_SERVER_USERNAME` | `otel_reader` | Dedicated SQL reader login. |
| `SQL_SERVER_PASSWORD_FILE` | `/run/sql-server-secrets/password` | Path to a readable password file. |
| `SQL_SERVER_CA_FILE` | `/run/sql-server-secrets/ca.pem` | Path to a PEM CA bundle; omit `ca_file` from YAML if platform trust is sufficient. |
| `SQL_SERVER_STATE_DIR` | `./state/sql_server` | Persistent, writable checkpoint storage. |
| `SQL_SERVER_CATCH_UP_MAX_PAGES` | `32` | Maximum page fetches per polling cycle. |

The default secret paths allow offline configuration validation; they do not
provide bundled credentials or certificates. Supply the files or override the
paths before starting. On Windows, set `checkpoint.directory` directly through
`SQL_SERVER_STATE_DIR`; managed `engine.state_dir` provisioning is Linux-only.

From `rust\otap-dataflow`, run:

```powershell
.\target\debug\df_engine.exe `
    --config configs\sql-server-console.yaml `
    --http-admin-bind 127.0.0.1:0
```

On Linux, use `./target/debug/df_engine` with the same arguments and a
shell-appropriate line continuation.

**The console demonstrates collection and feedback, not durable delivery.**
Stop the previous receiver before starting a replacement. If shutdown is
unconfirmed, follow [quarantine recovery](#recovering-a-quarantined-source)
instead of restarting automatically.

## Configuration

### Example

The following is a receiver node under a pipeline's `nodes:` block, not a
complete pipeline. The [console pipeline][example] includes the credential
extension, single-core allocation, exporter, and connections.

```yaml
sql-events:
  type: urn:otel:receiver:sql_server
  capabilities:
    basic_auth_provider: sql-credentials
  config:
    source_id: sql-events
    connection:
      host: sql.example.com
      port: 1433
      database: events
      tls:
        enabled: true
        trust_server_certificate: false
    query:
      statement: >
        SELECT ID, EVENT_TS, PAYLOAD
        FROM dbo.OTAP_SQL_EVENTS
        WHERE EVENT_TS > @last_timestamp
          OR (EVENT_TS = @last_timestamp AND ID > @last_id)
        ORDER BY EVENT_TS ASC, ID ASC
      interval: 1m
      timeout: 30s
      max_rows_per_poll: 1000
      fetch_size_rows: 100
      max_batch_bytes: 4 MiB
      catch_up:
        max_pages: 32
        max_duration: 10s
    watermark:
      mode: composite
      timestamp:
        column: EVENT_TS
        bind: last_timestamp
        initial: "2026-01-01T00:00:00Z"
        timezone: UTC
      tie_breaker:
        column: ID
        bind: last_id
        initial: 0
    checkpoint:
      directory: ./state/sql_server
      on_nack: rewind
      on_permanent_nack: pause
      nack_backoff: 1s
      max_consecutive_failures: 5
```

This example relies on platform certificate trust. For a private CA, add
`connection.tls.ca_file` with the path to its PEM bundle.

### Authentication

Credentials come from the `basic_auth_provider` capability, never a connection
string or inline receiver password. Integrated/Entra authentication is not
supported.

For a mounted password file, configure the
[flat-file username/password extension](../../../../contrib-extensions/src/flat_file_user_pass_auth/README.md)
under the same pipeline's `extensions:` block:

```yaml
sql-credentials:
  type: urn:otel:extension:flat_file_user_pass_auth
  config:
    username: '${env:SQL_SERVER_USERNAME:-otel_reader}'
    password_secret_file: '${env:SQL_SERVER_PASSWORD_FILE:-/run/sql-server-secrets/password}'
    password_secret_file_refresh: 1m
```

For startup-only Azure Key Vault credentials, see the
[AKV Basic Auth provider](../../../../contrib-extensions/src/azure_key_vault_user_pass_auth/README.md)
and [complete AKV pipeline](../../../../../configs/sql-server-akv-console.yaml).
It uses the same `basic_auth_provider` binding and verified TLS.

New connections and reconnects acquire the current credentials. Refreshing
credentials does not reauthenticate an already-open session or reset progress.

### Top-Level Fields

All fields below belong inside the receiver's `config:` block. Unknown fields,
invalid types, and unsupported enum values are rejected rather than replaced
with defaults.

| Field | Default | Description |
| --- | --- | --- |
| `source_id` | **required** | Stable logical source identity: 1-256 ASCII letters, digits, underscores, hyphens, or dots. |
| `connection` | **required** | SQL Server endpoint and TLS settings. |
| `query` | **required** | One read-only statement with explicit row and byte limits. |
| `watermark` | **required** | `snapshot`, `scalar`, or `composite`; see [cursor modes](#cursor-modes). |
| `checkpoint` | **required** | Persistent state directory and feedback policy. |

### Connection

| Field | Default | Description |
| --- | --- | --- |
| `connection.host` | **required** | One DNS name or IP address, not a URL or connection string. |
| `connection.port` | `1433` | Nonzero TCP port; SQL Browser/instance discovery is not supported. |
| `connection.database` | **required** | Database name. |
| `connection.connect_timeout` | `10s` | `1s`-`300s`, in whole seconds; one absolute budget for credential acquisition and connection setup. |
| `connection.tls` | Verified TLS | SQL Server-specific settings in the [TLS reference](#sql-server-tls-settings). |

### Query

| Field | Default | Description |
| --- | --- | --- |
| `query.statement` | **required** | One read-only `SELECT` in the [restricted T-SQL shape](#restricted-t-sql), at most 16 KiB. |
| `query.interval` | `1m` | `1m`-`24h`, in whole seconds. Delay after a polling cycle, not between acknowledged catch-up pages. |
| `query.timeout` | `30s` | `1s`-`300s`, in whole seconds. Absolute deadline for metadata validation or a metadata-plus-fetch operation, not a downstream-ACK deadline. |
| `query.max_rows_per_poll` | **required** | `1`-`10000`. Incremental per-page ceiling; snapshot complete-result ceiling. |
| `query.fetch_size_rows` | `min(300, max_rows_per_poll)` | Cooperative decode-chunk size, no greater than the row limit; not a TDS packet size. |
| `query.max_batch_bytes` | **required** | `1` byte-`256 MiB`, as integer bytes or a size string such as `4 MiB`. Separate normalized-row and encoded-OTLP ceilings, not a combined memory limit. |
| `query.catch_up.max_pages` | `32` | `1`-`1024`, including empty probes. Set to `1` for single-page cycles. |
| `query.catch_up.max_duration` | `10s` | `1ms`-`5m`. Cycle budget for admitting another fetch, not an in-flight deadline. |

An explicit fetch size above the row limit is rejected, not reduced.
Interval and timeout values require whole seconds: `1.5m` is valid (90 seconds),
but `60.5s` is not.

Catch-up is ACK-gated, with one pending page. It stops on an empty page, budget
exhaustion, downstream/process-memory admission pressure, or a stop request.
Snapshots are periodic complete observations, not incremental catch-up.

### Checkpoint

| Field | Default | Description |
| --- | --- | --- |
| `checkpoint.directory` | **required** | Persistent, writable directory used for checkpoint storage and source ownership. Required in every mode, including snapshot. |
| `checkpoint.on_nack` | **required** | Must be `rewind`: re-query from durable progress after retryable negative acknowledgement. |
| `checkpoint.on_permanent_nack` | `pause` | `pause` stops this source until repaired and restarted; `retry` re-queries with capped backoff. |
| `checkpoint.nack_backoff` | **required** | `1ms`-`5m`; delay before replaying a retryable NACK. The example uses `1s`. |
| `checkpoint.max_consecutive_failures` | **required** | `1`-`1000`; consecutive durable checkpoint-write failures before termination. The example uses `5`. |

Configuration preparation does not create directories, acquire leases, or
read/write checkpoints. See [delivery guarantees](#delivery-guarantees) before
changing source identity or reusing state.

### SQL Server TLS Settings

TLS is required by default, using platform certificate roots and an optional
additional CA bundle. These settings belong to `connection.tls`.

| Field | Default | Meaning |
| --- | --- | --- |
| `enabled` | `true` | Require TLS with no silent non-TLS fallback. |
| `trust_server_certificate` | `false` | When true, bypass both certificate-chain and hostname validation; controlled testing only. |
| `ca_file` | Omitted | Optional additional PEM CA bundle, at most 4 MiB; never opened during preparation. |
| `server_name_override` | Omitted | Optional DNS name or unbracketed IP for TLS identity/SNI, instead of `connection.host`; not a new connection address. |

Omitting `tls` or supplying an empty object preserves TLS and certificate/name
verification. Explicit null or nonboolean flags are rejected. Optional strings
must not be null; CA paths must be nonempty, and identity overrides follow the
same DNS/IP rules as `host`, rejecting whitespace, wildcards, URLs, ports,
userinfo, bracketed IPs, and control characters. DNS names normalize to lowercase.

`enabled: false` requests a non-TLS connection; SQL Server policy may refuse it
or require encryption. A disabled TLS contract rejects `ca_file`,
`server_name_override`, and `trust_server_certificate: true` as inapplicable;
an omitted or explicit false trust flag is allowed.

With TLS enabled, both `ca_file` and `server_name_override` conflict with
`trust_server_certificate: true`. Preparation rejects these combinations
instead of silently ignoring CA trust or a requested identity. Certificate
bypass disables SNI as well as **both certificate-chain and hostname
validation**. Configurations that previously combined bypass and an override
must remove the override or restore verification.

**Use non-TLS or certificate-bypass modes only with non-sensitive test data in
an isolated environment.** The factory warns when either is selected. They may
expose credentials/rows or permit server impersonation. Defaults never enable
them implicitly.

## Cursor Modes

Choose the mode according to the source's publication and retention guarantees,
not just the available column types. Incremental initial values are
**exclusive lower bounds**; an existing compatible checkpoint takes precedence.

| Mode | Required source contract | Collection behavior |
| --- | --- | --- |
| `snapshot` | Complete bounded result, no cursor | Re-emit the complete current result each interval. |
| `scalar` with `int64` | Unique, non-null, stable signed integer with commit-visible ordering | Incremental pages after one acknowledged key. |
| `scalar` with `timestamp` | Unique, non-null, stable UTC `datetime2` with commit-visible ordering | Incremental pages after one acknowledged timestamp. |
| `composite` | Non-null, stable UTC `datetime2` plus signed-int tie-breaker; unique pair with commit-visible ordering | Incremental pages after the acknowledged timestamp/key pair. |

The fragments below replace `query` and `watermark` inside the receiver's
`config:` block. Keep `source_id`, `connection`, `checkpoint`, and the credential
binding from the [configuration example](#example). Use a new source/checkpoint
identity when changing modes; do not reuse incompatible progress.

### Composite: Timestamp and Tie-Breaker

Use composite mode when multiple rows can share a timestamp. The tie-breaker
must make the pair unique.

```yaml
query:
  statement: >
    SELECT EVENT_TS, EVENT_ID, PAYLOAD FROM dbo.EVENTS
    WHERE EVENT_TS > @last_timestamp
       OR (EVENT_TS = @last_timestamp AND EVENT_ID > @last_id)
    ORDER BY EVENT_TS ASC, EVENT_ID ASC
  max_rows_per_poll: 1000
  max_batch_bytes: 4 MiB
watermark:
  mode: composite
  timestamp:
    column: EVENT_TS
    bind: last_timestamp
    initial: "2026-01-01T00:00:00Z"
    timezone: UTC
  tie_breaker:
    column: EVENT_ID
    bind: last_id
    initial: 0
```

Both columns must be non-null. The timestamp uses UTC `datetime2`; the
tie-breaker uses a supported native signed-integer type. A composite key
distinguishes equal timestamps but **does not solve late commit visibility**.

### Scalar: One Typed Key

Use one unique, non-null, stable cursor. SQL Server supports `int64` and
`timestamp` scalar values, not unsigned integers or strings.

```yaml
query:
  statement: >
    SELECT EVENT_ID, PAYLOAD FROM dbo.EVENTS
    WHERE EVENT_ID > @last_key
    ORDER BY EVENT_ID ASC
  max_rows_per_poll: 1000
  max_batch_bytes: 4 MiB
watermark:
  mode: scalar
  column: EVENT_ID
  bind: last_key
  initial:
    type: int64
    value: 0
```

For a timestamp cursor, use `column: EVENT_TS` in the watermark and that same
column in the projection, predicate, and ordering. Replace `initial` with:

```yaml
initial:
  type: timestamp
  value: "2026-01-01T00:00:00Z"
```

A timestamp shared by multiple rows is not a valid scalar key; use composite
mode instead.

### Snapshot: Complete Result

Snapshot mode needs no cursor predicate, bind, or initial value. Ordering is
optional; this example uses it for predictable presentation.

```yaml
query:
  statement: >
    SELECT EVENT_ID, PAYLOAD FROM dbo.EVENTS
    ORDER BY EVENT_ID ASC
  max_rows_per_poll: 1000
  max_batch_bytes: 4 MiB
watermark:
  mode: snapshot
```

Each result must fit the row, normalized-memory, and encoded-OTLP limits
**completely**. Overflow rejects the observation without emitting a truncated
snapshot. Empty results emit no logs. Snapshot checkpoints retain ownership
and acknowledged revisions, not a resumable row offset. A retry queries current
data, not a stored copy of an earlier observation.

### Cursor Fields and Timestamp Precision

Scalar cursors require `column`, `bind` (without `@`), and typed `initial`.
Composite cursors require `timestamp` (`column`, `bind`, `initial`,
`timezone: UTC`) and `tie_breaker` (`column`, `bind`, signed-integer `initial`).
Bind names use ASCII letters, digits, and `_`, starting with a letter or `_`.

Initial timestamps normalize to UTC with exactly seven fractional digits.
Offset-qualified RFC 3339 and timezone-less UTC date/time input are accepted
only when exactly representable as `datetime2`: years 0001..9999, no leap
seconds, and no sub-100ns precision loss. Input beyond nanosecond precision is
rejected rather than truncated. Live cursor values and initial/checkpoint
values must also fit the native signed-integer range or `datetime2(p)`
precision.

## Restricted T-SQL

Only one `SELECT` from one schema-qualified table or view is accepted.
Projections must have 1-256 explicit, case-insensitively unique names. Cursor
columns must be projected directly, without aliases, using their configured
spelling.

Scalar predicates must be exactly `column > @bind`, with ascending ordering by
that column. Composite predicates must use the strict keyset shape in the
[composite example](#composite-timestamp-and-tie-breaker).

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
leaves no executable plan.

Database acceptance does not replace preparation's polling and checkpoint
rules, or prove the source ordering, uniqueness, and immutability requirements
described below.

## Output

Each selected row becomes one structured OTLP log:

| Element | Mapping |
| --- | --- |
| Resource attributes | `db.system.name=microsoft.sql_server` and `receiver.database.source_id=<source_id>`. |
| Event name | `database.query.row`. |
| Body | Selected column names and their converted values, including cursor columns. |
| Event timestamp | Composite timestamp or scalar timestamp cursor; observation time for integer scalar and snapshot modes. |
| Observed timestamp | Time the receiver observes the page. |

If a source timestamp is outside OTLP's unsigned nanosecond range, the record
uses observation time and retains the original value in its body. Selecting a
timestamp in snapshot mode or alongside an integer cursor does not make it the
event timestamp.

## Delivery Guarantees

**Conditional at-least-once delivery** requires all of the following:

- A feedback-preserving path to a durable destination. Console feedback alone
  is not durable storage.
- Unique, non-null, stable incremental cursors with commit-visible ordering:
  no row may become visible later at or below an acknowledged cursor.
- Unchanged selected values and source retention covering the replay window.
- One active poller per logical source range, with persistent checkpoints and
  stop-before-start replacement.

An increasing timestamp or identity/sequence allocation order alone does not
prove commit-visible ordering: transactions can commit out of order.
Preparation and live metadata checks cannot prove source uniqueness,
immutability, or publication ordering. Unsupported sources need a suitable
publication-order guarantee or source view, or a different ingestion strategy.

Progress advances only after downstream ACK and durable checkpoint
installation. Crash recovery may replay rows already accepted downstream.
There is **no exactly-once, deduplication, lookback, or automatic distributed
partitioning guarantee**.

Checkpoint ownership does not detect equivalent queries under different source
identities or state directories. Do not bypass locks or reset progress to work
around ownership. Snapshots are periodic observations, not guaranteed replay
of transient rows.

## Implementation Notes

The following details are intended for adapter maintainers and embedding
applications. Operators should use the configuration and recovery sections.

<details>
<summary>Preparation API, fingerprint, driver integration, and cancellation internals</summary>

### Preparation API

Deserialize `SqlServerReceiverConfig` with Serde. Successful preparation exposes
the logical `source_id`, validated connection settings, a shared `CompiledQuery`,
checkpoint policy, semantic fingerprint, and a `SqlServerStatement`.
The statement retains canonical operator SQL separately from generated native
SQL and ordered `SqlServerParameter` roles. Getters expose these values for the
adapter; deserialization and getters do not execute the query or access files.

Configuration errors and `Debug` output redact SQL, endpoints, and cursor
values. Explicit SQL and connection getters are not redacted and must not be
logged as diagnostics.

Connection accessors `tls_enabled()`, `trust_server_certificate()`, and
`server_name_override()` expose the TLS contract. The override getter returns
`None` when omitted, meaning use `host()` for TLS identity. Disabled TLS maps
to Tiberius `NotSupported`, not `Off` (which uses login-only TLS).

Native SQL injects a bound `TOP` and maps configured cursor binds to positional
`@P1`, `@P2`, etc., without interpolating values. Snapshot limits include one
extra row to detect overflow. Token-aware rendering preserves quoted literals,
national strings, escaped bracket identifiers, and bind-like text inside them.
The adapter supplies every value through typed parameter binding.

Deterministic TDS fixtures cover locally accepted expressions, missing objects,
denied permissions, metadata-reported errors, and nullable cursors at startup
and after reconnect. Shared-controller tests assert no emitted page, unchanged
checkpoints, and no extra fetch or reconnect after permanent rejection. These
protocol fixtures exercise the real driver; the explicitly requested live suite
separately exercises the pinned SQL Server fixture.

### Compatibility Fingerprint

The versioned BLAKE3 fingerprint includes database system, source ID, endpoint,
database, canonical SQL, and normalized cursor configuration. SQL whitespace and
DNS-name case are normalized. Changing source/query/cursor semantics changes
the fingerprint.

Polling budgets, timeouts, all TLS transport settings (including encryption,
trust bypass, CA paths, and identity override), checkpoint location, and retry
tuning are excluded. Credentials are not accepted by this configuration and
cannot enter the fingerprint. No shared checkpoint format is changed.

### Adapter and Shared Delivery

`SqlServerAdapter<C>` implements the existing local `DriverAdapter` contract.
Its futures, session, metadata plan, and cancellation state stay on the pipeline
executor. The transport is `Send` only because Tiberius requires it; this does
not introduce a session pool or multithreaded polling runtime.

The factory supplies `TcpConnector` and the local `BasicAuthProvider`,
then constructs the shared `DatabaseReceiver`. Embedding harnesses may still
inject a connector for deterministic tests. The shared scraper owns
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

Connection establishment uses at most one owned worker thread per receiver,
with no queued jobs or Tokio blocking-pool job. It owns synchronous DNS and CA
loading/native TLS setup, using the pipeline's existing Tokio reactor for
socket I/O. It resolves names on that worker rather than abandoning a Tokio
blocking-pool DNS task when the connect future is dropped.
After handoff, query I/O remains local. Dropping the connect future requests
cancellation; shutdown confirms completion before releasing ownership.
Synchronous OS/file work cannot be forcibly interrupted. If it does not finish,
the shared shutdown deadline quarantines ownership instead of permitting a
replacement receiver to overlap it.

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

</details>

## Recovery Boundaries

| Failure | Receiver behavior | Operator action |
| --- | --- | --- |
| Allowlisted connection/network failure with confirmed setup cleanup | Retry with bounded backoff, then revalidate metadata. | Restore network/server availability; check DNS if retries persist. |
| Query timeout with acknowledged cancellation | Retry on a fresh validated connection from the last durable cursor. | Investigate query cost or timeout settings. |
| Invalid SQL, metadata, authorization, credentials, conversion, or certificate/identity failure | Terminal; no automatic retry. | Repair the cause before restarting safely. |
| Unconfirmed cancellation, transport completion, or owned-worker cleanup | Block reconnect/reuse and quarantine source ownership. | Follow [quarantined-source recovery](#recovering-a-quarantined-source); do not bypass locks. |

Retry delays start at one second, double to a 30-second cap, and reset after a
successful poll. No failed operation or retry advances progress before ACK.

<details>
<summary>Detailed retry classification, transport cleanup, and redacted diagnostics</summary>

The shared controller retries a metadata/fetch timeout only after the server
acknowledges cancellation. The adapter returns a distinct `QueryTimeout` in
this case; other unconfirmed query deadlines remain terminal. Retry uses shared
bounded backoff, reconnects, revalidates metadata, and resumes at the last
durable cursor.
Neither the failed operation nor its retry advances the checkpoint before a
downstream ACK.

Initial connections and reconnects use the same recovery loop for the total
credential/connection deadline, socket timeouts, unavailable hosts/networks,
and allowlisted connection resets/refusals. DNS failures also retry, including
unclassified OS resolver causes: the platform can erase temporary resolver
failures into a generic I/O category. This fallback applies only to name
resolution, not arbitrary socket, TLS, or query errors. Known invalid-input,
permission, and unsupported resolver errors remain terminal. An unresolvable
but syntactically valid hostname can therefore keep retrying until DNS or the
configuration is corrected. Invalid configuration, unusable credentials,
certificate/identity/expiry failures, and unclassified TLS errors stay terminal.

Each failed connection attempt must confirm owned-worker shutdown within one
second before it can enter backoff. Failed or unconfirmed cleanup blocks
reconnect and keeps source ownership quarantined; it never admits a second
worker. Dropping setup also blocks reuse until shutdown confirms completion.
Retry delays start at one second, double to a 30-second cap,
and reset after a successful poll. Recovery revalidates the query and preserves
the last durably acknowledged cursor, with the same at-least-once semantics.

Transport errors are retryable only when there is no outstanding submitted
response, or when ATTENTION has explicitly confirmed completion. An established
query is marked pending before submission, so even an error during its write
is treated conservatively: the adapter cannot infer from the error that no
bytes reached the server. A reset before the connector returns a session uses
the connector's cancellation/owned-work contract. A reset after submission or
during streaming requires cleanup confirmation; socket closure alone never
permits recovery.

During connection setup, Tiberius turns native-TLS failures into text. The
connector separately records the first underlying socket read failure/EOF,
retaining only `io::ErrorKind`. This lets the shared controller retry an
allowlisted network failure after owned setup cleanup, without parsing native
error strings or broadly retrying TLS failures. Certificate, identity, expiry,
and unclassified TLS failures remain permanent. A failed write of a TLS
rejection alert is not evidence of a retryable handshake; write-only failures
whose structured cause is erased remain conservatively terminal.

If an established session is closed while idle, the next metadata request is
marked pending before submission. A failed socket write/read cannot prove
whether that request ran. Without acknowledged ATTENTION the receiver stops
and quarantines ownership, preserving the last acknowledged checkpoint. This
accepted availability cost is covered over the production native-TLS connector,
not treated as safe automatic reconnect.

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

SQL, metadata, authorization, conversion, certificate, and unconfirmed failures
are not automatically retried. Raw driver error text is
discarded, keeping safe categories and numeric server codes. Tiberius can log
server error text and row contents, so its futures run with a suppressed tracing
subscriber; shared redacted diagnostics remain enabled.

</details>

## Recovering a Quarantined Source

1. Stop automatic replacement/restart for this source range. Preserve the error,
   last durable checkpoint, configuration identity, and restricted diagnostics.
   Do not delete locks, reset progress, or launch a differently named poller.
2. Have the database operator identify the prior receiver's session/request
   using the dedicated login, database, application name `otel-arrow-sql-server`,
   and deployment connection records. Check active requests and rollback state
   through the operator's approved SQL Server administration procedure.
3. Wait for confirmed completion or have the authorized database operator
   terminate the specific request/session and verify termination/rollback.
   Socket closure, a client timeout, or killing/restarting the engine alone
   does **not** prove that prior server work stopped. If this cannot be
   established, keep the source stopped and escalate; do not bypass quarantine.
4. After server-side quiescence is established, stop the previous engine process
   completely so its quarantined lease is released. Start exactly one replacement
   with the same source identity, compatible query, and preserved checkpoint.
   Validate source retention and unchanged replayable rows; expect duplicates
   if downstream accepted rows before their checkpoint was installed.

The fixture's `disconnect-readers` command is narrowly scoped fault injection
for its disposable database, **not** a general production recovery command.

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

The receiver uses the published Tiberius 0.13.0 driver with native TLS:
Schannel on Windows, OpenSSL on Linux, and Security Framework on macOS.
It needs no customer-installed ODBC/JDBC driver or Rustls provider.

Binary distributors must supply the matching native libraries and root store,
and qualify the actual engine feature set and deployment image.

<details>
<summary>Dependency features, platform packaging, and governance evidence</summary>

Tiberius is optional and resolves from crates.io at 0.13.0 in the workspace
lockfile, with default features disabled and `tds73,native-tls` enabled.
Both Tiberius and `tiberius-macros` are registry dependencies; no Git pin or
producer-local patch is required.

SQL Server explicitly uses platform-native TLS: Schannel on Windows, OpenSSL
on Linux, and Security Framework on macOS. This scoped choice is independent
of the engine's Rustls `crypto-*` selection; other components' provider features
and dependencies are unchanged. The native driver does not require a
process-installed Rustls provider. Do not enable another Tiberius TLS backend
through feature unification.

Linux builds require suitable OpenSSL development libraries and discovery
tooling. Deployments require the corresponding native libraries and trusted
root store. The default non-vendored Linux build discovers system OpenSSL
through `openssl-sys`/`pkg-config`; distributors must record the actual linked
`libssl`/`libcrypto` ABI in the built artifact rather than assume a version from
Cargo.lock. Supply the matching runtime packages and CA bundle in the deployed
image. The OS/image owner patches those libraries and roots; a static/custom
build shifts library updates and relinking to the binary distributor.

Windows uses system Schannel/CryptoAPI libraries and certificate stores,
serviced by Windows update policy. It does not need an OpenSSL SDK or DLL
for the SQL Server path. An embedding process does not need a Rustls provider
for SQL Server; the sample engine still selects `crypto-ring` for unrelated
components. Test-only Rustls/rcgen dependencies belong to the loopback peer,
not the production SQL Server dependency path.

Record both `cargo tree --edges normal` for `sql-server` alone and for the full
intended engine feature set, plus actual binary imports (`dumpbin /dependents`
on Windows, `readelf -d`/`ldd` in a trusted Linux build environment). Retain
crate/native-library versions, linking mode, image/OS and root-store ownership,
scan/SBOM results, and applicable organizational component-governance approval.
`cargo deny` alone does not scan system OpenSSL or establish compliance.
Using this component or an OS-native crypto library does not imply applicable
external governance approval.

The production connector honors CA/name verification and the explicit SQL
Server test modes without changing checkpoint fingerprints. Its owned
connection worker keeps synchronous CA loading/native setup off the local
executor. The loopback TLS tests use the real native client and a Rustls test
peer, not the TLS-disabled injected protocol connector. They qualify actual
handshakes, expiry/replacement/identity checks and encrypted cancellation
containment, but do not substitute for the pinned SQL Server live suite.

This adapter does not require customer-installed ODBC/JDBC drivers, a JVM, or
native database client. No server, container, or certificate-generation tools
are required for its deterministic tests.

</details>

## Focused Development Checks

Deterministic preparation, conversion, protocol, connector, factory, and
shared-delivery tests require no SQL Server. Linux all-feature CI and the
synchronized Windows CI/cache feature sets include the opt-in receiver.

Run from `rust\otap-dataflow`:

```powershell
python -B -m unittest discover `
    -s crates\contrib-nodes\src\receivers\sql_server_receiver\ci `
    -p test_sql_server_fixture.py
cargo check -p otel-arrow-dfe-contrib-nodes --no-default-features `
    --features sql-server
cargo test -p otel-arrow-dfe-contrib-nodes --no-default-features `
    --features sql-server --lib sql_server_receiver
cargo clippy -p otel-arrow-dfe-contrib-nodes --no-default-features `
    --features sql-server --lib --tests -- -D warnings
cargo test -p otel-arrow-dfe-contrib-nodes -p otel-arrow-dfe-scraper `
    --no-default-features --features sql-server,oracle,postgresql --lib
cargo tree -p otel-arrow-dfe-contrib-nodes --no-default-features `
    --features sql-server --edges normal
```

## Live Fixture and Qualification

Live tests are ignored by default and **fail, rather than skip**, if explicitly
requested without fixture inputs. Python, OpenSSL, and Docker with Linux
containers are developer-only prerequisites. Windows setup discovers OpenSSL
bundled with Git for Windows when it is absent from PATH.

Starting the fixture accepts the SQL Server Developer container license terms;
review and authorize that locally first. Use a new private directory outside
the checkout, and do not proceed if setup fails. For an interactive collection
demo rather than the full test suite, use [DEMO.md](DEMO.md).

<details>
<summary>Run the live suite, manage fixture cleanup, and exercise certificate rotation</summary>

```powershell
# From the repository root; use a new directory for each run.
python rust\otap-dataflow\crates\contrib-nodes\src\receivers\sql_server_receiver\ci\sql-server-fixture.py start `
    --directory C:\temp\otel-sql-fixture --allow-unencrypted
if ($LASTEXITCODE -ne 0) { throw 'Fixture setup failed' }
$fixture = Get-Content C:\temp\otel-sql-fixture\fixture.json -Raw | ConvertFrom-Json
foreach ($entry in $fixture.env.PSObject.Properties) {
    Set-Item -Path "Env:$($entry.Name)" -Value $entry.Value
}
Push-Location rust\otap-dataflow
try {
    cargo test --locked -p otel-arrow-dfe-contrib-nodes --no-default-features `
        --features sql-server sql_server_receiver::live_tests `
        -- --ignored --nocapture --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'Live tests failed' }
} finally {
    Pop-Location
    python rust\otap-dataflow\crates\contrib-nodes\src\receivers\sql_server_receiver\ci\sql-server-fixture.py stop `
        --directory C:\temp\otel-sql-fixture
}
```

The fixture pins SQL Server 2022 build 16.0.4295.3 to image digest
`sha256:4402d880dd4c34bfa7d8705e56a86cd6c88da80a1f6bbbe741f999e76264a090`.
It binds an ephemeral loopback port, generates short-lived certificates and
isolated credentials, and provisions a SELECT-only reader. Setup also verifies
TLS. Stop verifies container ownership and removes only owned resources and
generated secrets. Keep its directory private; never upload credential/key
files. Certificates expire after two days.

Recovery metadata is saved before generating secrets or launching Docker, and
manifest updates are atomic. If setup fails before launch, generated secrets
are removed without contacting Docker. After a launch attempt, cleanup verifies
ownership and targets the inspected container ID, including when `docker run`
times out without returning an ID. If inspection or removal fails, setup reports
unconfirmed cleanup and retains the private directory and recovery metadata.
Retry `stop` with the same directory once Docker is available. A failed inspection
does not prove the container is absent; if it still cannot be inspected, verify
the reserved container name in `fixture.json` and the launch outcome with a
Docker administrator before discarding recovery files. Do not reuse that
directory for another launch.

Omitting `--allow-unencrypted` enforces TLS on the server (the demo default).
The explicit flag permits the full suite's non-TLS cases; secure-default tests
still verify TLS. Use a fresh three-row fixture for tests, not one modified
during the interactive demo.

Eight live scenarios cover polling modes/type fidelity, CA/hostname rejection,
connection cancellation, factory NACK/replay/ACK/restart, explicit TLS policies,
native TLS without a Rustls provider, certificate rotation state, and a healthy
session disconnected between completed polls. Run the live suite serially on
an exclusively owned fixture: the idle-fault test deliberately disconnects its
reader sessions.

To exercise replacement/expiry, stop all receiver clients, then run
`sql-server-fixture.py rotate-certificate --directory <fixture> --certificate
replacement` (or `expired`, then `valid`). This restarts only the owned
container and does not change host trust. Docker may reassign the ephemeral
host port when it restarts; rotation refreshes the loopback-only port in the
manifest. Reload `fixture.json` environment values and run only
`live_certificate_rotation_state` after each rotation.
Expired-certificate readiness uses an explicit bypass solely for the fixture
administrator probe; the receiver test must reject expiry. Restore `valid`
before any other live scenario or engine trial. Stop removes all generated
original/replacement/expired private keys.

The component-local [`ci/integration-test.json`](ci/integration-test.json)
registers an advisory Linux live job with the shared contrib integration
workflow. It runs the serial live suite, rotates replacement/expired/valid
certificates, and then builds/runs the exact example YAML. The engine trial
requires console payload output, an acknowledged ID-3 checkpoint, and clean
shutdown. The trial sets `SQL_SERVER_CATCH_UP_MAX_PAGES=1` so shutdown after
that checkpoint cannot race an immediately started catch-up query. The example
defaults to 32 pages when the variable is omitted; mid-query cancellation and
quarantine remain covered separately by the receiver tests.
It stores only a non-secret result summary in CI artifacts; fixture
credentials/keys and engine output stay outside the upload directory. The
cleanup command always attempts owned removal. Windows CI includes real
Schannel loopback/protocol coverage, not a hosted live SQL Server job.

Live SQL Server and full engine qualification must be executed explicitly in
the target Windows/Schannel and Linux/OpenSSL environments. Passing ignored
test compilation, dependency graphs, or a loopback peer is not a live-server
pass. Fixture startup accepts the license and requires the operator's approval.

</details>

[rfc]: https://github.com/open-telemetry/otel-arrow/issues/3918
[tiberius-token-stream]: https://docs.rs/crate/tiberius/0.13.0/source/src/tds/stream/token.rs
[attention-bug]: https://github.com/tiberius-rs/tiberius/issues/459
[attention-fix]: https://github.com/tiberius-rs/tiberius/pull/460
[example]: ../../../../../configs/sql-server-console.yaml
