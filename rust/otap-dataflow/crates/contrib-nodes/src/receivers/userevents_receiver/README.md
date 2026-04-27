<!-- markdownlint-disable MD013 -->

# Linux Userevents Receiver

**URN:** `urn:otel:receiver:userevents`

This receiver ingests Linux
[`user_events`](https://docs.kernel.org/trace/user_events.html) tracepoints
through `perf_event_open` and converts them into OTAP logs for downstream
processing.

> **Note:** This receiver is vendor-neutral. It performs Linux `user_events`
> collection and structural decoding only; schema-specific mappings such as
> Microsoft Common Schema should be modeled outside the receiver.

It follows the OTAP Dataflow thread-per-core model:

- one receiver instance per pipeline thread
- one perf session per assigned CPU
- one bounded drain loop per receiver
- no shared hot-path state across pipeline threads

## Architecture

`user_events` writes enter Linux tracing as user-space tracepoint events. The
kernel stores trace records in per-CPU ring buffers; each receiver instance
drains the ring for the CPU/pipeline core it owns, decodes the tracepoint
record, and emits OTAP logs into that same pipeline.

```text
+------------------------------- Same Linux Host -------------------------------+
|                                                                               |
|    App thread(s)                                                              |
|         |                                                                     |
|         |  user_events tracepoint writes                                      |
|         v                                                                     |
|    +---------------------+                                                    |
|    |  Linux tracepoint   |                                                    |
|    |  / perf subsystem   |                                                    |
|    +----------+----------+                                                    |
|               |   (per-CPU fan-out; writer's runtime CPU selects the ring)    |
|      +--------+--------+---------+--- ... ---+                                |
|      v                 v         v           v                                |
|  +---------+     +---------+   +---------+   +---------+                      |
|  | CPU 0   |     | CPU 1   |   | CPU 2   |   | CPU N   |                      |
|  | perf    |     | perf    |   | perf    |   | perf    |    lockless         |
|  | ring    |     | ring    |   | ring    |   | ring    |    per-CPU          |
|  +----+----+     +----+----+   +----+----+   +----+----+    kernel buffers   |
|       |               |             |             |                           |
+-------|---------------|-------------|-------------|---------------------------+
        |               |             |             |
        v               v             v             v
   +----------+    +----------+  +----------+  +----------+
   | pipeline |    | pipeline |  | pipeline |  | pipeline |   df_engine
   | core 0   |    | core 1   |  | core 2   |  | core N   |   user space
   |          |    |          |  |          |  |          |
   | receiver |    | receiver |  | receiver |  | receiver |   one receiver
   | decode   |    | decode   |  | decode   |  | decode   |   instance per
   | batch    |    | batch    |  | batch    |  | batch    |   pipeline core
   | export   |    | export   |  | export   |  | export   |
   +----------+    +----------+  +----------+  +----------+
```

At a high level the Linux tracing ring buffer is a per-CPU producer/reader
structure: writers append on the CPU where they execute, and readers can drain
the per-CPU buffers independently. The receiver consumes those buffers via
`perf_event_open` / `one_collect`; it does not implement the kernel ring-buffer
algorithm itself.

## CPU Pinning and Coverage

**Placement rule:** a pipeline assigned to core `K` opens the perf ring for
CPU `K` only. If an application thread writes a `user_events` record while the
kernel has scheduled it on CPU `X`, that record lands in the CPU `X` ring and
is only visible to the receiver instance that owns CPU `X`. No other receiver
instance sees it, and there is no cross-CPU aggregation layer in this receiver.

```text
  user_events write on CPU K
             |
             v
       CPU K perf ring
             |
             v
  receiver pipeline pinned to CPU K

  No other receiver pipeline reads CPU K's ring.
```

For an 8-core host, coverage looks like this:

```text
  CPU:        0      1      2      3      4      5      6      7
              |      |      |      |      |      |      |      |
  ring:      R0     R1     R2     R3     R4     R5     R6     R7
              ^      ^      ^      ^      ^      ^      ^      ^
              |      |      |      |      |      |      |      |
  pipeline:  P0     P1     P2     P3     P4     P5     P6     P7

  P0 reads only R0. P1 reads only R1. ... P7 reads only R7.
```

If a CPU has no corresponding pipeline, writes on that CPU are not collected by
this receiver deployment. They are not seen and then discarded; no receiver is
attached to that CPU's ring. The engine does not emit a metric for these
missing records because they are never observed by the receiver; they are lost
silently.

| Configuration                    | Writer pinned? | Result                                                     |
|----------------------------------|----------------|------------------------------------------------------------|
| `--num-cores 1`                  | No             | Racy; writes on CPUs 1..N not collected by this receiver   |
| `--num-cores 1`                  | `taskset -c 0` | Reliable single-core capture                               |
| `--core-id-range 0-3`            | No             | Writes on CPUs 4..N not collected by this receiver         |
| `--core-id-range 0-3`            | Pinned to 0..3 | Reliable within the range                                  |
| `--num-cores 0` (all host CPUs)  | No             | Complete host-wide coverage                                |

For production, choose the engine's `--num-cores` / `--core-id-range` settings
carefully. Prefer full coverage (`--num-cores 0`) unless you intentionally want
to sample only a subset of CPUs; for development/testing, a pinned writer on a
single-core engine keeps the moving parts minimal.

## NUMA Locality

The receiver is **designed** to preserve NUMA locality under the df_engine
thread-per-core model. This is a locality design, not a hard guarantee - the
bullets below describe the intended behavior and the conditions under which it
can be weakened.

Design properties that support locality:

- The controller pins each pipeline thread to its assigned CPU **before**
  building the pipeline (`core_affinity::set_for_current`, logged as
  `core_affinity.set_failed` on failure). Under Linux's default first-touch
  policy, allocations made by that pipeline thread afterwards generally land
  on the CPU's local NUMA node.
- This receiver opens the perf ring only for `pipeline.core_id()` - i.e. its
  own pinned CPU - so ring reads happen from the thread pinned to the same CPU
  that the receiver is configured to drain.
- No receiver hot-path state crosses pipeline threads. Decoded records, Arrow
  builders, per-record payload buffers, decoded attribute strings, the metric
  set, and the admission state are all thread-local (`!Send`).

On a multi-socket host with `--num-cores 0`, the intended result is one
pipeline per CPU, each reading and processing its own ring on the local NUMA
node:

```text
  df_engine --num-cores 0          2-socket, 8-CPU host

  +----- NUMA node 0 -----+        +----- NUMA node 1 -----+
  |  CPU 0  1  2  3       |        |  CPU 4  5  6  7       |
  |  ring ring ring ring  |        |  ring ring ring ring  |
  |   ^    ^    ^    ^    |        |   ^    ^    ^    ^    |
  |   |    |    |    |    |        |   |    |    |    |    |
  |  pipe pipe pipe pipe  |        |  pipe pipe pipe pipe  |
  |  core core core core  |        |  core core core core  |
  |   0    1    2    3    |        |   4    5    6    7    |
  +-----------------------+        +-----------------------+

  Writes on CPUs on node 0 are read and processed by pipelines on node 0;
  writes on node 1 are read and processed by pipelines on node 1.
```

Each `user_events` write enters the ring of the CPU on which the writer
executes, and only a pipeline pinned to that CPU drains that ring. With one
pipeline per covered CPU and successful CPU affinity, collection does not
intentionally require cross-NUMA reads; each pipeline drains the ring for its
own CPU.

### What weakens locality

This is a locality-preserving design, not a hard guarantee. Locality can be
weakened if:

- CPU affinity cannot be set. This can happen when the requested CPU is outside
  the process cgroup/cpuset, the CPU is offline, or a container/seccomp policy
  blocks affinity changes. The controller logs `core_affinity.set_failed` and
  continues unpinned.
- The process is started with a NUMA policy that overrides first-touch
  placement, such as `numactl --interleave=all`.
- The receiver is changed to read perf rings for CPUs other than its assigned
  pipeline core. Today `session.rs` intentionally constructs
  `cpu_ids = vec![pipeline.core_id()]`.

## Platform Support

This receiver is **Linux-only**.

It does **not** work on macOS or Windows because `user_events` is a Linux
kernel tracing feature. Windows support would require a separate ETW receiver.

## Current Scope

Current implementation supports:

- one or more tracepoint subscriptions per receiver
- `tracefs`, which decodes standard Linux tracefs fields into typed log
  attributes
- `event_header`, which decodes EventHeader self-describing fields into typed
  log attributes

## Configuration

Configure one or more tracepoints with `subscriptions`. A receiver subscribes
to one tracepoint with a one-item list, or to multiple tracepoints with
multiple list entries.

### Subscriptions

```yaml
nodes:
  ingest:
    type: receiver:userevents
    config:
      subscriptions:
        - tracepoint: "user_events:myprovider_L2K1"
          format:
            type: tracefs
      session:
        per_cpu_buffer_size: 1048576  # bytes
        late_registration:
          enabled: true
          poll_interval_ms: 100
      drain:
        max_records_per_turn: 1024
        max_bytes_per_turn: 1048576
        max_drain_ns: 2ms  # duration string: 2ms, 500us, 1000000ns
      batching:
        max_size: 512
        max_duration: 50ms
      overflow:
        on_downstream_full: drop
```

Add another list entry when one receiver should listen to several tracepoints:

```yaml
nodes:
  ingest:
    type: receiver:userevents
    config:
      subscriptions:
        - tracepoint: "user_events:myprovider_L2K1"
          format:
            type: tracefs
        - tracepoint: "user_events:app_L2K1"
          format:
            type: event_header
      session:
        per_cpu_buffer_size: 1048576  # bytes
```

With `subscriptions`, each per-CPU receiver instance still opens one
one_collect/perf session for its assigned CPU only. The receiver registers all
configured tracepoints in that session, tags each sample with a subscription
index, and uses that index to select the configured decoder format. This keeps
the per-CPU locality model intact while allowing related tracepoints to share a
receiver.

The tradeoff is shared fate and shared budgets: all subscriptions in the
receiver share the same perf session, drain limits, batching policy, overflow
policy, and metrics. A high-volume tracepoint can consume the shared drain or
batch budget and affect quieter tracepoints. If tracepoints need independent
resource limits or failure isolation, configure separate receiver nodes instead.

`subscriptions` must contain at least one entry. `tracefs` is the default
`subscriptions[].format.type`.

`session.wakeup_watermark` exists as a reserved configuration field for future
one_collect wakeup support, but is currently ignored.

TODO: Wire `session.wakeup_watermark` into the perf ring setup once
`one_collect` exposes wakeup/readiness and watermark configuration for
tracepoint sessions.

### Configuration Reference

| Field | Default | Description |
| --- | --- | --- |
| `subscriptions` | none | Required non-empty list of tracepoints. Each entry must use `user_events:<event>`. |
| `subscriptions[].format.type` | `tracefs` | Decode format for one subscription. Supported values: `tracefs`, `event_header`. |
| `session.per_cpu_buffer_size` | `1048576` | Requested per-CPU perf ring size in bytes. Rounded by the underlying perf/ring setup. |
| `session.wakeup_watermark` | `262144` | Reserved for future one_collect wakeup support; currently ignored. |
| `session.late_registration.enabled` | `false` | When true, keep retrying if the tracepoint is not registered yet. |
| `session.late_registration.poll_interval_ms` | `1000` | Retry interval for late tracepoint registration. |
| `drain.max_records_per_turn` | `1024` | Maximum records popped from the receiver's pending queue per drain turn. |
| `drain.max_bytes_per_turn` | `1048576` | Maximum payload bytes popped per drain turn. |
| `drain.max_drain_ns` | `2ms` | Total drain-turn budget. Accepts duration strings such as `2ms`, `500us`, or `1000000ns`; must be greater than zero. |
| `batching.max_size` | `512` | Flush once this many logs are buffered in the current Arrow batch. |
| `batching.max_duration` | `50ms` | Flush interval for partially-filled batches. |
| `overflow.on_downstream_full` | `drop` | Drop the batch if downstream is full; blocking the perf drain loop is intentionally avoided. |

### Tracepoint Naming

Tracepoints must be configured with the `user_events:` group prefix. The receiver
rejects other groups because collection always uses the Linux `user_events`
tracefs group:

```text
user_events:<provider>_L<level>K<keyword>
```

EventHeader-style names such as `<provider>_L<level>K<keyword>` are accepted,
but the generic receiver does not interpret the level or keyword as OTLP
severity.

## Decode

### `tracefs`

Decodes standard Linux tracepoint fields using the tracefs `format` metadata
exposed for each registered `user_events` tracepoint. Common tracepoint fields
such as `common_pid` are skipped; producer-declared fields are emitted as typed
log attributes. After a successful decode, the receiver forwards the decoded
OTAP log record; it does not forward the original raw tracepoint sample bytes.
Unknown static fields may be preserved as per-field base64 string attributes.

### `event_header`

Decodes EventHeader self-describing fields into typed log attributes. Nested
EventHeader structs are flattened with dot-separated attribute names. If an
EventHeader payload cannot be decoded, the raw user payload is preserved in the
`linux.userevents.payload_base64` attribute.

## Output Shape

The receiver emits OTAP logs. Structural data is represented as flat log
attributes with source types preserved where possible (`Int`/`Bool`/`Double`/
`Str`). The typed `event_name` field is set to the configured tracepoint name,
and `time_unix_nano` uses the perf sample timestamp. Schema-specific promotion
to typed OTLP fields is intentionally left to processors. The original raw
tracepoint sample is not part of the normal output contract after structural
decode succeeds.

The receiver intentionally does **not** emit receiver-internal
transport/diagnostic fields such as tracepoint name, provider name,
EventHeader level/keyword, CPU, PID/TID, sample id, payload size, body
encoding, or decode mode. These describe the receiver itself rather than the
application payload; surfacing them as OTLP log attributes would pollute
downstream backends with receiver implementation details.

## Receiver Internals

The per-pipeline-thread pipeline inside the receiver has four stages, each
bounded by a specific config field. This diagram shows where the `drain.*`,
`batching.*`, `overflow.*`, and memory-pressure knobs take effect:

```text
  perf ring (per-CPU, kernel-owned)
         |
         |  parse_for_duration(max_drain_ns / 2)       // drain budget
         |  pop up to max_records_per_turn / max_bytes_per_turn
         v
  +---------------------+
  | drain (bounded)     |  drain.max_records_per_turn
  |                     |  drain.max_bytes_per_turn
  |                     |  drain.max_drain_ns   (split: parse / pop)
  +----------+----------+
             |
             v
  +---------------------+   tracefs fields -> typed attributes
  | decode              |   EventHeader fields -> typed attributes
  +----------+----------+
             |
             v
  +---------------------+
  | Arrow batch builder |  append one log per record
  |                     |  flush when len >= batching.max_size
  |                     |  flush on batching.max_duration tick
  +----------+----------+
             |
             |  flush_batch -> effect_handler.try_send_message(...)
             v
  +---------------------+      ok    -> forwarded_samples += n, flushed_batches++
  | downstream channel  |      full  -> dropped_downstream_full += n
  |                     |               (overflow.on_downstream_full = drop)
  +---------------------+      memory pressure (should_shed_ingress):
                                     records    -> dropped_memory_pressure++
                                     buffered batch on ctrl event -> drop_batch
```

Notes:

- The drain budget is split in half between parsing the kernel ring and popping
  the receiver's pending queue; `drain.max_drain_ns` must be non-zero or config
  validation rejects it.
- Memory pressure takes effect at two points: per-record during the pop loop
  (records are counted toward `dropped_memory_pressure` instead of being
  appended to the batch) and on ctrl events where any already-buffered batch is
  dropped rather than flushed.
- `overflow.on_downstream_full = drop` is currently the only mode; the perf
  drain loop is never blocked on downstream.

## Runtime Behavior

### Backpressure

`user_events` perf rings cannot be backpressured like a socket. For that
reason, the receiver does not block the perf drain loop when downstream is
full; it drops already-drained batches and reports them as
`dropped_downstream_full`. Separately, if the kernel/perf ring overruns before
the receiver drains it, that loss is reported by the perf path and counted as
`lost_perf_samples`.

TODO: Plumb corrupt perf event and corrupt perf buffer counters once
`one_collect` exposes them.

### Memory Pressure

When process-wide memory pressure indicates ingress shedding, the receiver
drops buffered batches rather than blocking on downstream flush.

### Late Registration

If `late_registration.enabled` is true, the receiver will keep retrying
tracepoint attachment until the producer has registered the tracepoint.

If late registration is disabled and any configured tracepoint is missing or
invalid, session startup fails and the receiver does not start. With multiple
subscriptions, one missing tracepoint fails the whole session.

TODO: Add mid-stream session recovery once the underlying collection layer
exposes enough error classification to distinguish recoverable session loss from
fatal data, permission, or configuration errors.

### Shutdown and Drain

On `DrainIngress`, the receiver performs one final bounded perf-ring drain
before flushing or dropping the in-memory Arrow batch according to memory
pressure. On immediate shutdown, the receiver reports terminal state without an
extra drain/flush; use graceful drain when minimizing buffered data loss matters.

## Metrics

The receiver reports these counters under `userevents.receiver.metrics`:

| Metric | Meaning |
| --- | --- |
| `received_samples` | Perf samples drained from the kernel/perf path. |
| `forwarded_samples` | Log records successfully forwarded downstream. |
| `dropped_downstream_full` | Batches dropped because the downstream channel was full. |
| `dropped_memory_pressure` | Records or batches dropped because process memory pressure requested ingress shedding. |
| `dropped_no_subscription` | Samples that did not map to a configured subscription index. This should normally stay zero. |
| `lost_perf_samples` | Lost sample count reported by the perf ring. |
| `late_registration_retries` | Late-registration retry attempts while waiting for tracepoints. |
| `sessions_started` | Receiver perf sessions successfully opened. |
| `flushed_batches` | Arrow log batches flushed downstream. |

TODO: Add metrics for corrupt perf events and corrupt perf buffers once the
underlying collection layer exposes those counters.

## Linux Requirements

This receiver requires all of the following on the host:

- Linux kernel **6.4 or later**, built with `CONFIG_USER_EVENTS=y`
- tracefs available, typically under `/sys/kernel/tracing`
- permission to read tracefs metadata
- permission to use `perf_event_open` for the configured tracepoints

`user_events` was merged in Linux 6.4; earlier kernels do not expose
`user_events_data` / `user_events_status` in tracefs. Distro support also
depends on whether `CONFIG_USER_EVENTS` is enabled in the shipped kernel.
Distributions that ship it enabled by default:

- **Ubuntu 23.10** and later
- **Azure Linux 3.0** and later

Other distributions may work on any 6.4+ kernel built with
`CONFIG_USER_EVENTS=y`, but this is not exhaustively verified.

The exact permission model depends on the host kernel and security settings.

## Docker

Sometimes, but not automatically.

It can work in Docker **only if the host kernel supports `user_events`** and
the container is given access to the host tracing and perf facilities.

Important implications:

- containers share the **host kernel**
- Docker on native Linux may work
- Docker Desktop on macOS or Windows does **not** make this a macOS or Windows
  feature; it only works if the Linux VM kernel behind Docker Desktop supports
  `user_events` and the necessary interfaces are exposed into the container

In practice, for Docker you usually need some combination of:

- access to `/sys/kernel/tracing`
- relaxed `perf_event_open` restrictions or appropriate privileges
- permission to write to `user_events_data` if the producer runs in-container

For reliable testing, prefer:

- native Linux first
- then privileged or carefully configured Linux containers
- not macOS as the host runtime

## Testing

Recommended test layers:

- unit tests for tracefs structural decoding and EventHeader payload handling
- Linux-only receiver integration tests using a real kernel tracepoint
- pipeline-level schema mapping tests in the processor that owns that schema
