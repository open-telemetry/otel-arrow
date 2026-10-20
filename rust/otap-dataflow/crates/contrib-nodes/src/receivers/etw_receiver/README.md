<!-- markdownlint-disable MD013 -->

# Windows ETW Receiver

## Metadata

- Type: `receiver:etw` (`urn:otel:receiver:etw`)
- Feature gate: `etw`
- Stability: Experimental

## Overview

This receiver ingests Windows [Event Tracing for Windows (ETW)](https://learn.microsoft.com/en-us/windows/win32/etw/event-tracing-portal) events and converts them into OTAP Arrow logs for downstream processing.

## Getting Started

Subscribe to an ETW provider by name or GUID:

```yaml
type: receiver:etw
config:
  session_name: "my-etw-session"
  providers:
    - name: "Microsoft-Windows-Kernel-Process"
  batching:
    max_size: 512
    max_duration: 100ms
```

## Metrics

These tables list telemetry emitted directly by this receiver.

Native OTLP emits each metric with an **Instrumentation Scope** (the metric set name) and an **Instrument Name** (the individual metric). For convenience and counter algebra notation, the combined dotted notation `<scope>.<instrument>` (such as `receiver.etw.dropped.dropped`) is noted as shorthand. The admin Prometheus endpoint also separates scope and instrument.

### Metric Sets

#### `receiver.etw.events`

| Instrument | Shorthand | Unit | Attributes | Description |
| --- | --- | --- | --- | --- |
| `received` | `receiver.etw.events.received` | `{event}` | None | Total ETW events produced/received by the session before channel enqueue (ingress denominator). |
| `forwarded` | `receiver.etw.events.forwarded` | `{event}` | None | Number of ETW events successfully forwarded downstream in batches. |

#### `receiver.etw.dropped`

| Instrument | Shorthand | Unit | Attributes | Description |
| --- | --- | --- | --- | --- |
| `dropped` | `receiver.etw.dropped.dropped` | `{event}` | `reason` | Number of dropped ETW events partitioned by drop reason (`slow_worker`, `forward_failed`, `memory_pressure`). |

#### `receiver.etw.decode`

| Instrument | Shorthand | Unit | Attributes | Description |
| --- | --- | --- | --- | --- |
| `errors` | `receiver.etw.decode.errors` | `{event}` | None | Number of ETW events that failed TDH decode/parsing. These events are still forwarded with empty decoded fields. |

#### `receiver.etw.kernel`

| Instrument | Shorthand | Unit | Attributes | Description |
| --- | --- | --- | --- | --- |
| `events_lost` | `receiver.etw.kernel.events_lost` | `{event}` | None | Number of events lost inside Windows kernel ETW buffers before delivery (buffer overrun). |

#### `receiver.etw.buffers`

| Instrument | Shorthand | Unit | Attributes | Description |
| --- | --- | --- | --- | --- |
| `written` | `receiver.etw.buffers.written` | `{buffer}` | None | Total number of kernel ETW buffers written by the session (activity/throughput denominator). |

#### `receiver.etw.buffer_losses`

| Instrument | Shorthand | Unit | Attributes | Description |
| --- | --- | --- | --- | --- |
| `lost` | `receiver.etw.buffer_losses.lost` | `{buffer}` | `path` | Number of kernel ETW buffers lost partitioned by delivery path (`real_time`, `log`). |

### Counter Algebra

There is a single ingress denominator (expressed here using shorthand notation):

```text
receiver.etw.events.received
    = receiver.etw.events.forwarded                             (events in successfully-sent batches)
    + receiver.etw.dropped.dropped{reason="forward_failed"}     (events in batches that failed to build/send)
    + receiver.etw.dropped.dropped{reason="slow_worker"}        (events dropped by internal backpressure)
    + receiver.etw.dropped.dropped{reason="memory_pressure"}    (events dropped due to memory limits)
    + events still buffered in the per-core channel and/or in the in-flight builder at snapshot time
```

Important operational notes:

- **Ingress Denominator**: `receiver.etw.events.received` (scope `receiver.etw.events`, instrument `received`) counts all events produced by the ETW session callback before channel enqueue.
- **Decode Quality**: `receiver.etw.decode.errors` (scope `receiver.etw.decode`, instrument `errors`) tracks events where TDH decode failed. Note that decode failures are still forwarded downstream with empty decoded fields, so they are counted in both `received` and `forwarded`.
- **Kernel Loss**: `receiver.etw.kernel.events_lost` (scope `receiver.etw.kernel`, instrument `events_lost`) counts events lost inside Windows kernel buffers before the receiver ever saw them (kernel buffer overrun). These are independent from receiver-level drops and are not included in `received`.
- **Kernel Buffers**: `receiver.etw.buffers.written` (scope `receiver.etw.buffers`, instrument `written`) tracks total buffers written as an activity/throughput denominator. Buffer losses are reported separately via `receiver.etw.buffer_losses.lost` (scope `receiver.etw.buffer_losses`, instrument `lost`) with `path="real_time"` (consumer draining too slowly) and `path="log"` (disk flush failure).
- **Derived Rates**:
  - Slow-worker drop rate = `receiver.etw.dropped.dropped{reason="slow_worker"} / receiver.etw.events.received`
  - Forward-failure rate = `receiver.etw.dropped.dropped{reason="forward_failed"} / receiver.etw.events.received`
