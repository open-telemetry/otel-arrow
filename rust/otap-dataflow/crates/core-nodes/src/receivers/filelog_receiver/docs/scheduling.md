# Reader scheduling

The scheduler decides which log file to read next and how much work it can do
before giving another file a turn.

For example, suppose one file receives logs continuously while another receives
only a few lines. Both get reading turns. If the open-file limit is reached, the
scheduler can close an eligible file, preserve its unfinished record and
ownership, and check that it is safe to resume when reopened. Files with no new
data wait for a later check instead of being read repeatedly.

This module handles reading and resource limits. Batch construction, delivery,
checkpoint updates, and rotation decisions belong to other components.

The synchronous [scheduler](../scheduler/mod.rs) runs on the read/checkpoint
worker. It consumes `ReaderSeed` values from the identity component and retains
their leases independently of open file handles.

Reading and framing have separate turn budgets, including buffered replay.
Readers share one source buffer and reserve capacity before framing. Eviction
preserves unfinished state; reopening validates source continuity before work
resumes. Unresolved batch deltas prevent eviction. Lifecycle pins also
prevent descriptor eviction. Empty partial slots are released even when a reader
halts; unfinished input, rejected output and its lease remain owned until an
explicitly authorized cleanup.

Validated resident readers at EOF with no partial state check metadata before
requesting a slot. Unchanged EOF re-parks without a content read or partial
construction and does not refresh eviction age. Growth follows normal capacity
admission; evicted readers still reopen and validate continuity.

Idle partial flush is not wired: nonzero `framing.force_flush_period` is not
serviced by this scheduler. Idle-flush deadlines and source revalidation,
startup descriptor-budget checks, and admission gating during global descriptor
backoff remain receiver-integration prerequisites. Authorized disposal of
unfinished or rejected state for descriptor-free disappearance containment is
also not implemented; receiver integration must remain disabled until that
cleanup path is provided. Retirement below only removes eligible clean readers.

The caller supplies a policy-checked source opener, processing permission,
cancellation and [batch sink](../scheduler/batch.rs). Sink preflight must account
for retained batches and carry-over. Both `Continue` and `Stop` consume offered
output; unexpected rejection retains the returned frame and stops scheduling.

Before the first turn, composition registers each scheduler key with bounded
sink context: the identity seed's file ID, epoch, initial offset and framing
resume, plus configured file metadata. The concrete sink adapter owns
registration and metadata updates. Acceptance freezes metadata and observed
time for each output, including carry-over; later updates affect future output.

Worker composition must service the concrete sink's batch-flush deadline even
when no reader is ready or source work is paused. Combine it with eligible
reader and control deadlines, and service expiry without waiting for a source
event. The scheduler's reader deadlines alone do not provide a batch timer.
Concrete batch timing and registration are implemented with the batch builder.
Its [scheduler hooks][batch-hooks] also request a flush when descriptor eviction
finds no eligible victim. Receiver integration depends on that hook: protected
open-batch progress must be flushed and resolved before retrying eviction.

`unresolved` covers open, retained and carry-over progress for the reader/epoch.
Sealing, sending or Ack alone does not release the descriptor-eviction veto.
After authorized progress application, composition releases the retained batch,
admits any carry-over into the next batch, refreshes reader evidence, and only
then calls `resume_sink`. Carry-over keeps its unresolved veto, and independent
permission checks still govern whether source work can resume.

A reader already waiting for probe access can finish opening before reacquiring
partial-state capacity. It releases probe access and keeps an evictable handle
while waiting for a slot. Validation and content reads require that slot;
fresh opens still reserve partial capacity first. An unused slot grant can be
handed to the next FIFO waiter completing such an open, avoiding repeated
descriptor turnover before either reader can make progress.

Route probe-release notifications to `probe_available` and descriptor, pin or
batch-delta changes to `resources_changed`. After authorized checkpoint
application, `observe_applied` refreshes cached evidence from the identity
owner's record and restarts in-flight validation when that evidence changes.
Probe-deferred readers reactivate within processing budgets when the request
completes or clears; protected-descriptor waiters need their own resource event.
The scheduler does not authorize delivery, checkpoint or lifecycle transitions.

## OTAP batch integration

The concrete `Batcher` registers each scheduler key with the file ID, epoch,
initial offset and continuation from that reader's identity seed. It projects
the configured file metadata once per registration or metadata update, using
immutable worker-local ownership. An accepted carry-over keeps its original
metadata and observed timestamp even if the source changes while delivery waits.
Raw frame payloads are copied directly into the existing Arrow builders; there
is no additional record queue. Per-row source ranges and bounded decoding,
truncation and fallback evidence remain internal, independent of attributes.

`turn` polls the batch deadline before servicing readers, and `next_deadline`
includes the sink deadline even when source permission is absent. Worker hosting
must service this combined deadline without waiting for another file event.
Known record, byte and distinct-file bounds seal before another source turn.
When every possible descriptor victim owns unresolved progress, the scheduler
requests an open-batch flush instead of waiting indefinitely.

After sealing, `retained()` exposes one immutable payload and one complete
progress transaction. Composition owns completion correlation, retries and
checkpoint writes. Only after matching aggregate completion (or an explicit
authorized loss policy) and successful application of the entire transaction
may it call `release_applied`. A Nack alone never permits release. Refresh
reader evidence through `observe_applied`, then call `resume_sink`; release
has already seeded any carry-over before this can permit another read.
Unfinished decoder/framer state stays charged to the logical reader under the
existing slot/yield rules. The batch builder does not create a second copy.

Inspect `Batcher::failure` on terminal scheduler failure. On direct shutdown,
`BatchBuilder::abort` reports the records and logical bytes abandoned without
advancing progress; do not treat dropping in-memory payload as capture success.
The batch size model is logical, not an Arrow allocation or total process-memory
limit. Source contexts, Arrow buffers and per-row provenance remain bounded by
the validated file/record/byte limits; aggregate memory qualification remains
the memory-admission task's responsibility.

See the [behavioral specification][spec] for scheduling and resource rules;
implementation details and tests live beside the code.

[spec]: ../../../../../../docs/filelog-receiver-phase1-spec.md#reader-scheduling-and-descriptor-resources
[batch-hooks]: https://github.com/open-telemetry/otel-arrow/pull/4648

## Reader lifecycle controls

`pin(key)` protects an already resident descriptor from eviction; it refuses a
nonresident or stale key. It does not pause reading. `unpin(key)` removes only
that protection and wakes resource waiters; unresolved batch progress still
prevents eviction. Both operations are idempotent for a valid resident pin.

Composition stops source work, seals batches, resolves delivery, applies progress
and calls `observe_applied` before requesting retirement. `retire(key, batcher)`
checks both components in one synchronous worker-local operation. It refuses:

- A stale key, a pinned reader or a terminal scheduler/batcher failure.
- Buffered input, unfinished line/multiline framing or continuation recovery.
- Rejected output, or a provisional frontier not matching applied evidence.
- Missing or inconsistent batch registration, or open, retained or carry-over
  progress. Sealing, sending or Ack alone is insufficient.

A halted reader can retire only if these same checks pass. Retirement does not
finalize EOF or authorize abandoning unfinished capture. Those policies remain
separate integration work.

The batcher validates and unregisters only after scheduler checks pass. From that
point scheduler cleanup cannot return a refusal: exclusive mutable borrows
prevent source or batch work from interleaving. Any refusal preserves both
registrations and their resources. The operation removes ready, blocked,
capacity-wait, deferred-probe, deadline, resident, grant and selection references,
and cancels the reader's pending probe request without requeueing it. It closes
the descriptor and drops partial allocations before releasing their reservation.

Success returns the still-held locator lease to composition, which must retain
and account for it until any authorized identity transition and explicit lease
release finish. Retirement neither changes identity/checkpoint state nor grants
permission to release that lease. Lease-release failure follows the identity
component's fail-closed rules.

Vacant reader slots are reused with a new generation. Complete keys identify
source callbacks, framed output and batch registrations. Stale lifecycle and
metadata updates are refused, and late output is returned intact. Scheduler
preflight only uses live keys: missing registration is a terminal composition
error. Keys are local to one scheduler instance and must not be
routed across worker instances. Internal queues use slot indices only because
retirement removes all references before reuse. A generation never wraps: its
exhausted slot remains unavailable, and admission returns the intact seed if no
capacity remains.

Reader/free-slot storage and scheduling references are bounded by
`max_tracked_files`, independent of lifetime admissions. Batcher unregister drops
source metadata and restores registration capacity. Cleanup scans bounded
scheduling collections on the blocking worker; it performs no source reads or
checkpoint I/O. Worker hosting, lifecycle policy and receiver registration remain
separate.
