# File identity and local ownership

`identity::IdentityOwner` decides whether to resume, register or quarantine a
file. It implements Task 12 on a blocking worker, using the existing Linux source
primitives and checkpoint store.

This is a standalone component. It does not register a receiver, schedule
readers, finalize rotated files or decide which progress an Ack authorizes.

## Admission protocol

Admission has four stages: restore, prepare, commit and read.

### 1. Restore discovery state

Restore durable records into discovery. Use complete saved native paths when
available; never open a file using a truncated advisory suffix.

Map each fresh discovery candidate to `identity::Candidate`. Task 7b's
`Replacement::Possible` maps to `Context::Replacement`. If the candidate has an
exact-locator checkpoint record, that record takes precedence over the context.

### 2. Prepare a candidate

Call `prepare`. It acquires the process-wide locator lease first. A checkpoint
quarantine or profile block can return immediately without opening the file.
Otherwise, preparation reserves the shared probe budget and calls the supplied
source opener.

**The opener must check current policy before returning a `SourceFile`.** This
includes link policy, resolved-target exclusions and checkpoint namespace
exclusion. `SourceFile::open_at` checks source type and locator, but does not
perform all these policy checks. The integration adapter must reuse discovery's
checks against the pinned file.

Preparation checks the fresh prefix, file size and committed-frontier evidence.
It returns a proposed file ID and decision without writing a checkpoint. It
closes temporary source descriptors before releasing the probe reservation.
Dropping an uncommitted plan releases its lease.

### 3. Select and commit a group

Prepare one bounded group at a time. Ordinary progress may continue while the
group is being prepared.

1. Pass at most `admission_limit()` plans to `select_batch`, before discovery
   preflight. Selection counts the group's net increase in durable records.
   Recovery and same-locator replacement need no extra slot.
2. Report capacity feedback for rejected plans, then drop them on the worker.
3. Have discovery preflight exactly the selected group, keeping its pass and
   offers frozen. Call `commit_batch` inside discovery's durable callback.
4. Publish the whole group only after commit succeeds or a pending write is
   successfully resolved. Never publish a subset.

`commit_batch` accepts plans prepared at one identity revision. Its limit allows
for up to three operations per plan and the maximum encoded operation size,
within the WAL limits of 256 operations and 16 MiB. If a preflighted group exceeds
capacity, commit rejects the whole group; it does not select a smaller subset.

Checkpoint append/replay remains responsible for durable invariants and sync
barriers. The admission boundary lets discovery inspect proposed identities
before durable work; it does not add another transaction manager.

### 4. Hand the result to a reader

Move each returned `ReaderSeed` into its logical reader. Revalidate the source
when reopening it. Closing or evicting a descriptor does not release the lease;
keep it until the logical reader retires. Discovery's `TrackedFile.resident`
means logical reader ownership, not an open descriptor. Keep it set through
descriptor eviction and while delivery work prevents retirement.

Before releasing a lease or superseding an identity, lifecycle handling must
prove that no unresolved source delta remains in an open batch, retained batch
or carry-over.

Reader seeds, plans and lease guards must be dropped on the blocking worker.
Lease release takes the process-local registry mutex, so it must not run on an
async pipeline task.

### When a plan becomes stale

Each plan records its lease owner, checkpoint generation, namespace identity
revision and a compact stamp of its exact-locator record.

- The stamp checks lifecycle, epoch, offset, frontier and framing-resume state.
- The revision covers fingerprint, profile and path changes, so plans do not
  copy that evidence. It advances after a successful non-progress operation or
  finalizing progress operation. Rejected appends do not advance it.
- Ordinary Ack progress on other files does not invalidate the plan. Identity
  changes anywhere in the namespace can invalidate it; this is intentionally
  conservative.

On `StalePlan`, gather fresh evidence. Validate all group members together so
committing one file does not invalidate the remaining plans one by one.

## Results, failures and shutdown

### Admission results

Early `Admission::Quarantined` and `ProfileBlocked` report checkpoint state for
the offered locator. They do not prove current path validity, policy eligibility
or successful reconnection. They neither open source content nor mutate state.

After commit or pending resolution, `CommittedAdmission::decision()` returns the
original decision, including reset and superseded-ID details for health events.
`ResolvedWrite.admissions` contains the original reader/quarantine results for
admissions, and is empty for progress or maintenance transactions.

A committed quarantine still reports its ID if lease cleanup fails. In that
case, `release_error` is present and the owner is stopped. Do not schedule any
returned readers when ownership cleanup has stopped the owner.

### Checkpoint failures

| Outcome | Required handling |
| --- | --- |
| `AppendRejected` | Nothing was written. Invalid encoding/state requests leave the owner usable; compaction and authority failures stop it. |
| `PendingWrite` | A recoverable write attempt has an unresolved outcome. The owner retains the exact result and leases. Back off on the worker, then call `resolve_pending`. |
| Failed sync, corruption, authority loss or exhausted failure budget | Terminal: stop the owner. |

After `SyncFailed` or `SyncPoisoned`, the supervisor must prevent reopening the
namespace in-process, including automatic receiver replacement. This applies to
errors from `CheckpointStore::open` before an owner exists, errors returned by
owner operations, and errors inside `SyncOutcome.growth`. A successful required
progress barrier does not make a later growth sync failure recoverable.

For `PendingWrite`, never resubmit the operation or reopen the store. Resolution
uses the store's original-handle protocol and returns the retained result once,
for both admissions and ordinary progress. The configured consecutive failure
budget includes the initial error.

Resolution revalidates retained admission leases first. A registry failure stops
the owner and retains the pending result.

### Progress and shutdown

Delivery calls `append_checkpoint` only for authorized progress. Use
`sync_checkpoint` for interval and drain barriers. An in-process resume may use
already applied Acked progress, but the crash-replay window does not change until
sync completes.

Shutdown must first end worker access. The owner closes checkpoint handles before
dropping retained pending outcomes and leases. Existing reader leases remain
owned by their readers. Tasks 14/24 own bounded cross-thread requests, responses
and cancellation.

## Decisions and association

### Recovery and replacement

| Evidence | Decision |
| --- | --- |
| Exact locator is quarantined | Keep it quarantined, even if content changes. |
| Exact record has an incompatible profile | Block it without changing its identity or applying mismatch policy. |
| Compatible Active record has matching prefix/frontier and a saved offset within the file | Resume it. |
| Same-locator Active record fails continuity | Atomically remove it and register a new ID. Quarantine, when required, is part of that transaction. |
| Recognized replacement | Start a new identity at zero, including under `fail` and `start_at: end`. |
| Changed locator | Never inherit the old progress or remove the old record. |

Recognized replacement context can come from a saved binding after restart; it
is not limited to rotation witnessed by a running reader. It does not prove that
the candidate is the old stream or authorize use of the old offset.

### Association without recognized replacement

Association examines retained Active records using either:

- **Full path evidence:** path kind, complete native-path length and digest
  match. A truncated suffix never associates.
- **Prefix evidence:** a nonempty common raw prefix matches at the same
  ignored-header offset. Short files and growing prefixes count; empty evidence
  does not. Different framing settings do not reinterpret these raw bytes.

Association applies the mismatch policy to choose a new starting offset. No old
progress transfers. It bypasses `start_at` and age filtering.

This conservative rule helps capture surviving bytes from copy/unlink and
copytruncate archives when matching evidence remains. Independent files with
common prefixes can also associate. The configuration guide explains that
tradeoff, especially for small windows and `fail`. Association is neither proof
that two files are one stream nor a guarantee of lossless copytruncate capture.

### Read failures and age filtering

Frontier EOF, a short frontier window or a digest mismatch means continuity
failed. Environmental I/O errors instead preserve checkpoint state and propagate
to the worker for bounded backoff and health reporting. Prefix and frontier reads
share a cancellation-aware helper that retries `Interrupted` at most eight times
per bounded read.

This PR updates the receiver specification and conformance matrix for this
frontier-error rule. The [design rationale][clarifications] explains the change
from the earlier wording and its availability tradeoff. Reviewers must approve
the amendment before it supports a Phase 1 conformance claim.

Age filtering uses one supplied wall-clock time and the open handle's mtime.
Files exactly at the age threshold remain eligible. Future mtimes count as age
zero. An unavailable mtime does not exclude a file; the prepared result exposes
it for a bounded health event. Ignored files create no checkpoint record.

## Fingerprint growth and synchronization

A reader seed retains the freshly validated prefix. When the reader observes a
valid extension, it must call `queue_fingerprint`, including after an empty-file
registration. The bounded queue combines updates for each file. If the call
returns false, retain the bytes on the reader and retry after a flush.

`flush_fingerprints` writes a separate, bounded non-progress transaction. Growth
and progress never share a WAL frame. Admission does not add a growth fsync for
each resumed file. Extensions for stale lifecycles or epochs are discarded.

`sync_checkpoint` first completes the required progress barrier, then tries
optional growth. Its `SyncOutcome` reports both the confirmed sequence and the
growth result. **Callers must handle both fields:** a growth failure cannot undo
the confirmed barrier, but may still require pending resolution or shutdown.

| Growth outcome | Effect |
| --- | --- |
| Pre-write `CompactionRequired` | Keep the queue; do not stop the owner. |
| Known encoding/replay rejection | Report the error and discard that optional batch, rather than retrying it forever. |
| Authority, write or sync failure | Report it and apply the normal pending-write or terminal-failure handling. |

## Resources and remaining integration

### Lookup and memory costs

Exact-locator recovery reuses `ReplayState`'s existing index. Only new unmatched
candidates scan Active history, checking cancellation between records. Worst-case
association work is O(tracked records * fingerprint width); this is not a finite
latency guarantee. Measure churn before adding indexes. Identity does not keep a
second authoritative record or path table. The adapter needs a bounded mapping
between discovery's `TrackedId` and checkpoint `FileId`; this is correlation
state, not a copy of checkpoint records.

The executor keeps one configured fingerprint buffer. Each prepared plan holds
one reader seed with a bounded prefix, a compact exact-record stamp, at most
three checkpoint operations and bounded evidence copies. The stamp does not copy
the checkpoint's retained fingerprint. `scratch_bytes()` reports allocated
scratch capacity.

Keep at most `admission_limit()` prepared plans outstanding, including plans
waiting for commit or pending-write resolution. Discovery may retain more offers;
prepare them one bounded group at a time. The lease reserve covers one group,
not the entire offer population. Tasks 19/36 must account for evidence copies,
channel storage and checkpoint staging.

### Lease ownership

Construct each lease owner with `IdentityOwner::required_lease_capacity(inputs)`.
This checked limit is `max_tracked_files` plus one bounded admission group. The
extra leases let existing records recover or be replaced even while speculative
new-file plans hold leases. Durable records remain separately limited by
`max_tracked_files`; count the temporary leases as admission memory.

Lease entries are bounded by the sum of owner capacities. The B-tree holds
locator keys and small owner references, and frees nodes on deletion instead of
retaining a high-water hash-table allocation. Owner names are shared across their
entries and limited to 255 bytes. `LeaseRegistry::len()` reports entry count;
include allocator and node overhead in integrated memory measurements.

Use one process registry across all nodes, regardless of checkpoint namespace.
Conflicts report the existing owner's name for `filelog.local_locator_conflict`;
do not silently ignore them. Tests use isolated registries. The registry mutex
covers only map and counter operations: no I/O, channel waits or telemetry
formatting occurs while held. The checkpoint store owns namespace locking.
These are local ownership guarantees, not distributed fencing. A conflict exposes
`same_owner`, based on registry identity rather than node name. Self-conflict
means the existing reader or plan still owns the locator; it must not produce a
cross-node conflict event or release that ownership.

### Discovery and lifecycle integration

The merged discovery API is `Reconciler::admit_batch`. The worker adapter owns
this sequence:

1. Prepare at most one admission group and call `select_batch`.
2. Map the selected plans to exactly the corresponding `AdmissionChange` values.
3. Call `admit_batch`, which preflights the entire group before its callback.
4. In the callback, commit that exact group. Resolve recoverable pending writes
   there, with bounded worker backoff and cancellation handling, while discovery
   remains exclusively borrowed. Do not return `Ok` while the outcome is uncertain.
   If cancellation leaves the write unresolved, return `Err`, stop reconciliation,
   and retain ownership until the worker has stopped accessing the store.
5. After successful publication, hand reader seeds to the reader scheduler.

The adapter maps lifetime-unique 64-bit discovery IDs to 128-bit durable file
IDs. Bound this mapping by tracked capacity plus one admission group, including
reserved IDs. Restore terminal records too: they consume tracked capacity until
retention removes them. Membership must agree after publication and retirement;
a frozen durable transition may temporarily precede discovery publication.

Map a decision's `superseded: Some(id)` to `replace_id`. This same-locator
replacement requires the old discovery entry to be non-resident and eligible
for replacement. It needs no extra tracked slot. Changed-locator rotation keeps
its predecessor and needs another slot. Publish committed quarantine with
`active: false` and no reader.

Callback results have distinct meanings:

| Result | Meaning |
| --- | --- |
| `Ok(Some(time))` | The entire group committed or its pending write resolved successfully, and the complete outcome permits publication. |
| `Ok(None)` | Known-no-write deferral with the owner still running, such as a stale plan or batch-capacity rejection. |
| `Err` | The owner stopped, the outcome remains uncertain, or another failure requires aborting publication. |

Check `is_stopped()` and all committed outcomes, including quarantine lease
cleanup, before returning success. Encoding or replay rejection is visible to
the worker even when it permits known-no-write deferral; do not silently retry
an invalid request forever.

Preflight rejection writes nothing and rejects the whole group. Capacity becomes
capacity feedback. Incompatible group members may be separated before a new
preflight; invalid mappings or lifecycle states need correction. Never commit a
subset of a group already preflighted, or repeatedly reform a rejected group
without addressing the cause.

Do not flush fingerprint growth or perform other identity-changing maintenance
between preparation and commit. Schedule maintenance between groups, including
when `sync_interval` is zero: immediate progress durability does not itself
flush fingerprint growth.

A reusable policy-qualified opener is still required integration work. Add it to
discovery/source access by reusing the existing pin-bound policy checks, then
use it in the worker adapter for admission and reader reopen with the shared
probe budget. The existing `SourceFile::open_at` enforces leaf symlink policy,
but does not perform the complete policy check. Locator equality alone is not
a substitute. Adapter and lifecycle tests must establish descriptor bounds,
pending-write correlation, Ack/Nack authorization and safe retirement.
Standalone identity batch tests do not prove those integration contracts.

## Validation

Unit tests use real temporary source files and the real checkpoint store. They
cover restart anchors, prefix/frontier mismatch, quarantine, incompatible
profiles, replacement context in both processing orders, association, capacity,
cancellation, source substitution, stale plans and lease contention.

Supplying replacement context in an identity test does not test Task 7b's ability
to recognize it. Integrated reconciliation and worker tests remain Tasks 14/24.

## Demo stopping limit before compaction

Measure checkpoint appends per second, not source lines per second. Without
compaction, remaining time is approximately the smaller of:

```text
(max_wal_transactions - applied_sequence) / appends_per_second
(max_wal_bytes - wal_bytes) / WAL_bytes_per_second
```

Restarting does not clear either WAL cap. Without compaction or a supported
change to the limits, the next required append will encounter the same limit.

Defaults are 10,000 transactions and 64 MiB. Registration, resets, metadata updates
and quarantine consume this budget too. The store uses these append caps
directly; WAL recovery memory limits do not enlarge them. Optional fingerprint
growth can bring `CompactionRequired` forward, leaving less room for progress.

For example, assume 10 Acks/second and one append per Ack, excluding startup and
other transactions:

| Clean progress updates per Ack | Encoded transaction bytes | First cap | Approximate time |
| --- | --- | --- | --- |
| 1 | 129 | 10,000 transactions | 1,000 seconds (16m 40s) |
| 512 | 45,608 | 1,471 transactions fit in 64 MiB | 147.1 seconds (2m 27s) |

One initial registration leaves 9,999 transaction slots. The one-file example
then has 999.9 seconds of progress capacity at that rate; the next append is
rejected. Startup WAL bytes also reduce the byte-limited example. Continuation
records have larger encodings. A unit test derives these sizes from the real
codec.

These are examples, not measured Ack rates or promised receiver run times.
Integration must measure its rates and remaining capacity before quoting a demo
limit. Task 13 compaction is still needed; raising limits does not remove the
limitation. Each Ack can already include multiple authorized file deltas in one
`append_checkpoint` call without an unbounded accumulation queue.

[clarifications]: ../../../../../../docs/filelog-identity-clarifications.md
