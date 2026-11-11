# Retained-batch delivery

The internal [delivery module](../delivery/mod.rs) authorizes one retained batch
at a time. It does not register or enable the Filelog receiver. Source admission,
worker hosting, topology validation and lifecycle composition remain separate.

The read/checkpoint worker owns `Delivery<B>`, its batch builder, scheduler and
`IdentityOwner`. Only `builder_mut(now)` grants access to source/builder work.
A sealed batch stops that work until matching aggregate Ack, atomic checkpoint
application and any required sync have succeeded. The host must check independent
source permissions, including memory pressure, before resuming reads.

## Pipeline boundary

`next_attempt(now)` returns one shared-buffer clone and a unique batch/attempt
pair. Pass it through the single bounded worker-to-pipeline handoff slot. The
pipeline task calls `send_attempt`, which uses the engine's source-send path and
subscribes to both Ack and Nack with the pair in `CallData` before sending.

The pipeline task must concurrently service lifecycle/control messages and cancel
the send when delivery stops. A full output channel leaves the send pending;
cancellation drops its clone without releasing worker-owned work. A closed
channel is a retryable publication failure. Other typed engine errors are
returned to the host, which must stop delivery and preserve unresolved work.
No send timeout is converted into an Ack or an assumed Nack. The engine currently
has no typed local `NoRoute` outcome, so the corresponding conformance case is
not yet implemented. Missing-default-port errors remain terminal configuration
errors rather than entering indefinite delivery pause.

Forward the send outcome with `sent` before forwarding that attempt's aggregate
completion through `complete`. The host must preserve this ordering through its
bounded command channel even when completion arrives before send-result handling.
A matching completion received during `Sending` is a host ordering violation and
stops delivery visibly. Malformed, duplicate, wrong-batch and old-attempt
completions remain ignored and counted. A matching Ack authorizes checkpoint work;
it does not release anything.
The host must use engine aggregate completions, never synthesize success from
individual destinations or empty subscriber membership.

Topology/readiness checks must establish aggregate Ack propagation before any
source admission. This component does not certify arbitrary pipeline graphs.
The receiver remains disabled until those checks and worker integration exist.

## Checkpoint boundary

Call `checkpoint` on the blocking read/checkpoint worker after authorization.
It uses `IdentityOwner::append_checkpoint` for one complete progress transaction.
The existing authority validates file identities, expected offsets, epochs and
transaction bounds atomically, and performs required durability barriers.
Compaction exists in the store but is not yet integrated into `IdentityOwner`.
Reaching its WAL limit stops delivery with the retained batch intact; automatic
compaction and authority-refresh integration remain prerequisites for sustained
collection. This component does not bypass the authority to compact storage.

`checkpoint` returns `Applied`, `NotReady`, or `Blocked`. A blocked outcome names
the component responsible for resolution and carries the original error when one
was just observed. `Err` always means terminal failure; `is_stopped` also exposes
that state. For a batch-owned pending write, call `checkpoint` at `next_deadline`:
it resolves the original transaction through `IdentityOwner::resolve_pending`,
never resubmits it or resends the Acked payload. The authority enforces the
configured consecutive store-failure budget.

Host-owned admission and fingerprint writes remain the host's responsibility.
Delivery never resolves those writes or consumes their admission results. They
block source access, new sends and checkpoint application. Their wakeup belongs
to the host, so `next_deadline` suppresses delivery/sync timers while blocked.
After resolving its write, the host must call `sync` or `checkpoint` to refresh
the authority gate. Call `sync` after host mutations and before source admission,
even if no periodic sync is due. A stopped authority stops delivery.

Growth initiated by delivery's own `sync` has a separate blocked state and retry
deadline. Only `sync` resolves that growth, without releasing or applying any
retained batch. Transaction ownership is recorded at initiation, not inferred
from whether the resolution happens to return admissions. The host must not
resolve delivery-owned batch or growth writes.

After any successful `sync` result other than `Blocked`, the worker must service
`checkpoint` and then `next_attempt` before waiting again. This includes
`NotReady` and `MaintenanceRejected`: an Ack may have authorized a retained batch
while growth was blocked. Resolving growth removes its timer but does not apply
that batch. `next_deadline` describes timed work, not whether immediate work is
ready. Service these operations once per event or deadline, rather than spinning
on `NotReady`.

Store-resolution retries use `retry.max_backoff` as their fixed worker delay;
they do not consume the delivery attempt budget. This coupling is internal and
introduces no new configuration. Required-sync failures remain terminal; the
surrounding lifecycle must preserve that incident across replacement and must not
reopen to bypass it.

With zero sync interval, release requires a synced receipt. With deferred sync,
release follows append/application, and `next_deadline` retains the sync deadline.
Call `sync` at that deadline even during idle sources or delivery backoff. Source
access and new attempts are blocked at an overdue sync deadline. An unresolved
append must settle before periodic sync; its retry deadline takes precedence.
`sync` distinguishes `Synced`, `NotReady`, `Blocked`, and `MaintenanceRejected`.
Once required synchronization succeeds, its deadline is cleared even if optional
growth fails. Recoverable growth stays blocked; a pre-write rejection that leaves
the owner usable is reported without stopping. A failed sync, exhausted failure
budget or other terminal authority failure still returns `Err` and stops delivery.
Final drain synchronization and worker termination/join policy belong to
lifecycle composition.

## Batch-builder boundary

`RetainedBatch` is an internal adapter for the batch builder. It mirrors the
builder contract:

- `failure` returns the builder's latched error, if any, before delivery or
  checkpoint application. The adapter can clone `BatchBuilder::failure()`.
- `retained` borrows the builder's immutable `records` and `progress`.
- `release_applied` receives the exact applied progress slice and current time.
  The builder validates the slice and promotes its exact carry-over before
  source scheduling can resume. If promotion immediately seals another batch,
  that batch requires a fresh delivery identity and its own matching Ack.

The associated error type preserves the builder's concrete error as the source
of `DeliveryError::Batch`, including failure after checkpoint application. Such
a release failure stops delivery and never retries the applied transaction.

The adapter must delegate these operations to the builder. It must not rebuild
records, reconstruct evidence from mutable files, transfer carry-over to delivery,
or allow mutation of retained progress. Tests use a builder double while that
implementation is pending integration. They do not qualify the real builder's
promotion or scheduler integration.

After successful release, `checkpoint` returns the exact applied progress.
Worker composition uses its file identities to refresh scheduler evidence with
`observe_applied`, then calls `resume_sink` subject to source permissions and
remaining carry-over. Neither Ack alone nor send success permits these actions.

## Bounds and supported policy

There is one retained builder batch, at most one handoff clone, one current
correlation tuple and fixed-size retry bookkeeping. There is no per-attempt
history or overlapping send. Deadlines use caller-supplied monotonic time;
checked arithmetic prevents identity/deadline wraparound.

Nacks and retryable publication failures preserve the exact batch. Exponential
backoff saturates at the configured maximum. Default exhaustion pauses delivery
with periodic retries at that maximum; explicit `fail` stops without progress.
Pause-entry time and reason are not yet exposed; worker/telemetry integration
must add that observability before the receiver is enabled.
`drop_and_continue` is rejected by this internal coordinator until explicit-loss
reporting is integrated. No new receiver configuration is exposed.

Cancellation stops attempts and deadlines without releasing retained work. The
host retains ownership until worker access ceases; teardown may then abandon
volatile batches without advancing checkpoints. Recovery uses the existing
store/replay implementation and surviving source bytes. Exact in-process retry
does not imply identical record reproduction after a crash.
