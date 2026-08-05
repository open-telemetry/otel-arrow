# Console Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:console` (`urn:otel:exporter:console`)
- Feature gate: Default
- Stability: Experimental

## Overview

The console exporter prints OTLP logs to standard output. It supports a
human-readable hierarchical `pretty` format for interactive inspection and a
newline-delimited record JSON format for structured logging pipelines.

This node is intended for local inspection, demos, and debugging pipelines. It
is not a production exporter, durable export path, or stable machine-readable
storage path.

## Acknowledgment Semantics

Each payload is formatted into one complete frame and handed to the engine's
process-wide console writer. The exporter ACKs the message once that handoff
attempt resolves, including when the handoff fails.

An ACK therefore means only "handed to the process-wide writer". It does not
mean the bytes reached the terminal, were consumed by a downstream logging
agent, were persisted, or were delivered durably.

## Console Output Serialization

Console output from the engine's cooperating writers -- this exporter, node
`info` messages, and internal diagnostics -- flows through a single writer
thread per standard stream. That thread holds the stream lock for the whole
frame, so concurrent exporters on different cores can never interleave bytes
inside one frame, even when a payload is larger than a single underlying write.
Every `record_json` line therefore stays independently parseable.

The queue feeding the writer is bounded, so a console that cannot keep up slows
producers down instead of dropping data or growing without limit. Best-effort
internal diagnostics are the exception: they are dropped rather than allowed to
stall an engine core thread. Ordering is FIFO per producer; output from
different cores is not globally ordered by timestamp.

Defaults require no configuration: 1024 queued frames for stdout, 256 for
stderr, a flush whenever the queue goes idle, and a 5 second drain deadline at
shutdown. When an engine run ends, accepted frames are written and flushed
before it returns; anything still queued when that deadline expires is reported
instead of waited on. The writer threads are process-wide and stay running, so
another engine run in the same process keeps its console output.

A console that never accepts writes is a separate case. Because the queue
applies backpressure by design, producers wait once it fills, and the pipeline
stops making progress until the console drains. The drain deadline bounds the
final flush, not that upstream stall.

While any console exporter in the process is configured with
`format: record_json`, stdout is treated as a machine-readable stream for the
rest of the run: human-readable engine diagnostics and the output of any
`pretty` console exporter are routed to stderr instead, so they cannot corrupt
it.

That claim is made when each exporter is created, and pipelines are built
concurrently. A `pretty` exporter that emits during the startup window, before
any `record_json` exporter has been created, can still reach stdout. Configure
one console output format per process when stdout must be machine readable.

The integrity guarantee covers output submitted through this writer only. It
excludes writers that bypass it: raw file descriptors, inherited child
processes, the debug processor's console fallback, last-resort messages emitted
while a controller thread is being torn down, and standalone binaries such as
`ctl`.

## Getting Started

Use the console exporter when you want to inspect pdata directly from the
engine process:

```yaml
type: exporter:console
config:
  format: pretty
  color: true
  unicode: true
```

## Configuration

```yaml
type: exporter:console
config:
  # Output format: "pretty" (default) or "record_json".
  format: pretty

  # Enables ANSI color output (default: true).
  # Applies only to pretty output.
  color: true

  # Enables Unicode box-drawing output (default: true).
  # Applies only to pretty output.
  unicode: true

  # Format-specific record_json options.
  record_json:
    # "rfc3339" (default) or "unix_nano".
    timestamp_format: rfc3339

    # "body" (default) or "message".
    body_field: body

    # "number" (default) or "string".
    int64_format: number

    # Include resource attributes (default: false).
    resource: false

    # Include scope context (default: true).
    scope: true

    # Include OpenTelemetry bookkeeping fields (default: false).
    otel: false
```

Structured formats use named configuration blocks. A future `otlp_json` format
can therefore add an `otlp_json` block without changing `record_json` options.
The existing pretty-only `color` and `unicode` fields remain at the top level for
compatibility. Options belonging to an unselected format are accepted but
ignored.

## Examples

### ASCII-only pretty output

```yaml
type: exporter:console
config:
  format: pretty
  color: false
  unicode: false
```

### Record JSON output

`record_json` writes one compact JSON object followed by `\n` for each log
record. It uses logging-oriented snake_case fields and native JSON values.
Enabled resource and scope context are added as sibling objects.

The following outputs use the engine's `otlp.receiver.grpc.start` internal
event as a representative record.

With the default context configuration (`resource: false`, `scope: true`):

```yaml
type: exporter:console
config:
  format: record_json
```

```json
{"timestamp":"2025-01-15T10:30:00.000000000Z","severity_number":9,"body":"Starting OTLP gRPC receiver","event_name":"otlp.receiver.grpc.start","attributes":{"endpoint":"0.0.0.0:4317"},"scope":{"name":"otap-df-core-nodes","attributes":{}}}
```

With both resource and scope context disabled:

```yaml
type: exporter:console
config:
  format: record_json
  record_json:
    resource: false
    scope: false
```

```json
{"timestamp":"2025-01-15T10:30:00.000000000Z","severity_number":9,"body":"Starting OTLP gRPC receiver","event_name":"otlp.receiver.grpc.start","attributes":{"endpoint":"0.0.0.0:4317"}}
```

With both resource and scope context enabled:

```yaml
type: exporter:console
config:
  format: record_json
  record_json:
    resource: true
    scope: true
```

```json
{"timestamp":"2025-01-15T10:30:00.000000000Z","severity_number":9,"body":"Starting OTLP gRPC receiver","event_name":"otlp.receiver.grpc.start","attributes":{"endpoint":"0.0.0.0:4317"},"resource":{"service.name":"otap_engine"},"scope":{"name":"otap-df-core-nodes","attributes":{}}}
```

The top-level `attributes` object is always present. Enabled resource and scope
context also have stable empty-object representations. Other unavailable fields
are omitted.

`record_json` is designed for logging agents that treat each line as an
independent event. It is intentionally not a top-level OTLP JSON document.

### Compact value encoding

Bodies and attributes use native JSON values. Strings, booleans, finite
doubles, arrays, and key-value lists map directly to their JSON equivalents.
Bytes use standard base64 strings. Empty values and non-finite doubles use
`null`.

Int64 values are JSON integers by default. Set `int64_format: string` to emit
decimal strings instead. Duplicate attribute keys use the final occurrence;
if that occurrence has no value, the key is omitted.

`timestamp_format: rfc3339` emits UTC timestamps with exactly nine fractional
digits. Set it to `unix_nano` for decimal-string nanoseconds since the Unix
epoch. `body_field: message` emits the body only as `message` instead of
`body`.

When `otel: true`, each record includes an `otel` object with
`dropped_attributes_count` and any available `resource_schema_url` and
`scope_schema_url`.

## Record JSON and OTLP JSON

OpenTelemetry's
[file-exporter specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/file-exporter.md)
defines JSON Lines containing complete top-level `LogsData` objects. A single
standard line may therefore contain multiple resources, scopes, and log
records.

That standard format and `record_json` have different framing:

| Format | JSON value written per line | Availability |
| --- | --- | --- |
| `record_json` | One log record with optional repeated context | Supported |
| `otlp_json` | One complete OTLP signal batch | Planned |

`otlp_json` is reserved for a future implementation and is not currently an
accepted configuration value.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

| Metric | Unit | Description |
| --- | --- | --- |
| *None* | N/A | This node does not register a node-specific metric set. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `console.logs_view.otlp_create_failed` | `error` | Failed to create an OTLP logs view for console output. |
| `console.logs_view.otap_create_failed` | `error` | Failed to create an OTAP logs view for console output. |
| `console.traces.not_implemented` | `error` | The exporter received traces, which are not currently rendered. |
| `console.metrics.not_implemented` | `error` | The exporter received metrics, which are not currently rendered. |
| `console.format_failed` | `error` | Failed to format a payload for console output. |
| `console.write_failed` | `error` | Failed to write rendered output to stdout. |

## Limits

- Output is written to the process console and is not persisted.
- Large or high-rate telemetry streams can produce substantial console output.
- Formatting and writes are best effort. Payloads are ACKed after the export
  attempt, including when formatting or writing fails.
- Traces and metrics are not currently rendered in either format.
- OTAP views do not currently expose every scope field. In particular, scope
  name and version can be absent from `record_json` after conversion to OTAP,
  while scope attributes remain available.
- OTAP log views do not currently expose resource or scope schema URLs.
- `record_json` is a machine-readable debugging format, not the standardized
  OpenTelemetry file-exporter representation.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
