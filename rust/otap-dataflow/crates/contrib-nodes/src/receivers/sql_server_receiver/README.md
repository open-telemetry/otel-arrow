# SQL Server Receiver

<!-- markdownlint-configure-file
{ "MD013": { "tables": false, "code_blocks": false } }
-->

This is a **scheduled query-to-logs receiver**, not CDC or
database-monitoring metrics. Enable `sql-server` explicitly to register
`urn:otel:receiver:sql_server`. The feature is excluded from all aggregates;
the root feature also enables `flat-file-user-pass-auth`.

`TcpConnector` defaults to verified native TLS (Windows Schannel/Linux OpenSSL),
platform certificate roots, and an optional additional CA file. The factory
requires one pipeline core and `basic_auth_provider`, acquires shared checkpoint
ownership, and initializes admission from process memory-pressure state.
The shared scraper provides scheduling, backpressure, feedback, and persistence;
it contains no vendor driver and Oracle behavior is unchanged.

The receiver follows the shared database polling architecture described in
[RFC #3918][rfc]. See the operating limits and deployment qualification
requirements below.

## Getting Started

See the [PowerShell demo](DEMO.md) for a short fixture setup, run, verification,
and cleanup walkthrough.
The complete pipeline is [`configs/sql-server-console.yaml`][example].
Build from `rust/otap-dataflow` with:

```powershell
cargo build --locked --no-default-features `
    --features sql-server,core-nodes,crypto-ring --bin df_engine
```

Use a dedicated SQL login with CONNECT access to the configured database and
SELECT access only to the intended table/view. Metadata discovery requires
permission to execute the same query; no blanket administrator grant is needed.
Integrated/Entra authentication is not supported. Credentials come from the
`basic_auth_provider` capability, never a connection string or inline password.

Set `SQL_SERVER_HOST`, `SQL_SERVER_PORT`, `SQL_SERVER_DATABASE`,
`SQL_SERVER_USERNAME`, `SQL_SERVER_PASSWORD_FILE`, and `SQL_SERVER_CA_FILE`
for the example. Omit `ca_file` from the YAML when platform trust is sufficient.
The password and CA file paths default to `/run/sql-server-secrets/password`
and `/run/sql-server-secrets/ca.pem` so offline configuration validation needs
no fixture environment. These are paths, not bundled secrets or trust bypasses:
provide the files or override the paths before starting the receiver.
Set `SQL_SERVER_STATE_DIR` to persistent local checkpoint storage (default
`./state/sql_server`). On Windows use this receiver setting directly:
optional managed `engine.state_dir` provisioning is Linux-only.
Run the built engine with `--config configs/sql-server-console.yaml` and an
appropriate loopback admin bind, such as `--http-admin-bind 127.0.0.1:0`.

The console demonstrates collection and feedback, **not durable delivery**.
Conditional at-least-once delivery requires a feedback-preserving path to a
durable destination, unique stable commit-visible cursors, unchanged selected
values, and source retention covering the replay window. Crash recovery may
replay accepted rows. There is no exactly-once or deduplication guarantee,
lookback, or automatic distributed partitioning.

Use **one active poller per logical source range** and stop-before-start
replacement. Checkpoint ownership does not detect equivalent queries under
different identities or state directories. Do not bypass locks. Snapshots
are periodic observations, not guaranteed replay of transient rows.

Receiver operation needs no separately installed ODBC/JDBC driver, JVM, SQL
tools, or native database client. Docker, Python, and the OpenSSL command-line
tool are fixture-only. Linux receiver binaries also need OpenSSL runtime
libraries and an up-to-date platform root store.
Distributors should provide feature-enabled binaries; users need no Rust
toolchain to run an existing binary.

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

These `connection.tls` settings are enforced by the production connector.
They are SQL Server-specific; shared database/TLS configuration is unchanged.

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
SQL Server native TLS does not require a process-installed Rustls provider.
Unrelated engine components still honor the selected `crypto-*` features.

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

The factory warns when either unsafe test mode is selected. These modes may
expose credentials/rows or permit server impersonation; use only non-sensitive
test data in an isolated environment. Defaults never enable them implicitly.

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
exercise the real driver; the explicitly requested live suite below separately
exercises the pinned SQL Server fixture.

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

### Recovery Boundaries

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

### Recovering a Quarantined Source

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

## Focused Development Checks

Run from `rust/otap-dataflow`:

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

These deterministic preparation, conversion, protocol, connector, factory, and
shared-delivery tests require no SQL Server. Linux all-feature CI and the
synchronized Windows CI/cache feature sets include the opt-in receiver.

## Live Fixture and Qualification

Live tests are ignored by default and **fail, rather than skip**, if explicitly
requested without fixture inputs. Python, OpenSSL, and Docker with Linux
containers are developer-only prerequisites. Windows setup discovers OpenSSL
bundled with Git for Windows when it is absent from PATH.

Starting the fixture accepts the SQL Server Developer container license terms;
review and authorize that locally first. Use a new private directory outside
the checkout, and do not proceed if setup fails.

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

Driver-internal preallocation/RSS limits, exhaustive fault/cancellation/load
tests, clean-machine packaging, and broader deployment qualification remain
readiness work. This does not qualify Windows-hosted SQL Server, other server
versions, Azure SQL/Managed Instance, or ARM64.

[rfc]: https://github.com/open-telemetry/otel-arrow/issues/3918
[tiberius-token-stream]: https://docs.rs/crate/tiberius/0.13.0/source/src/tds/stream/token.rs
[attention-bug]: https://github.com/tiberius-rs/tiberius/issues/459
[attention-fix]: https://github.com/tiberius-rs/tiberius/pull/460
[example]: ../../../../../configs/sql-server-console.yaml
