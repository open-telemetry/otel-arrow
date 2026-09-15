# Oracle composite watermark receiver

The opt-in `oracle-receiver` feature registers
`urn:otel:receiver:oracle`. The receiver polls a customer-authored, read-only
`SELECT` after a durable composite watermark, emits one typed OTLP `LogRecord`
per row, and advances its checkpoint only after a downstream acknowledgement.

## Delivery guarantees

Delivery is **at least once**. A crash, a negative acknowledgement, or a drain
that ends before feedback arrives can re-emit rows, but an unacknowledged row is
never intentionally skipped.

Each source has at most one page awaiting downstream feedback. An ACK commits
that page's cursor; a NACK retains the durable cursor and replays the page.

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

Operational bounds are explicit:

- `query.max_rows_per_poll` -- hard row ceiling for one poll
- `query.fetch_size` -- target rows per Oracle driver fetch, capped by the row
  and byte limits
- `query.max_batch_bytes` -- **exact** serialized OTLP payload ceiling

The receiver emits the largest non-empty row prefix that fits
`max_batch_bytes`, and the committed candidate is always the cursor of the last
row actually emitted. The same limit bounds normalized rows before encoding.
Rows beyond a ceiling are returned by the next poll rather than dropped. If a
single first row exceeds the byte ceiling, the poll fails explicitly instead
of skipping it.

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

A filesystem lease keyed by the checkpoint identity prevents two receiver
processes sharing the state directory from advancing the same checkpoint.
Deployments must still ensure one replica owns each checkpoint source when the
state directory is not shared.

## Telemetry

The receiver registers the `receiver.database` metric set covering starts,
polls, query failures, batches, rows, encoded bytes, acknowledgements, negative
acknowledgements, replays, stale feedback, checkpoint commits, checkpoint
failures, checkpoint cleanup failures, cancellations, drains, and shutdowns.

## Running

Use `configs\oracle-oci-console.yaml` as the complete example. Credentials must
be regular UTF-8 files; keep their contents outside YAML and environment
variables. Run the receiver on a single pipeline core:

```powershell
cargo run --features oracle-receiver -- `
  --config configs\oracle-oci-console.yaml --num-cores 1
```

## Deterministic load generation

The `oracle_load_generator` example creates `OTAP_ORACLE_EVENTS` and upserts a
requested number of deterministic rows. `--collision-size` controls how many
consecutive rows share one timestamp, which verifies that the composite
timestamp plus tie-breaker cursor neither skips nor duplicates rows at page
boundaries. `--reset` recreates the table before loading it.

Run it directly against an existing Oracle instance using
`ORACLE_USERNAME`, `ORACLE_PWD`, and `ORACLE_CONNECT_STRING`:

```powershell
cargo run -p otel-arrow-dfe-contrib-nodes `
  --features oracle-receiver --example oracle_load_generator -- `
  --reset --rows 10000 --collision-size 100
```

For the opt-in live smoke test, set `OTAP_ORACLE_RECEIVER_E2E=1` plus
`ORACLE_CONNECT_STRING`, `ORACLE_INSTANT_CLIENT_DIR`,
`ORACLE_USERNAME_FILE`, and `ORACLE_PASSWORD_FILE`, then run:

```powershell
cargo test -p otel-arrow-dfe-contrib-nodes `
  --features oracle-receiver `
  emits_oracle_rows_when_live_test_is_enabled -- --nocapture
```
