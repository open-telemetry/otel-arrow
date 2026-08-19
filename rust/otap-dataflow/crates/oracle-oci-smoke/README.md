# Oracle OCI smoke test

This is a minimal development tool that verifies an OCI-backed Rust client can:

1. Load Oracle Instant Client.
2. Authenticate to Oracle Database Free.
3. Ping the database.
4. Execute a query and print a bounded number of rows.

It is not an OTAP receiver. It intentionally excludes polling, pooling,
checkpointing, ACK/NACK handling, and datatype-preserving OTAP conversion.

## Prerequisites

- Rust and Cargo.
- Docker.
- 64-bit Oracle Instant Client Basic on `PATH`.
- Microsoft Visual C++ 2015-2022 Redistributable on Windows.

## Local Oracle Free test

The runner starts or reuses a container named `oracle-free-1`, prompts for the
local database password, waits for the database to become healthy, and executes
the smoke query. It does not create credential files or store a password in the
repository.

From this directory:

```powershell
.\run-local.ps1 -OracleClientDir C:\path\to\instantclient_23_26
```

If Instant Client is already on `PATH`:

```powershell
.\run-local.ps1
```

The default connection is `PDBADMIN@//localhost:1521/FREEPDB1`, and the default
query is:

```sql
SELECT SYSDATE AS CURRENT_TIME FROM DUAL
```

Run a different read-only query:

```powershell
.\run-local.ps1 `
  -Query "SELECT TABLE_NAME FROM USER_TABLES ORDER BY TABLE_NAME" `
  -MaxRows 20
```

## Existing database

The binary still supports mounted credential files:

```powershell
cargo run -p otap-df-oracle-oci-smoke -- `
  "//database-host:1521/SERVICE_NAME" `
  "C:\path\username.txt" `
  "C:\path\password.txt" `
  "SELECT SYSDATE FROM DUAL" `
  10
```

## Pull request scope

Suggested PR title:

```text
chore: add Oracle OCI connectivity smoke prototype
```

The PR should state that this validates the native Oracle client path only and
does not yet implement the production receiver lifecycle.
