# Experimental single-file tailing

This Linux-only developer example exercises the existing checked source access,
raw line decoder/framer, and discovery/read worker transports in a real process.
It is **not a registered OTAP receiver or a production filelog executable**.
It does not wire the scheduler, identity resolver, checkpoint store, OTAP
pipeline, or production file exporter.

## Build and run

From `rust/otap-dataflow`:

```sh
cargo build --locked -p otel-arrow-dfe-core-nodes \
  --no-default-features --features filelog-experimental --example filelog_tail
target/debug/examples/filelog_tail /tmp/input.log /tmp/output.log
```

Create the input before starting the process. The output must not already exist.
Append LF-delimited records to the input while the executable runs, then stop
with Ctrl-C. It starts at byte zero, preserves raw bytes including CR and NUL,
and polls the one exact input name every 100 ms. Each complete record reaches
the file sink before the read worker accepts its acknowledgement.

The example has a 16 KiB source buffer, a 64 KiB line-body limit, bounded
one-element observation and command queues, and one output handoff slot.
Oversized records fail instead of silently splitting or truncating. Source
replacement, disappearance, detectable truncation, and final-entry symlinks
also fail. Input and output must be ordinary files on a local Linux filesystem.
Parent-directory paths are caller-trusted, not a production discovery policy.

There are no durable identities, namespace leases, checkpoints, replay or
rotation guarantees. A write acknowledgement means the file sink flushed its
buffer, not that data was synchronized to storage. Ctrl-C cancels intake and
joins the workers against a shared five-second deadline; it does not drain an
in-flight write or complete an unterminated record. A worker panic or join timeout
is a process failure, not successful cleanup.

## End-to-end test

The optional third argument names a **new status file**. Its fixed-width decimal
line reports the last temporary-EOF source offset. This diagnostic is only a
test handshake; it must never be treated as a durable checkpoint or delivery
frontier.

```sh
python3 crates/core-nodes/tests/filelog_tail_e2e.py \
  --executable target/debug/examples/filelog_tail \
  --artifacts /tmp/filelog-tail-e2e
```

The test waits for explicit EOF offsets before appending each group, rather
than sleeping to assume the reader is ready. It checks ordered byte-for-byte
delivery after repeated appends, a record spanning multiple reads, raw bytes,
empty lines, and a partial record held across a temporary EOF. An unterminated
suffix must remain absent even after shutdown. Separate cases check that
existing output is preserved and oversized lines and truncation fail explicitly.
Child-process waits and cleanup are bounded; fixtures and logs are retained in
the artifact directory.

[The GitHub Actions workflow][workflow] builds the executable, runs these tests,
and uploads the executable and test evidence. It runs on relevant pull requests
and can also be started manually once the workflow is available on the
repository's default branch. It uses no secrets or write permissions.

[workflow]: ../../../../../../../../.github/workflows/filelog-tail-e2e.yml
