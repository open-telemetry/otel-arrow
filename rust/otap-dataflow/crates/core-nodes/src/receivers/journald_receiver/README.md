# Journald Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:receiver:journald`
- Type shortcut: `receiver:journald`
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

## Configuration

Minimal configuration:

```yaml
type: receiver:journald
config:
  source_id: system
  start_at: end
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
| `source_id` | string | `system` | Stable source identifier used in telemetry and checkpoint paths. Must contain only ASCII alphanumerics, `_`, `-`, or `.`. |
| `journal.root_path` | path | `/` | Absolute UTF-8 filesystem root used for local or mounted host journal access. `..` path components are rejected. |
| `journal.namespace` | string | unset | Reserved for named systemd journal namespaces. Named namespaces are rejected in v1; omit this field to read the default namespace. |
| `units` | list of strings | `[]` | Exact `_SYSTEMD_UNIT` matches. Empty entries are rejected and duplicate entries are removed. |
| `identifiers` | list of strings | `[]` | Exact `SYSLOG_IDENTIFIER` matches. Empty entries are rejected and duplicate entries are removed. |
| `priorities` | list of integers | `[0, 1, 2, 3, 4, 5, 6, 7]` | Exact journald `PRIORITY` values to include. Entries must be in the range `0..=7`. |
| `max_priority` | enum | unset | Shorthand for all priorities from emergency through the named level. One of `emergency`, `alert`, `critical`, `error`, `warning`, `notice`, `info`, or `debug`. Mutually exclusive with `priorities`. |
| `start_at` | enum | `end` | Where to start when no checkpoint exists. One of `end` or `beginning`. |
| `batch.max_records` | integer | `1024` | Maximum records per emitted batch. Must be greater than zero. |
| `batch.max_flush_period` | duration | `200ms` | Maximum time to hold a partial batch before flushing. Must be greater than zero. |
| `checkpoint.directory` | path | `${engine.state_dir}/journald` | Root directory for cursor checkpoint files. The receiver appends pipeline and source path components. |
| `checkpoint.max_in_flight_batches` | integer | `1` | Maximum unacknowledged batches. Must be `1` in v1. |
| `checkpoint.on_nack` | enum | `rewind` | Behavior on downstream NACK. One of `rewind` or `fail`. |
| `checkpoint.max_consecutive_failures` | integer | `5` | Maximum consecutive checkpoint commit failures before failing the receiver. Must be greater than zero. |
| `wait_timeout` | duration | `1s` | `sd_journal_wait` timeout. Must be greater than zero and no more than `5s`. |
| `drain_timeout` | duration | `5s` | Drain deadline budget. Must be greater than `wait_timeout`. |

## Examples

See the minimal and mounted-host examples above.

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
