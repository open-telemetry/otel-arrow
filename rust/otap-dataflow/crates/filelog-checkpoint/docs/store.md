# Linux checkpoint store

`CheckpointStore` provides blocking filesystem storage for Filelog checkpoints.
It combines the version 1 codec and transactional replay with namespace
ownership, publication, WAL appends, synchronization, and startup recovery.

## Usage and ownership

`open` takes a borrowed directory descriptor from the engine's provisioned
`StateDirectory`, the exact checkpoint ID, and `StoreConfig`. The caller must
have validated and synchronized the root and its ancestors. The store uses
relative directory operations, never a pathname fallback.

Run the store on the dedicated read/checkpoint worker. It creates no threads
and performs no automatic retries. The caller owns bounded lock acquisition
and recoverable-I/O retries, cancellation, idle sync deadlines, drain, and
telemetry. Filesystem calls can block inside the kernel.

The namespace is `filelog/@v1/<checkpoint-id-hex>`, using lowercase hex of the
exact ID bytes. Every parent is synced, including existing directories.
Directories must be owned by the effective UID and have access bits 0700;
special mode bits are allowed on directories. Regular files require the same
owner, mode 0600 without special bits, and one hard link. Symlinks, special
files, identity substitution, and unexpected artifacts fail closed.
Permissions are not repaired automatically; the root must be protected from
untrusted writers.

A separate `ownership.lock` uses nonblocking `flock` to exclude independent
opens in the same process and other processes on supported local filesystems.
The lock outlives data handles and pending recovery. Drop releases ownership
but does not perform a drain sync; drop only after worker access has ceased.

## Publication and recovery

Only `checkpoint.db` is authoritative. Initial publication exclusively creates
`checkpoint.db.create.tmp`, writes and validates an empty generation-zero
container, syncs it, publishes with `RENAME_NOREPLACE`, and syncs the directory.
There is no fallback if the filesystem lacks no-replace rename or a required
sync.

Recovery follows the [publication protocol][publication]:

1. Validate the fixed header and snapshot byte budget before allocation, then
   validate the snapshot and replay one bounded WAL frame at a time.
2. Reject complete corruption and impossible transitions. Only a mechanically
   incomplete final frame at physical EOF may be discarded.
3. Sync the namespace directory before repair or temporary cleanup. Truncate
   only at or beyond `wal_offset`, preserving the header and snapshot.
4. Sync the data file, even if no repair was needed. Remove recognized leftover
   temporaries, sync their directory, and install the recovered state.

Missing authority is accepted only for recognized interrupted creation.
Creation and compaction temporaries are never promoted as saved state. A
compaction temporary without authority, both temporaries together, or unknown
artifacts fail closed. `RecoveryReport` exposes interrupted creation, repaired
byte count, and cleanup for caller-owned health reporting.

## Writes, synchronization, and failures

`append` encodes and preflights the entire transaction before writing, then
commits its state only after the complete write succeeds. Preparation stages
only touched records; ordinary progress copies no stored fingerprint or path
buffers. The caller must establish source identity/continuity, compatible
framing, Ack or explicit-loss authorization, and unresolved-delta ordering.

`Durability::Sync` synchronizes before returning success. `Deferred` applies
only to ordinary progress; all other operations, including rotation
finalization, still force sync. Drive explicit `sync()` while idle and on drain.
Append and ordinary sync verify that physical EOF matches the accepted append
boundary. A failed authority or length check latches failure in this instance.

A recoverable write error retains the exact attempt and blocks state access,
new appends, and ordinary `sync()`. A successful drain requires resolution.
Call `resolve_pending` under the held lock, even if no bytes were written:

1. Check store health, the pending attempt, file identity, and expected size.
2. Sync the original handle before reopening, preserving its writeback-error
   history for deferred progress and the failed attempt.
3. Validate recovery against the saved generation and append boundary. Sync a
   complete matching frame without duplication, or repair a proven torn suffix
   matching the retained frame's prefix, then append, apply, and sync once.

The returned receipt belongs to the original attempt: do not resubmit it.
Repeated recoverable failures retain the attempt and lock. Dropping the store
leaves startup recovery to inspect surviving state; it does not prove that
an attempted transaction was absent.

Any required file or directory sync failure returns `SyncFailed` and poisons
the instance. Subsequent state access, append, sync, and resolution return
`SyncPoisoned` without another barrier. This includes sync failures during
pending recovery. See the [failure contract][failures].

Poison is instance-local: there is no persistent marker or process-wide
history. A fresh handle may miss a writeback error reported to an earlier
instance or process, so reopen-and-sync cannot prove those bytes durable.
The lifecycle supervisor must preserve known terminal failures beyond teardown
and prevent automatic retry, reconfiguration, or restart from reopening the
namespace. This includes failures returned by `open` before a store exists.
Explicit storage incident recovery is required; restarting, deleting a marker,
or ordinary compaction does not restore lost writes. This library provides
no operational repair or clearance procedure.

## Resource bounds and limitations

The caller supplies snapshot and record budgets through `ReplayConfig` and
admits total receiver memory, including live recovery staging: the old table,
recovered state, and retained attempt can coexist. Snapshot input is released
before WAL scanning, which retains one frame rather than the whole WAL.
Allocation failure may abort the process; allocation is not fallibly reserved.

`StoreConfig` uses the [interacting WAL bound][publication]:
`min(max_wal_bytes, max_wal_transactions * WAL_MAX_TX_FRAME_BYTES)`.
The capped product uses saturating multiplication; subsequent file and
recovery sums use checked arithmetic and must fit 1 GiB. Raw thresholds remain
unchanged, and the byte threshold must fit one maximum-size frame.
`InvalidLimits` reports all four configured record, snapshot, WAL-byte, and
transaction limits. Some parameter checks share a reason, and budget errors
do not distinguish overflow from exceeding the ceiling. Receiver configuration
validation must provide those distinctions to satisfy the
[diagnostic contract](../../../docs/filelog-receiver-phase1-conformance.md#checkpoint-recovery).

WAL limits exclude the immutable prefix and allow equality. Recovery accepts
an older WAL above lowered thresholds within the hard 1 GiB file ceiling and
replay bounds. An append exceeding either threshold returns
`CompactionRequired` without writing. Compaction and retention are not
implemented, so further appends require a compaction implementation.

The store is Linux-only. Tests cover restart, locks, path safety, corruption,
tail preservation, admission, and injected I/O failures. They verify ordering
and failure latching, not kernel page-cache loss or power-loss durability;
integrated crash qualification is still required.

`nix` supplies descriptor-relative access. `rustix` supplies no-replace rename
on Linux GNU and musl, plus flock without raising the workspace MSRV.

[publication]: ../../../docs/filelog-receiver-phase1-spec.md#publication-compaction-cleanup-and-recovery
[failures]: ../../../docs/filelog-checkpoint-format.md#wal-append-failure-and-repair
