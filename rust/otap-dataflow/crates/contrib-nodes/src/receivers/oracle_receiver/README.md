# Oracle composite watermark receiver

The opt-in `oracle-receiver` feature registers
`urn:otel:receiver:oracle`. The receiver polls a customer-authored, read-only
`SELECT` after a durable composite watermark, emits one typed OTLP `LogRecord`
per row, and advances its checkpoint only after a downstream acknowledgement.

## Delivery guarantees

Delivery is **at least once**. A crash, a negative acknowledgement, or a drain
that ends before feedback arrives can re-emit rows, but an unacknowledged row is
never intentionally skipped.

At most one page is in flight per source. The receiver does not start another
query until the outstanding page is acknowledged, rejected, or the receiver
terminates.

## Supported watermark mode

Only `watermark.mode: composite` is implemented:

- an ordered timestamp column of an Oracle `DATE` or `TIMESTAMP`-family type
- a non-null `int64` tie-breaker that is unique within each timestamp group
- UTC semantics; `watermark.timestamp.timezone` must be `UTC`

`scalar` and repeating `snapshot` modes are rejected as unsupported and are
deferred to follow-up work.

## Required query shape

You supply the complete SQL. The receiver validates, before connecting, that the
statement:

- is a single `SELECT` without SQL comments or statement separators
- references both configured named binds as real bind markers -- a bind name
  appearing only inside a string literal, or only as a prefix of a longer bind,
  does not count
- ends with the outer ordering
  `ORDER BY <timestamp_column> ASC, <tie_breaker_column> ASC`; an ordering
  nested inside a subquery does not satisfy this

Cursor values are bound through Oracle named parameters and are never
interpolated into SQL text. Live result metadata is then checked so both cursor
columns exist with supported, deterministic types.

## Bounds

Every operational bound is a required field:

- `query.max_rows_per_poll` -- hard row ceiling for one poll
- `query.max_normalized_bytes` -- in-memory ceiling for normalized rows before
  encoding
- `query.max_batch_bytes` -- **exact** serialized OTLP payload ceiling

The receiver emits the largest non-empty row prefix that fits
`max_batch_bytes`, and the committed candidate is always the cursor of the last
row actually emitted. Rows beyond a ceiling are returned by the next poll rather
than dropped. If a single first row exceeds either byte ceiling, the poll fails
explicitly instead of skipping it.

## Checkpoints and replay

Checkpoints are revisioned files under `checkpoint.directory`, keyed by pipeline
group, pipeline, receiver name, and `source_id`. Each file records a schema
version, revision, source identity, configuration fingerprint, composite cursor,
and checksum. Writes use a same-directory temporary file, `fsync`, and an atomic
rename, and the two newest revisions are retained.

Reads fail closed on corruption, an unsupported version, or a revision, source,
or fingerprint mismatch, so a receiver never resumes from an unrelated position.
The configuration fingerprint covers semantic fields only, so rotating a mounted
credential does not invalidate durable state.

`checkpoint.on_nack` supports only `rewind`. A negative acknowledgement retains
the committed cursor and replays the same page after the fixed
`checkpoint.nack_backoff`. Stale or duplicate feedback is ignored. Reaching
`checkpoint.max_consecutive_failures` durable-write failures terminates the
receiver with a checkpoint error without advancing in-memory state.

A process-local lease keyed by the checkpoint identity prevents two receivers in
one process from advancing the same checkpoint. It complements, and does not
replace, the deployment requirement that exactly one collector replica owns a
checkpoint source.

## Telemetry

The receiver registers the `receiver.database` metric set covering starts,
polls, query failures, batches, rows, encoded bytes, acknowledgements, negative
acknowledgements, replays, stale feedback, checkpoint commits, checkpoint
failures, checkpoint cleanup failures, cancellations, drains, and shutdowns.

## One-command Docker demo

The Docker demo packages the receiver, load generator, Oracle Instant Client,
credential-file setup, and Oracle Free orchestration. Docker is the only local
prerequisite.

From `rust\otap-dataflow`, run:

```powershell
docker compose -f docker-compose.oracle-demo.yaml up --build
```

The first run downloads Oracle Free and builds the receiver image, so it can
take several minutes. The receiver container waits for Oracle, creates 25
deterministic rows, and starts the console pipeline automatically. Look for:

```text
Prepared OTAP_ORACLE_EVENTS with 25 deterministic rows and collision groups of 5
Starting the Oracle receiver. Composite watermark paging begins at the initial cursor.
```

Override the generated row count or timestamp collision size without editing
files:

```powershell
$env:ORACLE_DEMO_ROWS = "100"
$env:ORACLE_DEMO_COLLISION_SIZE = "10"
docker compose -f docker-compose.oracle-demo.yaml up --build
```

If local port 1521 is already occupied, set `ORACLE_DEMO_PORT` to another
loopback port such as `1522`. Communication between the demo containers still
uses Oracle's internal port 1521.

Stop the containers with `Ctrl+C`. Remove the containers and demo database
volume with:

```powershell
docker compose -f docker-compose.oracle-demo.yaml down --volumes
```

The default database password is a public, local-demo-only value. The Oracle
port binds only to `127.0.0.1`. Do not publish this Compose setup or use its
credentials in production. The image downloads Oracle Instant Client directly
from Oracle while building; review Oracle's license before redistributing a
built image.

The demo exercises composite watermark paging, including the deliberate
timestamp collision groups that require the tie-breaker to make progress.

## Local load-generator demo

The repository includes `oracle_load_generator`, a deterministic data
generator originally introduced with the earlier Oracle work. Its stable
timestamp collision groups exercise the receiver's composite cursor across
successive pages.

The generator creates this table:

```sql
CREATE TABLE OTAP_ORACLE_EVENTS (
    EVENT_TS TIMESTAMP(9) NOT NULL,
    EVENT_ID NUMBER(19) NOT NULL PRIMARY KEY,
    PAYLOAD VARCHAR2(200) NOT NULL
)
```

It inserts stable event IDs and timestamps, so rerunning it without `--reset`
keeps existing rows and adds only missing IDs. `--collision-size` controls how
many adjacent rows share a timestamp, verifying that the `int64` tie-breaker
advances every row within an equal-timestamp group.

### 1. Configure Oracle

From `rust\otap-dataflow`, make Oracle Instant Client available and set the
credentials used to create and populate the demo table:

```powershell
$env:PATH = "C:\oracle\instantclient;$env:PATH"
$env:ORACLE_INSTANT_CLIENT_DIR = "C:\oracle\instantclient"
$env:ORACLE_USERNAME = "PDBADMIN"
$env:ORACLE_PWD = "your-local-password"
$env:ORACLE_CONNECT_STRING = "//localhost:1521/FREEPDB1"
```

The generator account needs permission to create, drop, and write the demo
table. For a production receiver, use a separate principal with read-only
access to only the required tables or views.

### 2. Generate deterministic rows

```powershell
cargo run -p otel-arrow-dfe-contrib-nodes `
  --features oracle-receiver `
  --example oracle_load_generator -- `
  --reset --rows 25 --collision-size 5
```

`--reset` drops only `OTAP_ORACLE_EVENTS`, if it exists, and recreates it.
Omit `--reset` on later runs. For example, this keeps rows 1 through 25 and
adds rows 26 through 50:

```powershell
cargo run -p otel-arrow-dfe-contrib-nodes `
  --features oracle-receiver `
  --example oracle_load_generator -- `
  --rows 50 --collision-size 5
```

Keep `--rows` at or below the configured `max_rows_per_poll`. A larger table is
paged across successive polls rather than truncated. The receiver caps
`max_rows_per_poll` at 10,000 rows, bounds normalized in-memory rows with
`max_normalized_bytes`, and bounds the exact serialized OTLP payload with
`max_batch_bytes`. The checked-in example sets both limits to 10 MiB.

The minimal Oracle adapter uses one-row native fetch arrays because rust-oracle
does not expose current result widths until after allocating that array;
`fetch_size` instead bounds the receiver's row-vector preallocation.

### 3. Configure receiver credentials

The receiver reads credentials from files rather than environment variables.
Create the files, or point the YAML to existing mounted secrets:

```powershell
New-Item -ItemType Directory -Force C:\secrets | Out-Null
Set-Content -NoNewline C:\secrets\oracle-username $env:ORACLE_USERNAME
Set-Content -NoNewline C:\secrets\oracle-password $env:ORACLE_PWD
$env:ORACLE_USERNAME_FILE = "C:\secrets\oracle-username"
$env:ORACLE_PASSWORD_FILE = "C:\secrets\oracle-password"
```

The checked-in `configs\oracle-oci-console.yaml` accepts these four environment
overrides and supplies container-friendly local defaults:

```yaml
connection:
  connect_string: '${env:ORACLE_CONNECT_STRING:-//localhost:1521/FREEPDB1}'
  instant_client_dir: '${env:ORACLE_INSTANT_CLIENT_DIR:-/opt/oracle/instantclient}'
authentication:
  username_file: '${env:ORACLE_USERNAME_FILE:-/run/oracle-secrets/username}'
  password_file: '${env:ORACLE_PASSWORD_FILE:-/run/oracle-secrets/password}'
```

Single quotes are intentional: environment substitution occurs before YAML
parsing, and single-quoted YAML preserves Windows path backslashes.

Credential paths must reference regular UTF-8 files. Keeping credential
contents outside the YAML prevents environment substitution, debug output, and
effective-configuration snapshots from materializing the username or password.
`source_id` is limited to 256 bytes, and `query.timeout` must be between 1
millisecond and 5 minutes. Initial network connection attempts are capped at 10
seconds because Oracle cannot interrupt a connection attempt before the native
client returns a connection handle. Multi-address Easy Connect strings and
`retry_count` or `retry_delay` parameters are therefore rejected.

The checked-in query already reads the generated table with the required cursor
predicate and ordering:

```sql
SELECT EVENT_ID, EVENT_TS, PAYLOAD
FROM OTAP_ORACLE_EVENTS
WHERE (
  EVENT_TS > :last_timestamp
  OR (EVENT_TS = :last_timestamp AND EVENT_ID > :last_tie_breaker)
)
ORDER BY EVENT_TS ASC, EVENT_ID ASC
```

### 4. Run Oracle to console

```powershell
cargo run --features oracle-receiver -- `
  --config configs\oracle-oci-console.yaml --num-cores 1
```

The receiver emits one typed OTLP log record per generated row, advances its
durable checkpoint after each acknowledged page, and stops emitting once the
table is exhausted. Newly inserted rows are picked up on the next poll. Stop it
with `Ctrl+C`; restarting resumes from the last committed cursor.

## Live smoke test

Set paths for a local Oracle Instant Client and mounted credential files:

```powershell
$env:OTAP_ORACLE_RECEIVER_E2E = "1"
$env:ORACLE_CONNECT_STRING = "//localhost:1521/FREEPDB1"
$env:ORACLE_INSTANT_CLIENT_DIR = "C:\oracle\instantclient"
$env:ORACLE_USERNAME_FILE = "C:\secrets\oracle-username"
$env:ORACLE_PASSWORD_FILE = "C:\secrets\oracle-password"

cargo test -p otel-arrow-dfe-contrib-nodes `
  --features oracle-receiver `
  emits_oracle_rows_when_live_test_is_enabled -- --nocapture
```

The test connects, validates result metadata, executes a bounded query,
converts the row to OTLP logs, sends it through the receiver test pipeline,
and verifies that one item arrives downstream.
