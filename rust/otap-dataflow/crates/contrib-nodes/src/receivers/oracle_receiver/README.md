# Oracle OCI Receiver

This experimental receiver incrementally reads Oracle rows with an ascending
composite watermark:

```text
(TIMESTAMP column, signed 64-bit integer tie-breaker)
```

The operator supplies the complete SQL statement, including named Oracle binds
and the matching final `ORDER BY`. The receiver binds the last durably
acknowledged tuple, reads at most one page per poll interval, and emits one OTLP
logs batch at a time.

## Delivery and checkpoint behavior

- The initial tuple is explicit; a missing checkpoint starts there.
- The final emitted row supplies the next checkpoint candidate.
- The receiver requests native OTAP ACK/NACK feedback for each batch.
- ACK durably commits the candidate before another page can be in flight.
- NACK retains the committed tuple and replays after `nack_backoff`.
- Checkpoints are versioned, checksummed, configuration-fingerprinted, and
  revisioned. Each revision is fsynced and atomically renamed.
- Corrupt, incompatible, or configuration-mismatched state fails closed.
- Drain stops new polling and waits for the in-flight ACK/NACK until its
  deadline. Immediate shutdown never advances unacknowledged state.
- A process-local lease prevents two receivers from owning the same checkpoint.

The query must order the watermark columns ascending and make their tuple unique
for stable pagination. Inserts at or below an already committed tuple cannot be
observed. Use a database account with only the read permissions required by the
query.

Every selected column is currently requested as an optional string. SQL `NULL`
is emitted as JSON `null`; values that Oracle cannot convert to strings fail the
poll. Unbounded `CLOB`, `NCLOB`, `LONG`, and native `JSON` columns must be cast
to a bounded character type in the query. The two watermark columns are
additionally read as Oracle `TIMESTAMP` and signed 64-bit integer values.

## Configuration

```yaml
type: urn:otel:receiver:oracle
config:
  source_id: local-events
  connect_string: //localhost:1521/FREEPDB1
  username: PDBADMIN
  password_env: ORACLE_PWD
  query: >-
    SELECT EVENT_TS, EVENT_ID, PAYLOAD
    FROM OTAP_ORACLE_EVENTS
    WHERE (
      EVENT_TS > :last_ts
      OR (EVENT_TS = :last_ts AND EVENT_ID > :last_id)
    )
    ORDER BY EVENT_TS ASC, EVENT_ID ASC
  watermark:
    timestamp:
      column: EVENT_TS
      bind: last_ts
      initial: "1970-01-01 00:00:00"
    tie_breaker:
      column: EVENT_ID
      bind: last_id
      initial: 0
  checkpoint:
    directory: "${engine.state_dir}/oracle"
    max_consecutive_failures: 5
  nack_backoff: 1s
  poll_interval: 30s
  call_timeout: 10s
  max_rows: 100
  max_batch_bytes: 1 MiB
```

`source_id` is part of the durable checkpoint identity and must remain stable.
`password_env` names the environment variable containing the password; the
password is never stored in pipeline configuration. The checkpoint directory
placeholder uses `OTAP_DF_STATE_DIR`, falling back to `.otap-state`.

Configuration rejects unknown fields, zero or excessive limits, invalid
identifiers, missing/extraneous bind-name prefixes, multiple SQL statements,
comments, and a final ordering that does not exactly match the configured
watermark columns. The receiver requires a one-core source pipeline and one OCI
session.

`max_rows` bounds fetched rows. `max_batch_bytes` bounds the encoded OTLP
`LogsData`; if only a prefix fits, its final row becomes the candidate and the
remaining rows are fetched on a later poll. A first row that exceeds the byte
limit is an error rather than a skipped row.

Telemetry reports counts, batch sizes, revisions, batch IDs, and error classes.
It does not report SQL text, credentials, or watermark values.

## Local Oracle Database Free

Make Oracle Instant Client available through `PATH`, then prepare deterministic
rows with timestamp collisions:

```powershell
$env:PATH = "C:\path\to\instantclient_23_26;$env:PATH"
$env:ORACLE_USERNAME = "PDBADMIN"
$env:ORACLE_PWD = "your-local-password"
$env:ORACLE_CONNECT_STRING = "//localhost:1521/FREEPDB1"

cd rust\otap-dataflow
cargo run -p otap-df-contrib-nodes --features oracle-receiver `
  --example oracle_load_generator -- --reset --rows 1000 --collision-size 10
cargo run --features oracle-receiver -- `
  --config configs\oracle-oci-console.yaml `
  --num-cores 1
```

The generator is idempotent without `--reset`: existing event IDs are retained
and missing IDs are inserted deterministically.

## Tests

Unit tests require no Oracle installation:

```powershell
cargo test -p otap-df-contrib-nodes --features oracle-receiver `
  receivers::oracle_receiver
```

The opt-in live integration test creates an isolated table and covers
more-than-page input, a timestamp collision across a page boundary, NACK replay,
restart from an ACKed checkpoint, a concurrent ordered insert, and the final
checkpoint:

```powershell
$env:OTAP_ORACLE_RECEIVER_E2E = "1"
cargo test -p otap-df-contrib-nodes --features oracle-receiver `
  live_composite_watermark_checkpoint_when_configured -- --nocapture
```

It uses `ORACLE_USERNAME`, `ORACLE_PWD`, and `ORACLE_CONNECT_STRING` from the
load-generator example.

## Deferred scope

This slice does not implement snapshot or scalar modes, additional typed
mappings or SQL components, multiple queries or databases, multiple pages per
tick, multiple batches from one page, distributed ownership, checkpoint reset
or migration, exactly-once delivery, or dead-letter handling.
