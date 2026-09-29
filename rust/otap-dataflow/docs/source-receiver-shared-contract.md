# Shared contracts for checkpointed source receivers

**Status:** Proposed companion contract for Filelog and Journald. Agreement on
these boundaries precedes freezing their worker and checkpoint APIs. This does
not claim that existing implementations already satisfy the contract.

Related designs: [Filelog](filelog-receiver.md),
[Journald](journald-receiver.md), and
[common source coordination #4001](https://github.com/open-telemetry/otel-arrow/issues/4001).

## Scope and ownership

This document defines shared behavior, not a common Rust trait, worker topology,
checkpoint format, or new coordinator. Each receiver supplies its own progress
representation, validity checks, recovery rules, and persistence implementation.
A common implementation should be extracted only where both sources demonstrate
compatible requirements. Existing differences must be resolved explicitly during
integration rather than silently overridden by a helper library.

<!-- markdownlint-disable MD013 -->

| Shared boundary | Source-specific responsibility |
| --- | --- |
| Delivery identity and commit authorization | Meaning and validation of the proposed progress change |
| Stable state root and namespace exclusion | Source identity, checkpoint namespace recipe, and payload encoding |
| Commit outcome and durability reporting | Storage transactions, recovery evidence, and corruption rules |
| Lifecycle, readiness, and worker termination | Source API, worker arrangement, and source-specific drain limits |
| Exclusive administration and audit reporting | Valid reset targets and source evidence needed for each action |
| Future ownership authorization at commit | File partitions, journal subscriptions, and source-specific handoff state |

<!-- markdownlint-enable MD013 -->

Filelog owns file locators, byte offsets, decoding/framing, rotation, and its
snapshot/WAL format. Journald owns journal selection, opaque cursors, cursor
validation, and journal recovery. Journal cursors must not be converted into
numeric offsets or assumed to support ordering or arithmetic.

## Ack-correlated commit

Each bounded delivery associates its source-specific proposed progress with a
unique delivery identity. Sources may implement retained-payload retry or
explicit replay from committed source progress; these are different guarantees.
Filelog retains the same logical payload and progress proposal across retries,
with a distinct attempt identity for every resend. Journald's source-replay mode
may reconstruct different output and creates a new delivery identity after
rewind, invalidating the earlier delivery's completion authority. The adapter
must not start replay while an old accepted attempt can still mutate its progress.
Late, duplicate, or unrelated completions never authorize the new delivery.
Required fan-out completion is aggregated by the engine, not by independent
receiver-local subscriber state machines. Each source defines payload ownership
and replay limits explicitly; a common helper must not silently substitute replay
for retained-payload retry or assume record-identical reconstruction.

A matching aggregate Ack authorizes exactly the proposed progress transition.
Read completion, enqueue, Nack, NoRoute, cancellation, and timeout do not. Any
explicit loss policy is separate authorization with source-specific semantics;
it must not be disguised as Ack. Retryability metadata must be preserved end to
end before it controls policy; see
[typed Nack propagation #4065](https://github.com/open-telemetry/otel-arrow/issues/4065).

The persistence boundary distinguishes authorized, applied, and durable progress.
It reports failures without treating a possibly applied write as known absent.
An ambiguous write is reconciled using the source store's recovery protocol
before another mutation. Retrying persistence after Ack must not turn into a
fresh delivery retry. Payload release follows the receiver's documented commit
and sync policy. A shared helper must not silently impose delayed durability on
a source that requires synchronous commits.

The progress proposal remains opaque to shared plumbing. Filelog may commit an
atomic set of file deltas; Journald may commit a cursor. Their adapters establish
ordering and reachability. Common code must not compare opaque proposals or
merge them by taking a numeric maximum.

Restart guarantees depend on surviving source data and the durable checkpoint.
Retaining an in-memory batch does not establish durable record-level replay.
Each source documents its reconstruction and retention limitations separately.

## Stable state root and namespace ownership

The engine supplies a validated absolute durable root selected independently of
the working directory. Resolution, ownership/permissions, directory validation,
creation, and required parent-directory syncs complete before checkpoint access.
There is no receiver-local relative fallback. An intentional root change selects
different state unless an explicit migration preserves it. The
[Filelog root contract](filelog-receiver-phase1-spec.md#stable-engine-state-root)
defines the proposed engine integration requirements.

Each source defines a stable, collision-safe namespace derived from its logical
identity and receiver kind, independent of current core or runtime generation.
Exclusive ownership is required before recovery or mutation and remains effective
while any worker can access the store. Failed acquisition does not authorize a
new namespace or source reads. An ownership object must follow actual resource
lifetime, not merely the async task or join-handle lifetime.

Source stores retain their own publication and recovery barriers. A generic
success result cannot erase a requirement to sync a recovered authority marker
before new progress or cleanup. Sharing this boundary does not adopt Filelog's
CURRENT layout or v1 envelope for Journald.

## Lifecycle, readiness, and termination

Lifecycle control has bounded, independent delivery and cooperative cancellation
between bounded work units. Backpressure pauses source intake while completion,
commit, cancellation, and cleanup remain serviceable. Queues and retained work
have explicit owners and bounds; no shutdown path assumes channel capacity is
available or blocks the async runtime on an unbounded thread join.

Startup readiness requires runtime initialization, namespace ownership, and
successful source-specific checkpoint recovery sufficient to schedule safely.
Pipeline construction alone is not that signal. Readiness is one-time startup
status, not continuous source or downstream health. Engine integration is tracked
in [#4049](https://github.com/open-telemetry/otel-arrow/issues/4049).

Replacement must not wait for candidate ownership while the old owner waits for
candidate readiness before releasing its lock. Until coordinated handoff exists,
exclusive sources require an explicit stop-before-start path. Successful stop
requires confirmed worker termination and safe ownership release, not just a
cancellation request or drained message from the async task.

Delivery-drain and worker-termination deadlines are distinct: exhausting a
downstream wait must not consume the allowance for cooperative cancellation.
The engine must define the combined total stop bound and any process hard-stop
policy. A worker deadline starts once and is not renewed by repeated commands.

The termination contract distinguishes clean completion, source/drain failure,
and failure to terminate workers. Filelog Phase 1 selects the
[process-fatal join-timeout policy](filelog-receiver-phase1-spec.md#worker-termination-and-process-fatal-join-timeout).
Journald integration must explicitly select an equally bounded replacement
policy before its shared lifecycle API is finalized; it must not silently inherit
process-fatal behavior from a reusable helper. If another design retains detached
workers, process-wide permits must remain owned until actual thread exit and
must bound replacement. Such a permit mechanism is not specified here.

No path claims that cancellation or process termination immediately interrupts
an uninterruptible kernel call. Resource accounting and namespace exclusion
remain effective for surviving access. Shutdown never fabricates Ack or commits
unacknowledged progress.

## Administration

Inspection that requires a consistent store view and all mutating administration
use the store's explicit exclusion/snapshot protocol. Reset and removal cannot
race a live owner or worker. The interface identifies the namespace, intended
action, and expected state; the source adapter validates evidence and performs
its own atomic durable transition. Live reset is unsupported unless separately
designed with equivalent exclusion and fencing.

Source-specific procedures define backup, corruption containment, interruption
recovery, and observable replay or skip consequences. Administrative success is
reported only after its required durability conditions hold. Metrics and health
events are not a durable audit log; any durable audit requirement must be met
explicitly. A generic administrative helper must not turn a journal cursor reset
into a Filelog epoch reset or bypass either source's evidence checks.

## Future fencing boundary

Reserve an ownership-authorization boundary around acquisition, commit, revoke,
and administrative mutation. Future shared coordination may supply an opaque
fencing token bound to a namespace and ownership revision. The authoritative
store must reject stale authorization at the mutation boundary; checking a token
only before queuing work does not fence a delayed write.

Phase 1 local locks are not distributed fencing tokens. This document adds no
token to either checkpoint format and specifies no token encoding. Token
issuance, atomic validation, fixed-partition assignment, live handoff, and
versioned checkpoint migration remain under #4001. That design must settle how
late completion and queued commits interact with revocation before new owners
read. Shared plumbing must not bake core IDs or reusable runtime-instance IDs
into durable ownership authority.

## Integration evidence

Before the affected APIs are finalized, each receiver maps its concrete types
and state transitions to this contract and records intentional differences.
Source tests retain responsibility for payload correctness; common contract
scenarios cover:

- matching, stale, duplicate, and retried delivery completions;
- Ack followed by persistence failure or ambiguous write completion;
- working-directory changes, interrupted root creation, and namespace contention;
- delayed readiness, failed recovery, and exclusive-owner replacement;
- drain under backpressure, full channels, partially started workers, and workers
  that fail to terminate without releasing their resource ownership;
- administrative contention, invalid source evidence, and interrupted reset.

Fencing and mixed-owner tests become mandatory when the #4001 integration is
implemented. Agreement on this contract does not imply that shared code,
receiver integration, or those future capabilities are already complete.
