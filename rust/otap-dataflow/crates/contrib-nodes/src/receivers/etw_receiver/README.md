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
| `receiver.etw.events.events` | `{event}` | `outcome` | Number of ETW events partitioned by outcome (`received`, `forwarded`, `invalid`, `lost`). |

#### `receiver.etw.dropped`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.etw.dropped.dropped` | `{event}` | `reason` | Number of dropped ETW events partitioned by drop reason (`slow_worker`, `forward_failed`, `memory_pressure`). |

#### `receiver.etw.buffers`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.etw.buffers.buffers` | `{buffer}` | `outcome` | Number of kernel ETW buffers partitioned by buffer outcome (`written`, `real_time_lost`, `log_lost`). |
