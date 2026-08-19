# Oracle OCI Receiver

This is a minimal, experimental Oracle database receiver. It uses the
OCI-backed Rust `oracle` crate and Oracle Instant Client.

The implementation follows the design document's scraper-style boundary:

- `receivers/scraper.rs` defines the small reusable `Scraper` trait and
  `ScraperReceiver`;
- `receivers/sql_polling.rs` defines the narrow `SqlPollingAdapter`,
  `Credentials`, `PageRequest`, `Page`, and compound-watermark types;
- `OracleAdapter` implements connection, bounded page fetching, value
  extraction, and Oracle-specific error classification;
- `OracleScraper` loads credentials, owns the adapter session, and converts
  returned rows to OTLP; and
- `ScraperReceiver` owns interval scheduling, non-overlap, lifecycle calls,
  shutdown, and downstream backpressure.

The Oracle scraper:

- creates one OCI session pool containing one database session;
- polls one configured SQL query on a fixed interval;
- limits every poll to `max_rows`;
- converts each row to one OTLP log record; and
- represents the row as JSON in the log body.

The configured SQL must start with `SELECT` or `WITH`. Use a database account
that has only the read permissions required by that query.

This intentionally stops short of a universal SQL framework. The first version
does not implement checkpoints, ACK/NACK tracking, retries, bind parameters,
watermark query generation, or complete Oracle-to-OTel type mapping. The
adapter currently receives `PageRequest { watermark: None, ... }` and rejects
non-empty watermarks explicitly. Because polling is stateless, every poll can
emit the same rows again.

Every selected column is currently requested from the driver as an optional
string. SQL `NULL` remains JSON `null`; types that the driver cannot convert to
a string fail the poll. Queries can use `TO_CHAR` or another explicit Oracle
conversion while typed OTel mapping is developed.

## Configuration

```yaml
type: urn:otel:receiver:oracle
config:
  connect_string: //localhost:1521/FREEPDB1
  username: PDBADMIN
  password_env: ORACLE_PWD
  query: SELECT SYSDATE AS CURRENT_TIME FROM DUAL
  poll_interval: 30s
  call_timeout: 10s
  max_rows: 100
```

The password is read from the environment variable named by `password_env`.
It is not stored in the pipeline configuration.

The receiver currently requires a single-core pipeline. This prevents
multiple pipeline cores from polling and emitting the same rows.

## Local Oracle Database Free

Make Oracle Instant Client available through `PATH`, set the password, then
run the sample pipeline:

```powershell
$env:PATH = "C:\path\to\instantclient_23_26;$env:PATH"
$env:ORACLE_PWD = "your-local-password"

cd rust\otap-dataflow
cargo run --features oracle-receiver -- `
  --config configs\oracle-oci-console.yaml `
  --num-cores 1
```

## Live Receiver Test

The normal unit tests exercise the shared lifecycle with a fake scraper, config
validation, error classification, and row encoding without requiring Oracle:

```powershell
cd rust\otap-dataflow
cargo test -p otap-df-contrib-nodes --features oracle-receiver `
  receivers::scraper
cargo test -p otap-df-contrib-nodes --features oracle-receiver `
  receivers::oracle_receiver
```

To run the opt-in live test:

```powershell
$env:PATH = "C:\path\to\instantclient_23_26;$env:PATH"
$env:OTAP_ORACLE_RECEIVER_E2E = "1"
$env:ORACLE_USERNAME = "PDBADMIN"
$env:ORACLE_PWD = "your-local-password"
$env:ORACLE_CONNECT_STRING = "//localhost:1521/FREEPDB1"

cargo test -p otap-df-contrib-nodes --features oracle-receiver `
  oracle_receiver_emits_rows_when_configured -- --nocapture
```

The live test starts the receiver, executes the query through its one-session
OCI pool, verifies that one row reaches the OTAP pipeline, and shuts the pool
down through the shared scraper lifecycle.
