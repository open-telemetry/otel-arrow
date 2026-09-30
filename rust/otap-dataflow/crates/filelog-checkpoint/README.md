# Filelog checkpoint codec

`otel-arrow-dfe-filelog-checkpoint` defines the durable checkpoint format used
by the Filelog receiver.

It converts checkpoint values between Rust types and their exact on-disk byte
representation. The crate is specific to Filelog; it is not a general-purpose
storage or WAL library.

## What this crate provides

The crate encodes and decodes:

- a `checkpoint.db` container header identifying the WAL start offset;
- an embedded snapshot containing the complete tracked-file state;
- checkpoint operations; and
- atomic WAL transactions.

All decoding is bounded. Lengths, counts, versions, reserved fields, and
checksums are validated before variable-size data is trusted.

## Format compatibility

Merging this codec does not freeze the on-disk format. Version 1 remains
unfrozen until the first released conforming Filelog implementation.

The single-file layout is a pre-release revision of the proposed version 1
format. The earlier unshipped `CURRENT`/snapshot/WAL layout is not imported;
legacy artifacts must fail closed rather than start an empty checkpoint.

Once a Filelog release writes a version 1 checkpoint, later releases must
continue to interpret those bytes using the same field layout, byte order,
operation codes, checksums, and corruption rules. An incompatible on-disk
change requires a new checkpoint format version and an explicit migration or
rejection policy.

The crate's Rust API remains internal and experimental. After the format is
frozen, Rust types, module layout, and function names may still change while
the version 1 byte format remains compatible.

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

Snapshot decoding preserves reserved nonzero quarantine reason code `4`, but
the version 1 encoder rejects it with `ReservedReasonCode`. Store recovery
must reject this state before replay can change or remove the evidence.
Ordinary per-file administration requires successful recovery; structural
decoding alone does not authorize a reset or removal. Read-only diagnostic
tools may report the preserved value. Compaction must not pass it through,
rewrite it, or silently omit the record. The codec does not automatically
quarantine or repair state; any recovery procedure belongs to the separately
defined administrative contract.

`checkpoint.db` consists of a 24-byte checksummed container header, one
complete snapshot section, and zero or more WAL transactions. The header gives
the absolute WAL start offset. The snapshot retains its header, footer,
namespace binding, generation counter, and record encoding. The WAL has no
separate artifact header. Its first transaction sequence is one after creation
or compaction.

For bounded recovery, read exactly `CHECKPOINT_HEADER_BYTES` and call
`decode_checkpoint_header` with the configured maximum snapshot byte count
before reading its declared section. `decode_checkpoint_snapshot` validates
that complete prefix against the expected namespace, byte budget, and record
limit, then returns the snapshot and WAL offset. Extra WAL bytes in the input
are left unexamined, not accepted as valid transactions. A store can discard
the input prefix and read the WAL incrementally. The lower-level
`decode_snapshot` accepts only the exact snapshot section, excluding the
container header and WAL, and rejects trailing bytes. Its generation comes
from the snapshot itself; no external generation selector exists.

Compaction publishes a newly encoded empty-WAL container by syncing the
complete temporary file, atomically replacing `checkpoint.db`, and syncing the
namespace directory under the separate ownership lock. Recovery repeats the
directory barrier before new mutations. Appends must switch to the replacement
file handle. Tail repair may only truncate within the WAL section. These are
storage-layer obligations, not operations performed by this codec.

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

## Incomplete input and malformed payloads

`Truncated` means the supplied outer input lacks required bytes. It does not
prove physical EOF or show whether bytes are missing or a declared length is
wrong. `InvalidLength` similarly reports a fixed-width size mismatch; callers
must inspect its context and sizes. `SnapshotRecordCountExceedsPhysicalMaximum`
is an early allocation bound, not proof of truncation: missing bytes or an
invalid declared count can both trigger it. These errors never authorize repair.

After a record, operation, or transaction passes its complete frame and CRC
checks, an inner shortfall is `MalformedPayload` with the failing container,
required bytes, and remaining bytes. Other structural errors retain their
specific variants. This prevents an inner length error from looking like a
request to read more outer input. The store must combine codec errors with its
read/EOF evidence when reporting an incomplete authoritative generation.
For WAL scanning, only `Ok(TransactionScan::Incomplete)` (inside `Some`) denotes
a potentially incomplete suffix. Every scanner error is fail-closed corruption
or another explicit validation failure, never permission to truncate a tail.
