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

## Windows containers

Set `enable_in_containers: true` on a provider to also collect its events from
[process-isolated](https://learn.microsoft.com/en-us/virtualization/windowscontainers/manage-containers/hyperv-container)
Windows containers while the receiver runs on the host. The receiver enables
the provider with `EVENT_ENABLE_PROPERTY_ENABLE_SILOS` and
`EVENT_ENABLE_PROPERTY_SOURCE_CONTAINER_TRACKING`; existing `level`,
`keywords`, and `event_ids` filters still apply. The option defaults to
`false`.

```yaml
type: receiver:etw
config:
  session_name: "container-etw"
  providers:
    - guid: "<application-provider-guid>"
      level: information
      enable_in_containers: true
```

### Source container ID

Container events get a log attribute, `etw.container.id`. It is a **grouping
key**: all events from one container have the same value, and different
containers have different values. It does not tell you the container's name or
runtime ID. There is no separate setting for it.

Host events never have this attribute.

Windows assigns the value. It is **not** the containerd/CRI container ID, so
the receiver does not set `container.id`. On Windows Server 2025 (build 26100)
it is an uppercase GUID string, such as `EFF73D44-44E9-5C45-B432-7F67133D3923`,
and pod sandbox containers have their own values. Microsoft does not document
the format, so treat it as an opaque string. The receiver copies it unchanged.
Values longer than 256 bytes or with characters outside printable ASCII are
dropped (logged at debug level), and the event is still forwarded.

To find which container a value belongs to, take `etw.process.id` from one of
its events and, on the host while that process is running, look up the
process's session ID, then the HCS compute system with that
`ServiceSessionId` (`hcsdiag list -raw`). Its `Id` is the runtime container ID
(`crictl inspect <id>`).

For EventSource providers that send their manifest in-band and run on .NET
versions earlier than 8, source-container tracking can push large manifest
events over ETW's size limit, and they are then lost
([dotnet/runtime#87255](https://github.com/dotnet/runtime/issues/87255)).
Upgrade these providers to .NET 8 or later before enabling container collection.
TraceLogging providers do not use these in-band manifest events. Tracking still
adds metadata to every event, so ETW's general event-size limit applies to them
too.

### Container limitations

- Hyper-V-isolated containers run a separate kernel and are not reached.
- Collection covers every process-isolated container that uses the provider;
  it is not a container-selection filter.
- `etw.container.id` is a grouping key, not mapped to `container.id` or a
  container name yet.
- Decoding manifest-based events requires the provider manifest to be
  installed on the host.

## Metrics

The following table lists the telemetry emitted by this receiver. All metric sets include a fixed registration attribute `signal="logs"`.

| Scope | Instrument | Unit | Attributes | Meaning |
| --- | --- | --- | --- | --- |
| `receiver.etw.events` | `received` | `{event}` | None | Total ETW events produced/received by the session before channel enqueue (ingress denominator). |
| `receiver.etw.events` | `forwarded` | `{event}` | None | ETW events successfully handed off to the next pipeline stage in batches (not guaranteed delivery to final backend). |
| `receiver.etw.drops` | `events` | `{event}` | `reason` | Number of events dropped at the receiver boundary. Values: `slow_worker` (internal channel backpressure), `forward_failed` (batch build or forward error), and `memory_pressure` (reserved/not currently recorded; the receiver does not implement memory-pressure shedding). |
| `receiver.etw.decode` | `errors` | `{event}` | None | Number of ETW events that failed TDH decode/parsing. Decode-failed events remain eligible for forwarding downstream with empty decoded fields (successful forwarding is not guaranteed), so they are counted under `received` and (if handed off) `forwarded`. |
| `receiver.etw.session` | `events.lost` | `{event}` | None | Number of events lost inside Windows kernel ETW buffers before delivery (buffer overrun). |
| `receiver.etw.session` | `buffers.written` | `{buffer}` | None | Total number of kernel ETW buffers written by the session (activity/throughput denominator). |
| `receiver.etw.session` | `buffers.lost` | `{buffer}` | `delivery` | Number of kernel ETW buffers lost, distinguishing real-time consumer delivery (`real_time`) from trace-log-file output (`log_file`). |

### Counter Algebra

There is a single ingress denominator (expressed here using `<scope>.<instrument>` notation):

```text
receiver.etw.events.received
    = receiver.etw.events.forwarded                             (events successfully handed off to next pipeline stage)
    + receiver.etw.drops.events{reason="forward_failed"}        (events in batches that failed to build/send)
    + receiver.etw.drops.events{reason="slow_worker"}           (events dropped by internal backpressure)
    + receiver.etw.drops.events{reason="memory_pressure"}       (reserved; not currently recorded)
    + events still buffered in the per-core channel and/or in the in-flight builder at snapshot time
```

### Operational notes

- `receiver.etw.events.received` is the ingress denominator, counted on the session callback before channel enqueue.
- `receiver.etw.events.forwarded` tracks handoff to the next pipeline stage (batches sent downstream), not end-to-end backend delivery.
- `receiver.etw.decode.errors` tracks TDH decoding errors. Decode-failed events are still forwarded downstream with empty decoded fields when possible, so they appear in both `received` and `forwarded`.
- `receiver.etw.session` groups Windows ETW session properties. `events.lost` tracks kernel buffer overruns before the receiver saw the events. `buffers.lost` separates real-time consumer delivery drops (`real_time`) from trace log file drops (`log_file`).
- `receiver.etw.drops.events` partitions receiver drops by `reason` (`slow_worker` for channel backpressure, `forward_failed` for batch send failures). `memory_pressure` is reserved for future bounded shedding and is not currently emitted.
- Slow-worker drop rate = `receiver.etw.drops.events{reason="slow_worker"} / receiver.etw.events.received`
- Forward-failure rate = `receiver.etw.drops.events{reason="forward_failed"} / receiver.etw.events.received`
