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

For an interactive receiver-to-console demo, update the connection and
credential paths in `configs/oracle-oci-console.yaml`, then run:

```powershell
cargo run --features oracle-receiver -- `
  --config configs/oracle-oci-console.yaml --num-cores 1
```
