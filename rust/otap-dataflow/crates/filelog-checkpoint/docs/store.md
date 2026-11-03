# Linux checkpoint store

`CheckpointStore` provides blocking filesystem storage for Filelog checkpoints.
It combines the version 1 codec and transactional replay with namespace
ownership, publication, WAL appends, synchronization, and startup recovery.

## Usage and ownership

`CheckpointStore::open` takes `&StateRoot`, the exact checkpoint ID, and
`StoreConfig`. Construct `StateRoot` from the engine's provisioned
`StateDirectory` using `StateRoot::from_borrowed_fd`. The caller must first
validate and synchronize the root and its ancestors. Use a readable directory
descriptor; `O_PATH` handles are rejected.

`StateRoot` owns a duplicate of the supplied descriptor. Each store retains
its own root handle and revalidates it when opening the namespace, so the
caller may close the original descriptor and drop `StateRoot` after opening.
A root capability can serve multiple checkpoint IDs. Filesystem access stays
relative to held directory handles, without reopening paths.

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

Active-file writes use `O_APPEND`, so recovery reads and seeks cannot redirect
later writes into the snapshot or earlier WAL.

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

## Compaction and retention

`compact()` replaces the update history with a snapshot of the latest applied
state. It starts a new generation with an empty WAL. It does not advance a
source position.

Successful compaction makes all earlier applied progress durable, including
progress written with `Durability::Deferred`. `synced_sequence()` then starts
at zero, and the next append uses sequence one. Compare sequence numbers only
within the same generation.

`compact_with_retention(ids)` also removes the complete caller-approved set in
one snapshot. Unknown, duplicate, and quarantined IDs reject the whole set
before writing. An empty set does nothing; `compact()` still works without
removals.

The receiver decides when removal is safe. It must check continuous absence
and all remaining uses: discovery, readers, descriptors, leases, candidates,
open and retained batches, carry-over, and rotation state. Keep those checks
valid until publication finishes. Age alone does not allow removal, and
quarantined records are never ordinary retention candidates. The receiver also
schedules retention checks when the WAL is quiet.

The store holds the namespace lock throughout publication:

1. Write and validate a complete temporary checkpoint, then sync and close it.
2. Check and sync the old append handle before closing it, so earlier write
   errors cannot be missed.
3. Replace the active checkpoint atomically and sync the directory.
4. Reopen and validate the replacement before installing its state and resuming
   appends. Generation overflow fails before writing.

`CompactionStagingInvalid` means the temporary failed validation before
replacement. The active file is unchanged, but the attempt needs cleanup.
`AuthorityChanged` means an active-file or ownership check failed.

`has_unresolved_compaction()` identifies an attempt that still needs resolution.
State use, append, sync, and another compaction stay blocked. If recovery is
allowed, `recover_compaction()` validates the active file under the same lock
and cleans up the temporary. It returns `Some(receipt)` when the proposed
snapshot was published, or `None` when the old generation remains active.
If replacement returned success, recovery must validate the new generation;
it cannot fall back to the old one. If the outcome is uncertain, recovery may
accept either complete generation.
Recheck removal eligibility before another attempt.

`can_recover_compaction()` reports whether this store permits recovery. The
caller still decides whether the error is retryable and limits retries. Any
failed file or directory sync is terminal. Reopening cannot clear that failure.

When an append returns `CompactionRequired`, the worker keeps the authorized
operation, pending admissions, and leases until it can compact and retry the
append. Run compaction and recovery one at a time. Restart identity scans when
saved state changes, and remove runtime tracking only after the complete
removal set has been durably published.

## Resource bounds and limitations

The caller supplies snapshot and record budgets through `ReplayConfig` and
admits total receiver memory, including live recovery staging: the old table,
recovered state, and retained attempt can coexist. Snapshot input is released
before WAL scanning, which retains one frame rather than the whole WAL.
Compaction calculates the snapshot size without allocating or encoding record
buffers. It then allocates one output buffer and encodes borrowed records into
it. Writes cannot grow that buffer, even in release builds. The encoder checks
both file IDs and live locators for duplicates. The budget includes the snapshot
header and footer; the separate checkpoint header adds 24 bytes to the buffer.

Compaction keeps the old table while preparing the replacement. It does not
clone that table for encoding. Each validation stage still builds a replay
table, so validation can determine peak memory. The stages run one at a time.
See [compaction measurements](compaction-performance.md) for the complete-call
allocation and timing results.

The old complete file and replacement
snapshot coexist on disk. Provision space for the actual validated active-file
length (up to 1 GiB after thresholds are lowered) plus one replacement header
and snapshot within the configured snapshot bound. No old-WAL-sized allocation
is introduced. Allocation failure may abort the process; allocation is not
fallibly reserved.

If constructing `StoreConfig` directly, allow enough snapshot space for the
full retained table, including older fingerprint evidence. Receiver callers
should use `CheckpointBounds::for_population` to size that budget. Validation
checks resource ceilings; it does not guarantee that the chosen record count
fits the snapshot budget. An undersized budget can allow appends until the WAL
is full, then prevent compaction with `RecoveryLimit`. Retrying with the same
state and limits will not make it fit.

`StoreConfig` uses the [interacting WAL bound][publication]:
`min(max_wal_bytes, max_wal_transactions * WAL_MAX_TX_FRAME_BYTES)`.
The capped product uses saturating multiplication; subsequent file and
recovery sums use checked arithmetic and must fit 1 GiB. Raw thresholds remain
unchanged, and the byte threshold must fit one maximum-size frame.
`InvalidLimits` reports invalid structural parameters. `StoreError::Admission`
wraps the portable `AdmissionError` and retains the configured population,
snapshot, WAL-byte and transaction limits. It distinguishes arithmetic overflow
from a calculated value above its ceiling and names the failed formula.
Receiver configuration uses the same portable bounds and additionally reports
the contributing receiver settings, including the fingerprint window. See the
[diagnostic contract](../../../docs/filelog-receiver-phase1-conformance.md#checkpoint-recovery).

WAL limits exclude the immutable prefix and allow equality. Recovery accepts
an older WAL above lowered thresholds within the hard 1 GiB file ceiling and
replay bounds. An append exceeding either threshold returns
`CompactionRequired` without writing. The worker calls `compact()`, then retries
the still-authorized transaction. Equality does not trigger an extra compaction.
Compaction is synchronous on the dedicated checkpoint worker; it does not run
on the pipeline's async thread or create background tasks.

The store is Linux-only. Tests cover restart, locks, path safety, corruption,
tail preservation, admission, and injected I/O failures. They verify ordering
and failure latching, not kernel page-cache loss or power-loss durability;
integrated crash qualification is still required.

`nix` supplies descriptor-relative access. `rustix` supplies no-replace rename
on Linux GNU and musl, plus flock without raising the workspace MSRV.

[publication]: ../../../docs/filelog-receiver-phase1-spec.md#publication-compaction-cleanup-and-recovery
[failures]: ../../../docs/filelog-checkpoint-format.md#wal-append-failure-and-repair
