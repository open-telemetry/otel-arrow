# Linux Userevents Receiver

**URN:** `urn:otel:receiver:userevents`

This receiver ingests Linux
[`user_events`](https://docs.kernel.org/trace/user_events.html) tracepoints
through `perf_event_open` and converts them into OTAP logs for downstream
processing.

It follows the OTAP Dataflow thread-per-core model:

- one receiver instance per pipeline thread
- one perf session per assigned CPU
- one bounded drain loop per receiver
- no shared hot-path state across pipeline threads

## Architecture

`user_events` writes enter Linux tracing as user-space tracepoint events. The
kernel stores trace records in per-CPU ring buffers; each receiver instance
drains the ring for the CPU/pipeline core it owns, decodes the EventHeader
payload, and emits OTAP logs into that same pipeline.

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
attached to that CPU's ring.

| Configuration                    | Writer pinned? | Result                                                     |
|----------------------------------|----------------|------------------------------------------------------------|
| `--num-cores 1`                  | No             | Racy; writes on CPUs 1..N not collected by this receiver   |
| `--num-cores 1`                  | `taskset -c 0` | Reliable single-core capture                               |
| `--core-id-range 0-3`            | No             | Writes on CPUs 4..N not collected by this receiver         |
| `--core-id-range 0-3`            | Pinned to 0..3 | Reliable within the range                                  |
| `--num-cores 0` (all host CPUs)  | No             | Complete host-wide coverage                                |

For production, prefer full coverage (`--num-cores 0`) unless you intentionally
want to sample only a subset of CPUs; for development/testing, a pinned writer
on a single-core engine keeps the moving parts minimal.

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
  builders, per-record payload buffers, PartC attribute strings, the metric
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

- **`core_affinity::set_for_current` fails.** Some restrictive cgroups or
  seccomp sandboxes prevent setting affinity; the controller logs
  `core_affinity.set_failed` and continues without pinning. The first-touch
  guarantee is then lost.
- **Process-wide NUMA policy that overrides first-touch.** For example running
  the engine under `numactl --interleave=all`, or an explicit
  `set_mempolicy(MPOL_INTERLEAVE)`, forces pages to round-robin across nodes.
- **Allocator / arena state that predates pinning.** The global allocator may
  have arena pages that were seeded before the pipeline thread was pinned;
  those pages can physically reside on a different NUMA node, and subsequent
  allocations that reuse them inherit that placement.
- **Fewer pipelines than CPUs on a multi-socket host.** The covered CPUs still
  get NUMA-local handling, but producers on uncovered CPUs write to rings on
  *their* NUMA node that this deployment does not collect.
- **Future changes that open rings for CPUs other than the pipeline's core.**
  Today `session.rs` always constructs `cpu_ids = vec![pipeline.core_id()]`;
  opening rings for other CPUs would introduce remote memory reads from the
  pinned pipeline thread. The single-CPU invariant lives in `session.rs`.

## Platform Support

This receiver is **Linux-only**.

It does **not** work on macOS because `user_events` is a Linux kernel feature.

## Current Scope

Current implementation supports:

- single-tracepoint and multi-tracepoint configuration
- `common_schema_otel_logs` as the only supported decode format

## Configuration

You can configure the receiver in one of two ways.

### Single Tracepoint Shorthand

Use this when one receiver should listen to one tracepoint.

```yaml
nodes:
  ingest:
    type: receiver:userevents
    config:
      tracepoint: "user_events:myprovider_L2K1"
      format:
        type: common_schema_otel_logs
      session:
        per_cpu_buffer_size: 1048576
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

### Multiple Tracepoints

Use this when one receiver should listen to several tracepoints.

```yaml
nodes:
  ingest:
    type: receiver:userevents
    config:
      subscriptions:
        - tracepoint: "user_events:myprovider_L2K1"
          format:
            type: common_schema_otel_logs
        - tracepoint: "user_events:app_L2K1"
          format:
            type: common_schema_otel_logs
      session:
        per_cpu_buffer_size: 1048576
```

Exactly one of `tracepoint` or `subscriptions` must be configured.
`common_schema_otel_logs` is currently the only supported `format.type`.

`session.wakeup_watermark` exists as a reserved configuration field for future
one_collect wakeup support, but is currently ignored.

### Configuration Reference

| Field | Default | Description |
| --- | --- | --- |
| `tracepoint` | none | Single tracepoint shorthand. Must use `user_events:<event>`. Mutually exclusive with `subscriptions`. |
| `subscriptions` | none | List of tracepoints. Each entry must use `user_events:<event>` and `format.type: common_schema_otel_logs`. |
| `format.type` | `common_schema_otel_logs` | Only supported decode format. |
| `session.per_cpu_buffer_size` | `1048576` | Requested per-CPU perf ring size in bytes. Rounded by the underlying perf/ring setup. |
| `session.wakeup_watermark` | `0` | Reserved for future one_collect wakeup support; currently ignored. |
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

The `_L<level>K<keyword>` suffix follows EventHeader tracepoint naming. The
receiver uses `level` only as a fallback when PartB does not provide
`severityNumber`:

| EventHeader level | Fallback severity |
| --- | --- |
| `1` or `2` | `ERROR` / `17` |
| `3` | `WARN` / `13` |
| `4` | `INFO` / `9` |
| `5` | `DEBUG` / `5` |

If the suffix is absent or malformed, Common Schema payloads with no
`severityNumber` are emitted without a fallback severity.

## Decode

### `common_schema_otel_logs`

Intended for payloads produced by
[`opentelemetry-user-events-logs`](https://github.com/open-telemetry/opentelemetry-rust-contrib/tree/main/opentelemetry-user-events-logs).

Current behavior:

- decodes EventHeader-encoded Common Schema log payloads
- promotes `event_name`, `severityNumber`, `severityText`, `body`, and
  `eventId` from Common Schema PartB
- maps PartA fields including timestamp and trace/span context into typed
  OTLP log fields when present
- flattens eligible PartC scalar attributes into emitted log attributes
- falls back to preserving the payload as base64-encoded data when Common
  Schema decoding fails

The main field mapping is:

| Source field | OTAP / OTLP output |
| --- | --- |
| EventHeader name / PartA.name | fallback for typed `event_name` |
| PartA.time | `time_unix_nano` |
| PartA.ext_dt_traceId | typed `trace_id` when valid hex; otherwise `trace.id` attribute |
| PartA.ext_dt_spanId | typed `span_id` when valid hex; otherwise `span.id` attribute |
| PartA.ext_dt_traceFlags | log `flags` |
| PartB.name | preferred typed `event_name` |
| PartB.severityNumber | `severity_number` |
| PartB.severityText | `severity_text` |
| PartB.body | log `body` |
| PartB.eventId | typed `eventId` attribute |
| PartC scalar fields | flat log attributes with original field names and basic value types |

Decode failure means the payload could not be interpreted as the supported
EventHeader/Common Schema log shape. Examples include malformed EventHeader
bytes, `__csver__` missing or not first, `__csver__ != 0x400`, missing PartB,
PartB `_typeName != "Log"`, duplicate PartA/PartB/PartC structs, unknown PartB
fields, or invalid nesting. On failure, the receiver emits the raw payload as a
base64 body and increments `cs_decode_fallbacks`.

## Output Shape

The receiver emits OTAP logs. Payload data is represented as typed OTLP log
fields and flat application attributes; receiver transport/debug metadata is
not emitted as log attributes.

The only PartB field emitted as a log attribute is:

- `eventId` (typed Int, when the Common Schema payload carries PartB.eventId)

Typed OTLP log fields (not attributes) also carry:

- `body`
- `severity_number` / `severity_text`
- `event_name` (prefers PartB.name, falls back to EH.Name / PartA.name)
- `time_unix_nano` (from PartA.time when present, else the perf sample timestamp)
- `trace_id` / `span_id` / `flags` (from PartA.ext_dt_*)

PartC fields are emitted as flat attributes using their original names
(e.g. `user_name`, `user_email`) with their source types preserved
(`Int`/`Bool`/`Double`/`Str`).

The receiver intentionally does **not** emit receiver-internal
transport/diagnostic fields such as tracepoint name, provider name,
EventHeader level/keyword, CPU, PID/TID, sample id, payload size, body
encoding, or decode mode. These describe the receiver itself rather than the
application payload; surfacing them as OTLP log attributes would pollute
downstream backends (e.g. Geneva turns each attribute into a dynamic backend
column).

Similarly, no `cs.*` inspection attributes are emitted (e.g.
`cs.__csver__`, `cs.part_b._typeName`, `cs.part_b.name`). These would
otherwise surface as `cs_*` backend columns in Geneva; PartB.name is
already represented by the typed `event_name` column. The base64
fallback path remains available internally when Common Schema decoding
fails, but no `linux.userevents.body_encoding` marker is exposed to
consumers.

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
  +---------------------+   CS EventHeader decode succeeds -> typed logs
  | decode              |   CS EventHeader decode fails    -> base64 body
  |                     |                                     + cs_decode_fallbacks++
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
reason, the receiver defaults to dropping when downstream is full instead of
blocking the perf drain loop.

### Memory Pressure

When process-wide memory pressure indicates ingress shedding, the receiver
drops buffered batches rather than blocking on downstream flush.

### Late Registration

If `late_registration.enabled` is true, the receiver will keep retrying
tracepoint attachment until the producer has registered the tracepoint.

If late registration is disabled and any configured tracepoint is missing or
invalid, session startup fails and the receiver does not start. With multiple
subscriptions, one missing tracepoint fails the whole session.

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
| `cs_decode_fallbacks` | Samples that failed Common Schema decoding and were emitted with a base64 body fallback. |
| `lost_perf_samples` | Lost sample count reported by the perf ring. |
| `late_registration_retries` | Late-registration retry attempts while waiting for tracepoints. |
| `sessions_started` | Receiver perf sessions successfully opened. |
| `flushed_batches` | Arrow log batches flushed downstream. |

## Linux Requirements

This receiver requires all of the following on the host:

- Linux kernel with `CONFIG_USER_EVENTS`
- tracefs available, typically under `/sys/kernel/tracing`
- permission to read tracefs metadata
- permission to use `perf_event_open` for the configured tracepoints

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

- unit tests for tracepoint-format parsing and payload normalization
- Linux-only receiver integration tests using a real kernel tracepoint
- exporter-to-receiver end-to-end tests from
  `opentelemetry-user-events-logs` into this receiver

An ignored Linux-only end-to-end smoke test exists under
`crates/contrib-nodes/tests/userevents_exporter_receiver_e2e.rs`.
