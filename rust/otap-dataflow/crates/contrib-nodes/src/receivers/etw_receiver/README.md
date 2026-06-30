# ETW Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:etw` (`urn:otel:receiver:etw`)
- Feature gate: Default
- Stability: Experimental
- Platform: Windows only (`#[cfg(target_os = "windows")]`)

## Overview

The ETW (Event Tracing for Windows) receiver subscribes to one or more Windows
ETW provider sessions, decodes events using the Trace Data Helper (TDH) API
(including TraceLogging schemas), converts them to OTAP Arrow log record
batches, and forwards them through the pipeline.

## Getting Started

```yaml
etw:
  type: receiver:etw
  config:
    providers:
      - guid: "d2387720-2907-5677-8625-c1bdc4155197"
        level: verbose
```

## Configuration

```yaml
type: receiver:etw
config:
  # One or more ETW providers to subscribe to. Required.
  providers:
    - guid: "d2387720-2907-5677-8625-c1bdc4155197"
      level: verbose                             # see trace levels below
      keywords: 0x0000000000000010               # optional keyword bitmask

  # Name of the ETW trace session. Optional, defaults to "OtelArrowETW".
  # Windows permits only one real-time session per name; use distinct names
  # when running multiple receiver:etw nodes simultaneously.
  session_name: "OtelArrowETW"

  # In-memory OTAP log batching limits. Optional.
  batching:
    max_size: 512        # max log records per emitted Arrow batch (1..=65535, default: 512)
    max_duration: "100ms" # max time to hold a non-empty batch before flush (default: 100ms)
```

### Provider fields

| Field | Required | Description |
| ----- | -------- | ----------- |
| `guid` | Yes (or `name`) | Provider GUID string, e.g. `"d2387720-2907-5677-8625-c1bdc4155197"`. Mutually exclusive with `name`. |
| `name` | Yes (or `guid`) | Provider name string. **Not yet implemented** - use `guid` for now. |
| `level` | No | Trace level filter. Default: `information`. |
| `keywords` | No | 64-bit keyword bitmask for additional event filtering. Default: all keywords. |

### Trace levels

| Value | Description |
| ----- | ----------- |
| `critical` | Critical errors only |
| `error` | Errors and critical events |
| `warning` | Warnings, errors, and critical events |
| `information` | Informational events and above (**default**) |
| `verbose` | All events including verbose/debug output |

## Multi-core fan-out

Windows allows only one real-time ETW session per session name. The engine may
instantiate the receiver on multiple cores. Internally, one session is created
per `session_name` with one consumer channel per core, and the `ProcessTrace`
callback round-robins events across those channels so each core receives an
equal share of the event stream.

## Output: Log Record fields

Each decoded ETW event becomes one OTAP log record with the following fields.

### Log record

| Field | Value |
| ----- | ----- |
| `timestamp` | Event timestamp converted from QPC ticks to Unix nanoseconds |
| `severity_number` | Mapped from the ETW level (see table below) |
| `severity_text` | Original ETW level name (e.g. `"WARNING"`, `"INFO"`, `"VERBOSE"`) |
| `event_name` | TDH event name (e.g. `"AppStarted"`), or `"etw.<event_id>"` as a fallback |
| `body` | Empty - decoded fields go into attributes |

### ETW level to OTel severity mapping

| ETW level | ETW name | OTel severity number | OTel severity text |
| --------- | -------- | -------------------- | ------------------ |
| 0 | LOG_ALWAYS | 0 (UNSPECIFIED) | `LOG_ALWAYS` |
| 1 | CRITICAL | 21 (FATAL) | `CRITICAL` |
| 2 | ERROR | 17 (ERROR) | `ERROR` |
| 3 | WARNING | 13 (WARN) | `WARNING` |
| 4 | INFO | 9 (INFO) | `INFO` |
| 5 | VERBOSE | 5 (DEBUG) | `VERBOSE` |
| 6-255 | *(reserved / provider-defined)* | 0 (UNSPECIFIED) | *(absent)* |

### Attributes

The following attributes are always set on every log record:

| Attribute | Type | Description |
| --------- | ---- | ----------- |
| `etw.event.id` | int | Numeric ETW event identifier |
| `etw.level` | int | Raw ETW level value (preserved so the mapping is reversible) |
| `etw.opcode` | int | ETW opcode |
| `etw.version` | int | ETW event version |
| `etw.keywords` | int | ETW keyword bitmask |
| `etw.process.id` | int | PID of the process that emitted the event |
| `etw.thread.id` | int | Thread ID of the thread that emitted the event |
| `etw.provider.id` | string | Provider GUID as a lowercase hyphenated hex string |

The following attribute is set only when a non-zero activity ID is present:

| Attribute | Type | Description |
| --------- | ---- | ----------- |
| `etw.activity.id` | string | Correlation activity ID as a lowercase hyphenated hex string |

TDH-decoded payload fields are appended as additional string, int, double,
bool, or bytes attributes using the field name from the event manifest or
TraceLogging schema.

## Known limitations

- Provider `name`-to-GUID resolution is not yet implemented. Always use `guid`.
- Manifest-based events with provider-defined levels (6-255) report
  `severity_number = 0` (UNSPECIFIED).
- This component is Windows-only and produces no output on other platforms.

## Telemetry

See [telemetry.md](telemetry.md) for the ETW receiver's emitted metrics,
internal log events, counter relationships, and maintenance checklist.

<!-- markdownlint-enable MD013 -->
