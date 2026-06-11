# Journald Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:journald` (`urn:otel:receiver:journald`)
- Feature gate: Default
- Stability: Experimental

## Overview

Linux-only receiver for local `systemd-journald` entries. It targets structured
journal records, uses journald source selection such as units, identifiers, and
priorities, and is designed around journald cursors for checkpointed progress.

The current implementation is the first receiver slice: it registers the
factory, validates configuration, enforces a process-local source lease, and
handles lifecycle, drain, shutdown, and telemetry control messages. The blocking
`sd-journal` worker, batch handoff, ACK/NACK tracking, and checkpoint
persistence are follow-up work.

## Getting Started

Start at the end of the default system journal:

```yaml
type: receiver:journald
config:
  source_id: system
  start_at: end
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
- Must run in a one-core source pipeline. Use `receiver:journald` followed by a
  topic exporter to fan out to multicore downstream processing.
- Named journal namespaces are not supported in v1.
- Kernel ring-buffer (`dmesg`) ingestion is not supported by this receiver.
- `checkpoint.max_in_flight_batches` must be `1` in v1.
- `wait_timeout` must be no more than `5s`, and `drain_timeout` must be greater
  than `wait_timeout`.
- Only one receiver in a process can target the same concrete journal source.
- The current implementation does not yet emit journal records; the blocking
  `sd-journal` worker, batch handoff, ACK/NACK tracking, and checkpoint
  persistence are follow-up work.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Journald receiver design](../../../../../docs/journald-receiver.md)
- [Core node catalog](../../../README.md)
