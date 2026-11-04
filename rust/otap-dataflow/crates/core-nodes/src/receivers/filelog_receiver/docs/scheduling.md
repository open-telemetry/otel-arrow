# Reader scheduling (Task 15)

## Ownership and boundaries

The synchronous scheduler runs on Task 14's read/checkpoint worker. It reuses
`LineFramer`, `MultilineFramer`, `SourceFile`, frontier guards and `ProbeBudget`.
It creates no worker, batch builder, durable identity or checkpoint writer.

`Scheduler::from_inputs` uses the same validated configuration as Task 12.
`admit` consumes its actual `ReaderSeed`, preserving file ID, epoch, initial
position, recovered continuation, frontier bytes, fingerprint and non-cloneable
lease. A refusal returns the intact seed, including its lease. The fingerprint
is checked at the configured ignored-header offset. Descriptor eviction does
not release the lease or reset framing.

After authorized checkpoint application, `observe_applied` accepts the matching
record from the identity owner's authoritative state. It refreshes cached guard
evidence without changing the live read frontier or writing a checkpoint. Stale
identity/epoch, backward progress and progress beyond observed source are refused.
Task 24 supplies the existing pin-bound policy opener to `LinuxIo`; there is no
default unchecked pathname opener.

## Scheduling and resource bounds

Ready readers rotate after source service. Each turn bounds source bytes and
processing steps independently, including zero-consumption replay. Evidence
reads count against the byte limit. One reusable source buffer serves all
readers; only unread remainders are copied into reserved reader storage.
Cancellation is checked between operations. An operation cannot interrupt a
blocked kernel call or a bounded multiline pattern search.

Eviction selects the least-recently-served eligible handle. Unresolved sink
deltas and pinned rotation handles veto closure. Eviction/validation work can
resume over multiple calls, including when successful eviction uses the last
step. A pending probe request belongs to one reader, which receives the handoff
before other ready readers. Other resident readers can still run while it waits.

Each sleeping reader has one deadline entry. Pauses retain deadlines and suppress
expired-timer wakeups. Environmental backoff grows from 250 ms to 30 seconds;
`EMFILE`/`ENFILE` also defer other new opens. That deferral is `OpenBackoff`, not
EOF evidence. Resident EOF reprobes check handle size and read at the frontier
without rereading fingerprints. Full validation is required after reopening or
discarding speculative input, including when the handle remains resident.

## Partial capacity

The conservative slot includes the existing framer payload peak, one maximum
source-turn remainder, inline state and bounded raw boundary history. History
is 136 bytes for newline framing, or twice the maximum decoded line size plus
136 bytes for multiline lookahead. An additional 64 bytes covers overlap when
initial frontier history grows to that ceiling. The whole charge is reserved
before framing; payload allocation is lazy.

Capacity admission is FIFO. A released slot is granted to the oldest waiter
before the releasing reader competes again. A holder receives the specified
source-byte allowance across completed logical records; split fragments alone
do not release capacity. Clean yields discard only later speculative input and
require source revalidation before replay. Resumption preserves encoding and
does not restart BOM detection at nonzero offsets.

Open backoff, descriptor waits and probe waits release empty slots and unused
grants. They retain any unfinished input, framing or split continuation. This
allows resident readers to progress while other sources cannot open. EOF also
drops empty retained capacity before releasing its slot. Unterminated EOF records
can retain reservations indefinitely when idle flush is disabled.

Task 19 still owns complete receiver accounting, including reader tables, matcher
scratch, allocator overhead and process-pressure integration. These component
bounds do not establish a receiver RSS or throughput guarantee.

## Task 16 batch interface

The small interface is in [batch.rs](../scheduler/batch.rs).

- `preflight(key, epoch)` refuses known limits, retained batches and waiting
  carry-over before source work or any framing step, including replay.
- `offer(output)` consumes an owned frame on both `Continue` and `Stop`. Stop
  cannot create another carry-over. Output preserves the supplied durable file
  ID and epoch, source ranges, fragment/ending evidence and frame-end guard.
- `unresolved(key, epoch)` includes open batch, retained batch and carry-over
  deltas when vetoing eviction.

Unexpected rejection returns the exact output. The scheduler retains it under
its originating reservation and stops; it does not retry an ambiguous offer.
`resume_sink` cannot override separate read, processing or memory permission.
The scheduler does not authorize aggregate Ack, progress, loss, idle completion
or drain. Those decisions belong to Tasks 17/20/24.

## Probe handoff

Registered reader and discovery demand alternate at release. A completed reader
conversion does not wake its own worker. Cancellation, sink Stop and terminal
faults relinquish pending probe priority immediately while retaining reader and
output ownership. Route a probe wake to `probe_available`; descriptor/pin/delta
changes use `resources_changed`.

The scanner waits for notification when contending with a registered reader.
A 10 ms park timeout samples its existing cancellation callback; fairness comes
from the remembered turn, not polling or elapsed time. The scanner has no pinned
source handle while waiting. Unregistered external reservations still produce
`ProbeBusy` and incomplete coverage. Task 12's opportunistic admission checks
remain nonblocking and defer behind registered worker requests. Worker scheduling
must service a granted reader before attempting further identity admission.

## Integration and validation

Task 12 integration is implemented. Task 14 supplies worker control and wakeups;
Task 16 supplies the actual sink; Tasks 17/20/24 supply lifecycle authority and
final receiver composition. The receiver is not registered by this component.

Tests use controlled time/I/O, real Task 12 leases and committed seeds, a fake
sink, and channel-synchronized probe contention. They include complete real
Linux discovery passes under sustained reader requests. Detailed commands and
results are recorded in [scheduling-validation.md](scheduling-validation.md).
This standalone component changes no registered receiver behavior and needs no
release-note entry.
