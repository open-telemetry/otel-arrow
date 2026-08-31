# Oracle receiver foundation

The opt-in `oracle-receiver` feature registers
`urn:otel:receiver:oracle`. This initial receiver runs one bounded,
read-only snapshot query and emits one typed OTLP `LogRecord` per row.

Watermark progression, query rewriting, checkpoint persistence, and
Ack/Nack rewind are not implemented. The optional `watermark` and
`checkpoint` configuration sections are accepted only so the stable
configuration can be introduced incrementally. When `watermark` is present,
its columns are validated against live result metadata and its timestamp
column supplies OTLP event time.

## Local load-generator demo

The repository includes `oracle_load_generator`, a deterministic data
generator originally introduced with the earlier Oracle work. This foundation
reuses only the generator; it does not reuse or enable watermarking.

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
many adjacent rows share a timestamp. Those collisions are useful for future
composite-watermark work but do not activate watermarking in this receiver.

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

Keep `--rows` at or below the configured `max_rows_per_poll`. An oversized
snapshot fails the batch rather than silently truncating it.

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

The checked-in `configs\oracle-oci-console.yaml` reads these four environment
variables:

```yaml
connection:
  connect_string: '${env:ORACLE_CONNECT_STRING}'
  instant_client_dir: '${env:ORACLE_INSTANT_CLIENT_DIR}'
authentication:
  username_file: '${env:ORACLE_USERNAME_FILE}'
  password_file: '${env:ORACLE_PASSWORD_FILE}'
```

Single quotes are intentional: environment substitution occurs before YAML
parsing, and single-quoted YAML preserves Windows path backslashes.

The checked-in query already reads the generated table:

```sql
SELECT EVENT_ID, EVENT_TS, PAYLOAD
FROM OTAP_ORACLE_EVENTS
ORDER BY EVENT_TS, EVENT_ID
```

### 4. Run Oracle to console

```powershell
cargo run --features oracle-receiver -- `
  --config configs\oracle-oci-console.yaml --num-cores 1
```

The receiver emits one typed OTLP log record per generated row. Because this
foundation is snapshot-only, it reads and emits the same rows every five
seconds. Stop it with `Ctrl+C`.

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
