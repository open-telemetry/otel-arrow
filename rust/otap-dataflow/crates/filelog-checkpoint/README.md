# Filelog checkpoint codec

`otel-arrow-dfe-filelog-checkpoint` defines the durable checkpoint format used
by the Filelog receiver.

It converts checkpoint values between Rust types and their exact on-disk byte
representation. The crate is specific to Filelog; it is not a general-purpose
storage or WAL library.

## What this crate provides

The crate encodes and decodes:

- `CURRENT`, which identifies the active checkpoint generation;
- snapshots containing the complete tracked-file state;
- WAL headers;
- checkpoint operations; and
- atomic WAL transactions.

All decoding is bounded. Lengths, counts, versions, reserved fields, and
checksums are validated before variable-size data is trusted.

## Format compatibility

Once a Filelog release writes a version 1 checkpoint, later releases must
continue to interpret those bytes using the same field layout, byte order,
operation codes, checksums, and corruption rules. An incompatible on-disk
change requires a new checkpoint format version and an explicit migration or
rejection policy.

The crate's Rust API remains internal and experimental. Rust types, module
layout, and function names may change while the version 1 byte format remains
compatible.

## Scope

This crate only handles checkpoint values and bytes. It does not:

- access the filesystem;
- create or lock checkpoint directories;
- append or synchronize a WAL;
- publish checkpoint generations;
- apply operations to previously stored state;
- compact or recover a checkpoint store; or
- implement the Filelog receiver.

Filesystem storage, replay, compaction, and receiver integration are separate
layers built on this codec.

## Decoding and replay

The codec validates the structure of each operation and transaction. Rules
that depend on a previously stored record are checked later while replaying the
operation against the checkpoint table.

For example, the decoder preserves structurally decodable `keep_failed` values
for later replay checks. The current producer rejects the locally impossible
case where `resulting_epoch` differs from `expected_quarantine_epoch`. Replay
must still compare the offset, frontier guard, fingerprint, framing state, and
all other stored quarantined state exactly.

Snapshot decoding takes the caller's current tracked-file limit. Before record
storage is allocated or a body is decoded, the authenticated count must fit
both that limit and the maximum number of minimum-width record frames
physically possible in the supplied snapshot bytes.

WAL recovery is incremental. `scan_next_transaction` returns at most one
validated transaction, allowing the caller to apply and drop it before
decoding the next transaction. The codec does not collect the complete WAL in
memory. `TransactionScan::Incomplete` means only that the supplied non-empty
slice cannot hold the complete next transaction. The codec cannot know whether
the slice reaches physical EOF and never authorizes truncation; a future store
must read again unless it independently confirms the permitted final torn-tail
condition at EOF.

## Consumers

The intended consumers are:

- the core-nodes Filelog receiver; and
- offline Filelog checkpoint administration in `dfctl`.

Both depend on this crate. The checkpoint crate does not depend on the
receiver, `dfctl`, the engine, controller, OTAP, Arrow, discovery, reader,
configuration, or telemetry layers.
