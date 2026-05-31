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

## Configuration Options

| Field | Type | Default | Description |
| ----- | ---- | ------- | ----------- |
| `source_id` | string | `system` | Stable source identifier used for checkpoint paths and telemetry labels. |
| `journal.root_path` | path | `/` | Local or mounted host root used for `sd-journal` access. |
| `journal.namespace` | string | unset | Named namespaces are rejected in v1. |
| `units` | list | `[]` | Exact `_SYSTEMD_UNIT` matches. |
| `identifiers` | list | `[]` | Exact `SYSLOG_IDENTIFIER` matches. |
| `priorities` | list | unset | Exact journald `PRIORITY` values to include. When unset, no priority filter is installed. |
| `max_priority` | enum | unset | Shorthand for all priorities up to the selected level. Mutually exclusive with `priorities`. |
| `start_at` | enum | `end` | `end` reads new entries only when no checkpoint exists; `beginning` reads existing entries. |
| `batch.max_records` | integer | `1024` | Maximum log records per emitted batch. |
| `batch.max_flush_period` | duration | `200ms` | Maximum time to hold a partial batch. |
| `extraction.max_entry_bytes` | byte size | `1MiB` | Maximum copied bytes per journal entry. |
| `extraction.max_field_bytes` | byte size | `256KiB` | Maximum copied bytes per field value. |
| `extraction.max_fields_per_entry` | integer | `256` | Maximum copied fields per journal entry. |
| `extraction.large_field_policy` | enum | `drop_and_count` | Oversized fields are dropped and counted. |
| `checkpoint.directory` | path | `${engine.state_dir}/journald` | Root directory for durable cursor checkpoints. |
| `checkpoint.max_in_flight_batches` | integer | `1` | Must be `1` in v1. |
| `checkpoint.on_nack` | enum | `rewind` | Either `rewind` or `fail`. |
| `checkpoint.max_consecutive_failures` | integer | `5` | Consecutive checkpoint write failures before failing the source. |
| `wait_timeout` | duration | `1s` | Bounds idle wait and shutdown responsiveness. |
| `drain_timeout` | duration | `5s` | Drain deadline budget; must exceed `wait_timeout`. |

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
  Startup fails clearly when journal files are present but unreadable.
- Must run in a one-core source pipeline. Use `receiver:journald` followed by a
  topic exporter to fan out to multicore downstream processing.
- Run one active journald receiver per host journal source
  (`journal.root_path` + namespace) unless duplicate collection is intentional.
  The receiver rejects duplicate journal sources inside one process, but v1 does
  not coordinate ownership across separate collector processes.
- Named journal namespaces are not supported in v1.
- Kernel ring-buffer (`dmesg`) ingestion is not supported by this receiver.
- `checkpoint.max_in_flight_batches` must be `1` in v1.
- `wait_timeout` must be no more than `5s`, and `drain_timeout` must be greater
  than `wait_timeout`.
- Only one receiver in a process can target the same concrete journal source.
- Duplicate journald fields are emitted as repeated same-key attributes in the
  first implementation. Array coalescing is planned as a follow-up.
- Cross-process source locking is not implemented in v1; run one owner for a
  checkpoint identity (`source_id` and checkpoint directory) to avoid cursor
  races.
- NUMA pinning and placement are future work.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Journald receiver design](../../../../../docs/journald-receiver.md)
- [Core node catalog](../../../README.md)
