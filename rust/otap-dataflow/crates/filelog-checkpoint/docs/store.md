# Linux checkpoint store

Task 11 adds blocking storage to the existing codec and replay crate. The
wire format is unchanged. This is an internal foundation for the receiver and
future administration; it does not register a runnable Filelog receiver.

## Ownership and integration

`CheckpointStore::open` takes the borrowed descriptor of the engine's
provisioned `StateDirectory`, the exact checkpoint ID, and `StoreConfig`.
The engine must already have validated and synchronized the root and its
ancestors. A random directory descriptor is not an alternative provisioning
contract. The store checks root ownership and mode but cannot prove how the
caller established that capability. No engine dependency or pathname fallback
is introduced.

Use the dedicated read/checkpoint worker. All filesystem calls are blocking.
The store performs no automatic retry loop and does not spawn threads. The
worker owns lock acquisition deadlines, recoverable-I/O failure limits,
cancellation, periodic sync deadlines, and graceful drain. A failed file or
directory sync is terminal and must not enter an automatic receiver/store
replacement loop. Linux filesystem calls can still block inside the kernel.

The poison latch belongs to one store instance. Task 11 does not persist a
poison marker or keep a process-wide history, and a fresh `open` cannot detect
a failure reported to an earlier instance. Tasks 17 and 24 must enforce the
no-automatic-reopen policy in the lifecycle/supervision layer, including sync
failures returned by `open` before it constructs a store. That integration
must retain the known failure beyond instance teardown and prevent retry,
reconfiguration, or process supervision from automatically reopening the
affected namespace. It is a required integration gate, not enforcement
already supplied by this library. The storage incident recovery procedure and
any persistent marker/clearance protocol belong to Task 22a; deleting a marker
alone would not restore lost writes.

The exact ID is validated and mapped to lowercase hexadecimal bytes below
`filelog/@v1`. Each descendant is opened relative to its held parent handle.
Existing and newly created parents are synced unconditionally. Directories
require effective-UID ownership and mode 0700; regular artifacts require the
same owner, mode 0600, and one hard link. Directory special bits, including
inherited setgid, are accepted consistently with engine provisioning; the
low nine access bits must still be exactly 0700. Regular files retain the
strict 0600 check including rejection of special bits. Symlinks, special files,
identity substitution, and unknown artifact names fail closed. Permissions are
never silently repaired. The trusted service identity and privileged local users
remain outside the filesystem substitution threat boundary.

The separate `ownership.lock` uses nonblocking Linux `flock`. Independent
opens in one process and different processes exclude each other. Namespace
ownership survives data-file reopening and all failed append retries. Drop
releases handles, but does not perform or certify drain synchronization.

## Recovery and publication

Only `checkpoint.db` is authoritative. First publication exclusively creates
`checkpoint.db.create.tmp`, writes and validates an empty generation-zero
container, syncs the file, publishes with `RENAME_NOREPLACE`, and syncs the
namespace directory. No replacement fallback is used if the filesystem does
not support that operation.

Missing authority is accepted only with the exact interrupted-creation
inventory. A creation temporary is discarded, never promoted as saved state.
A compaction temporary or unexpected artifact with missing authority fails
closed. With valid authority, either recognized leftover temporary can be
cleaned after recovery and the directory barrier. Two temporaries fail closed.

Recovery validates the fixed container header and configured byte cap before
allocating the snapshot prefix. It releases that input before reading the WAL,
then validates and applies one bounded frame at a time. A complete invalid
frame or impossible transition fails the namespace closed. Only a mechanically
incomplete final frame at the observed physical EOF can be removed. Repair
truncates at an absolute offset at or beyond `wal_offset`, preserving every
header and snapshot byte.

The namespace directory is synced after replay and before repair or cleanup.
The recovered data file is synced even when no repair was needed. That barrier
covers ordinary recovery, but cannot prove durability after a prior process's
writeback failure during the same boot. Kernel error reporting can forget an
already-observed error while affected bytes have not reached disk. The caller
must require explicit storage incident recovery before reopening a namespace
with a known failed sync; dropping the store, repeating fsync, or restarting
the process is not a repair. Temporary cleanup has its own directory sync.
`RecoveryReport` exposes interrupted creation, repaired byte count, and cleanup
for future bounded health reporting; this crate does not emit telemetry.

## Live transactions and durability

`append` owns a vector of operations and assigns the next sequence. The complete
frame is encoded, wire-validated, and semantically prepared before any write.
Preparation reuses replay's touched-record overlay and distinct-file progress
validation. It does not clone the full table. Commit follows successful append;
no operation prefix becomes live on a partial write.

The caller establishes downstream Ack or explicit-loss authorization, source
proof, compatible framing state, and unresolved-delta ordering. Storage does
not infer any of these from valid checkpoint bytes.

`Durability::Sync` syncs before returning a successful receipt.
`Durability::Deferred` is allowed only for ordinary progress; registration,
metadata, fingerprint, reset, quarantine, removal, and rotation finalization
still force sync. This conservative policy is stronger than required for
metadata and fingerprint updates. The worker must drive explicit `sync` while
idle as well as on drain. Before ordinary sync, the store checks that physical
EOF still equals its accepted append boundary; a mismatch or failed metadata
check latches an authority failure without advancing the synced sequence.
A successful sync covers all prior applied sequences.
Any failed sync returns `SyncFailed` and permanently poisons the store.
Subsequent state access, append, sync, and pending resolution return
`SyncPoisoned` without attempting another barrier. This includes a sync failure
inside recovery of a pending write. Previously unsynced progress is never
certified by a later fsync-only success. Namespace ownership remains held
until the store is dropped after worker quiescence. The
[format failure contract](../../../docs/filelog-checkpoint-format.md#wal-append-failure-and-repair)
explains the storage-remediation requirement. Fresh-generation repair from
trusted state requires a separately approved implementation; Task 11 does not
supply it or claim that merely retrying Task 13 compaction repairs storage.

A recoverable append error retains the exact encoded attempt and blocks state
access and new appends. If no sync failure occurred, call `resolve_pending`
under the same held lock. Resolution first checks store health, the presence
of a pending attempt, file identity, and the expected file-size range. It then
syncs the original open handle before reopening for recovery, preserving that
handle's writeback-error history for earlier deferred progress and the failed
attempt. Failure at this barrier poisons the store before repair or re-append.
After that barrier:

- A complete valid matching transaction is synced without appending it again.
- A proven incomplete suffix matching the beginning of the retained frame at
  the saved boundary is repaired and synced, then the retained transaction is
  appended, applied, and synced.
- A changed generation, section boundary, earlier accepted boundary, or
  complete mismatching transaction fails closed before repair.

Even a no-write error follows this conservative reopening path. Resolution
returns the receipt for the original attempt, so the caller releases that
attempt instead of resubmitting it. Repeated failures retain the frame and lock.
Dropping the store after a recoverable write error leaves ordinary startup
recovery responsible for surviving state; it does not promise the attempt was
absent. Dropping a sync-poisoned store never authorizes automatic reopening.

A failed authority check is also latched: a replaced inode, changed file
length, or invalid namespace must not leave stale state readable. Ownership is
declared last in the store so it outlives every data handle during drop.

## Bounds and remaining work

The caller derives `ReplayConfig`'s snapshot and record budgets from receiver
configuration. `StoreConfig` additionally caps WAL bytes and transaction count
and checks the documented conservative file, snapshot-recovery, and WAL-recovery
formulas against 1 GiB with checked arithmetic. Admission uses the
[interacting WAL bound](../../../docs/filelog-receiver-phase1-spec.md#publication-compaction-cleanup-and-recovery):
`min(max_wal_bytes, max_wal_transactions * WAL_MAX_TX_FRAME_BYTES)`.
The capped product uses saturating multiplication before taking the minimum:
if the product exceeds `u64::MAX`, the representable byte threshold must be
the smaller bound. File and recovery sums still use checked arithmetic and
must fit the ceiling. Raw thresholds remain unchanged for append enforcement,
including the requirement that the byte threshold fits one maximum frame.
Rejections retain the four
configured record, snapshot, WAL-byte, and transaction limits in the error.
Recovery applies the fixed 1 GiB file ceiling independently of the current
append thresholds, so lowering those thresholds still permits opening a valid
older WAL. Snapshot, record, and transaction validation remain bounded. The
receiver still admits the combined reader, checkpoint, worker, batch, and
framing memory budget.

Snapshot input is bounded before allocation. WAL input retains only one frame,
not the entire WAL. A failed append retains one encoded frame; resolution also
holds the previous live table while reconstructing recovery state. Caller
admission must include live recovery staging as well as ordinary preparation.
Progress preparation copies no stored fingerprint or advisory-path buffers.
Standard Rust allocation failure may abort; allocation is not fallibly reserved.

WAL limits exclude the immutable prefix. Equality is allowed; an append that
would exceed a limit returns `CompactionRequired` without writing. This also
applies after recovery of a WAL larger than a lowered threshold. Task 13 must
publish a compacted generation before retrying that logical transaction. This
module does not implement compaction or retention, and a store that reaches
its threshold cannot make further append progress through this API yet.
Receiver deadlines, retries, Ack integration, and telemetry remain later tasks.

Linux tests cover restart, nonempty-snapshot preservation, every truncation
position in a progress frame, corruption, locks, namespace/path safety,
preflight rejection, threshold changes, admission, and injected write/sync
failures. The sync tests prove the terminal latch prevents a successful retry;
they do not emulate kernel page-cache loss or demonstrate recovery from a
real writeback failure. Fault injection and ordinary filesystem restart tests
do not establish power-loss
qualification. Integrated Linux crash qualification remains Task 26; macOS
and Windows storage remain Task 28.

The syscall wrappers have different portability roles: `nix` supplies the
existing descriptor-relative access patterns, while `rustix` supplies
no-replace rename on both Linux GNU and musl and flock without requiring a
standard-library API newer than the workspace MSRV.
