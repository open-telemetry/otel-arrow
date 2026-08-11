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

An ACK therefore means only "the handoff attempt finished". The attempt may have
failed, and even a successful one does not mean the bytes reached the terminal,
were consumed by a downstream logging agent, were persisted, or were delivered
durably.

## Console Output Serialization

Console exporters can run concurrently on multiple engine cores, but stdout and
stderr are shared by the entire process. To prevent output from different cores
from being interleaved, cooperating engine writers use a process-wide output
service.

producer -->   bounded queue -->  dedicated writer
complete frame  -->  stdout queue  ------>  lock stdout, write, flush
complete frame  -->  stderr queue  ------>  lock stderr, write, flush

Each producer formats a complete frame before submitting it. A frame may contain
one pretty-printed payload or several newline-terminated `record_json` records.
The writer holds the stream lock while writing the entire frame, so another
producer cannot insert bytes into it. Ordering is preserved for each producer,
but output from different cores is not globally ordered.

The queues bound both the number of frames and the number of bytes waiting to be
written, because a frame owns its payload and payloads vary in size. Console
exporters wait when either limit is reached, applying backpressure to the
pipeline. Internal diagnostics use a best-effort path instead: they are dropped
when the stderr queue is full so logging cannot stall an engine core. By default
stdout holds up to 1024 frames or 64 MiB, and stderr up to 256 frames or 16 MiB.
A frame larger than the whole byte budget is rejected rather than queued, since
draining could never make room for it.

Human-readable engine diagnostics always go to stderr. When the accepted
configuration contains a `record_json` console exporter, stdout is reserved for
structured records and any pretty console exporter also writes to stderr. The
standard engine binary applies this policy before starting its pipelines.
Applications embedding `Controller` directly must call
`claim_structured_stdout` on the validated configuration before starting it.
The first `record_json` exporter cannot be introduced through live control after
a pretty-only process has started.

At the end of an engine run, the controller waits up to five seconds for
accepted frames to be written and flushed. The process-wide writer threads
remain available for later runs in the same process. On final process shutdown,
the engine attempts to drain and join them. Writer failures and incomplete
drains are reported with the number of frames still pending.

Only output submitted through this service receives the frame-integrity
guarantee. Direct file-descriptor writes, child-process output, standalone
binaries, and the debug processor's console fallback remain outside it.

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
