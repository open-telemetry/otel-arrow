<!-- markdownlint-disable MD013 -->

# Linux user_events Receiver

**URN:** `urn:otel:receiver:user_events`

This receiver ingests Linux
[`user_events`](https://docs.kernel.org/trace/user_events.html) tracepoints
through `perf_event_open` and converts them into OTAP logs for downstream
processing.

> **Note:** This receiver is vendor-neutral. It performs Linux `user_events`
> collection and structural decoding only; schema-specific mappings should be
> modeled outside the receiver.

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
is visible to receiver instances attached to CPU `X`. Within one df_engine
instance, no other receiver instance sees it, and there is no cross-CPU
aggregation layer in this receiver.

```text
  user_events write on CPU K
             |
             v
       CPU K perf ring
             |
             v
  receiver pipeline pinned to CPU K

  No other receiver pipeline in this df_engine instance reads CPU K's ring.
```

For a 4-core host, coverage looks like this:

```text
  CPU:        0      1      2      3
              |      |      |      |
  ring:      R0     R1     R2     R3
              ^      ^      ^      ^
              |      |      |      |
  pipeline:  P0     P1     P2     P3

  P0 reads only R0. P1 reads only R1. P2 reads only R2. P3 reads only R3.
```

If a CPU has no corresponding pipeline, writes on that CPU are not collected by
this receiver deployment. They are not seen and then discarded; no receiver is
attached to that CPU's ring. The engine does not emit a metric for these
missing records because they are never observed by the receiver; they are lost
silently from the engine's point of view. A producer may still use
`user_events` listener availability to detect whether it has an active reader.

| Configuration                    | Writer pinned? | Result                                                     |
|----------------------------------|----------------|------------------------------------------------------------|
| `--num-cores 1`                  | No             | Incomplete unless the writer stays on CPU 0                |
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
kernel tracing feature.

## Current Scope

Current implementation supports:

- one or more tracepoint subscriptions per receiver
- `tracefs`, which decodes standard Linux tracefs fields into typed log
  attributes
- optional `event_header` decoding when the `user_events-eventheader` feature is
  enabled

## Configuration

Configure one or more tracepoints with `subscriptions`. A receiver subscribes
to one tracepoint with a one-item list, or to multiple tracepoints with
multiple list entries.

### Subscriptions

```yaml
nodes:
  ingest:
    type: urn:otel:receiver:user_events
    config:
      subscriptions:
        - tracepoint: "myprovider_L2K1"
          format:
            type: tracefs
      session:
        per_cpu_buffer_size: 1048576  # bytes
        late_registration_poll_interval: 100ms
      drain:
        max_records_per_turn: 1024
        max_bytes_per_turn: 1048576
        max_drain_ns: 2ms  # duration string: 2ms, 500us, 1000000ns
      batching:
        max_size: 512
        max_duration: 50ms
```

Add another list entry when one receiver should listen to several tracepoints:

```yaml
nodes:
  ingest:
    type: urn:otel:receiver:user_events
    config:
      subscriptions:
        - tracepoint: "myprovider_L2K1"
          format:
            type: tracefs
        - tracepoint: "app_L2K1"
          format:
            type: tracefs
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
receiver share the same perf session, drain limits, batching policy, downstream
backpressure boundary, and metrics. A high-volume tracepoint can consume the
shared drain or batch budget and affect quieter tracepoints. If tracepoints need
independent resource limits or failure isolation, configure separate receiver
nodes instead.

The receiver bounds the in-process pending queue between one_collect perf
callbacks and the drain loop with `session.max_pending_events` and
`session.max_pending_bytes`. When parse rate exceeds drain rate and either cap
is reached, new events are dropped before their payload is copied into user
space and counted as `dropped_pending_overflow`. This cap covers only the
adapter pending queue; kernel perf ring memory is governed by
`session.per_cpu_buffer_size`, Arrow batch memory by `batching.*`, and
downstream channel pressure by the engine's bounded local channel.

Late registration is also currently all-or-nothing for multiple subscriptions:
if any configured tracepoint is missing, the receiver retries opening the
entire session and does not collect already-registered tracepoints yet.
Supporting partial startup and later subscription registration is a future
improvement.

`subscriptions` must contain at least one entry. `tracefs` is the default
`subscriptions[].format.type`. `event_header` requires the
`user_events-eventheader` feature.

`session.wakeup_watermark` exists as a reserved configuration field for future
one_collect wakeup support, but is currently ignored.

`session.per_cpu_buffer_size` is a requested perf-ring size, not an exact byte
count. The one_collect adapter rounds it up to at least one system page and
then to the next power of two before converting it to a perf page count.

TODO: Wire `session.wakeup_watermark` into the perf ring setup once
`one_collect` exposes wakeup/readiness and watermark configuration for
tracepoint sessions.

### Configuration Reference

| Field | Default | Description |
| --- | --- | --- |
| `subscriptions` | none | Required non-empty list of `user_events` tracepoints. Each entry may be either `<event>` or `user_events:<event>`. |
| `subscriptions[].format.type` | `tracefs` | Decode format for one subscription. `tracefs` is always supported; `event_header` requires the `user_events-eventheader` feature. |
| `session.per_cpu_buffer_size` | `1048576` | Requested per-CPU perf ring size in bytes. Rounded up to at least one page and then to the next power of two. |
| `session.wakeup_watermark` | `262144` | Reserved for future one_collect wakeup support; currently ignored. |
| `session.max_pending_events` | `4096` | Maximum parsed events buffered between one_collect callbacks and the receiver drain loop. New events are dropped when this cap is reached. |
| `session.max_pending_bytes` | `16777216` | Maximum raw event payload bytes buffered between one_collect callbacks and the receiver drain loop. New events are dropped when this cap would be exceeded. |
| `session.late_registration_poll_interval` | none | Optional retry interval for late tracepoint registration. When absent, missing tracepoints fail startup immediately. |
| `drain.max_records_per_turn` | `1024` | Maximum records popped from the receiver's pending queue per drain turn. |
| `drain.max_bytes_per_turn` | `1048576` | Maximum payload bytes popped per drain turn. |
| `drain.max_drain_ns` | `2ms` | Total drain-turn budget. Accepts duration strings such as `2ms`, `500us`, or `1000000ns`; must be greater than zero. |
| `batching.max_size` | `512` | Flush once this many logs are buffered in the current Arrow batch. |
| `batching.max_duration` | `50ms` | Flush interval for partially-filled batches. |

TODO: Add human-readable byte sizes such as `10KB`/`16MiB` if the dataflow
configuration layer standardizes byte-size parsing.

Batching avoids sending one Arrow payload per perf sample. Each flush has fixed
Arrow/message overhead, so the receiver groups records by count or time to keep
low-volume traces timely while making higher-volume traces more efficient.

### Tracepoint Naming

Tracepoints may be configured as bare `user_events` event names because this
receiver only collects from the Linux `user_events` tracefs group:

```text
<provider>_L<level>K<keyword>
```

The explicit `user_events:<event>` form is also accepted and normalized
internally. Other tracefs groups are rejected.

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
`linux.user_events.payload_base64` attribute.

This decoder is optional and requires the `user_events-eventheader` feature.

Only scalar EventHeader values that map to the receiver's current attribute
types are surfaced: strings, signed integers, booleans, and floating point
values. EventHeader arrays, binary blobs, and other non-scalar encodings are
not emitted as attributes yet.

TODO: Add support for non-scalar EventHeader values once there is a stable
mapping to OTLP log attributes.

## Producers

Applications can emit Linux `user_events` directly or through OpenTelemetry
libraries that target `user_events`:

- [LinuxTracepoints-Rust](https://github.com/microsoft/LinuxTracepoints-Rust)
  provides non-OpenTelemetry Rust APIs for Linux Tracepoints and EventHeader
  events.
- [opentelemetry-user-events-logs](https://github.com/open-telemetry/opentelemetry-rust-contrib/tree/main/opentelemetry-user-events-logs)
  provides an OpenTelemetry Rust log exporter that writes to Linux `user_events`.

## Output Shape

The receiver emits OTAP logs. Structural data is represented as flat log
attributes with source types preserved where possible (`Int`/`Bool`/`Double`/
`Str`). The typed `event_name` field is set to the configured tracepoint name,
and `time_unix_nano` and `observed_time_unix_nano` use the perf sample
timestamp. Processors may later replace `time_unix_nano` with a producer-provided
event time, for example Common Schema `PartA.time`, while preserving the perf
sample timestamp as observed time. The receiver also emits
`linux.user_events.process.pid` and `linux.user_events.thread.id` from perf sample
metadata when available, because multiple processes or threads can emit the same
tracepoint. Schema-specific promotion to typed OTLP fields is intentionally left
to processors. The original raw tracepoint sample is not part of the normal
output contract after structural decode succeeds.

The receiver intentionally does **not** emit receiver-internal
transport/diagnostic fields such as tracepoint name, provider name,
EventHeader level/keyword, CPU, sample id, payload size, body encoding, or
decode mode. These describe the receiver itself rather than the application
payload; surfacing them as OTLP log attributes would pollute downstream
backends with receiver implementation details.

## Receiver Internals

The per-pipeline-thread pipeline inside the receiver has four stages, each
bounded by a specific config field. This diagram shows where the `drain.*`,
`batching.*`, downstream backpressure, and memory-pressure knobs take effect:

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
             |  flush_batch -> effect_handler.send_message(...).await
             v
  +---------------------+      ok      -> forwarded_samples += n, flushed_batches++
  | downstream channel  |      full    -> receiver task waits for capacity
  |                     |      waiting -> downstream_send_blocked_ns += elapsed
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
- When the downstream channel is full, the receiver awaits channel capacity.
  While it is waiting, the perf ring is the intentional overflow point.

## Runtime Behavior

### Backpressure

`user_events` perf rings cannot be backpressured like a socket. For that
reason, the receiver applies backpressure at the engine channel boundary
instead. When the downstream channel is full, the receiver awaits the async
send, parks the receiver task, and stops draining new perf samples until
capacity is available. Under sustained downstream saturation, the kernel perf
ring is deliberately the overflow point; perf-ring loss is reported by the perf
path and counted as `lost_perf_samples`. Time spent waiting for downstream
capacity is counted as `downstream_send_blocked_ns`.

TODO: Plumb corrupt perf event and corrupt perf buffer counters once
`one_collect` exposes them.

### Memory Pressure

When process-wide memory pressure indicates ingress shedding, the receiver
drops buffered batches rather than blocking on downstream flush.

### Late Registration

If `late_registration_poll_interval` is set, the receiver will keep retrying
tracepoint attachment until the producer has registered the tracepoint.

TODO: For EventHeader producers, investigate whether preregistered tracepoint
definitions can avoid polling once the Rust collection path exposes support
equivalent to the C++ `tracepoint_control::TracepointCache::PreregisterTracepointDefinition`
API.

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

The receiver reports these counters under `receiver.user_events`:

| Metric | Meaning |
| --- | --- |
| `received_samples` | Perf samples drained from the kernel/perf path. |
| `forwarded_samples` | Log records successfully forwarded downstream. |
| `downstream_send_blocked_ns` | Time spent waiting for downstream channel capacity. |
| `dropped_memory_pressure` | Records or batches dropped because process memory pressure requested ingress shedding. |
| `dropped_no_subscription` | Samples that did not map to a configured subscription index. This should normally stay zero. |
| `dropped_pending_overflow` | Samples dropped before allocation because the adapter pending queue reached its configured event or byte cap. |
| `dropped_send_error` | Records dropped because a downstream send failed. |
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

- unit tests for tracefs structural decoding, plus EventHeader payload handling
  when `user_events-eventheader` is enabled
- Linux-only receiver integration tests using a real kernel tracepoint
- pipeline-level schema mapping tests in the processor that owns that schema

### Manual tracefs E2E

The tracefs debug pipeline exercises a real Linux `user_events` producer, this
receiver, the debug processor, and the noop exporter:

```bash
cargo build --features user_events-receiver
cargo build -p otap-df-contrib-nodes \
  --features user_events-receiver \
  --example user_events_tracefs_producer

sudo ./target/debug/df_engine \
  --config configs/user-events-tracefs-debug.yaml \
  --num-cores 1

sudo taskset -c 0 \
  ./target/debug/examples/user_events_tracefs_producer \
  otap_df_tracefs_demo 3 100

sudo cat /tmp/user_events_tracefs_debug.log
```

Expected debug output includes `EventName: user_events:otap_df_tracefs_demo`,
`ci_answer: 0`, `ci_answer: 1`, `ci_answer: 2`, and
`ci_message: hello-from-ci`.

### Manual EventHeader E2E

The EventHeader debug pipeline uses the optional `user_events-eventheader`
feature and validates EventHeader payload decoding through the same receiver
and debug processor path:

```bash
cargo build --features user_events-eventheader
cargo build -p otap-df-contrib-nodes \
  --features user_events-eventheader \
  --example user_events_eventheader_producer

sudo ./target/debug/df_engine \
  --config configs/user-events-eventheader-debug.yaml \
  --num-cores 1

sudo taskset -c 0 \
  ./target/debug/examples/user_events_eventheader_producer \
  otap_df_eventheader_demo 3 100

sudo cat /tmp/user_events_eventheader_debug.log
```

Expected debug output includes
`EventName: user_events:otap_df_eventheader_demo_L4K1`,
`ci_answer: 0`, `ci_answer: 1`, `ci_answer: 2`, and
`ci_message: hello-from-ci`.

The `sudo` usage is intentional for hosts that restrict tracefs writes or
`perf_event_open`. If the current user already has those permissions, sudo is
not required. `taskset -c 0` keeps the producer on the CPU read by the
single-core receiver process used in these examples.
