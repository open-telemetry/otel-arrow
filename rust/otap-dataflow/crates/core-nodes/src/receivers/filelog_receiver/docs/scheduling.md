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
resumes. Unresolved batch progress and lifecycle pins prevent eviction. A halt
releases empty partial slots but keeps the reader's lease, unfinished input and
rejected output.

Validated resident readers at EOF with no partial state check metadata before
requesting a slot. Unchanged EOF re-parks without a content read or partial
construction and does not refresh eviction age. Growth follows normal capacity
admission; evicted readers still reopen and validate continuity.

Idle partial flush is not wired: this scheduler does not service a nonzero
`framing.force_flush_period`. Receiver integration also needs source checks for
idle flush, startup descriptor-budget checks, and admission pauses during global
descriptor backoff.

Keep receiver integration disabled until it also supports authorized cleanup
when a file disappears after its descriptor has closed. That cleanup may need
to discard unfinished input or rejected output. It is not implemented here;
`retire` only removes clean readers.

Before enabling frequent retirement, measure cleanup latency with many tracked
files and repeated admissions/removals. Retirement still scans deadlines and
queues. Every idle EOF reader can have a deadline; storing that deadline on the
reader would allow direct removal. Defer queue indexing until measurements show
it is needed.

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

`pin(key)` keeps an open descriptor from being evicted. It rejects stale keys and
readers without an open descriptor, and does not pause reads. `unpin(key)` clears
only that pin and wakes waiting readers. Unresolved batch progress still prevents
eviction. Repeating either operation is safe for a valid resident reader.

To retire a reader, composition must:

1. Stop reads and seal outstanding batches.
2. Resolve delivery, apply checkpoint progress and call `observe_applied`.
3. Call `retire(key, batcher)` on the same worker.

Retirement refuses stale keys, pinned readers and terminal component failures.
It also refuses readers with:

- Buffered input, unfinished framing or continuation recovery.
- Rejected output or read progress that does not match applied checkpoint evidence.
- Missing or inconsistent batch registration.
- Progress in an open batch, retained batch or carry-over record.

A halted reader must pass the same checks. Sealing, sending or receiving Ack is
not enough. Retirement does not finish EOF or authorize discarding pending data.

Both components are checked before either is removed. A refusal preserves their
registrations and resources. After batcher unregister succeeds, the remaining
cleanup cannot return an error. It removes all scheduling, timer, probe and
selection references, closes the descriptor, then drops partial allocations
before returning their capacity.
Internal queues store slot indices; every collection holding a reader index must
remove it during retirement, before the slot is reused.

Success returns the still-held locator lease. Composition must keep and account
for that lease until it authorizes any identity transition and explicit release.
Retirement performs no checkpoint writes or identity transitions. Lease-release
failures follow the identity component's fail-closed rules.

Reused slots get a new generation, so old keys cannot control the replacement
reader or update its metadata. Late output is returned intact. Keys belong to
one scheduler instance and must not cross workers. Scheduler preflight uses only
live keys; missing batch registration is a terminal composition error.

Generations never wrap. An exhausted slot stays unavailable; admission returns
the intact seed if no capacity remains. Reader slots, free slots and scheduling
references stay bounded by `max_tracked_files`, regardless of lifetime admissions.
Unregistering also releases batch metadata and registration capacity. Worker
hosting, lifecycle policy and receiver registration remain separate.
