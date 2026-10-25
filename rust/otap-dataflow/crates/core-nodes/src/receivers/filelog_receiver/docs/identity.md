# File identity and local ownership

`identity::IdentityOwner` implements Task 12 admission on a blocking worker.
It uses the existing Linux source primitives and checkpoint store. It does not
register a receiver, schedule readers, perform rotation finalization or advance
Ack-based progress.

## Admission protocol

1. Restore durable records into discovery, using complete saved native paths
   where available. Never use a truncated advisory suffix as a path to open.
2. Map a fresh candidate into `identity::Candidate`. Task 7b's
   `Replacement::Possible` maps to `Context::Replacement`; the candidate's own
   exact-locator record takes precedence over that context.
3. Call `prepare`. This acquires the process-wide locator lease first. Existing
   checkpoint blocks can return immediately; otherwise it reserves the shared
   probe budget and invokes the supplied source opener.
4. The opener must validate current link and resolved-target exclusion policy,
   including checkpoint namespace exclusion, before returning a `SourceFile`.
   `SourceFile::open_at` alone validates source type and locator, not that policy.
   Reuse discovery's pin-bound checks in the receiver integration adapter.
5. Preparation validates fresh prefix, size and committed-frontier evidence.
   It returns a proposed file ID and decision without writing checkpoints.
   Temporary source descriptors close before the probe reservation is released.
6. Before discovery preflight, call `select_batch` on at most `admission_limit()`
   plans. It counts net durable growth across the group: recovery and same-locator
   replacement cost no extra slot. Report capacity feedback for rejected plans,
   then drop them on the worker. Preflight exactly the selected group while its
   pass/offers are frozen; call `commit_batch` inside its durable callback and
   publish only after success or successful pending resolution.
   A plan is bound to its lease owner, checkpoint generation, identity revision
   and a compact stamp of its exact-locator record. Ordinary progress on other
   files does not invalidate it. Finalization and non-progress mutations do.
7. Move the returned `ReaderSeed` into the logical reader. Scheduling must
   revalidate the source when reopening it. Keep the lease when evicting a
   descriptor; release it only when that logical reader actually retires.
   Before release or supersession, lifecycle integration must prove that no
   unresolved source delta remains in an open batch, retained batch or carry-over.

The prepare/commit boundary lets Task 7b inspect the proposed identity before
its durable callback. It does not create a second transaction manager. Store
append/replay still enforce all durable invariants and synchronization barriers.
Task 14/24 owns bounded cross-thread requests, responses and cancellation.
Delivery uses `append_checkpoint` for authorized progress and `sync_checkpoint`
for interval/drain barriers. An in-process resume can use already applied Acked
progress; it does not change the configured crash-replay window before sync.
The identity revision advances only after a successful non-progress operation
or a finalizing progress operation. The exact-record stamp also checks lifecycle,
epoch, offset, frontier and resume state. Fingerprint/profile/path changes are
covered by the revision, so plans do not copy that evidence. A rejected append
does not advance the revision. Treat `StalePlan` as a request for fresh evidence;
ordinary Ack traffic on other files no longer forces a re-probe. The revision
remains deliberately conservative for identity changes anywhere in the namespace.
Prepare one bounded admission group at a time while allowing ordinary progress;
validate all members together rather than invalidating a wave one file at a time.

Dropping an uncommitted plan releases its lease. Known pre-write rejections
report `AppendRejected`; invalid encoding/state requests leave the owner usable,
while compaction and authority failures stop it. A recoverable write attempt
reports `PendingWrite` and retains its exact outcome and leases. The worker must
back off and call `resolve_pending`, never resubmit the original operation or
reopen the store. Resolution uses the store's original-handle protocol and
publishes the retained result once, for both admissions and ordinary progress.
The configured consecutive failure budget includes the initial error. Retained
admission leases are revalidated before resolution; registry failure stops the
owner and retains the pending result.
Failed sync, corruption, authority loss and budget exhaustion are terminal.

`commit_batch` supports a bounded group of plans prepared at one revision. Its
limit accounts conservatively for three operations per plan and maximum encoded
operation size under the 256-operation/16 MiB format limits. Discovery must
preflight the entire group before calling it and publish the whole group only
after success or successful pending resolution. That 7b group transition and
same-locator net-zero replacement are not implemented by this component yet.
Do not treat a successful standalone batch test as evidence of that integration.
`commit_batch` rejects an over-capacity preflighted group atomically; it never
selects a partial subset after discovery has approved the group.

`ResolvedWrite.admissions` is empty for progress and maintenance transactions;
for admission it contains the original reader/quarantine results. A committed
quarantine reports its ID even if lease cleanup fails; `release_error` is then
present and the owner is stopped. Do not schedule returned readers if ownership
cleanup has stopped the owner.

Early `Admission::Quarantined` and `ProfileBlocked` describe checkpoint state for
the offered locator. They do not establish fresh path validity, policy eligibility
or successful reconnection. Neither opens source content nor mutates state.
Reader seeds and lease guards must be dropped on the blocking worker, not an
async pipeline task, because release takes the process-local registry mutex.

Shutdown must first end worker access. The owner drops checkpoint handles before
its retained pending outcomes/leases. Existing reader leases remain reader-owned.

## Decisions and association

Quarantined exact locators remain quarantined, even if content changes. Profile
incompatibility blocks the original record without rekeying or applying the
mismatch policy. Compatible Active recovery needs matching prefix and frontier
plus an offset within the file size. A same-locator mismatch atomically removes
that Active record and registers a new ID, optionally quarantining it in the
same transaction. Changed locators never inherit progress or remove the old
record.

A recognized replacement starts a new identity at zero, including under `fail`
and `start_at: end`. This context can come from a saved binding after restart;
it is not restricted to rotations witnessed by a live reader. It does not prove
that the candidate is the old stream, nor authorize old-offset inheritance.

Without recognized replacement context, association examines retained Active
records. A full native path match uses path kind, complete length and digest;
a suffix alone never associates. Otherwise compare the common nonempty raw prefix
at the same ignored-header offset. This includes short files and growing prefixes;
empty evidence never associates. Different framing settings do not reinterpret
these raw bytes. The mismatch policy chooses a new anchor; no old progress moves.

This conservative rule captures surviving copy/unlink and copytruncate archive
bytes when matching evidence remains available. Independent files sharing common
prefixes can also associate, bypassing start/age policy. The configuration guide
explains that tradeoff, especially for small windows and `fail`. This is not a
claim of lossless copytruncate capture or proof that two files are one stream.

Frontier EOF/short evidence or a digest mismatch is a continuity failure.
Environmental I/O errors preserve checkpoint state and propagate to worker-owned
bounded backoff and health reporting. Prefix/frontier reads share a cancellation-
aware helper that retries Interrupted at most eight times per bounded read.
The explicit frontier-error clarification remains in [the follow-up proposal][clarifications];
PR #4443 is unchanged.

Reader seeds retain the freshly validated prefix. The reader must submit a valid
prefix extension with `queue_fingerprint` when observed, including growth after
empty-file registration. The bounded queue coalesces each file's extension; a
false result means retain the reader-owned bytes and retry after a flush.
`flush_fingerprints` persists a separate bounded non-progress transaction.
`sync_checkpoint` performs the required progress barrier first, then attempts
optional growth. A pre-write CompactionRequired result from growth retains its
queue and does not stop the owner; encoding, replay, authority and actual write
or sync failures are not hidden. `SyncOutcome` separately reports the sequence
certified by the required barrier and the optional growth result. Callers must
handle both: growth can still require pending resolution or terminal shutdown,
but it cannot turn an already-confirmed barrier into an unconfirmed one.
Known encoding/replay rejection discards and reports that optional batch rather
than retrying it forever. Capacity rejection retains the growth queue. Growth and
progress never mix in one WAL frame. Admission itself adds no per-file growth
fsync. Stale lifecycle/epoch extensions are discarded.

Age uses one supplied wall-clock observation and handle-derived modification
time. Equality at the threshold is eligible. Future mtimes clamp to zero age;
unavailable mtime does not exclude and is exposed on the prepared result for a
bounded health event. No checkpoint record is created for ignored files.

## Resources and remaining integration

Exact-locator recovery reuses ReplayState's existing index. Only new unmatched
candidates scan Active history, with cancellation between records. Cost is
O(tracked records * fingerprint width) in the worst association case;
no finite latency guarantee is implied. Measure this path under churn before
introducing additional indexes. There is no duplicate record or path table.

The executor retains one configured fingerprint buffer. A prepared plan holds
one reader seed with a bounded prefix, one compact exact-record stamp, at most
three checkpoint operations and bounded evidence copies. It does not duplicate
the checkpoint's retained fingerprint into the stamp.
The integration must cap outstanding plans by its candidate-offer bound and
include these copies, channel storage and checkpoint staging in Task 19/36
accounting. `scratch_bytes()` exposes allocated scratch capacity.

Construct each lease owner with `IdentityOwner::required_lease_capacity(inputs)`.
That checked bound is max_tracked_files plus one bounded admission group, so
speculative new-file plans cannot consume all leases needed by existing-record
recovery or net-zero replacement. Durable population is still bounded separately
by max_tracked_files. Charge the extra temporary leases to admission state.
Lease entries are bounded by the sum of these owner capacities. The B-tree stores
locator keys and small owner references; deletion reclaims its nodes rather
than retaining a high-water hash allocation. Owner names are capped at 255 bytes
and shared across their entries. `LeaseRegistry::len()` exposes actual entries;
allocator/node overhead still belongs in integrated memory measurements. The
registry mutex protects only map/counter operations. No I/O, channel waits or
telemetry formatting occurs while it is held.

Use one process registry across all nodes, regardless of checkpoint namespace.
Conflicts carry the existing owner's name for `filelog.local_locator_conflict`;
they are never silently ignored. Tests inject isolated registries. Namespace
locking remains the checkpoint store's responsibility. These are local ownership
guarantees, not distributed fencing.

Task 7b is not part of this main-based change. Its adapter must also handle
net-zero same-locator supersession without demanding another tracked slot:
preflight retirement/replacement of the discovery view before committing the
atomic checkpoint operation. Do not remove a record merely to make room for a
changed-locator rotation, which still needs an independent durable slot.

## Validation

The identity unit tests use real temporary source files and the real checkpoint
store. They cover restart anchors, prefix/frontier mismatch, quarantine,
profile incompatibility, replacement context in both processing orders,
association, capacity, cancellation, source substitution, stale plans and
lease contention. They do not claim that a mocked replacement context tests
Task 7b itself; integrated reconciliation/worker tests remain Tasks 14/24.

## Demo stopping limit before compaction

Use actual checkpoint appends per second, not source lines per second. With no
compaction, remaining time is approximately the smaller of:

```text
(max_wal_transactions - applied_sequence) / appends_per_second
(max_wal_bytes - wal_bytes) / WAL_bytes_per_second
```

Defaults are 10,000 transactions and 64 MiB. Registration, resets, metadata
updates and quarantine also consume that budget. The current store uses these
configured append caps directly. WAL recovery memory bounds do not enlarge them.

For an illustrative 10 Acks/second with one append per Ack, excluding startup
and other transactions:

| Clean progress updates per Ack | Encoded transaction bytes | First cap | Approximate time |
| --- | --- | --- | --- |
| 1 | 129 | 10,000 transactions | 1,000 seconds (16m 40s) |
| 512 | 45,608 | 1,471 transactions fit in 64 MiB | 147.1 seconds (2m 27s) |

One initial registration leaves 9,999 transaction slots, so the one-file example
has 999.9 seconds of subsequent progress capacity at that rate; its next append
is rejected. Actual startup WAL bytes reduce the byte-limited example too.
Continuation records have larger encodings. A unit test derives the frame sizes
through the real codec. These are explicit example assumptions, not a measured
Ack rate or a promised duration for an unwired receiver. Integration must record
its measured rates and remaining capacities before describing the demo limit.
This limitation remains until Task 13 adds compaction; changing limits does not
remove it. Each Ack can already carry multiple authorized file deltas in one
`append_checkpoint` call without adding an unbounded accumulation queue.

[clarifications]: ../../../../../../docs/filelog-identity-clarifications.md
