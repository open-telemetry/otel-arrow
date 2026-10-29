# File identity and local ownership

`identity::IdentityOwner` decides whether to resume a file, register a new file
ID, or quarantine it for operator attention. It runs on a blocking worker and
uses the existing Linux file-access code and checkpoint store.

This component does not register a receiver, schedule readers, finalize rotated
files, or decide which progress an Ack authorizes. The [specification][spec] sets
the behavior; the [conformance matrix][conformance] lists required test cases.
This guide explains the implementation and the work needed to connect it.

## Start with one log file

Suppose Filelog saved progress at byte 1,000 of `app.log`, under file ID A. A
**locator** is the operating system's file reference (device and inode on Linux).
The path is its name; the file ID identifies the stream in the checkpoint.
A **fingerprint** checks the start of the file. A **frontier guard** checks a
small window just before the saved offset. A **lease** reserves the file for
one logical reader.

Consider these cases independently. The saved record is Active and its framing
settings are compatible. Resume still requires the prefix, size and frontier
checks described below.

| What happens next? | Identity decision |
| --- | --- |
| Filelog restarts and the same file passes validation | Keep ID A and resume at byte 1,000. |
| The file is renamed to `app.log.1`; its locator and evidence still validate | Keep ID A and resume at byte 1,000, despite the new name. |
| A copy appears at a different locator and its prefix associates with A | Give the copy a new ID. With the default mismatch policy, start at byte zero; some bytes may be read again. |
| Discovery recognizes a previously untracked move/create replacement at `app.log` | Give the replacement a new ID and start at byte zero. Keep the old file's progress separate. |
| Reading the saved file's frontier returns a disk error | Keep ID A and byte 1,000, but do not resume until validation succeeds. |

If another logical reader already holds the locator's lease, admission reports a
conflict before opening the file. Closing that reader's descriptor does not
release its lease.

## Admission: before a reader starts

Admission has four stages: restore, prepare, commit and read.

### 1. Restore saved records

Restore checkpoint records into discovery, including terminal records: they
still use tracked slots until retention removes them. Saved paths start as
`Unverified`; fresh evidence must establish their current role. Use complete
saved native paths when available. Never open a truncated advisory suffix.

Map each discovery candidate to `identity::Candidate`.
`Replacement::Possible` maps to `Context::Replacement`. If the candidate has its
own exact-locator checkpoint record, that record takes precedence.

### 2. Prepare a candidate

Call `prepare`. It takes the process-wide locator lease first. Quarantine or
incompatible framing settings can return a blocked result without opening the
file. Otherwise, preparation reserves the shared probe budget and calls the
supplied opener.

**The opener must check current policy against the pinned file.** Check symlink
policy, resolved-target exclusions and checkpoint-namespace exclusion. Reuse
discovery's checks. `SourceFile::open_at` checks file type, locator and leaf
symlink policy, but does not perform the complete policy check.

Preparation reads fresh prefix, size and frontier evidence. It returns a plan
with a proposed file ID and decision, without writing a checkpoint. Temporary
file handles close before the probe reservation is released. Dropping an
uncommitted plan releases its lease.

### Association takes bounded worker turns

For an unmatched candidate with retained history, `prepare` returns
`Admission::Scanning`. Keep that scan on the worker. It owns a locator lease,
a bounded path/prefix, and a cursor; it retains no source descriptor or shared
probe reservation and does not borrow the checkpoint table.

Call `advance_association` once per worker turn. Each call visits at most 64
records and compares at most 64 KiB of prefix bytes. It can pause partway through
one comparison. Service reads, checkpoint commands, cancellation and discovery
between calls; do not loop to completion in a production turn. The returned
`AssociationWork` reports actual charged work and completion. These are CPU work
bounds, not filesystem or wall-clock latency guarantees.

When complete, call `finish_prepare` with a fresh policy-aware opener. It checks
the owner/revision, reopens the source under the shared probe budget and validates
its locator, size and scanned prefix again. A changed prefix, including growth
of a short prefix, returns `Deferred`; discard the result and prepare again.
If the shared probes are busy, finishing returns the completed scan unchanged;
retry finishing on a later turn without repeating association. The fresh handle
supplies mtime and the final start anchor. No scan result alone permits a reader
or a checkpoint write.

Ordinary non-finalizing progress can run between steps without invalidation.
Identity/metadata mutations, finalization, or pending-write recovery invalidate
the cursor; discard it on `StalePlan`. Recovery may reorder the table even for a
progress-only write, so it has a separate cursor revision. Cancellation also
requires discarding the scan. A failed step never authorizes admission.

Count waiting scans, prepared plans and pending outcomes together against the
single `admission_limit()` group reserve. Drop them on the worker. Finish all
members before selecting/preflighting the group; identity mutations while other
scans wait require fresh preparation of those scans.

### 3. Select, check and commit a group

Keep at most `admission_limit()` scans/plans outstanding, including plans waiting
for commit or pending-write resolution. Discovery may retain more offers; prepare
one group at a time. Ordinary progress on other files may continue.

1. Call `select_batch` before discovery preflight. It counts the group's net
   increase in durable records. Resume and same-locator replacement need no
   extra slot. Report capacity feedback for rejected plans and drop them on
   the worker.
2. Map the selected plans to exactly the corresponding `AdmissionChange` values.
   Call `Reconciler::admit_batch`. Its preflight checks the entire group before
   any checkpoint write.
3. Inside its callback, call `commit_batch` for that exact group. Keep discovery
   exclusively borrowed so offers and bindings cannot change. Resolve any
   recoverable pending write inside the callback, using bounded worker backoff.
4. Publish all members only after durable success or successful pending
   resolution. Then give the reader results to the scheduler.

Never commit or publish a subset of a preflighted group. `commit_batch` checks
capacity again and rejects the whole group if it no longer fits. Plans must
share one identity revision. The group limit allows up to three operations per
plan at the maximum encoded operation size, within the WAL limits of 256
operations and 16 MiB.

The callback returns:

| Result | Meaning |
| --- | --- |
| `Ok(Some(time))` | The whole group committed or resolved successfully, and every result permits publication. |
| `Ok(None)` | Nothing was written and the owner is still running. Examples: stale plan or batch-capacity rejection. |
| `Err` | The owner stopped, the write remains uncertain, or another failure prevents publication. |

Do not return `Ok` while a write is uncertain. If cancellation leaves it
unresolved, return `Err`, stop reconciliation and retain ownership until worker
access has ended. Before reporting success, check `is_stopped()` and every
result, including quarantine lease cleanup.

A preflight rejection writes nothing. Report capacity failures as capacity
feedback. Incompatible group members may be separated before a new preflight;
invalid mappings or lifecycle states need correction. Do not silently retry a
bad encoding/state request or reform the same rejected group forever.

Checkpoint append/replay owns persistence and sync rules. This admission flow
adds no second transaction manager.

### 4. Keep ownership with the reader

Move each `ReaderSeed` into its logical reader. Revalidate the source on reopen.
Closing or evicting a file descriptor does not retire the reader or release its
lease. `TrackedFile.resident` means the reader still owns the file; keep it set
through descriptor eviction and while delivery work prevents retirement.

Before releasing a lease or replacing an identity, prove that no unresolved
source delta remains in an open batch, retained batch or carry-over.

Drop seeds, plans and lease guards on the blocking worker. Lease release takes
the registry mutex and must not run on an async pipeline task.

### When a plan becomes stale

A plan records its lease owner, checkpoint generation, identity revision and a
small stamp of its exact-locator record.

- The stamp checks lifecycle, epoch, offset, frontier and framing-resume state.
- The revision is a counter. It advances after successful non-progress writes
  and finalization, covering fingerprint, profile and path changes. Rejected
  writes do not advance it or durable progress.
- Ordinary Ack progress on another file does not invalidate a plan. Changes to
  the exact record's progress do. Identity changes elsewhere in the namespace
  can invalidate it too; the check is deliberately conservative.

On `StalePlan`, collect fresh evidence. Validate and commit the group together;
committing members one by one can invalidate the remaining plans.

Do not flush fingerprint growth or make other identity-changing writes between
prepare and commit. Run maintenance between groups, including when
`sync_interval` is zero: immediate progress sync does not flush growth.

## Results and failures

### Admission results

Early `Admission::Quarantined` and `ProfileBlocked` describe checkpoint state
for the offered locator. They open no source content and change no state. They
do not prove that the current path is valid, eligible or reconnected.

After commit or resolution, `CommittedAdmission::decision()` returns the
original decision, including reset and superseded-ID details for health events.
`ResolvedWrite.admissions` returns the original reader/quarantine outcomes once.
It is empty for progress or maintenance writes.

If lease cleanup fails after quarantine commits, the result still reports the
quarantined ID, with `release_error` set. The owner stops. Do not schedule any
returned readers after that stop.

### Checkpoint failures

| Outcome | Required handling |
| --- | --- |
| `AppendRejected` | Nothing was written. Encoding/replay errors leave the owner usable. Required-write compaction and authority failures stop it. |
| `PendingWrite` | The write may have reached disk. Keep its original result and leases; back off on the worker, then call `resolve_pending`. |
| Failed sync, corruption, authority loss or exhausted failure budget | Stop the owner. |

Never resubmit a pending operation or reopen the store to resolve it. Use the
store's existing recovery protocol with its original file handle. Resolution
returns the retained outcome once, for both admission and progress. The failure
budget includes the initial error. Resolution rechecks admission leases first;
a registry failure stops the owner and keeps the pending result.

After `SyncFailed` or `SyncPoisoned`, the supervisor must prevent in-process
namespace reopening, including automatic receiver replacement. This applies to
`CheckpointStore::open`, owner operations and `SyncOutcome.growth`. A successful
progress sync does not make a later growth sync failure recoverable.

### Progress and shutdown

Call `append_checkpoint` only for authorized progress. Use `sync_checkpoint`
for interval and drain barriers. An in-process resume may use Acked progress
already applied by the store; only a successful sync confirms the durable
frontier for crash recovery.

Stop worker access before destroying the owner. Checkpoint handles close before
retained pending outcomes and leases are dropped. Existing reader leases stay
with their readers. The worker/lifecycle integration owns bounded cross-thread
requests, responses and cancellation.

## Identity decisions

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

### Association: choosing a starting point for a new identity

Association means a candidate may be related to a retained Active record. It
uses either of these checks:

- **Full path:** path kind, complete native-path length and digest match. A
  truncated suffix never associates.
- **Prefix:** start at the same ignored-header offset and compare the whole
  shorter prefix with the start of the longer one. At least one byte must be
  present, and all compared bytes must match. Short files and growing prefixes
  count; different framing settings do not reinterpret these raw bytes.

Association uses the mismatch policy instead of `start_at` or age filtering.
It never transfers the old file's ID or progress. Matching evidence can help
capture copied bytes, but unrelated files can share the same prefix. See the
[configuration warning][fingerprints], especially for small windows and `fail`.
This is not a guarantee of lossless copytruncate capture.

### Why not require the candidate to contain the whole saved fingerprint?

That alternative would reduce some false matches, but could skip surviving data:

1. A 100-byte file is copied; its last 20 bytes have not been acknowledged.
2. The original grows to 200 bytes. Filelog saves a 200-byte fingerprint, but
   those 20 bytes are still unacknowledged. Evidence growth does not advance
   acknowledged progress.
3. The original is deleted.
4. After restart, the 100-byte copy is the only surviving source of those bytes.

The copy matches the first 100 bytes of the saved fingerprint, but cannot contain
all 200. Requiring the whole saved fingerprint would reject the association;
with `start_at: end`, the unread tail would be skipped. Common-prefix association
with the default `beginning` mismatch policy reads it again, accepting duplicates.
A regression test covers this order of events.

Keep common-prefix matching for the MVP. Narrowing it requires either accepting
this capture gap or collecting additional evidence. We cannot assume that every
copy was made after the latest fingerprint was saved.

A larger minimum fingerprint length cannot remove all false matches. Waiting
for the old locator to disappear also misses copytruncate, which leaves the
original file in place. An incomplete scan cannot prove disappearance.

### Read errors

A different frontier digest or EOF before the required window is complete is a
continuity mismatch. A successful partial read is not: continue reading the
bounded window. I/O errors preserve saved state and return to the worker for
operation-specific retry or termination. Environmental retries need bounded
backoff and rate-limited health reporting, including for persistent failures.

Prefix and frontier reads share a helper that retries `Interrupted` at most
eight times per bounded read and checks cancellation between attempts. Neither
exhaustion nor cancellation authorizes admission or changes saved progress.

### Example: the disk fails, then recovers

The saved offset is 1,000 and the locator still matches the saved Active record.
With compatible framing settings, Filelog must still validate the prefix, size
and bytes just before the offset before resuming.

| Attempt | Observation | Result |
| --- | --- | --- |
| First | The disk returns EIO instead of bytes | Keep the saved record at 1,000. Report the error; do not resume. |
| Second | The disk returns EIO again | Keep the same record. Repetition does not prove replacement. |
| Later | The prefix, size and frontier all validate | Resume the original file ID at 1,000. |

If the later read succeeds but the frontier bytes differ, that is a mismatch and
uses the configured mismatch policy. An error and a successful read of different
bytes provide different evidence.

A persistently unreadable file keeps its tracked slot and cannot resume until
validation succeeds. Repeated errors do not prove replacement. Failed checkpoint
syncs remain terminal; these source-read rules do not relax them.

### Age filtering

Only unrelated new files use `ignore_older_than`. Use one supplied wall-clock
time and mtime from the open handle. A file exactly
at the age threshold remains eligible. A future mtime counts as age zero. If
mtime is unavailable, do not exclude the file; the prepared result exposes this
for a bounded health event. Ignored files create no checkpoint record.

## Fingerprint growth and sync

The seed keeps the freshly validated prefix. When a reader observes a valid
extension, call `queue_fingerprint`, including after empty-file registration.
The bounded queue combines updates per file. It checks coverage and capacity
before allocating, retains only the proposed prefix, and reuses its buffer for
accepted extensions. The expected old prefix is obtained from authoritative
state at flush. If it returns false, keep the bytes on the reader and retry
after a flush.

`flush_fingerprints` writes a separate bounded non-progress transaction. Growth
and progress never share a WAL frame. Resume does not add one growth fsync per
file. Extensions with stale lifecycles or epochs are discarded. Growth
throttling is an optional performance improvement.

`sync_checkpoint` syncs required progress first, then tries optional growth.
`SyncOutcome` reports the confirmed sequence and the growth result. **Handle
both fields:** a growth failure cannot undo confirmed progress, but may still
require pending-write resolution or shutdown.

| Growth outcome | Effect |
| --- | --- |
| Pre-write `CompactionRequired` | Keep the queue; do not stop the owner. |
| Encoding/replay rejection | Report the error and discard that optional batch instead of retrying it forever. |
| Authority, write or sync failure | Report it and apply the normal pending-write or terminal-failure handling. |

## Resource limits

### Lookup and memory

Exact-locator recovery uses `ReplayState`'s existing index. Only new unmatched
candidates scan retained history, skipping inactive records and checking cancellation
between bounded comparisons. Total worst-case association work remains
O(tracked records * fingerprint width), but each worker turn has explicit record
and byte limits. There is no secondary fingerprint/path index or copied ID list.
The authoritative table uses `IndexMap` for constant-time positional traversal
and hash-based keyed lookup. Removal swaps the last entry into the gap; revision
checks reject old cursors before they can observe that change. Include the dense
table and hash index capacity in integrated memory qualification.

See [identity admission measurements](identity-performance.md) for the Linux
bulk/churn fixture, work counts, timing limitations and separate heap results.

There is one authoritative checkpoint table. The adapter maps lifetime-unique
64-bit `TrackedId` values to 128-bit `FileId` values. Bound that map by tracked
capacity plus one admission group, including reserved IDs. It stores correlation
keys, not copies of checkpoint records. Membership must agree after publication
and retirement; a frozen durable write can precede discovery publication.

The executor keeps one configured fingerprint buffer; `scratch_bytes()` reports
its allocated capacity. Each plan also holds a reader seed with a bounded prefix,
a compact exact-record stamp, at most three checkpoint operations and bounded
evidence copies. The stamp does not copy the checkpoint's retained fingerprint.
Receiver memory accounting must include these copies, channels, checkpoint
staging, the ID map and temporary leases.

### Leases

Construct the lease owner with `IdentityOwner::required_lease_capacity(inputs)`:
`max_tracked_files` plus one admission group. This reserve protects recovery and
replacement while speculative new-file plans hold leases. It covers only one
group, not all retained discovery offers. Durable records have their separate
`max_tracked_files` limit.

Use one process registry across nodes and checkpoint namespaces. Tests use
isolated registries. A conflict's `same_owner` flag compares registry identity,
not node names. Self-conflict means an existing reader or plan still owns the
locator; do not release that ownership or report a cross-node conflict. Other
conflicts report the existing owner's name for `filelog.local_locator_conflict`.
Never silently ignore them.

Registry entries are bounded by the sum of owner capacities. The B-tree stores
locators and small owner references; it frees nodes on deletion. Shared owner
names are limited to 255 bytes. Use `LeaseRegistry::len()` for entry counts and
include allocator/node overhead in memory measurements.

The mutex protects map and counter operations only. No I/O, channel waits or
telemetry formatting runs under it. The checkpoint store owns namespace locking.
These guarantees are process-local, not distributed fencing.

## Remaining integration work

A reusable policy-checked opener is still needed. Add it to discovery/source
access using the existing pin-bound checks. Share it between admission and
reader reopen, under the shared probe budget. Locator equality alone does not
establish current policy.

Map a decision's `superseded: Some(id)` to discovery's `replace_id`. The old
entry must be non-resident and eligible for replacement. Same-locator replacement
needs no extra tracked slot. Changed-locator rotation keeps its predecessor and
needs another slot. Publish committed quarantine as `active: false`, with no
reader.

Adapter and lifecycle tests must prove descriptor bounds, correlation between
pending writes and their groups, Ack/Nack authorization, and safe retirement.
Unit tests use real files and the real store to cover restart anchors,
prefix/frontier mismatch, quarantine, profiles, replacement order, association,
capacity, cancellation, source substitution, stale plans and lease contention.
They supply replacement context directly; they do not prove that discovery
recognizes it or that the worker is correctly integrated.

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

[spec]: ../../../../../../docs/filelog-receiver-phase1-spec.md
[conformance]: ../../../../../../docs/filelog-receiver-phase1-conformance.md
[fingerprints]: configuration.md#fingerprint-ambiguity
