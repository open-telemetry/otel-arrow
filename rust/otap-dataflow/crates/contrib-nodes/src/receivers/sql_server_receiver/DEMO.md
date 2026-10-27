# Demo: SQL Server rows to OpenTelemetry logs

This walkthrough demonstrates the SQL Server receiver on Windows
using PowerShell and a disposable SQL Server 2022 Linux container.
Use only generated demo data. This walkthrough covers functionality, not
deployment qualification or exactly-once delivery.

The [receiver README](README.md) is authoritative for configuration, delivery
guarantees, operating limits, and native dependencies. The console demonstrates
feedback, **not durable delivery**.

## 1. Set up

Use PowerShell at the repository root and an engine built with `sql-server`
as described in [Getting Started](README.md#getting-started). Fixture setup
needs Python 3, OpenSSL (Git for Windows' copy is discovered automatically),
and Docker Desktop in Linux-container mode.

**Approve the SQL Server Developer EULA before running setup.** The script
accepts it and may download the pinned image. It starts one 3-GiB container on
an ephemeral loopback-only port; the test CA is installed only inside the
container, never in host trust. See [fixture details](README.md#live-fixture-and-qualification).

In **Terminal A**:

```powershell
$engine = (Resolve-Path .\rust\otap-dataflow\target\debug\df_engine.exe).Path
$demo = Join-Path $env:TEMP ('otel-sql-demo-' + [guid]::NewGuid().ToString('N'))
python .\rust\otap-dataflow\crates\contrib-nodes\src\receivers\sql_server_receiver\ci\sql-server-fixture.py start --directory $demo
if ($LASTEXITCODE -ne 0) { throw 'Fixture setup failed; do not continue' }

$fixture = Get-Content (Join-Path $demo 'fixture.json') -Raw | ConvertFrom-Json
foreach ($entry in $fixture.env.PSObject.Properties) {
    $name = $entry.Name -replace '^OTAP_', ''
    Set-Item -Path "Env:$name" -Value $entry.Value
}
$env:SQL_SERVER_STATE_DIR = Join-Path $demo 'checkpoints'
Write-Host "Fixture ready: $demo"
```

Keep this directory private: it contains credentials and private keys.
Copy the printed **fixture directory**, not the repository path, for Terminal B.
The fixture contains three seed rows and a SELECT-only receiver login;
certificates expire after two days.

## 2. Run

In **Terminal A**, use two-row pages without modifying repository files:

```powershell
$yaml = Get-Content .\rust\otap-dataflow\configs\sql-server-console.yaml -Raw
$yaml = $yaml.Replace('max_rows_per_poll: 1000', 'max_rows_per_poll: 2')
$yaml = $yaml.Replace('fetch_size_rows: 100', 'fetch_size_rows: 1')
$env:SQL_SERVER_DEMO_CONFIG = $yaml
& $engine --config env:SQL_SERVER_DEMO_CONFIG --http-admin-bind 127.0.0.1:0
```

Expect IDs **1 and 2**, then **3** after acknowledgement. Unchanged rows are
not re-emitted; after catch-up the receiver checks for new rows every minute.
Leave this terminal running.

## 3. Verify

Open **Terminal B**. Its variables are independent of Terminal A:

```powershell
$demo = Read-Host 'Paste the directory printed after Fixture ready'
$fixture = Get-Content (Join-Path $demo 'fixture.json') -Raw | ConvertFrom-Json
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

The admin password expands inside the owned container, and TLS stays verified.
Insert ID 4 only once; it should appear in Terminal A on the next scheduled poll.
Then inspect the latest acknowledged checkpoint:

```powershell
Get-ChildItem (Join-Path $demo 'checkpoints') -Recurse -File `
    -Filter '*.checkpoint.*.json' |
    Sort-Object LastWriteTime |
    Select-Object -Last 1 |
    Get-Content -Raw
```

Its cursor should contain ID 4 after ACK. To verify resume, press **Ctrl+C** in
Terminal A and wait for a clean exit before rerunning the same engine command.
Keep the same configuration and checkpoint directory: acknowledged IDs 1-4
should not repeat. Insert ID 5 with a fresh timestamp to observe another poll.

Use **stop-before-start**, never a second concurrent poller or deleted locks.
If shutdown/cancellation is unconfirmed, preserve the checkpoint and follow
[quarantine recovery](README.md#recovering-a-quarantined-source): a database
operator must establish prior server work has ended before replacement.
Client exit or socket closure alone is not proof.

## 4. Clean up

Stop the engine in Terminal A and wait for its prompt. From the repository root:

```powershell
python .\rust\otap-dataflow\crates\contrib-nodes\src\receivers\sql_server_receiver\ci\sql-server-fixture.py stop --directory $demo
if ($LASTEXITCODE -ne 0) { throw 'Fixture cleanup failed; inspect the error' }
```

Cleanup verifies ownership, removes only that container and generated secrets,
and retains public certificates, logs, and checkpoints for inspection. Start
fresh demos with a new fixture directory and checkpoints; do not reuse old
source progress against a recreated database.

For automated TLS/fault/replay tests, use the
[live-suite instructions](README.md#live-fixture-and-qualification) with a
separate fresh fixture. That suite modifies certificates and disconnects reader
sessions: do not run it alongside this demo.
