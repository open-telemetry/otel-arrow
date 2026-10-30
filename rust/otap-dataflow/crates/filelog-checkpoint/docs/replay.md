# Filelog checkpoint replay

Checkpoint replay validates stored transactions against an in-memory file table.
The [format contract][format] defines the transitions. The codec owns wire
structure, lengths, checksums, and transaction classes; replay adds stored-state
preconditions and atomic application. The durable format is unchanged.

## Caller responsibilities

The checkpoint store owns filesystem access, locks, append/sync, physical EOF
classification, tail repair, and compaction. Source readers establish identity
and continuity. Receiver integration establishes Ack authorization and
unresolved-delta ordering. Replay cannot establish those facts from bytes.

## API

`ReplayState::from_checkpoint(bytes, checkpoint_id, config)` validates the
immutable container prefix and returns `(state, wal_offset)`. Its `ReplayConfig`
is constructed with `ReplayConfig::new(max_tracked_files, max_snapshot_bytes,
&profile)`. It derives the compatibility digest and fingerprint window from the
same validated `FramingProfileParams`; callers cannot override that pair.
The namespace is the raw checkpoint ID; it is never derived from a path.
Trailing WAL bytes are not accepted as valid merely because prefix decoding
succeeded.

Call `state.replay_next(wal_suffix)` for each bounded input slice:

- `Applied { sequence, consumed }`: advance the input cursor by `consumed`.
  All operations and the sequence have committed to the in-memory table.
- `Incomplete { bytes, total_len }`: retain the suffix and obtain more input.
  No record or sequence changed. Only a store that proves physical EOF can
  classify a torn tail and authorize repair.
- `Empty`: the supplied slice was empty; this does not prove filesystem EOF.
- `Err`: abort recovery. Records and sequence remain unchanged by this call,
  but the error does not authorize skipping input or using a partial recovery
  result to resume source collection.

Replay starts at sequence one for each snapshot generation. It applies every
complete valid transaction present, including a transaction that survived an
unsynced append. There is no persisted last-synced sequence to use as a cutoff.
Sequence exhaustion fails before wrapping; an empty suffix remains valid.

`get` and `records` expose immutable records; iteration order is unspecified.
`generation` and `last_sequence` identify the recovered position. Snapshot
construction for compaction remains a store responsibility.

## Transaction rules

Operations observe earlier changes within the same transaction. Registration
and quarantine permit only their specified exact idempotent repetitions.
Progress checks the stored epoch and offset and preserves continuation until
its known boundary. Zero-delta progress cannot replace the guard or framing
state. Epoch-changing resets replace fingerprint evidence together with the
offset, guard, and framing state.

Administrative reset and removal validate the raw namespace before lookup,
including removal of an absent file. `keep_failed` is an exact operational
no-op, including preservation of last-seen time; attempted changes return
`KeepFailedStateChange`.

A non-administrative removal must be accompanied by a new, different file ID
registered at the same locator in that transaction. Removal must precede the
new registration, and a live replacement must remain in the final table.
Finalized records hold no live locator claim. Retention is not represented by
these removal transactions; it uses vetted snapshot compaction instead.

Reserved quarantine evidence fails before subsequent operations can erase it.
Nonreserved unknown diagnostic reason codes remain valid opaque values.

## Profiles and source-derived evidence

Framing-profile version/digest incompatibility is a per-file condition.
Replay preserves the record and its historical operations; callers must check
`profile_compatible(file_id)` before resuming a source. A false result never
authorizes a new identity, changed profile, or automatic offset reset.

For matching profiles, snapshot records and resulting operation state must
fit the configured fingerprint window. Incompatible records retain their old
bounded evidence because the selected window may differ from their original
configuration. The format's absolute fingerprint limit still applies.

Replay checks guard shape and zero-offset canonicality but cannot verify a
nonempty guard digest against actual source bytes. It also cannot prove the
LF ending a scan-to-LF continuation, an externally selected reset-to-end offset,
or the source correspondence of replacement fingerprints. Those proofs belong
to the reader and administrative caller before a transaction is encoded.

## Resource and performance model

The table and live-locator index contain at most `max_tracked_files` entries.
Each decoded transaction is bounded by the existing format caps: 4,096 distinct
progress operations, or 256 non-progress operations, and a 16 MiB body limit.
The caller releases each transaction's input before loading further WAL data.

Progress transactions validate every operation against immutable records before
changing any state. The codec guarantees distinct file IDs, so one progress
update cannot alter another update's preconditions. Commit changes only fixed-size
progress fields and releases finalized locator claims. It copies no fingerprint
or advisory-path buffers. An isolated allocation test compares 128 updates with
minimal and maximum evidence: allocation must be identical and below 128 KiB.

Non-progress transactions clone each touched existing record once into an
overlay. Their 256-operation cap limits duplication to at most 256 fingerprints
and paths: about 17 MiB of variable payload at the absolute format maxima,
plus fixed record and map storage. Unaffected records are never copied or
scanned. A locator overlay tracks released/new claims without full-table scans.
Expected work is proportional to transaction data and touched record payloads.
The conservative recovery model includes `3 * maximum snapshot bytes` during
WAL replay; it is not limited to one copy of checkpoint record state. Adoption
for live mutation would also need explicit runtime staging admission.

The record cap applies to the resulting table. An overlay may temporarily hold
up to 256 extra registrations, bounded by the non-progress transaction limit;
these are part of transaction staging. Registering one locator before removing
another is allowed at capacity if the final count fits. Same-locator replacement
still removes the prior live owner first. Commit removes records before inserting
new records so the authoritative table never temporarily exceeds its limit.
Returned errors discard the non-progress overlays or leave progress untouched.
Standard Rust allocation failure can abort the process; replay does not provide
a recoverable allocator-failure API.

The store still derives and enforces aggregate recovery memory, total WAL bytes,
transaction count, and full checkpoint-file bounds. It must validate the fixed
container header before allocating the declared snapshot input. This module
does not replace the admission formulas in the [conformance specification][bounds].

[format]: ../../../docs/filelog-checkpoint-format.md#replay-preconditions-idempotency-and-exact-transition-restrictions
[bounds]: ../../../docs/filelog-receiver-phase1-conformance.md#checkpoint-recovery
