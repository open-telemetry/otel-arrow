# Journald Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:journald` (`urn:otel:receiver:journald`)
- Feature gate: Default
- Stability: Experimental

## Overview

Linux-only receiver for local `systemd-journald` entries backed by the
`sd-journal` API. It reads structured journal records through runtime-loaded
`libsystemd.so.0`, uses journald source selection such as units, identifiers,
and priorities, and emits OTAP log records for downstream processing.

The receiver does not exec `journalctl` and does not read `.journal` files
directly. It uses journald cursors for progress tracking and advances the
durable checkpoint only after downstream Ack.

## Getting Started

Start at the end of the default system journal:

```yaml
groups:
  host:
    pipelines:
      collect:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          journald:
            type: receiver:journald
            config:
              source_id: system
              start_at: end
          publish:
            type: exporter:topic
            config:
              topic: journald_logs
        connections:
          - from: journald
            to: publish
```

## Configuration

```yaml
type: receiver:journald
config:
  # Stable source identifier used in telemetry and checkpoint paths
  # (default: system). Only ASCII alphanumerics, "_", "-", and "." are allowed.
  source_id: system

  # Journal source. Named namespaces are reserved and rejected in v1.
  journal:
    root_path: /
    # namespace: default

  # Exact source filters. Empty entries are rejected and duplicates are removed.
  units: []
  identifiers: []

  # Priority filters. Use either priorities or max_priority, not both.
  priorities: [0, 1, 2, 3, 4, 5, 6, 7]
  # max_priority: warning

  # Where to start when no checkpoint exists: "end" or "beginning"
  # (default: end).
  start_at: end

  batch:
    # Maximum records per emitted batch (default: 1024).
    max_records: 1024
    # Maximum time to hold a partial batch before flushing (default: 200ms).
    max_flush_period: 200ms

  extraction:
    # Maximum copied bytes per journal entry (default: 1MiB).
    max_entry_bytes: 1MiB
    # Maximum copied bytes per field value (default: 256KiB).
    max_field_bytes: 256KiB
    # Maximum copied fields per journal entry (default: 256).
    max_fields_per_entry: 256
    # Behavior for oversized fields: "drop_and_count" (default).
    large_field_policy: drop_and_count

  checkpoint:
    # Root directory for cursor checkpoint files (default:
    # ${engine.state_dir}/journald).
    directory: "${engine.state_dir}/journald"
    # Maximum unacknowledged batches. Must be 1 in v1.
    max_in_flight_batches: 1
    # Behavior on downstream NACK: "rewind" or "fail" (default: rewind).
    on_nack: rewind
    # Maximum consecutive checkpoint commit failures (default: 5).
    max_consecutive_failures: 5

  # sd_journal_wait timeout (default: 1s, max: 5s).
  wait_timeout: 1s

  # Drain deadline budget (default: 5s, must be greater than wait_timeout).
  drain_timeout: 5s
```

Read a host journal mounted into a container and keep only warning-or-higher
records from selected units:

```yaml
type: receiver:journald
config:
  journal:
    root_path: /host
  units: ["ssh.service", "systemd-journald.service"]
  max_priority: warning
  batch:
    max_records: 512
    max_flush_period: 500ms
  checkpoint:
    directory: "${engine.state_dir}/journald"
    on_nack: rewind
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.journald`

| Metric | Unit | Description |
| --- | --- | --- |
| `receiver.journald.starts` | `{start}` | Number of times the receiver was started. |
| `receiver.journald.drains` | `{drain}` | Number of clean drain transitions. |
| `receiver.journald.shutdowns` | `{shutdown}` | Number of clean shutdown transitions. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `journald_receiver.start` | `info` | Receiver started for the configured `source_id` and journal root path. |
| `journald_receiver.drain_ingress` | `info` | Receiver ingress drain started for the configured `source_id`. |
| `journald_receiver.shutdown` | `info` | Receiver shutdown completed for the configured `source_id`. |

## Limits

- Linux only.
- Requires `libsystemd.so.0` and permission to read the selected journal.
- Must run in a one-core source pipeline. Use `receiver:journald` followed by a
  topic exporter to fan out to multicore downstream processing.
- Named journal namespaces are not supported in v1.
- Kernel ring-buffer (`dmesg`) ingestion is not supported by this receiver.
- `checkpoint.max_in_flight_batches` must be `1` in v1.
- `wait_timeout` must be no more than `5s`, and `drain_timeout` must be greater
  than `wait_timeout`.
- Only one receiver in a process can target the same concrete journal source.
- Duplicate journald fields are emitted as repeated same-key attributes in the
  first implementation. Array coalescing is planned as a follow-up.
- Cross-process duplicate readers are not prevented in v1.
- NUMA pinning and placement are future work.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Journald receiver design](../../../../../docs/journald-receiver.md)
- [Core node catalog](../../../README.md)
