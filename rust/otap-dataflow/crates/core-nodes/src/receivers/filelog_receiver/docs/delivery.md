# Retained-batch delivery

The [delivery module](../delivery/mod.rs) coordinates delivery and checkpointing
for one batch at a time. The receiver remains disabled until worker hosting,
topology checks and lifecycle handling are integrated.

The main flow is:

```text
sealed batch -> send -> matching aggregate Ack -> checkpoint -> release
```

Send success alone does not authorize progress. The worker keeps the batch until
its checkpoint has been applied with the configured durability. Retries use the
same retained data.

## Ownership

The read/checkpoint worker owns delivery, the batch builder, the scheduler and
`IdentityOwner`. The pipeline task handles sending and engine completions.

- Use `builder_mut(now)` to check whether source/builder work is allowed. The
  host must also check independent permissions, such as memory pressure.
- Keep at most one send in flight through a bounded handoff slot. Sending uses
  a shared-buffer clone; the builder retains ownership of the batch.
- Use `RetainedBatch` to borrow records and progress and release them after
  application. The adapter must preserve the builder's exact carry-over and
  must not reconstruct records from source files.

## Sending and completion

Pass `next_attempt(now)` to `send_attempt`. Forward its result through `sent`
**before** forwarding the matching aggregate completion through `complete`, even
if the completion arrives first. Each attempt has its own correlation identity;
stale or duplicate completions cannot authorize progress.

Use engine aggregate Ack/Nack, not individual destination acknowledgements or
assumed success from an empty subscriber set. Topology checks must establish
aggregate Ack propagation before source admission.

The pipeline task must keep servicing lifecycle messages while a send is pending
and cancel it when delivery stops. A closed channel follows the configured retry
policy. Other engine errors return to the host, which must stop delivery while
preserving unresolved work. A timeout must not be treated as an Ack or an assumed
Nack.

## Checkpointing and worker wakeups

Run checkpoint I/O on the blocking read/checkpoint worker through
`IdentityOwner`. Apply the entire batch's progress atomically. An Ack authorizes
this operation; it does not release the batch itself.

A pending write must be resolved by the component that started it:

| Pending work | Who resolves it |
| --- | --- |
| Acked batch checkpoint | Delivery, through `checkpoint` |
| Fingerprint growth started by delivery | Delivery, through `sync` |
| Admission or fingerprint write started by the host | The host |

Do not resend an Acked batch or append its progress again to recover a pending
write. Host-owned writes suppress delivery timers; the host must wake delivery
after resolving them. Call `sync` after host mutations and before admitting
source work, even when no periodic sync is due.

Service `next_deadline` even when sources are idle or delivery is backing off.
With zero sync interval, release waits for synchronization. With deferred sync,
release follows application, but an overdue sync blocks new source work and
attempts until serviced. Terminal storage failures stop delivery; reopening the
store must not be used to bypass a failed required sync.

After a successful, non-blocked `sync`, service `checkpoint` and then
`next_attempt` before waiting again. Resolving fingerprint growth may leave an
Acked batch ready without another timer. Do this once per event or deadline;
do not spin on `NotReady`. Errors are terminal; blocked outcomes identify whose
work must be resolved.

After application and release, refresh scheduler evidence with `observe_applied`
and call `resume_sink`, subject to source permissions. The builder promotes
carry-over first. If that seals another batch, it needs its own delivery and Ack.
A release failure stops delivery without retrying the applied transaction.

## Retry and shutdown policy

Nacks and retryable publication failures retain the exact batch and use bounded
exponential backoff. On exhaustion, the default pause policy keeps retrying at
maximum backoff; `fail` stops without advancing progress. `drop_and_continue` is
unsupported until explicit-loss reporting is integrated.

Cancellation stops attempts and timers without releasing retained work. The host
must wait until worker access ends before dropping it. Uncommitted progress is
not saved; recovery depends on the checkpoint and surviving source bytes.

## Integration still required

- Real batch-builder integration; current delivery tests use a builder double.
- Worker hosting, topology/readiness checks, final drain and worker shutdown.
- Automatic checkpoint compaction through the identity authority. Until then,
  reaching the WAL limit stops delivery with the retained batch intact.
- Pause/loss observability and the engine's typed local `NoRoute` outcome.

See the [implementation](../delivery/mod.rs),
[delivery tests](../delivery/tests.rs) and
[checkpoint tests](../delivery/checkpoint_tests.rs) for API details and failure
scenarios.
