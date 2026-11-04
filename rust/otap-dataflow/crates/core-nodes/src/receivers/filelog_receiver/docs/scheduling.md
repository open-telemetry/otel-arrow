# Reader scheduling (Task 15)

## Plan and ownership

Start from upstream main, independently of Tasks 12 and 13. Reuse `LineFramer`,
`MultilineFramer`, `SourceFile`, the frontier guard recipe, and `ProbeBudget`.
The synchronous scheduler belongs on Task 14's read/checkpoint worker. Task 24
will supply lifecycle permission and compose the components.

Keep bounded ready and partial-capacity queues, one deadline per waiting reader,
resident descriptors, retained framing state, and one reusable source-turn
buffer. Bound source bytes and processing steps independently. Preserve state
on eviction and pauses. Use FIFO partial-slot admission and yield completed
records after the specified source-byte allowance. Revalidate discarded
read-ahead before reuse, including on a still-resident descriptor.

Keep source access behind a small synchronous adapter. Production opening must
reuse pin-bound policy validation; an unchecked pathname open is not an adapter.
Use an explicit shared probe handoff and release notification for contention.
Keep batch preflight, owned output transfer, Stop, failure disposition, and
unresolved-delta queries together for Task 16 review. Do not build OTAP batches.

Task 12 (#4566) and Task 13 (#4623) were unmerged at implementation start.
Reader construction remains test-only until `ReaderSeed` and its lease can be
consumed directly. There is no production identity generator, admission bypass,
checkpoint writer, or recovery decision in this component. Refresh Task 12
before delivery; identity and production composition remain explicit gates.

Use controlled clocks, I/O, cancellation and a fake batch sink for scheduling
tests. Cover hot-reader fairness, byte/step limits, replay, pause/deadline
interaction, descriptor pressure, partial-slot accounting, probe handoff, Stop
ownership, and failure ordering. Validate real source access on Linux `testvm`
with native storage, then run affected-crate checks, Markdown/sanity checks and
the full workspace check. Record focused and full-workspace outcomes separately.

## Implemented core

`Scheduler::turn` does synchronous work only. Its caller supplies `Instant`, a
cancellation predicate, independent read/processing/memory permission, a wake
handle, and a sink. There is no thread, async runtime, timer task, checkpoint
writer, or production reader-admission constructor in this module.

Ready readers rotate after source service. A source read uses the shared buffer;
only an unread remainder is copied into the reader's reserved storage. Every
framer call costs a processing step, even with zero fresh-byte consumption.
Evidence reads also consume the source-byte budget. Eviction scans and reopen
validation can resume over several calls; the selected service keeps its place
until validation can make progress, while each return permits worker control
and checkpoint work. This avoids repeatedly evicting partially validated handles.
A bounded step cannot interrupt a kernel call or a multiline pattern search.

One ordered deadline entry is retained per sleeping reader. Pauses leave it
intact and suppress the wake deadline. There is no stale-entry heap or polling
loop. Resident EOF reprobes check handle size and read at the frontier without
rereading the fingerprint. Environmental retries use the specified 250 ms
exponential ramp capped
at 30 seconds. Descriptor exhaustion also defers other new opens. A file
transition or deterministic framing failure stops that reader and returns the
reason; composition must resolve earlier deltas and authorize the next action.

Descriptor service uses least-recently-served ordering with bounded resumable
victim inspection. Unresolved sink deltas and pinned rotation handles veto
closure. A descriptor wait requests surrounding batch/lifecycle work; it does
not seal a batch or decide that a file can be removed. Framing state, source
remainders, frontier evidence and reservations survive descriptor closure.

## Partial-state accounting and fairness

The local conservative slot charge is the existing framer payload peak plus:

- one maximum source-turn remainder;
- inline partial/framer state;
- 136 bytes of raw boundary history for newline framing, or twice the configured
  maximum line size plus 136 bytes for multiline lookahead.

The bounded raw history preserves exact trailing-window evidence at emitted
boundaries, including a multiline boundary behind a consumed lookahead line.
It is not another durable buffer. Reservations cover allocation-growth overlap
already included by the framer peak; retained capacity is checked separately.
Completed output transfers to the sink's pre-admitted capacity. A rejected
output remains scheduler-owned under its originating reservation and stops all
further work until explicit external failure disposition.

Capacity waiters are FIFO, with one entry per reader. A released slot is granted
to its oldest waiter before the releasing reader can compete again. Complete
logical records count their original source-frame bytes toward the service
allowance. Nonfinal fragments do not release the slot. At a clean yield, only
later speculative input is discarded; the saved source boundary is revalidated
before rereading, including when its handle remains resident. A resumed decoder
uses its configured encoding and never restarts initial BOM recognition.
An unfinished EOF record may retain a slot indefinitely with idle flush disabled.

The configuration component currently validates a payload/remainder floor;
production composition must additionally apply this scheduler's derived charge.
Task 19 still owns complete receiver accounting, allocator overhead, matcher
scratch, and integration with process-pressure signals. These tests do not
establish an RSS bound or whole-receiver throughput.

## Boundary for Task 16

Review [batch.rs](../scheduler/batch.rs) independently of scheduling mechanics.

| Method | Required ownership behavior |
| --- | --- |
| `preflight(key, epoch)` | Refuse known batch limits, retained batches and waiting carry-over before reading or resuming any framing, including replay. |
| `offer(output)` | Consume the owned frame on both `Continue` and `Stop`; Stop cannot create a second carry-over. |
| `unresolved(key, epoch)` | Include the open batch, retained batch and carry-over when vetoing eviction. |

The output carries the opaque reader key, supplied epoch, exact primitive frame
(including body/frame ranges, fragment and ending evidence), and the frame-end
guard. Task 12 integration will map the opaque key to its supplied durable file
ID. No ID or epoch is generated here. `Rejected` returns the exact owned output;
the scheduler retains it and returns a terminal disposition rather than retrying
an offer that may have been partially applied.

`resume_sink` is an explicit composition notification after resolution. It
cannot override the independent read, processing or memory permission. The
scheduler never handles aggregate Ack, retries an OTAP batch, or advances an
applied/durable checkpoint.

## Probe coordination and integration gates

`ProbeBudget::request_reader` registers one read-worker request before trying
to acquire the shared two-descriptor allowance. Discovery cannot reacquire
ahead of that request. Releasing the allowance wakes the registered worker;
dropping a request cancels its priority. The worker routes that notification to
`resources_changed`. No discovery-thread body or worker host is introduced.
Atomic request state and a registered waker keep acquisition nonblocking; no
source I/O runs under shared synchronization.

Before production integration:

1. Consume Task 12's `ReaderSeed` and actual lease directly, carrying its file ID,
   epoch, starting/continuation state and validated evidence. Retain that lease
   until externally authorized cleanup. Test-only tokens are not qualification
   of the actual identity lease.
2. Supply the existing pin-bound opener, including target/path/namespace policy.
   `LinuxIo` reuses `SourceFile` for metadata and reads; it deliberately provides
   no unchecked pathname opener. Refresh the cached applied guard from the
   authoritative identity/checkpoint component after authorized progress.
3. Task 14 supplies control wakeups and cancellation; Task 16 implements the sink.
   Tasks 17/20/24 own delivery, idle/permanent-EOF authorization, transition
   disposition, drain and final composition. The scheduler does not infer those
   permissions from elapsed time, EOF, removal or a returned failure.

Task 15 remains incomplete until its required identity and production boundaries
are connected and qualified. This standalone implementation changes no registered
receiver behavior, so its PR uses `chore` and has no release-note entry.
