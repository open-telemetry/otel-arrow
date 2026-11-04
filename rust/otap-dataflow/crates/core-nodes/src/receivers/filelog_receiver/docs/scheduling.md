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

See the [behavioral specification][spec] for scheduling and resource rules;
implementation details and tests live beside the code.

[spec]: ../../../../../../docs/filelog-receiver-phase1-spec.md#reader-scheduling-and-descriptor-resources
