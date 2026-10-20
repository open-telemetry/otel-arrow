# Demo: SQL Server rows to OpenTelemetry logs

<!-- markdownlint-configure-file { "MD013": { "tables": false } } -->

This walkthrough demonstrates the experimental SQL Server receiver on Windows
using PowerShell and a disposable SQL Server 2022 Linux container. It shows
initial catch-up, polling for new rows, and resuming from acknowledged progress.

Use only generated demo data. This is not a production-readiness or
exactly-once-delivery demonstration.

## What we are demonstrating

```text
SQL Server table
  -> verified native TLS connection using a read-only SQL login
  -> parameterized timestamp + ID keyset query
  -> one structured OpenTelemetry LogRecord per row
  -> console exporter
  -> acknowledgement
  -> file checkpoint
  -> next eligible query page
```

The receiver uses the existing shared scraper for scheduling, feedback,
checkpoints, backpressure, and memory-pressure admission. SQL Server-specific
code handles connection setup, T-SQL, metadata, and value conversion.

| Demo stage | What the audience should see |
| --- | --- |
| Initial catch-up | IDs 1 and 2 arrive in the first page, followed by ID 3 in the next page. |
| Idle polling | Unchanged rows are not repeatedly emitted. The receiver checks again after the configured interval. |
| New data | Inserting ID 4 produces a new log on a subsequent poll. |
| Checkpoint | Saved progress advances after downstream acknowledgement. |
| Restart | The same configuration and checkpoint directory resume after saved progress rather than rereading every row. |

## Prerequisites

| Requirement | Why it is needed |
| --- | --- |
| PowerShell on Windows | Runs the commands below. |
| Docker Desktop in Linux-container mode | Runs the disposable database, not the receiver. Allow a 3 GiB container plus host overhead. |
| Python 3 | Runs the fixture setup/cleanup script; it requires no third-party Python packages. |
| OpenSSL | Generates the fixture's CA and server certificate. The script automatically discovers Git for Windows' bundled OpenSSL when it is absent from PATH. |
| An engine built with `sql-server` | Registers the receiver and the flat-file credential extension. |

No host installation of SQL Server, an ODBC driver, JDBC/JVM, or SSMS is
required. Docker, Python, OpenSSL, and the database tools inside the container
are **demo/developer tooling**, not extra receiver runtime dependencies.
The Windows receiver uses system Schannel, not OpenSSL or the engine's Rustls
provider. Linux receiver deployments require system OpenSSL runtime libraries
and a current root store; see the [native deployment guidance][readme].

The fixture script accepts the SQL Server Developer container license terms.
Review and approve those terms before running setup; preparing/building this
demo does not authorize starting the fixture.
Use this fixture for development/testing only. The first run may need to
download the pinned database image.

### Understand the two directories

The **worktree root** and **demo directory** are different:

| Name | Meaning | Example contents |
| --- | --- | --- |
| Worktree root | The checkout belonging to this Copilot session. Run repository-relative commands here. | `rust`, `go`, `.github`, `Makefile` |
| Demo directory (`$demo`) | A new temporary directory generated during setup. Its exact path is printed after `Fixture ready:`. | `fixture.json`, CA certificate, password file, checkpoints |

`Get-Content ...\fixture.json` needs the **demo directory**, not the worktree
root. A second PowerShell terminal does not inherit `$demo` or `$fixture`.

## 1. Prepare Terminal A

Open a PowerShell terminal in the Copilot session's worktree root. If opening
an external terminal, first use `Set-Location` with that session's checkout
path, not a different main checkout.

```powershell
$repo = (Get-Location).Path
$engine = Join-Path $repo 'rust\otap-dataflow\target\debug\df_engine.exe'

Get-Location
Test-Path .\rust\otap-dataflow\scripts\sql-server-fixture.py
Test-Path $engine
```

Both `Test-Path` results should be `True`. If the script is missing, correct
your current directory before continuing.

If only the engine is missing or needs rebuilding, use:

```powershell
Push-Location .\rust\otap-dataflow
try {
    cargo build --locked --no-default-features `
        --features sql-server,core-nodes,crypto-ring --bin df_engine
    if ($LASTEXITCODE -ne 0) { throw 'Engine build failed' }
} finally {
    Pop-Location
}
```

Building requires the repository's pinned Rust toolchain and Windows MSVC
build tools. Those are not needed just to run an existing binary. This debug
build is for demonstrating behavior, not measuring production performance.

## 2. Create the fixture and load connection settings

Run in **Terminal A**:

```powershell
$demo = Join-Path $env:TEMP ('otel-sql-demo-' + [guid]::NewGuid().ToString('N'))
$fixture = $null

python .\rust\otap-dataflow\scripts\sql-server-fixture.py start --directory $demo

if ($LASTEXITCODE -eq 0) {
    $fixture = Get-Content (Join-Path $demo 'fixture.json') -Raw |
        ConvertFrom-Json

    foreach ($p in $fixture.env.PSObject.Properties) {
        $name = $p.Name -replace '^OTAP_', ''
        Set-Item -Path "Env:$name" -Value $p.Value
    }

    $env:SQL_SERVER_STATE_DIR = Join-Path $demo 'checkpoints'
    Write-Host "Fixture ready: $demo"
} else {
    Write-Error 'Fixture setup failed. Stop here; do not start the receiver.'
}
```

**Do not proceed until `Fixture ready:` appears. Copy the printed path for
Terminal B.**

Setup does the following:

- Checks OpenSSL availability and Docker's Linux-container mode before writing
  fixture files.
- Starts the digest-pinned SQL Server 2022 container on an ephemeral host port
  bound to loopback.
- Generates a short-lived CA and a server certificate for `localhost`, and
  requires encrypted connections.
- Creates database `otap_fixture`, table `dbo.OTAP_SQL_EVENTS`, and three rows.
- Creates `otel_reader` with SELECT access to that table; it is not the setup
  administrator.
- Writes connection settings and secret **file paths** into `fixture.json`.
  The receiver's password stays in its separate generated file.

The seeded rows include Unicode text, a precision-38 decimal, binary bytes,
nulls, `datetime2(7)`, and `datetimeoffset(7)`. Their event timestamps begin in
January 2026; this is intentional fixture data.

Keep the demo directory private. Its certificates expire after two days.

## 3. Start the receiver with two-row pages

In **Terminal A**, use a copy of the [example configuration][example] held in
an environment variable. The following does not modify repository files:

```powershell
$yaml = Get-Content .\rust\otap-dataflow\configs\sql-server-console.yaml -Raw
$yaml = $yaml.Replace('max_rows_per_poll: 1000', 'max_rows_per_poll: 2')
$yaml = $yaml.Replace('fetch_size_rows: 100', 'fetch_size_rows: 1')
$env:SQL_SERVER_DEMO_CONFIG = $yaml

& $engine --config env:SQL_SERVER_DEMO_CONFIG --http-admin-bind 127.0.0.1:0
```

Leave Terminal A running. The admin endpoint binds to loopback on an ephemeral
port so the demo does not require an unused fixed admin port.

### What to explain while it runs

The configuration selects `urn:otel:receiver:sql_server`, binds
`basic_auth_provider` to the existing `flat_file_user_pass_auth` extension,
and routes the emitted logs to the console exporter.

The receiver validates live metadata, then issues a parameterized query
equivalent to:

```sql
SELECT TOP (@P3) ID, EVENT_TS, PAYLOAD, AMOUNT, RAW, AT_TIME
FROM dbo.OTAP_SQL_EVENTS
WHERE EVENT_TS > @P1
   OR (EVENT_TS = @P1 AND ID > @P2)
ORDER BY EVENT_TS ASC, ID ASC
```

The parameters are native bound values, not concatenated SQL text:

| Parameter | Meaning in this demo |
| --- | --- |
| `@P1` | Initial or last acknowledged event timestamp. |
| `@P2` | Initial or last acknowledged ID at that timestamp. |
| `@P3` | Page-row ceiling, set to 2. |

Expected sequence with a fresh checkpoint directory:

1. The first page emits IDs **1 and 2**.
2. After its matching ACK and checkpoint installation, the next page emits
   ID **3** without waiting a full polling interval.
3. An empty fetch ends catch-up. The receiver waits **one minute** before
   checking for more rows.

There is only one pending page. The catch-up defaults allow up to 32 page
attempts or 10 seconds of elapsed admission time per cycle. They do not
replace the query deadline. `fetch_size_rows: 1` controls cooperative decoding;
it does **not** mean one SQL statement is executed per row.

The console prints individual records, so page boundaries are not necessarily
labeled. Look for the IDs and the structured body fields. The first payload
is `hello` followed by the Greek letter pi, and its decimal is:

```text
123456789012345678901234567890.12345678
```

The displayed event time is the source timestamp, not the current wall clock.
The receiver intentionally does not print raw SQL, credentials, or native
driver traces. Silence after catch-up is normal: unchanged rows are not emitted
again in incremental mode.

## 4. Insert new data from Terminal B

Open a **second PowerShell terminal**. It can start in any directory because
this step reads the full demo path.

```powershell
$demo = Read-Host 'Paste the directory printed after Fixture ready'
$fixture = Get-Content (Join-Path $demo 'fixture.json') -Raw |
    ConvertFrom-Json
```

Paste only the actual directory path at the prompt, without `Fixture ready:`
and without surrounding quotes. It will usually be beneath your user's
`AppData\Local\Temp` directory. **Do not paste the worktree root.**

Insert ID 4 once:

```powershell
$insert = @'
SQLCMDPASSWORD="$MSSQL_SA_PASSWORD" /opt/mssql-tools18/bin/sqlcmd \
  -S localhost -U sa -N -b \
  -Q "INSERT INTO otap_fixture.dbo.OTAP_SQL_EVENTS
        (ID, EVENT_TS, PAYLOAD)
      VALUES (4, SYSUTCDATETIME(), N'new row after startup');"
'@

docker exec $fixture.container bash -c $insert
if ($LASTEXITCODE -ne 0) { throw 'Demo insert failed' }
```

This uses the SQL tool already inside the container. The administrator password
is expanded **inside that disposable container** rather than copied into the
host command. TLS remains verified; there is no `-C` trust bypass.

Administrative access is used only to add fixture data. The receiver continues
using the SELECT-only `otel_reader` account.

Watch Terminal A. ID **4** should appear on the next scheduled poll, normally
within about a minute plus query processing time. Its timestamp is current
UTC time and therefore later than the seeded cursor values.

Do not rerun the same INSERT: ID 4 is a primary key. For another demonstration,
choose a new ID and a later timestamp.

## 5. Inspect acknowledged progress

After ID 4 appears, use **Terminal B** to inspect the newest checkpoint:

```powershell
$checkpointDir = Join-Path $demo 'checkpoints'

Get-ChildItem -LiteralPath $checkpointDir -Recurse -File `
    -Filter '*.checkpoint.*.json' |
    Sort-Object LastWriteTime |
    Select-Object -Last 1 |
    Get-Content -Raw
```

The file envelope contains the saved cursor, source identity, compatibility
fingerprint, and revision information. After ID 4 is acknowledged, its
timestamp/ID should be the saved cursor. There may be more than one revision
file; do not mistake an older retained revision for the current checkpoint.

Checkpoint installation follows ACK, so allow a moment after console output.
The checkpoint is not a stored copy of every row. A replay re-queries the
database, which is why retained, unchanged source rows matter.

The demo's console acknowledgement shows feedback plumbing. It is **not**
proof that a remote production destination has durably stored the records.

## 6. Restart and demonstrate resume

In **Terminal A**, press **Ctrl+C** and wait until the engine exits and the
PowerShell prompt returns.

Run the same command again without changing environment variables:

```powershell
& $engine --config env:SQL_SERVER_DEMO_CONFIG --http-admin-bind 127.0.0.1:0
```

Because the receiver uses the same configuration, logical identity, and
checkpoint directory, IDs 1 through 4 should not be emitted again once their
progress was acknowledged and installed.

To make resume visible, repeat the INSERT command in Terminal B with **ID 5**
and a new `SYSUTCDATETIME()` value. Terminal A should emit the new row rather
than the historical rows.

Explain the boundary: a crash after downstream acceptance but before checkpoint
installation may replay data. This is not exactly once. A normal completed
checkpoint is what enables the resume behavior shown here.

Do not run a second receiver against the same source while the first is still
active. Use stop-before-start replacement and never delete locks to bypass
ownership.

## 7. Stop and clean up

First stop the engine in Terminal A with **Ctrl+C**. Then, from the worktree
root in that terminal:

```powershell
python .\rust\otap-dataflow\scripts\sql-server-fixture.py stop --directory $demo
```

Cleanup verifies the container's fixture ownership label, removes that
container, and deletes generated credentials/private keys. It retains the
manifest, certificates, logs, and checkpoint files for inspection.

For a fresh demonstration, repeat setup with a new generated directory and
new checkpoints. Do not attach old source checkpoints to a newly recreated
logical database without an intentional reset or migration decision.

## Troubleshooting

| Symptom | Meaning and action |
| --- | --- |
| OpenSSL not found | Use the updated fixture script. It discovers Git for Windows' bundled OpenSSL; otherwise provide OpenSSL on PATH. No database client installation is needed. |
| `fixture.json` does not exist | Setup failed or the wrong directory was supplied. Stop and fix setup; do not continue with an empty `$fixture`. |
| Directory already exists | Use a new GUID-based `$demo` directory. An earlier failed attempt may have left partial files. |
| `$demo` is empty in Terminal B | PowerShell variables are not shared between terminals. Paste the path printed by Terminal A using the prompt in step 4. |
| Docker unavailable or wrong mode | Start Docker Desktop and select Linux containers. |
| Certificate/hostname error | Use the generated CA and `localhost` endpoint from the manifest. Do not substitute `127.0.0.1` for a certificate issued only to `localhost`. Expired fixture certificates require a new fixture. |
| Engine rejects `engine.state_dir` on Windows | Leave `engine: {}` as in the example; use `SQL_SERVER_STATE_DIR` for receiver checkpoints. Managed engine state provisioning is Linux-only. |
| No output after the first rows | Expected when no newer rows exist. Insert a later row and wait for the next one-minute poll. |
| Old event timestamps in the console | Expected for the January 2026 seed data; these are not the observation time. |
| INSERT reports a duplicate key | ID 4 or 5 was already inserted. Use a new ID, or start a fresh demo. |
| Checkpoint compatibility or ownership error | Preserve the error, stop the old receiver, and check source/configuration identity. Do not delete locks or silently reuse incompatible state. |

## Optional automated live suite

For automated mode, TLS rejection, cancellation, and NACK/replay/restart
coverage, use the live-test commands in the [receiver README][readme].
Run them against a **fresh, unmodified three-row fixture**, before adding IDs
4 or 5; their assertions expect the original seed data.

The full live suite also exercises explicit non-TLS testing and therefore needs
the fixture's `--allow-unencrypted` flag, as shown in the README. The interactive
demo above deliberately keeps server-enforced TLS enabled and does not need
that flag. Keep the secure demo and permissive test fixture separate.

These tests require Rust build tooling. They are not necessary for presenting
the already-built engine demo.

The full suite also disconnects the fixture's idle reader sessions; run it with
`--test-threads=1` and do not run the interactive demo at the same time.
The [receiver README][readme] describes explicit certificate replacement and
expiry trials, restoring the valid certificate, and an automated example run.
Neither the helper nor these commands modifies host certificate trust.

## If the receiver stops after a connection fault

Unconfirmed cancellation, including a connection lost while idle before the
next submission, can intentionally stop the receiver and quarantine its source
until process exit. Preserve the checkpoint. Do not delete locks, rename the
source, or repeatedly restart to bypass this protection.

For a real deployment, follow the [quarantine recovery procedure][recovery]:
the database operator must establish prior server work has ended before a
single replacement starts. A stopped client process or closed socket is not
proof. For this disposable demo, first stop the engine and explicitly stop
the owned fixture; use a new fixture and new checkpoints for a fresh demo,
not the old source cursor against a recreated database.

## What this demo does not claim

- The console is not a durable production destination.
- Page-byte limits do not yet bound every Tiberius-internal allocation or RSS.
- A timestamp/ID keyset does not solve late-committing transactions, mutable
  rows, deletes, or insufficient retention.
- This is scheduled query-to-logs polling, not CDC or database-metrics scraping.
- The tested SQL Server 2022 Linux fixture does not qualify all server versions,
  Windows-hosted databases, Azure SQL, ARM64, or every failure scenario.
- No driver installation is required for the receiver, but distribution-level
  clean-machine packaging and broader readiness work remain separate.

[example]: ../../../../../configs/sql-server-console.yaml
[readme]: README.md
[recovery]: README.md#recovering-a-quarantined-source
