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

Route probe-release notifications to `probe_available` and descriptor, pin or
batch-delta changes to `resources_changed`. After authorized checkpoint
application, `observe_applied` refreshes cached evidence from the identity
owner's record. The scheduler does not authorize delivery, checkpoint or
lifecycle transitions.

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
