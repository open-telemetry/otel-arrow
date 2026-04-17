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

## Platform Support

This receiver is **Linux-only**.

It does **not** work on macOS because `user_events` is a Linux kernel feature.

## Current Scope

Current implementation supports:

- single-tracepoint configuration
- multi-tracepoint configuration
- configurable decode modes
- raw payload preservation
- partial Common Schema-aware promotion

Current implementation does **not** yet provide full binary EventHeader field
decoding.

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
        wakeup_watermark: 262144
        late_registration:
          enabled: true
          poll_interval_ms: 100
      drain:
        max_records_per_turn: 1024
        max_bytes_per_turn: 1048576
        max_drain_ns: 2ms
      batching:
        max_size: 512
        max_duration: 50ms
      overflow:
        on_downstream_full: drop
```

### Multiple Tracepoints

Use this when one receiver should listen to several tracepoints with
per-tracepoint decode settings.

```yaml
nodes:
  ingest:
    type: receiver:userevents
    config:
      subscriptions:
        - tracepoint: "user_events:myprovider_L2K1"
          format:
            type: common_schema_otel_logs
        - tracepoint: "user_events:app_L5K1"
          format:
            type: eventheader_flat
            flatten_prefix: "userevents.field"
        - tracepoint: "user_events:legacy_L4K1"
          format:
            type: raw
      session:
        per_cpu_buffer_size: 1048576
        wakeup_watermark: 262144
```

Exactly one of `tracepoint` or `subscriptions` must be configured.

## Decode Modes

### `raw`

The receiver does not interpret the payload. It stores the payload as a
base64-encoded log body and attaches metadata as attributes.

This is the safest mode for unknown producers.

### `common_schema_otel_logs`

Intended for payloads produced by
[`opentelemetry-user-events-logs`](https://github.com/open-telemetry/opentelemetry-rust-contrib/tree/main/opentelemetry-user-events-logs).

Current behavior:

- decodes EventHeader-encoded Common Schema log payloads
- promotes `event_name`, `severityNumber`, `severityText`, `body`, and
  `eventId` from Common Schema PartB
- maps PartA fields including timestamp, trace/span context, and service
  metadata when present
- flattens eligible PartC scalar attributes into emitted log attributes
- falls back to preserving the payload as base64-encoded data when Common
  Schema decoding fails

### `eventheader_flat`

Intended for generic EventHeader-based producers.

Current behavior decodes EventHeader payload bytes to JSON and flattens the
result into attributes using the configured prefix. Richer typed promotions and
semantic mapping are still future work.

### `custom_eventheader`

Intended for producer-specific mappings where configured fields should
eventually be promoted into OTel log fields.

When EventHeader JSON decoding succeeds, the receiver applies the configured
`body_field`, `event_name_field`, severity field mappings, and
`attributes_from` extraction to the emitted log record. Unsupported mappings
and richer semantic promotions remain future work.

## Output Shape

The receiver emits OTAP logs.

Current output always includes these attributes:

- `linux.userevents.tracepoint`
- `linux.userevents.cpu`
- `process.pid`
- `thread.id`
- `linux.userevents.sample_id`
- `linux.userevents.payload_size`
- `linux.userevents.body_encoding`
- `linux.userevents.decode.mode`

Depending on configured format, it may also include:

- `event.provider`
- `event.name`
- `eventheader.level`
- `eventheader.keyword`
- `cs.__csver__`
- `cs.part_b._typeName`
- `cs.part_b.name`

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
