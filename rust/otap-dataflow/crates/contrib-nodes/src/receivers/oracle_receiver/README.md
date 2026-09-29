# Oracle Receiver

This receiver is currently pre-1.0. Its public API may evolve between minor
releases.

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `urn:otel:receiver:oracle`
- Crate: `otel-arrow-dfe-contrib-nodes`
- Feature gate: `oracle-receiver` (opt-in; not enabled by `contrib-receivers`)
- Stability: Experimental
- Output: OTLP logs, one log record per selected database row
- Execution: One query per receiver, in a single-core pipeline

## Overview

The Oracle receiver polls an operator-authored, read-only SQL query using a
composite cursor: a timestamp and a signed integer tie-breaker. It maps returned
rows to structured OTLP logs and saves acknowledged progress in filesystem
checkpoints.

Oracle connection management, SQL validation, and native type conversion live in
this receiver. Scheduling, log encoding, downstream ACK/NACK handling, and
checkpoint storage use the [shared scraper][scraper].

This is scheduled query polling, **not Change Data Capture (CDC)**. Delivery is
conditionally at least once, subject to the
[source-data and downstream requirements](#delivery-guarantees). It is not
exactly once and does not implement every capability proposed in the
[database receiver RFC][database-rfc].

## Getting Started

1. Prepare a source with stable rows and a unique, commit-ordered composite
   cursor. Read [Delivery guarantees](#delivery-guarantees) before choosing a
   timestamp column.
2. Install [Oracle Instant Client](#oracle-instant-client-installation) on the
   engine host and provision a least-privileged account with read access to the
   selected data.
3. Mount the username and password as files readable by the engine, and provide
   a persistent, writable checkpoint directory.
4. Adapt the [complete pipeline example](#full-configuration), then use the
   [Windows or Linux commands](#running) to start it on one core.

Fields marked **required** must be provided. Collection interval, query timeout,
fetch size and catch-up budgets have defaults. Unknown fields are rejected.

## Configuration

### Native Schema and Design Proposals

This reference documents the implemented native OTEL-Arrow schema, matching the
updated **composite** configuration design. An upstream deployment system may
translate customer settings into this YAML and provision credentials and
checkpoint storage; pre-DCR/DCR translation and mount orchestration are separate
deployment work, not implemented by this receiver.

Do not copy proposed or older schema variants into the native configuration:

- `credentialReference`, `tnsAdminReference`, `startAt`, and destination `stream`
  are not native receiver fields. Credentials use file paths; destination
  routing belongs to the surrounding pipeline/exporter configuration.
- `connection.instant_client_dir` is required, even if the loader can already
  find the Oracle libraries. `connection.tns_admin` is not supported.
- Watermarks use nested `timestamp` and `tie_breaker` objects, explicit `bind`
  and `initial` values, and `mode: composite`; flat `timestamp_column`,
  `tie_breaker_column`, and `start_at` fields are not accepted.
- Scalar and snapshot configurations are proposals, not implemented modes.
- A deployment system may choose defaults, but the native schema requires all
  fields marked required, including `max_batch_bytes` and `nack_backoff`.

### Top-Level Fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `source_id` | string | **required** | Non-empty logical source identity, at most 256 UTF-8 bytes. Used in checkpoints and emitted telemetry; do not include credentials or sensitive connection details. |
| `connection` | object | **required** | Oracle connection string and Instant Client directory. |
| `authentication` | object | **required** | Paths to mounted username and password files. |
| `query` | object | **required** | One SQL statement and its polling, row, byte, and timeout limits. |
| `watermark` | object | **required** | Composite timestamp/tie-breaker cursor definition and initial position. |
| `checkpoint` | object | **required** | State directory, NACK policy, replay backoff, and checkpoint-write failure limit. |

These fields belong inside a node's `config:` block, not at the pipeline root.
There is no `queries` list, `query.name`, configurable `query.output`, or
`query.error_policy` field.

### Connection

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `connection.connect_string` | string | **required** | Non-empty, single-address Easy Connect string, for example `//database.example.com:1521/ORCL`. |
| `connection.instant_client_dir` | string | **required** | Non-empty path to the extracted Instant Client libraries on the engine host. All Oracle receivers in one process must use the same directory. |

The adapter adds `connect_timeout` and `transport_connect_timeout`, each capped
at 10 seconds and bounded by the configured query timeout, rounded to at least
one second. It rejects connection descriptors beginning with `(`, multiple
comma-separated addresses, and user-supplied `connect_timeout`,
`transport_connect_timeout`, `retry_count`, or `retry_delay` options.
These bounds do not establish a whole-poll deadline.

Setting `instant_client_dir` does not replace operating-system library-loader
setup. See [Oracle Instant Client installation](#oracle-instant-client-installation).
Changing this directory requires a process restart because client initialization
is process-global.

### Authentication

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `authentication.username_file` | string | **required** | Path to a regular UTF-8 file containing the Oracle username. |
| `authentication.password_file` | string | **required** | Path to a regular UTF-8 file containing the Oracle password. |

Each file must be non-empty after removing trailing CR/LF characters and no
larger than 64 KiB. Restrict file permissions to the engine's identity. Configure
file paths, not secret values, in YAML or environment-variable substitutions.

Files are read when opening a connection. Replacing a mounted secret does not
invalidate the checkpoint, but an already-open session continues using its
existing credentials; new values are used on the next connection.

There are no receiver-level shared-auth-extension, wallet, or TLS configuration
blocks. Do not assume these examples establish encrypted transport; qualify
your Oracle client and connection security separately for the deployment.

### Query

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `query.statement` | string | **required** | One SELECT in the [supported query shape](#required-query-shape), at most 16 KiB after trimming the optional trailing semicolon and surrounding whitespace. |
| `query.interval` | duration string | `1m` | Between `1m` and `24h`, inclusive, with whole-second precision. `1.5m` is valid (90 seconds); `60.5s` is not. Delay after a poll cycle ends; acknowledged catch-up pages do not wait this interval. |
| `query.fetch_size_rows` | integer | `300` | Between `1` and `10000`, and no greater than `max_rows_per_poll`. Target rows per native fetch, further capped by row and byte budgets. |
| `query.max_rows_per_poll` | integer | **required** | Between `1` and `10000`. Maximum rows fetched per page, including catch-up pages. |
| `query.max_batch_bytes` | byte count or size string | **required** | Between 1 byte and 256 MiB, inclusive. For example, `10485760` or `10 MiB`. Applied separately to normalized row storage and encoded OTLP. |
| `query.timeout` | duration string | `30s` | Positive whole seconds from `1s` through `5m`. Native Oracle call timeout, not a whole-query, whole-poll or downstream-ACK deadline. |
| `query.catch_up.max_pages` | integer | `32` | Between `1` and `1024`. Maximum page fetches per cycle, including empty probes. Set to `1` for single-page cycles. |
| `query.catch_up.max_duration` | duration string | `10s` | Between `1ms` and `5m`. Elapsed cycle budget for admitting another fetch, not an in-flight query deadline. |

Catch-up remains ACK-gated with one pending page. It stops on an empty page,
budget exhaustion, downstream admission pressure, or a stop request. The receiver
also observes the pipeline's process-memory admission state.

Omitting the three operational settings selects `interval: 1m`, `timeout: 30s`,
and `fetch_size_rows: 300`. Explicit blank, null, negative or invalid values are
rejected, not replaced with defaults or rounded. A configured page limit below
300 requires an explicitly smaller fetch size. Product configuration may hold
the timeout and fetch size fixed while still using these native defaults.

There is no independent `query.max_normalized_bytes` setting. For example,
`max_batch_bytes: 10 MiB` caps each of the normalized-row and encoded-payload
representations at 10 MiB separately; it does not cap their combined memory use.

The encoder emits the largest non-empty row prefix that fits the encoded
ceiling. The checkpoint candidate is the last row actually emitted. Remaining
rows are queried again after that cursor. If the first row alone exceeds either
budget, the receiver fails explicitly rather than skipping the row.
Native fetch buffers, metadata, allocator overhead, and intermediate
representations are additional memory; this is not a process-RSS limit.

Earlier pre-release configurations must rename `query.fetch_size` to
`query.fetch_size_rows`. The old name is rejected, not silently ignored.
The 300-row default, native fetch bounds, and checkpoint fingerprint are unchanged.

### Watermark

Only `watermark.mode: composite` is supported. The timestamp and tie-breaker
form an exclusive lower bound: the first poll selects rows strictly after the
configured initial pair; subsequent polls use the last committed pair.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `watermark.mode` | string | **required** | Must be `composite`; `scalar` and `snapshot` are not implemented. |
| `watermark.timestamp.column` | string | **required** | Unquoted Oracle identifier for the timestamp cursor column. |
| `watermark.timestamp.bind` | string | **required** | Named parameter without `:`. ASCII letters, digits, and `_` only, starting with a letter or `_`. |
| `watermark.timestamp.initial` | string | **required** | Oracle timestamp text, such as `"1970-01-01 00:00:00"`, with at most nine digits per numeric component. Fractional precision above nine digits is rejected, not truncated. Used only when no checkpoint exists. |
| `watermark.timestamp.timezone` | string | **required** | Must be `UTC`, ignoring ASCII case. The adapter sets its Oracle session timezone to UTC. |
| `watermark.tie_breaker.column` | string | **required** | Unquoted Oracle identifier for the signed integer tie-breaker column. |
| `watermark.tie_breaker.bind` | string | **required** | Named parameter with the same syntax as the timestamp bind. |
| `watermark.tie_breaker.initial` | signed 64-bit integer | **required** | Initial tie-breaker; zero is valid but is not a default. |

Cursor columns and bind names must each be distinct, ignoring ASCII case.
Cursor column identifiers must start with an ASCII letter, followed by letters,
digits, `_`, `$`, or `#`. Quoted or qualified cursor identifiers are rejected.

Both cursor columns must be declared `NOT NULL`. Supported timestamp metadata
is Oracle `DATE`, `TIMESTAMP`, `TIMESTAMP WITH TIME ZONE`, or
`TIMESTAMP WITH LOCAL TIME ZONE`. Tie-breaker metadata must be `Int64` or
`NUMBER(p,0)` with `1 <= p <= 18`; unconstrained, fractional, unsigned, and
wider numeric cursor types are rejected.

The tie-breaker must be unique within each timestamp group. The receiver checks
column presence, types, and nullability, but cannot prove source uniqueness,
commit ordering, or immutability.

### Checkpoint

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `checkpoint.directory` | string | **required** | Non-empty state directory without `..` components. Supports the `${engine.state_dir}` prefix; keep this directory persistent across restarts. |
| `checkpoint.on_nack` | string | **required** | Only `rewind`: retain committed progress and re-query after a matching retryable NACK. |
| `checkpoint.on_permanent_nack` | string | `pause` | Pause until repair and restart, or explicitly `retry` with shared capped backoff. Neither policy skips rows. |
| `checkpoint.nack_backoff` | duration string | **required** | Between `1ms` and `5m`, inclusive. Fixed NACK replay delay; also used between checkpoint-write retries. |
| `checkpoint.max_consecutive_failures` | integer | **required** | Between `1` and `1000`. Consecutive checkpoint-write failures before termination; not a query-error or NACK retry limit. |

See [Checkpoints and ownership](#checkpoints-and-ownership) for identity,
durability, and recovery behavior.

### Required Query Shape

The following example matches the cursor configuration in the complete pipeline:

```sql
SELECT EVENT_ID, EVENT_TS, PAYLOAD
FROM OTAP_ORACLE_EVENTS
WHERE (
    EVENT_TS > :last_timestamp
    OR (
        EVENT_TS = :last_timestamp
        AND EVENT_ID > :last_tie_breaker
    )
)
ORDER BY EVENT_TS ASC, EVENT_ID ASC
```

Configuration validation requires:

- One SELECT, with no SQL comments or extra statements. One final semicolon is
  accepted and removed.
- No subqueries or set operations such as `UNION ALL`, `INTERSECT`, or `MINUS`.
- Exactly the top-level predicate
  `timestamp > :timestamp OR (timestamp = :timestamp AND id > :id)`, optionally
  enclosed in one additional pair of parentheses. Arbitrary extra predicates
  are not supported.
- Both configured binds as real parameter tokens, not string literals or
  prefixes of other parameter names.
- A final outer `ORDER BY <timestamp> ASC, <tie_breaker> ASC`. Trailing clauses,
  including `FOR UPDATE`, are rejected.

The adapter binds cursor values through named parameters, uses read-only
transactions, and checks live result metadata before polling. Result-column
names must be unique, ignoring ASCII case.

**Source-query responsibility:** selected cursor values must represent the same
values used in the predicate and ordering. Do not alias a different column or
expression to a configured cursor name. The validator does not prove that
projection identity, nor does it prove that every selected database function
is free of side effects. Use a least-privileged account and a reviewed query.

### Output Mapping

| Output | Mapping |
| --- | --- |
| Signal | Logs only; one `LogRecord` per emitted row. |
| Body | Structured key-value body containing all selected columns under their returned names. |
| Event timestamp | Configured timestamp cursor column; observation time is used if the source time cannot fit OTLP's unsigned nanosecond range. |
| Observed timestamp | Time the page was observed by the receiver. |
| Severity / event name | `INFO` / `database.query.row`. |
| Resource attributes | `db.system.name=oracle.db` and `receiver.database.source_id=<source_id>`. |
| Log attributes | `receiver.database.source_id` and `receiver.database.query.name`, both using `source_id` because there is one query per receiver. |

Precision-sensitive values such as Oracle NUMBER values are emitted as decimal
strings; binary values remain OTLP bytes. Timestamp and interval body values are
text, and NULL maps to an empty OTLP `AnyValue`. Unsupported native types and
non-finite floating-point values fail explicitly rather than using a lossy
fallback. There is no configurable per-column mapping or separate metrics/traces
output in this receiver.

**Database text limitation:** the Oracle driver's character-to-`String`
conversion can replace invalid UTF-8 sequences with replacement characters
instead of returning an error. The adapter propagates conversion errors that
the driver reports, but does not independently detect or reject these
replacements. Strict invalid-UTF-8 rejection for database text remains follow-up
work. Credential files are handled separately and require valid UTF-8; invalid
credential bytes are rejected.

### Schema Changes

Changing the query's result schema while the receiver is running is unsupported.
The Oracle driver caches column metadata on reused statements, so the per-poll
metadata comparison may not detect changed column names or types.

Stop the receiver instance before making these schema changes, then restart it
to create a new connection, prepare the query, and validate fresh metadata.
Restarting does not make unsupported types valid. If SQL or cursor definitions
also change, resolve checkpoint-compatibility errors explicitly; do not delete
checkpoints to bypass validation.

## Delivery guarantees

Delivery is **at least once only when all requirements below hold**. Replays and
duplicates are possible; exactly-once delivery is not provided.

### Source-Data Requirements

| Requirement | Why it is necessary |
| --- | --- |
| Commit-visible cursor ordering | Once progress passes a cursor, no transaction may later expose a row at or before that cursor. Increasing IDs or append-only storage alone do not ensure this. |
| Unique composite cursor | Rows must not share the same timestamp/tie-breaker pair; otherwise a page boundary can exclude an unread row with an equal cursor. |
| Stable cursor and selected values | A NACK executes the query again, rather than replaying an immutable saved payload. Updates can change both which rows qualify and what they contain. |
| Sufficient retention | Keep rows available and unchanged throughout the maximum outage, downstream retry, and checkpoint-recovery window. Deletion or expiration before replay can lose the original data. |
| Stable query semantics | The selected cursor must match the predicate/order values, and the query must continue to describe the same logical stream. |

For example, transaction A assigns timestamp 10:00 but stays uncommitted.
Transaction B commits a row at 10:05, which is delivered and checkpointed.
If A commits afterward, the next query starts after 10:05 and misses A.
**Append-only data does not prevent this late-commit case.**

Likewise, if a returned row is changed or deleted before a NACK retry, the
original row cannot necessarily be reproduced. Backdated inserts, late commits
behind the checkpoint, cursor reuse, mutable selected values, and insufficient
retention are unsupported for the at-least-once guarantee.

### Downstream Acknowledgement Boundary

Progress advances only after matching downstream acknowledgement **and** a
successful checkpoint installation. Successful enqueue alone is not a durable
delivery boundary.

Choose an ACK boundary that represents the durability your deployment needs.
Intermediate processors, buffers, topics, and fan-out must preserve feedback to
that boundary. The receiver does not validate an entire downstream graph's
delivery guarantees. The console example below demonstrates collection and
formatting; it is not a durable production destination.

### Failure Handling and Retries

| Condition | Receiver behavior |
| --- | --- |
| Matching ACK | Write the last emitted cursor to the checkpoint, then advance in-memory progress. |
| Matching retryable NACK | Keep the committed cursor and re-query after `nack_backoff`. The returned page may differ if source rows changed. |
| Matching permanent NACK | Pause this source by default, retaining ownership and committed progress; opt-in `retry` uses capped backoff. |
| Stale or duplicate feedback | Do not advance progress. |
| Empty query result | Wait for the next polling interval without advancing the cursor. |
| Transient database availability failure | Discard failed session state and partial results; reconnect and revalidate through the shared polling runtime. |
| Non-retryable query, conversion, or encoding failure | Report an error and terminate the receiver; do not silently skip a bad row. |
| Checkpoint-write failure | Retry without advancing in-memory progress, up to `max_consecutive_failures`, subject to shutdown/drain deadlines. |
| Crash after downstream acceptance but before checkpoint commit | Resume from committed progress and allow duplicates. |
| Slow or missing downstream feedback | Keep at most one page pending; normal-operation ACK deadlines are not implemented. |

The shared runtime owns retry scheduling: transient database failures retry at
1, 2, 4, 8, 16 and then 30 seconds, capped, until recovery or stop. Oracle uses
an explicit allowlist for broken sessions, listener/database unavailability and
native call timeouts. Authentication, permissions, invalid SQL/schema/values,
and unconfirmed worker cleanup are not retryable. A native operation must
finish and drop failed session state on its worker before reporting a retryable
failure; a stuck operation follows shared bounded cleanup and ownership quarantine.

Reconnect replaces the session on the existing query worker and rereads
credentials. Shared code revalidates metadata before fetching again.
After a recovered execution page is acknowledged, subsequent catch-up waits
for the normal polling interval. Late cancellation from a previous attempt
cannot cancel a newer attempt.

Permanent rejection with `pause` retains the checkpoint and lease without
restarting the receiver; repair the destination and restart the source.
With `retry`, shared capped backoff continues until a matching ACK is durably
committed. The engine's separate runtime recovery policy can restart terminal
pipeline failures; disable it when testing source-local recovery in isolation.

## Checkpoints and Ownership

Checkpoint identity includes the state directory, pipeline group, pipeline,
receiver name, and `source_id`. Revisioned, checksummed files record the cursor
and configuration fingerprint. Corruption, unsupported versions, and
revision/source/fingerprint mismatches fail explicitly instead of resetting to
the initial cursor.

The receiver acquires a shared `SourceBinding` in the factory. It couples the
checkpoint store with its native-path lease and derives emitted source identity
from that store rather than accepting independent identity arguments.
Its current checkpoint namespace distinguishes IDs that differ
only in case, including on Windows. The factory, checkpoint writer and ownership
lock therefore refer to the same storage identity. The shared store owns layout
compatibility and recovery; Oracle has no separate checkpoint implementation.

Writes use a same-directory temporary file, file synchronization, and rename.
The store attempts to retain the newest two revisions; cleanup failures are
reported. Unix also synchronizes the checkpoint's parent directory, but newly
created ancestor directories are not synchronized. Windows has no portable
directory-fsync step. Power loss can therefore lose a new state tree or an
installation and cause replay. Power-loss behavior has not been experimentally
qualified; retention must account for that recovery window.

The fingerprint includes the source ID, connection string, SQL, and cursor
column/bind/initial values. Credential paths, Instant Client directory, and
polling interval are excluded. Changing semantic fields can invalidate saved
state; do not remove checkpoints merely to bypass that protection.

A process-local registry and advisory filesystem lock exclude competing owners
of the **same checkpoint identity**, provided the filesystem honors the lock.
They do not discover overlapping database queries. Different pipeline names,
receiver names, state directories, or unshared replica filesystems can still
poll the same data. Enforce one active poller per logical source range.
Renaming a receiver or moving its state directory does not transfer its progress.

## Live configuration changes

**Stop the existing pipeline before starting it with changed configuration,
including an interval-only change.**

The controller starts a replacement before stopping the old receiver. The old
receiver still holds the checkpoint lease, so a replacement using that identity
cannot acquire ownership. Excluding `query.interval` from the fingerprint does
not remove this lease conflict.

The required procedure is:

1. Stop the existing pipeline and wait for its receiver and native worker to
   finish.
2. Update the configuration, preserving checkpoint identity when continuing the
   same logical stream.
3. Start the pipeline with the new configuration. If semantic fields changed,
   resolve any checkpoint-compatibility error explicitly.

Until exclusive-source replacement support is available, do not use
start-before-stop live replacement for this receiver. Follow
[open-telemetry/otel-arrow#4049][readiness] for startup-readiness and safe rollout
support, and [open-telemetry/otel-arrow#4001][source-coordination] for source
coordination. If cleanup cannot be confirmed, restart the process rather than
deleting a lock or starting an overlapping owner.

## Oracle Instant Client Installation

Download **Basic** or **Basic Light** from [Oracle Instant Client downloads][instant-client].
Select a client compatible with your database and matching the engine's OS and
architecture. The examples use placeholder installation directories; substitute
the actual directory created by your ZIP package.

### Windows

Follow Oracle's [Windows ZIP installation guide][instant-client-windows]:

1. Extract the matching Basic or Basic Light package into one directory.
2. Install the Microsoft Visual C++ Redistributable required by that client
   version; Oracle's download page identifies the prerequisite.
3. Put the library directory first in `PATH`, and set
   `connection.instant_client_dir` to the same directory.

For the example configuration:

```powershell
$env:ORACLE_INSTANT_CLIENT_DIR = "C:\oracle\instantclient"
$env:PATH = "$env:ORACLE_INSTANT_CLIENT_DIR;$env:PATH"
```

Existing terminals and services do not inherit later environment changes;
restart them or configure their service environment before launching the engine.

### Linux

Follow Oracle's [Linux ZIP installation guide][instant-client-linux]:

1. Extract the selected Basic or Basic Light package into one directory readable
   by the engine's service account.
2. Install the OS dependencies required by your client version, including
   `libaio` (`libaio1` on some distributions).
3. Make the libraries discoverable through the system loader with `ldconfig`,
   or set `LD_LIBRARY_PATH` **before** starting the engine.

For a ZIP installation using the environment-variable approach:

```sh
export ORACLE_INSTANT_CLIENT_DIR=/opt/oracle/instantclient
export LD_LIBRARY_PATH="$ORACLE_INSTANT_CLIENT_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
```

For the `ldconfig` alternative, follow Oracle's instructions for adding the
actual directory to `/etc/ld.so.conf.d/` and refreshing the loader cache.
Set `connection.instant_client_dir` to that same library directory. Configure
the environment in the service or container that actually starts the engine.

## Examples

### Full Configuration

The repository provides this single-core Oracle-to-console pipeline in
[`configs/oracle-oci-console.yaml`][example-config]. Environment substitutions
refer to connection settings and secret **paths**, not secret contents.

```yaml
version: otel_dataflow/v1
engine: {}
policies:
  resources:
    core_allocation:
      type: core_count
      count: 1
groups:
  default:
    pipelines:
      main:
        nodes:
          oracle-audit:
            type: urn:otel:receiver:oracle
            config:
              source_id: oracle-audit
              connection:
                connect_string: '${env:ORACLE_CONNECT_STRING:-//localhost:1521/FREEPDB1}'
                instant_client_dir: '${env:ORACLE_INSTANT_CLIENT_DIR:-/opt/oracle/instantclient}'
              authentication:
                username_file: '${env:ORACLE_USERNAME_FILE:-/run/oracle-secrets/username}'
                password_file: '${env:ORACLE_PASSWORD_FILE:-/run/oracle-secrets/password}'
              query:
                statement: >
                  SELECT EVENT_ID, EVENT_TS, PAYLOAD
                  FROM OTAP_ORACLE_EVENTS
                  WHERE (
                    EVENT_TS > :last_timestamp
                    OR (
                      EVENT_TS = :last_timestamp
                      AND EVENT_ID > :last_tie_breaker
                    )
                  )
                  ORDER BY EVENT_TS ASC, EVENT_ID ASC
                interval: 1m
                fetch_size_rows: 300
                max_rows_per_poll: 10000
                max_batch_bytes: 10 MiB
                timeout: 30s
              watermark:
                mode: composite
                timestamp:
                  column: EVENT_TS
                  bind: last_timestamp
                  initial: "1970-01-01 00:00:00"
                  timezone: UTC
                tie_breaker:
                  column: EVENT_ID
                  bind: last_tie_breaker
                  initial: 0
              checkpoint:
                directory: ${engine.state_dir}/oracle
                on_nack: rewind
                nack_backoff: 1s
                max_consecutive_failures: 5
          console:
            type: exporter:console
            config:
              format: pretty
        connections:
          - from: oracle-audit
            to: console
```

### Running

Install Instant Client first, ensure the source table exists, and provision the
credential files. Run these commands from the repository root. The environment
variables below override the paths and connection string in the example.

**Windows (PowerShell):**

```powershell
Set-Location rust\otap-dataflow
$env:ORACLE_INSTANT_CLIENT_DIR = "C:\oracle\instantclient"
$env:PATH = "$env:ORACLE_INSTANT_CLIENT_DIR;$env:PATH"
$env:ORACLE_CONNECT_STRING = "//database.example.com:1521/ORCL"
$env:ORACLE_USERNAME_FILE = "C:\secrets\oracle-username"
$env:ORACLE_PASSWORD_FILE = "C:\secrets\oracle-password"
cargo run --no-default-features --features crypto-ring,oracle-receiver -- `
  --config configs\oracle-oci-console.yaml --num-cores 1
```

**Linux (shell):**

```sh
cd rust/otap-dataflow
export ORACLE_INSTANT_CLIENT_DIR=/opt/oracle/instantclient
export LD_LIBRARY_PATH="$ORACLE_INSTANT_CLIENT_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export ORACLE_CONNECT_STRING=//database.example.com:1521/ORCL
export ORACLE_USERNAME_FILE=/run/oracle-secrets/username
export ORACLE_PASSWORD_FILE=/run/oracle-secrets/password
cargo run --no-default-features --features crypto-ring,oracle-receiver -- \
  --config configs/oracle-oci-console.yaml --num-cores 1
```

The runtime rejects multi-core placement for this unpartitioned receiver.
The `crypto-ring` build feature selects the engine's TLS provider; it does not
by itself configure Oracle transport encryption. No bundled database or Docker
Compose environment is provided.

### Development Load Generation

The [`oracle_load_generator` example][load-generator] creates
`OTAP_ORACLE_EVENTS` with a non-null `TIMESTAMP(9)` and `NUMBER(18)` primary-key
cursor. It inserts deterministic rows; `--collision-size` controls how many rows
share each timestamp. Existing IDs are left unchanged.

Run against a **disposable test database**, from `rust/otap-dataflow`, after
setting `ORACLE_USERNAME`, `ORACLE_PWD`, and `ORACLE_CONNECT_STRING` and configuring
the native library path. This developer tool uses environment credentials;
the receiver itself uses mounted files.

```sh
cargo run -p otel-arrow-dfe-contrib-nodes --features oracle-receiver --example oracle_load_generator -- --rows 10000 --collision-size 100
```

The optional `--reset` flag **drops and recreates the table**. Never use it on
production data. Load rows before starting the receiver for a repeatable test;
this tool is not evidence that arbitrary concurrent writers satisfy commit
ordering.

### Tests

Unit tests do not require a database or an installed Instant Client:

```sh
cargo test -p otel-arrow-dfe-contrib-nodes --features oracle-receiver --lib oracle_receiver
cargo test -p otel-arrow-dfe-scraper
cargo test -p otel-arrow-dfe-contrib-nodes --features oracle-receiver --example oracle_load_generator
```

The factory tests cover existing shared checkpoints, exact lease ownership and
case-distinct sources. The generator regression test keeps its `NUMBER(18)`
cursor compatible with the adapter's supported integer precision.

For the opt-in live smoke test, provision Instant Client and source data, then
set `OTAP_ORACLE_RECEIVER_E2E=1`, `ORACLE_CONNECT_STRING`,
`ORACLE_INSTANT_CLIENT_DIR`, `ORACLE_USERNAME_FILE`, and `ORACLE_PASSWORD_FILE`:

```sh
cargo test -p otel-arrow-dfe-contrib-nodes --features oracle-receiver emits_oracle_rows_when_live_test_is_enabled -- --nocapture
```

Without `OTAP_ORACLE_RECEIVER_E2E`, that test returns without contacting Oracle;
a passing unit-test run is not a live-database qualification.

## Telemetry

The receiver uses the shared `receiver.database` metric set. These instruments
describe receiver activity, not database monitoring metrics.

### Metric Sets

#### `receiver.database`

Metrics are counters except for the rejection-pause gauge, with no custom measurement attributes.
Common engine resource and node context may still accompany them.

| Metric | Unit | Description |
| --- | --- | --- |
| `receiver.database.starts` | `{start}` | Receiver starts. |
| `receiver.database.polls` | `{poll}` | Attempted bounded query polls. |
| `receiver.database.query_failures` | `{failure}` | Failed query executions. |
| `receiver.database.reconnects` | `{attempt}` | Reconnect attempts after transient database failures. |
| `receiver.database.batches_sent` | `{batch}` | Pages sent downstream. |
| `receiver.database.rows_sent` | `{row}` | Rows sent downstream. |
| `receiver.database.encoded_bytes_sent` | `By` | Encoded OTLP bytes sent downstream. |
| `receiver.database.event_time_fallbacks` | `{record}` | Records using observation time because source time is outside OTLP's range. |
| `receiver.database.acks` | `{ack}` | Matching downstream ACKs; not necessarily successful checkpoint commits. |
| `receiver.database.nacks` | `{nack}` | Matching downstream NACKs. |
| `receiver.database.rejection_paused` | `1` | Gauge: one while permanent rejection pauses this source, otherwise zero. |
| `receiver.database.replays` | `{replay}` | Replay requests recorded when a matching NACK rewinds the pending page. |
| `receiver.database.stale_feedback` | `{feedback}` | Correlated feedback rejected because it does not match the pending page. |
| `receiver.database.checkpoint_commits` | `{commit}` | Successful checkpoint commits. |
| `receiver.database.checkpoint_failures` | `{failure}` | Failed checkpoint writes. |
| `receiver.database.checkpoint_cleanup_failures` | `{failure}` | Reported stale-revision cleanup failures. |
| `receiver.database.cancellations` | `{cancellation}` | Cancellation requests for active operations. |
| `receiver.database.drains` | `{drain}` | Ingress drain requests handled by the receiver. |
| `receiver.database.shutdowns` | `{shutdown}` | Immediate shutdown requests handled by the receiver. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `database_receiver.start` | `info` | Startup with source, database system, checkpoint revision, and ownership generation. |
| `database_receiver.page_sent` | `debug` | Page sent, including row/byte counts and deferred rows. |
| `database_receiver.page_nacked` | `warn` | Checkpoint retained and replay scheduled. |
| `database_receiver.retry_scheduled` | `warn` | Transient database recovery scheduled without restarting the pipeline. |
| `database_receiver.recovered` | `info` | Database polling recovered after transient failures. |
| `database_receiver.source_paused` | `warn` | Permanent rejection paused the source until repair and restart. |
| `database_receiver.rejection_retry_scheduled` | `warn` | Permanent-rejection replay scheduled with capped backoff. |
| `database_receiver.rejection_recovered` | `info` | A previously rejected page was acknowledged and checkpointed. |
| `database_receiver.checkpoint_committed` | `debug` | Matching progress committed. |
| `database_receiver.checkpoint_failed` | `warn` | Checkpoint write failed; includes attempt count. |
| `database_receiver.checkpoint_cleanup_failed` | `warn` | Stale revisions could not be removed. |
| `database_receiver.event_time_fallback` | `warn` | Source event time required an observation-time fallback. |
| `database_receiver.drain_deadline_reached` | `warn` | Drain deadline reached during unresolved delivery. |
| `database_receiver.cancellation_failed` | `warn` | An active operation could not be interrupted. |
| `database_receiver.worker_abandoned` | `warn` | Worker cleanup could not be joined; ownership retained until process exit. |

Native Oracle failures expose the operation and available numeric OCI/DPI codes,
not raw native messages or their error-source chains. Native and OS error
payloads are discarded after extracting safe codes/categories. Oracle-specific
validation errors do not echo configured bind or column names, and factory
lease-acquisition errors do not echo checkpoint paths. The development load
generator uses the same sanitized Oracle failures and does not echo unknown
argument values.

This is diagnostic redaction, not filtering of exported database rows: selected
columns still form the intended telemetry payload. `source_id`, pipeline/node
identifiers and checkpoint paths must be safe to expose. The unchanged shared
runtime can include source identity and filesystem paths in its own events and
checkpoint errors; the Oracle adapter does not rewrite those diagnostics.

## Limits

- One query and one pending page per receiver; one pipeline core. Deploy one
  active collector replica per logical source. Automatic partitioning and
  distributed source discovery are not implemented.
- Composite watermark and `on_nack: rewind`; permanent rejection separately supports `pause` or `retry`. No snapshots, scalar cursor,
  CDC, delete capture, multiple named queries, or configurable output mapping.
- No whole-poll deadline, normal-operation ACK deadline, or process-RSS ceiling.
- Only explicitly classified transient database failures retry. Conversion and
  unsupported schema failures remain terminal. Checkpoint retries have their own
  explicit failure limit; source-local availability retries continue until stop or recovery.
- Query/session work and native cancellation use separate bounded OS workers,
  each with one queue slot; the cancellation worker starts only when needed.
  Neither belongs to Tokio's blocking pool. Cancellation is checked between
  native calls, fetches, and column conversions.
- Stop/cancellation waits are bounded by the earlier active deadline and a
  five-second worker-stop cap. If cleanup cannot be joined, ownership is held
  until process exit. Shutdown confirms both workers' cleanup before releasing
  ownership. Stuck native work may outlive the runtime, but does not make Tokio
  join these workers. It still requires process-level recovery; configure the
  service supervisor with a hard process-stop timeout.
- Do not delete lock/generation files or start a replacement while abandoned
  work may still be running. Stop-before-start is required for configuration
  changes; see [Live configuration changes](#live-configuration-changes).

## Related Docs

- [Shared database scraper][scraper]
- [Complete Oracle-to-console example][example-config]
- [Configuration model](../../../../../docs/configuration-model.md)
- [Contrib node catalog](../../../README.md)
- [Oracle Instant Client downloads][instant-client]
- [Oracle Instant Client Windows installation][instant-client-windows]
- [Oracle Instant Client Linux installation][instant-client-linux]
- [Database receiver RFC: open-telemetry/otel-arrow#3918][database-rfc]
- [Original implementation: open-telemetry/otel-arrow#3969][source-pr]
- [Exclusive-source rollout: open-telemetry/otel-arrow#4049][readiness]
- [Source coordination: open-telemetry/otel-arrow#4001][source-coordination]

[scraper]: ../../../../scraper/README.md
[example-config]: ../../../../../configs/oracle-oci-console.yaml
[load-generator]: ../../../examples/oracle_load_generator.rs
[instant-client]: https://www.oracle.com/database/technologies/instant-client/downloads.html
[instant-client-windows]: https://docs.oracle.com/en/database/oracle/oracle-database/26/ntcli/installing-oracle-instant-client-using-zip-files.html
[instant-client-linux]: https://docs.oracle.com/en/database/oracle/oracle-database/21/lacli/install-instant-client-using-zip.html
[database-rfc]: https://github.com/open-telemetry/otel-arrow/issues/3918
[source-pr]: https://github.com/open-telemetry/otel-arrow/pull/3969
[readiness]: https://github.com/open-telemetry/otel-arrow/issues/4049
[source-coordination]: https://github.com/open-telemetry/otel-arrow/issues/4001
