# Internal Telemetry Collection Architecture & Development Plan

## Architecture

OTAP-Dataflow uses a highly-configurable internal telemetry data
plane.  We believe in supporting many alternatives because users have
a wide range of observability requirements, therefore we present a
number of orthogonal choices when configuring internal telemetry.

OTAP-Dataflow supports a self-hosted internal diagnostics data path,
which means it is designed to safely consume its own telemetry. While
this requires care and attention, the engine contains within
`otap-df-otap` all the building blocks to "be" an OpenTelemetry SDK.

Consuming internal telemetry presents a potential for self-induced
telemetry, a harmful form of self-dependency. The OTAP-Dataflow
internal telemetry pipeline is rigorously safeguarded against these
pitfalls through:

- OTAP-Dataflow components downstream of an ITR cannot be configured
  to send to an ITR node to avoid feedback
- Thread-local variable to avoid self-induced log events the global
  logs collection threads (both OpenTelemetry SDK or OTAP-Dataflow
  cases)
- Routing to on-core OTAP-Dataflow pipeline for log events within an
  engine thread avoids blocking the engine and isolates the cores 
  that are able to process their own telemetry.
- Option to fall back to no-op and raw LoggerProviders.

As a key design decision, the OTAP-Dataflow internal telemetry data
path takes an OTLP-first approach. This is appropriate for the
OTAP-Dataflow engine because OTLP bytes is one of the builtin
`OtapPayload` formats, and as soon as we have an OTLP bytes encoding
we are able to send to any OTAP-Dataflow pipeline. To obtain these
bytes, we will build a custom [Tokio `tracing` Event][TOKIOEVENT] handler.

[TOKIOEVENT]: https://docs.rs/tracing/latest/tracing/struct.Event.html

As a matter of last resort, we support directly formatting a message
for the console directly from OTLP bytes, based on the
`otap_df_pdata::views::logs::LogsDataView` and associated types, which
supports zero-copy traversal of OTLP bytes. We refer to "Raw" logging
as a handler for OTLP bytes that prints synchronously, direcly to the
console. Raw logging is used before the OTAP-Dataflow engine starts,
and it is provided as an option for internal telemetry collection since
it always avoids self-dependency.

There are two internal logs data paths:

- Tokio `tracing` global subscriber: third-party log events, instrumentation
  in code without access to an OTAP-Dataflow `EffectHandler`.
- `EffectHandler` supports a direct logging interface for components, these
  are routed using local- or shared-specific synchronization logic, and these
  interfaces will introduce attributes specific to the OTAP-Dataflow engine.
  
We establish a global logs collection thread (potentially multiple of
them). The global collection thread is used as the primary collection
point for multi-threaded applications, for processing Tokio `tracing`
events in threads not belonging to OTAP-Dataflow.

An internal telemetry `TelemetryRouter` concept will be developed, supporting 
per-component configuration of [`TelemetrySettings` as this type of runtime
configuration is called in the OpenTelemetry Collector][TELSETTINGS].

[TELSETTINGS]: https://github.com/open-telemetry/opentelemetry-collector/blob/bf28fa76882d0d6e40457db8bfffb86a4efcdfbf/component/telemetry.go#L14

TelemetrySettings will be configurable, allowing fine-grain control
over logging behavior in the components.  The router is configurable
with:

- No-op logging
- Raw logging
- Global logging (to OTel SDK)
- Global logging (to dedicated OTAP-Datflow)
- Internal routing (to EffectHandler logging buffer)

We use a thread-local variable for routing Tokio `tracing` events
through the effective `EffectHandler` instance, for OTAP-Dataflow
threads. This prevents third-party log events from impacting the
engine directly.

Whether the OpenTelemetry SDK is used or the ITR is used instead, the
global logs collection thread itself configures a special bit in the
thread-local state to avoid self-induced logging events within any
thread that is a last-resort for telemetry export. These threads are
forbidden from logging.

While we can extend this design to other OpenTelemetry signals, this
design is focused on logs. We anticipate that the global logs
collection thread described here will only process logs, that we will
use other separate solutions for other signals. However, we anticipate
adding similar configurability for `MeterProvider` and
`TracingProvider` in the future.

## Telemetry Data Paths

```
                    OTAP-Dataflow Telemetry Routing
                    
┌─────────────────────────────────────────────────────────────────┐
│                      Telemetry Sources                          │
└─────────────────────────────────────────────────────────────────┘
        │                                    │
        │                                    │
        ▼                                    ▼
┌──────────────────┐              ┌──────────────────────┐
│ Tokio Subscriber │              │  Effect Handler      │
│ tracing::info!() │              │  otel_info!(effect)  │
│ (3rd party libs) │              │  (OTAP components)   │
└────────┬─────────┘              └──────────┬───────────┘
         │                                   │
         │ Global                            │ Per-thread
         │ Subscriber                        │ Buffer
         ▼                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│              TracingLogRecord (LogRecordView)                   │
│              Structured data capture (no formatting)            │
└─────────────────────────────────────────────────────────────────┘
         │                                   │
         │                                   │
         ▼                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                   StatefulOtlpEncoder                           │
│                   Streaming OTLP bytes encoding                 │
└─────────────────────────────────────────────────────────────────┘
         │                                   │
         │                                   │
         ▼                                   ▼
    OTLP Bytes                          OTLP Bytes
    (per-event)                         (batched)
         │                                   │
         │                                   │
         └─────────┬─────────────────────────┘
                   │
                   │ Thread-local routing
                   ▼
         ┌─────────────────────┐
         │ 1. On-Core ITR?     │
         │    (if configured)  │
         └──────┬─────────┬────┘
                │         │
          YES   │         │ NO or overflow
                │         │
                ▼         ▼
         ┌──────────┐   ┌────────────────────┐
         │  On-Core │   │ 2. Global Thread?  │
         │   ITR    │   │    (if configured) │
         └────┬─────┘   └──────┬──────┬──────┘
              │                │      │
              │          YES   │      │ NO or full
              │                │      │
              ▼                ▼      ▼
    ┌──────────────────┐  ┌────────────┐  ┌──────────────┐
    │ OTAP Pipeline    │  │ Global     │  │ 3. Raw       │
    │ • Batch          │  │ Collection │  │    Logger    │
    │ • Transform      │  │ Thread     │  │  (console)   │
    │ • Export         │  └─────┬──────┘  └──────────────┘
    │ (Internal        │        │
    │  Telemetry)      │        │
    └──────────────────┘        │
                                ▼
                    ┌───────────────────────┐
                    │   Destination         │
                    ├───────────────────────┤
                    │ • SDK Exporters       │
                    │   - Console           │
                    │   - OTLP              │
                    │   - File              │
                    │   - Custom            │
                    │                       │
                    │ • Dedicated OTAP      │
                    │   Pipeline            │
                    │   (Full dataflow      │
                    │    processing)        │
                    └───────────────────────┘

Legend:
  • On-Core ITR: Fast path, core isolation, preferred for engine threads
  • Global Thread: Overflow/fallback, handles 3rd party + overflow
  • Raw Logger: Ultimate fallback, synchronous console, never fails
  • OTLP Bytes: Universal format, native to OTAP pipeline
```

## Development plan

Each of the items below is relatively small, estimated at 300-500
lines of new code plus new tests.

### TracingLogRecord: Tokio tracing Event and Metadata to LogRecordView

When we receive a Tokio tracing event whether through a
`tracing::info!` macro (or similar) or through a dedicated
`EffectHandler`-based API, the same happens:

Create a `TracingLogRecord`, a struct derived from `tracing::Event`
and `tracing::Metadata`, containing raw LogRecord fields extracted
from the tracing macro layer. The `otap_df_pdata::views::logs::LogRecordView` is
implemented for `TracingLogRecord` making it the `TracingLogRecord` something
we can transcode into OTel-Arrow batches.

The `otap_df_pdata` crate currently has no OTLP bytes encoder for
directly accepting `otap_df_pdata::views::*` inputs (note the
OTAP-records-to-OTLP-bytes function bypasses the views and encodes
bytes directly). Therefore, this project implies we extend or refactor
`otap_df_ptdata` with an OTLP bytes encoder for its views interfaces.

Then, `TracingLogRecord` implements the log record view, we will encode
the reocrd as OTLP bytes by encoding the view.

### Stateful OTLP bytes encoder for repeated LogRecordViews

We can avoid sending a log record through a channel every time an event
happens by buffering log records. We will buffer them as OTLP bytes. Each 
receiver of events from `TracingLogRecord` OTLP bytes will use one stateful
encoder that is:

- Preconfigured with the process-level OpenTelemetry `Resource` value
- Remembers the OpenTelemetry `InstrumentationScope.Name` that was previously used
- Remembers the starting position of the current `ResourceLogs` and `ScopeLogs` of a 
  single OTLP bytes payload.
  
Whether a global logging collector thread or an effect handler thread
processing internal telemetry, we will enter the stateful encoder and
append a `LogRecordView` with its effective
`InstrumentationScope`. The stateful encoder will append the log
record correctly, recognizing change of scope and a limited buffer
size.  This re-uses the `ProtoBuf` object from the existing
OTAP-records-to-OTLP-bytes code path for easy protobuf generation
(1-pass encoder with length placeholders).

### OTLP-bytes console logging handler

We require a way to print OTLP bytes as human-readable log lines. We
cannot easily re-use the Tokio `tracing` format layer for this,
however we can use the `LogsDataView` trait with `RawLogsData` to
format human-readable text for the console directly from OTLP bytes.

This OTLP-bytes-to-human-readable logic will be used to implement raw
logging.

### Global logs collection thread 

An OTAP-Dataflow engine will run at least one global logs collection
thread. These threads receive encoded (OTLP bytes) log events from
various locations in the process. The global logs collection thread is
special because it sets a special anti-recursion bit in the
thread-local state to prevent logging in its own export path

The global logs collection thread is configured as one (or more, if
needed) instances consuming logs from the global Tokio `tracing`
subscriber. In this thread, we'll configure the OpenTelemetry SDK or a
dedicated OTAP-Dataflow pipeline (by configuration) for logs export.

Because global logs collection threads are used as a fallback for
`EffectHandler`-level logs and because third-party libraries generally
could call Tokio `tracing` APIs, we arrange to explicitly disallow
these threads from logging. The macros are disabled from executing.

### Global and Per-core Event Router

OTAP-Dataflow provides an option to route internal telemetry to a pipeline
in the same effect handler that produced the telemetry. When a component
logging API is used on the `EffectHandler` or when a tokio `tracing` event
occurs on the `EffectHandler` thread, it will be routed using thread-local
state so that event is immediately encoded and stored or flushed, without
blocking the effect handler.

When a telemetry event is routed directly, as in this case and
`send_message()` succeeds, it means there was queue space to accept
the log record on the same core. When this fails, the configurable
telemetry router will support options to use global logs collection
thread, a raw logger, or do nothing (dropping the internal log
record).

## Example configuration

T.B.D. needs development:

```yaml
service:
  telemetry:
    logs:
      level: info
      internal_collection:
        enabled: true
        
        # Per-thread buffer
        buffer_size_bytes: 65536
        
        # Individual record size limit
        max_record_bytes: 16384
        
        # Bounded channel capacity
        max_record_count: 10
        
        # Timer-based flush interval
        flush_interval: "1s"
```
