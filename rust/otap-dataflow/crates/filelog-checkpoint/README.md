# Filelog checkpoint codec

`otel-arrow-dfe-filelog-checkpoint` implements the filelog-specific version 1
checkpoint wire format. It provides durable value types and bounded codecs for
`CURRENT`, snapshots, WAL headers, operations, and transactions.

The version 1 on-disk representation is the compatibility contract. The Rust
API is internal and experimental under repository policy. This crate performs
no filesystem access or checkpoint publication, and it does not introduce a
production filelog receiver.

Future consumers are the core-nodes filelog receiver and offline checkpoint
administration in `dfctl`. Dependency direction is from those consumers to
this crate; this crate intentionally has no runtime, engine, controller, OTAP,
Arrow, configuration, discovery, reader, or telemetry dependencies.

Operation decoding validates framing and self-contained transaction structure.
Checks that require a previously stored record or staged checkpoint table are
intentionally deferred to replay. In particular, structurally valid
`keep_failed` values are preserved for replay to compare bit-for-bit with the
stored quarantined record.

Snapshot decoding requires the caller's current maximum tracked-file count and
rejects a larger authenticated record count before allocating record storage or
decoding record bodies. WAL recovery uses `scan_next_transaction` so a consumer
can validate, apply, and drop one transaction before decoding the next; the
production API does not collect an entire WAL in memory.
