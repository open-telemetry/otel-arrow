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

### Metric Sets

#### `receiver.etw.events`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.etw.events.received` | `{event}` | None | Total ETW events produced/received by the session before channel enqueue (ingress denominator). |
| `receiver.etw.events.forwarded` | `{event}` | None | Number of ETW events successfully forwarded downstream in batches. |

#### `receiver.etw.dropped`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.etw.dropped.dropped` | `{event}` | `reason` | Number of dropped ETW events partitioned by drop reason (`slow_worker`, `forward_failed`, `memory_pressure`). |

#### `receiver.etw.decode`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.etw.decode.errors` | `{event}` | None | Number of ETW events that failed TDH decode/parsing. These events are still forwarded with empty decoded fields. |

#### `receiver.etw.kernel`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.etw.kernel.events_lost` | `{event}` | None | Number of events lost inside Windows kernel ETW buffers before delivery (buffer overrun). |

#### `receiver.etw.buffers`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.etw.buffers.buffers` | `{buffer}` | `outcome` | Number of kernel ETW buffers partitioned by buffer outcome (`written`, `real_time_lost`, `log_lost`). |

### Counter Algebra

There is a single ingress denominator:

```text
receiver.etw.events.received
    = receiver.etw.events.forwarded                     (events in successfully-sent batches)
    + receiver.etw.dropped{reason="forward_failed"}     (events in batches that failed to build/send)
    + receiver.etw.dropped{reason="slow_worker"}        (events dropped by internal backpressure)
    + receiver.etw.dropped{reason="memory_pressure"}    (events dropped due to memory limits)
    + events still buffered in the per-core channel and/or in the in-flight builder at snapshot time
```

Important operational notes:
- **Ingress Denominator**: `receiver.etw.events.received` counts all events produced by the ETW session callback before channel enqueue.
- **Decode Quality**: `receiver.etw.decode.errors` tracks events where TDH decode failed. Note that decode failures are still forwarded downstream with empty decoded fields, so they are counted in both `receiver.etw.events.received` and `receiver.etw.events.forwarded`.
- **Kernel Loss**: `receiver.etw.kernel.events_lost` counts events lost inside Windows kernel buffers before the receiver ever saw them (kernel buffer overrun). These are independent from receiver-level drops and are not included in `receiver.etw.events.received`.
- **Derived Rates**:
  - Slow-worker drop rate = `receiver.etw.dropped{reason="slow_worker"} / receiver.etw.events.received`
  - Forward-failure rate = `receiver.etw.dropped{reason="forward_failed"} / receiver.etw.events.received`
