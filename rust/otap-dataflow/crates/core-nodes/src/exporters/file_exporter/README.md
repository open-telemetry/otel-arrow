# File Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:file` (`urn:otel:exporter:file`)
- Feature gate: Default
- Stability: Experimental

## Overview

The file exporter writes logs, metrics, and traces as newline-delimited OTLP
JSON. It accepts OTLP protobuf bytes and OTAP Arrow records. Each non-empty
input batch becomes one compact JSON object followed by `\n`.

Every physical file contains one signal type. The required path tokens keep
files exclusive to one signal, core, and deployment generation.

## Getting Started

```yaml
type: exporter:file
config:
  path: "/var/log/otel/telemetry-{signal}-{core_id}-{generation}.jsonl"
```

The parent directory must already exist unless `create_directories` is true.
For logs on core 3 in deployment generation 7, the example resolves to
`/var/log/otel/telemetry-logs-3-7.jsonl`.

## Configuration

| Field | Default | Description |
| --- | --- | --- |
| `path` | Required | Absolute template containing `{signal}`, `{core_id}`, and `{generation}` exactly once. |
| `create_directories` | `false` | Create missing parent directories. |
| `format` | `otlp_json` | Output format. OTLP JSON is the only supported value. |
| `open_mode` | `append` | First-open behavior: `append`, `truncate`, or `create_new`. |
| `durability` | `write` | ACK after `write`, or after `sync_data`. |
| `max_frame_bytes` | `67108864` | Maximum encoded frame size including `\n`; range 1 through 268435456. |
| `tail_recovery` | `truncate_partial` | Append-mode handling: `truncate_partial` or `fail`. |

Unknown fields, relative paths, missing or repeated tokens, unknown tokens,
tokens removed by lexical parent traversal, and explicit `tail_recovery`
settings outside append mode are rejected during configuration validation.

### Open modes

- `append` retains complete frames. If the file has an incomplete final frame,
  `truncate_partial` scans backward by at most `max_frame_bytes` and removes
  only that tail. `fail` rejects the file instead.
- `truncate` explicitly discards existing contents when a signal first opens.
- `create_new` rejects a signal path that already exists.

Unused signal files are not created. A signal writer is opened and its append
tail is checked when the first non-empty batch for that signal arrives. The
first actual frame exercises the write path.

### Durability and failures

`write` acknowledges after the complete frame is accepted and flushed by the
operating system. `sync_data` additionally synchronizes file data before the
ACK. A graceful shutdown flushes and synchronizes all open files within the
pipeline deadline regardless of the selected ACK durability.

The exporter records the previous file length before each write. A failed
write or sync is truncated back to that length and receives a retryable NACK.
A rollback failure also NACKs the batch and terminates the node because the
file state is indeterminate. Invalid pdata and oversized frames receive a
permanent NACK without modifying the file.

The exporter owns no retry queue. Compose the retry processor for redelivery
policy and the durable-buffer processor for crash-persistent pending work.

## Output

| Signal | `{signal}` | Top-level repeated field |
| --- | --- | --- |
| Logs | `logs` | `resourceLogs` |
| Metrics | `metrics` | `resourceMetrics` |
| Traces | `traces` | `resourceSpans` |

The encoding follows OTLP ProtoJSON rules, including quoted 64-bit integers,
hexadecimal trace and span IDs, base64 byte values, numeric enums, lower-camel-case
field names, and omission of default values. Field ordering and insignificant
whitespace are not part of the contract.

## Security and Operations

The destination contains full telemetry, including bodies and attributes.
Treat it as sensitive storage. On Unix, newly created files use mode `0600`
and newly created directories use mode `0700`, subject to the process umask.
Existing permissions are unchanged.

Paths are derived only from trusted configuration and bounded runtime values;
telemetry attributes never affect them. A process-local lease prevents two
live exporter writers from owning the same normalized path. It does not
coordinate separate processes. Filesystem quotas, retention, encryption,
mount policy, and cross-process ownership remain operator responsibilities.

Do not delete, rename, replace, or externally rotate an active destination
file. The exporter retains the open file handle and does not re-resolve the
configured path after opening it. Stop the exporter before performing
destination lifecycle operations.

## Telemetry

### Metric sets

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.file.exports.messages` | `{message}` | `signal`, `outcome` | Telemetry messages whose file export reached a terminal outcome. |
| `exporter.file.items` | `{item}` | `signal` | Signal items in successfully written frames. |
| `exporter.file.bytes` | `By` | `signal` | Successfully written bytes including delimiters. |
| `exporter.file.failures` | `{failure}` | `signal`, `operation` | Open, write, sync, or rollback failures. |
| `exporter.file.tail_recoveries` | `{recovery}` | `signal` | Incomplete final frames repaired at open. |
| `exporter.file.tail_recovered_bytes` | `By` | `signal` | Bytes removed by successful tail repair. |

No metric contains a destination path.

### Events

| Event | Severity | Attributes | Description |
| --- | --- | --- | --- |
| `otelcol.node.file.start` | `info` | `format`, `create_directories`, `open_mode`, `durability`, `tail_recovery`, `max_frame_bytes` | Exporter startup with its non-sensitive bounded configuration. |
| `otelcol.node.file.writer.start` | `info` | `signal` | A signal writer opened successfully on first use. |
| `otelcol.node.file.tail.recover` | `warn` | `signal`, `recovered_bytes` | An incomplete final frame was removed. |
| `otelcol.node.file.operation.fail` | `warn` | `signal`, `operation`, `error` | A signal writer entered an I/O failure state. |
| `otelcol.node.file.rollback.fail` | `error` | `signal`, `operation`, `error`, `rollback_error` | Rollback failed and the node will terminate. |
| `otelcol.node.file.stop` | `info` | `reason` | Graceful shutdown completed. |

## Limits

The first release does not support profiles, rotation, retention, compression,
protobuf output, plain-text templates, attribute-derived paths, an internal
retry queue, or exactly-once delivery. A completed frame may be replayed twice
if its write succeeded but its ACK was not observed before a crash.

## Related Docs

- [Architecture and design decisions](ARCHITECTURE.md)
- [Example configuration](../../../../../configs/trafficgen-file.yaml)
- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)

<!-- markdownlint-enable MD013 -->
