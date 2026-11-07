# Reader scheduling

The synchronous [scheduler](../scheduler/mod.rs) runs on the read/checkpoint
worker. It consumes `ReaderSeed` values from the identity component and retains
their leases independently of open file handles.

Reading and framing have separate turn budgets, including buffered replay.
Readers share one source buffer and reserve capacity before framing. Eviction
preserves unfinished state; reopening validates source continuity before work
resumes. Unresolved batch deltas and pinned rotation handles prevent eviction.

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

`unresolved` covers open, retained and carry-over progress for the reader/epoch.
Sealing, sending or Ack alone does not release the descriptor-eviction veto.
After authorized progress application, composition releases the retained batch,
admits any carry-over into the next batch, refreshes reader evidence, and only
then calls `resume_sink`. Carry-over keeps its unresolved veto, and independent
permission checks still govern whether source work can resume.

Route probe-release notifications to `probe_available` and descriptor, pin or
batch-delta changes to `resources_changed`. After authorized checkpoint
application, `observe_applied` refreshes cached evidence from the identity
owner's record. The scheduler does not authorize delivery, checkpoint or
lifecycle transitions.

See the [behavioral specification][spec] for scheduling and resource rules;
implementation details and tests live beside the code.

[spec]: ../../../../../../docs/filelog-receiver-phase1-spec.md#reader-scheduling-and-descriptor-resources
