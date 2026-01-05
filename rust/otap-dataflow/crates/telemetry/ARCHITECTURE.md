# Internal Telemetry Collection Architecture & Development Plan

## Architecture

OTAP-Dataflow uses a configurable internal telemetry data plane.  We
support alternatives to enable a range of observability requirements.
The internal telemetry SDK is designed for the engine to safely
consume its own telemetry, and we intend for the self-hosted telemetry
pipeline to be the standard configuration.

Consuming self-generated ("telemetry presents a potential a kind of
feedback loop, situations where a telemetry pipeline creates pressure
on itself and must not explode. 

## Internal telemetry receiver

The Internal Telemetry Receiver or "ITR" is an OTAP-Dataflow receiver
component that produces telemetry from internal sources. An internal
telemetry pipeline consists of one or more ITR components and any of
the connected processor and exporter components reachable from ITR
source nodes.

To begin with, every OTAP-Dataflow comonent is configured with an
internal telemetry SDK meant for primary instrumentation of that
component. Components are required to exclusively use the internal
telemetry SDK for self-diagnostics, as they are considered first party
in this exchange.

The internal telemetry receiver is the SDK's counterpart, making it
second party as it is responsible for routing internal telemetry. The
ITR cannot use the internal telemetry SDK itself, an invisible member
of the pipeline. The ITR can be instrumented using third-party
instrumentation (e.g., `tracing`, `log` crates) provided it can
guarantee there is no potential for feedback (e.g., a single
`tracing::info()` statement at startup).

## Pitfall avoidance

The OTAP-Dataflow engine is safeguarded against many self-induced
telemetry pitfalls, as follows:

- OTAP-Dataflow components reachable from an ITR cannot be configured
  to send to an ITR node. This avoids a direct feedback cycle for
  internal telemetry because the components cannot reach
  themselves. For example, ITR and downstream components may be
  configured for raw logging, no metrics, etc.
- ITR instances share access to one or more threads with associated
  async runtime. They use these dedicated threads to isolate internal
  telemetry processes that use third-party instrumentation.
- A thread-local variable is used to redirect third-party
  instrumentation in dedicated internal telemetry threads. Internal
  telemetry threads automatically configure a safe configuration.
- Components under observation (non-ITR components) have internal
  telemetry events routed queues in the OTAP-Dataflow pipeline on the
  same core, this avoids blocking the engine. First-party
  instrumentation will be handled on the CPU core that produced the
  telemetry under normal circumstances. This isolates cores that are
  able to process their own internal telemetry.
- Option to fall back to no-op, a non-blocking global provider, and/or
  raw logging.

## OTLP-bytes first

As a key design decision, the OTAP-Dataflow internal telemetry data
path produces OTLP-bytes first. Because OTLP bytes is one of the
builtin `OtapPayload` formats, once we have the OTLP bytes encoding of
an event we are able to send to an OTAP-Dataflow pipeline. To obtain
these bytes, we will build a custom [Tokio `tracing`
Event][TOKIOEVENT] handler to produce OTLP bytes before dispatching to
an internal pipeline, used (in different configurations) for first and
third-party instrumentation.

[TOKIOEVENT]: https://docs.rs/tracing/latest/tracing/struct.Event.html

## Raw logging

We support formatting events for direct printing to the console from
OTLP bytes, based on `otap_df_pdata::views::logs::LogsDataView` and
associated types, a zero-copy approach. We refer to this most-basic
form of printing to the console as raw logging because it is a safe
configuration early in the lifetime of a process.

## Routing

The two internal logs data paths are:

- Third-party: Tokio `tracing` global subscriber: third-party log
  events, instrumentation in code without access to an OTAP-Dataflow
  `EffectHandler`. These are handled in a dedicated internal telemetry
  thread.
- First-party: components with a local or shared `EffectHandler` use
  dedicated macros (e.g., `otel_info!(effect, "interesting thing")`),
  these use the configured internal telemetry SDK and for ordinary
  components (not ITR-downstream) these are routed through the ITR the
  same core. These are always non-blocking APIs, the internal SDK must
  drop logs instead of blocking the pipeline.

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
