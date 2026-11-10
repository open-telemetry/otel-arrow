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
resumes. Unresolved batch deltas prevent eviction. Rotation pin controls and
reader retirement remain receiver-integration prerequisites: retirement must
resolve retained output and authorize partial-state disposal before releasing
the lease and capacity. Until those controls exist, halted readers retain bounded
capacity for unfinished or rejected output, and receiver integration must remain
disabled. Empty partial slots are released even when a reader halts; the lease
and logical reader remain retained.

Validated resident readers at EOF with no partial state check metadata before
requesting a slot. Unchanged EOF re-parks without a content read or partial
construction and does not refresh eviction age. Growth follows normal capacity
admission; evicted readers still reopen and validate continuity.

Idle partial flush is not wired: nonzero `framing.force_flush_period` is not
serviced by this scheduler. Idle-flush deadlines and source revalidation,
startup descriptor-budget checks, and admission gating during global descriptor
backoff remain receiver-integration prerequisites alongside retirement and pins.

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

See the [behavioral specification][spec] for scheduling and resource rules;
implementation details and tests live beside the code.

[spec]: ../../../../../../docs/filelog-receiver-phase1-spec.md#reader-scheduling-and-descriptor-resources
[batch-hooks]: https://github.com/open-telemetry/otel-arrow/pull/4648
