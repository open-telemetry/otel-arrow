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

//// @@@@@ ////// @@@@@@
HUMAN AUTHORED ABOVE
MACHINE AUTHORED BELOW
//// @@@@@ ////// @@@@@@

OTAP-Dataflow uses a **two-path telemetry architecture** that provides
both high-performance pipeline instrumentation and compatibility with
standard tracing infrastructure:

1. **Slow Path** (Bootstrap/3rd Party): Standard `tracing::*!()` macros → Global subscriber → Console/SDK (~5-10μs per event)
2. **Fast Path** (Pipeline Components): `otel_*!(effect, ...)` macros → Effect handler buffer → OTLP bytes → Internal Telemetry Receiver (< 100ns target)

The system guarantees **bounded memory usage** and **never blocks component operations**, with graceful degradation under load.

## Core Design Principles

### 1. Bounded Memory Architecture

**Critical Principle**: All telemetry operations must have bounded memory and never block component operations.

**Memory Bounds**:
- **Per-thread buffer**: Pre-allocated StatefulOtlpEncoder (default 64 KiB)
  - Fixed capacity at creation
  - Individual log records limited to 16 KiB (enables encoder optimization)
  - Returns BufferFull error when capacity exceeded

- **Bounded channels**: Configurable capacity (default 1000 records)
  - Worst case: 1000 × 64 KiB = 64 MB
  - Non-blocking try_send for backpressure

- **Total per component**: ~64 KiB buffer + share of channel capacity

**Graceful Degradation**:
1. Buffer full → Flush buffer → Retry encoding
2. Channel full → Fall back to raw console logger
3. Flush fails → Fall back to raw console logger
4. **Never blocks** → Component operation continues regardless

### 2. Anti-Recursion Guarantees

**Problem**: Any logging in response to telemetry collection with amplification ≥1.0 creates feedback loops.

**Solution**: Complete isolation with multiple safeguards:

- **Component effect handlers**: 
  - Buffer fills → flush synchronously → retry encoding
  - Flushed bytes discarded if send fails (no queueing)
  - Never amplifies (one log event in = at most one flush attempt)

- **ITR pipeline components**:
  - ITR receiver and ALL downstream components use raw console logger only
  - NO effect handler telemetry
  - Production: ERROR level only (INFO/WARN suppressed)
  - Complete isolation prevents feedback loops

- **Raw logger fallback**:
  - Initialized early via `LoggerProvider::init_default_console_tracing()`
  - Synchronous console output (stderr)
  - Never fails, never allocates
  - No recursion risk (separate code path)

### 3. Zero-Copy Where Possible

- **Streaming encoding**: Build view, encode to OTLP bytes, append to reusable `Vec<u8>`
- **View pattern**: LogRecordView borrows data, no intermediate storage
- **Buffer reuse**: `Vec<u8>` cleared (not reallocated), capacity retained
- **Single allocation per core**: Buffer grows once to working set, then stable

## Two-Path Architecture

### Slow Path: Bootstrap and 3rd Party Logs

**Use Case**: Application startup, 3rd party libraries, test code, debugging

```
tracing::info!("starting")  [main.rs, 3rd party libs]
    ↓
OtlpTracingLayer (global subscriber)
    ↓
TracingLogRecord (implements LogRecordView)
    ↓ Captures full structure via Visit trait
    ↓ TracingAnyValue preserves arrays, maps, nested data
    ↓
StatefulOtlpEncoder
    ↓ encode_log_record() → OTLP bytes
    ↓
Thread-local routing:
    ├─ CORE_LOCAL_ITR_SENDER (try first - on-core ITR if configured)
    ├─ GLOBAL_TELEMETRY_SENDER (overflow - global collector)
    └─ Raw logger (last resort - never fails)
    ↓
On-core ITR (preferred) OR Global collector (fallback) OR Console (emergency)
```

**Characteristics**:
- Latency: ~5-10μs per event (global subscriber overhead)
- Full structure preservation (no string formatting)
- Thread-local routing provides core isolation
- Overflow mechanism prevents blocking
- Good for cold paths (not performance critical)

**Status**: ✅ Complete and working (demonstrated in examples)

### Fast Path: Pipeline Component Instrumentation

**Use Case**: Receivers, processors, exporters - performance-critical code

```
otel_info!(effect, "event", key=val)  [Pipeline components - FUTURE]
    ↓
Create tracing::Event + Metadata (bypass global subscriber)
    ↓
TracingLogRecord (LogRecordView)
    ↓
effect.log_event(&log_record_view)
    ↓
StatefulOtlpEncoder (per-thread buffer)
    ↓ Streaming encoding to Vec<u8>
    ↓ Check size threshold
    ↓
On overflow/threshold: flush OTLP bytes
    ↓
Thread-local routing:
    ├─ CORE_LOCAL_ITR_SENDER (try first - on-core ITR)
    ├─ INTERNAL_TELEMETRY_SENDER (overflow - dedicated thread, small capacity: 1-2 buffers)
    └─ Raw logger (last resort - never fails)
    ↓
On-core ITR (preferred) OR Internal telemetry overflow thread OR Console (emergency)
```

**Characteristics**:
- Target latency: < 100ns per event (when not flushing)
- Zero global state (effect handler owned)
- Streaming OTLP encoding
- Size-based batching with overflow protection
- Direct buffer access (no locks)
- Same thread-local routing as slow path (core isolation)

**Status**: 🚧 Infrastructure ready (Phase 0 complete), macros not yet updated (Phase 1)

## Three Operating Modes

The system supports flexible configuration through **orthogonal options** that compose independently:

### Configuration Dimensions

1. **ITR (On-Core Processing)**: Enabled or disabled per core
2. **Global Collection**: SDK, OTAP pipeline, console, or none
3. **Channel Isolation**: Per-core or shared channels
4. **Overflow Routing**: Global thread or direct console
5. **Anti-Recursion**: Automatic via thread-locals and component marking

See
[internal-telemetry-receiver-design.md](internal-telemetry-receiver-design.md#orthogonal-configuration-dimensions)
for detailed configuration matrix and examples.

### Common Mode Configurations

The system supports three distinct common configurations (many others possible):

### Mode 1: SDK Only (Default)

When `internal_collection.enabled: false` (default), all logging uses OpenTelemetry SDK.

**Thread-local routing**:
- CORE_LOCAL_ITR_SENDER: None (not configured)
- INTERNAL_TELEMETRY_SENDER: Routes effect handler overflow to internal telemetry thread
- GLOBAL_TOKIO_SENDER: Routes 3rd party logs to global tokio subscriber thread
- Both threads route to SDK exporters

**Internal telemetry collection thread**:
- Receives overflow from each engine core (dedicated channel per core)
- Small capacity: 1-2 buffers (~64-128 KiB) provides fast backpressure
- Decodes OTLP bytes → SDK LogRecord → SDK exporter
- Sets `IN_TELEMETRY_COLLECTION = true` during processing (prevents recursion)

**Global tokio subscriber thread** (separate):
- Receives 3rd party library logs from GLOBAL_TOKIO_SENDER
- Larger capacity (not overflow case)
- Routes to SDK exporters
- Sets `IN_TELEMETRY_COLLECTION = true` during processing

**Configuration**:
```yaml
service:
  telemetry:
    logs:
      level: info
      global_collection:
        destination: "sdk"
        sdk:
          processors:
            - batch:
                exporter:
                  console:
      internal_collection:
        enabled: false  # No on-core ITR
```

**Use case**: Simple deployments, development, testing

### Mode 2: Hybrid (On-Core ITR + SDK Overflow)

When `internal_collection.enabled: true` with SDK processors configured.

**Thread-local routing**:
- CORE_LOCAL_ITR_SENDER: Some (on-core ITR configured)
- Try on-core ITR first (core isolation, fast path)
- Overflow → INTERNAL_TELEMETRY_SENDER → SDK exporters

**Internal telemetry overflow thread**:
- Receives overflow from each core's effect handlers
- Per-core isolated channels (noisy logger doesn't affect other cores)
- Small capacity: 1-2 buffers per core (~64-128 KiB)
- Routes to SDK exporters (same as Mode 1)
- Sets `IN_TELEMETRY_COLLECTION = true` during processing

**Channel isolation benefit**: Each core has dedicated overflow channel - excessive logging on one core gets throttled without affecting other cores' telemetry or metrics reporting.

**Configuration**:
```yaml
service:
  telemetry:
    logs:
      level: info
      global_collection:
        destination: "sdk"
        sdk:
          processors:
            - batch:
                exporter:
                  otlp:
                    endpoint: "http://localhost:4317"
      internal_collection:
        enabled: true  # On-core ITR
        buffer_size_bytes: 65536
        overflow_destination: "global"  # Falls back to SDK
```

**OTAP pipeline** (pipelines.yaml):
```yaml
pipelines:
  - name: main
    receivers:
      - otlp
    exporters:
      - otlp:
          endpoint: "http://collector:4317"
  
  - name: internal_telemetry
    receivers:
      - internal_telemetry  # On-core ITR instances
    processors:
      - batch:
          timeout: 5s
    exporters:
      - otlp:
          endpoint: "http://observability:4317"
```

**Use case**: Production deployments wanting full control over component telemetry with SDK fallback

### Mode 3: OTAP Only (ITR without SDK)

When `internal_collection.enabled: true` without SDK processors configured.

**Thread-local routing**:
- CORE_LOCAL_ITR_SENDER: Some (on-core ITR configured)
- Try on-core ITR first (core isolation)
- Overflow → INTERNAL_TELEMETRY_SENDER → dedicated OTAP pipeline (no SDK)

**Internal telemetry overflow thread**:
- Receives overflow from each core's effect handlers
- Per-core isolated channels (1-2 buffers each)
- Routes to dedicated `internal_telemetry_overflow` pipeline
- Sets `IN_TELEMETRY_COLLECTION = true` during processing
- No SDK exporters configured

**Global tokio subscriber**:
- 3rd party logs also route through dedicated OTAP pipeline
- Separate GLOBAL_TOKIO_SENDER channel (larger capacity)

**Configuration**:
```yaml
service:
  telemetry:
    logs:
      level: info
      global_collection:
        destination: "otap"  # No SDK
        otap:
          pipeline_name: "internal_telemetry_overflow"
      internal_collection:
        enabled: true  # On-core ITR
        buffer_size_bytes: 65536
        overflow_destination: "global"  # Falls back to dedicated pipeline
```

**OTAP pipeline** (pipelines.yaml):
```yaml
pipelines:
  - name: main
    receivers:
      - otlp
    exporters:
      - otlp:
          endpoint: "http://collector:4317"
  
  - name: internal_telemetry
    receivers:
      - internal_telemetry  # On-core ITR instances
    processors:
      - batch:
          timeout: 5s
    exporters:
      - otlp:
          endpoint: "http://observability:4317"
  
  - name: internal_telemetry_overflow
    receivers:
      - telemetry_channel  # Global collector overflow
    processors:
      - batch:
          timeout: 5s
    exporters:
      - otlp:
          endpoint: "http://observability:4317"
```

**Use case**: Full OTAP control, zero SDK dependency, all telemetry flows through dataflow engine

## Additional Configuration Options

### Console-Only (Minimal)

**Configuration**: `global_collection.destination: "none"`

- CORE_LOCAL_ITR_SENDER: None
- INTERNAL_TELEMETRY_SENDER: None (direct console)
- GLOBAL_TOKIO_SENDER: None (direct console)
- All telemetry → raw logger (synchronous, no threads)
- Use case: Development, debugging, minimal footprint

### Per-Core Isolation Without ITR

**Configuration**: `global_collection.channel_per_core: true`, `internal_collection.enabled: false`

- CORE_LOCAL_ITR_SENDER: None (no ITR)
- INTERNAL_TELEMETRY_SENDER: Some (per-core dedicated channel, 1-2 buffers)
- GLOBAL_TOKIO_SENDER: Some (shared, larger capacity)
- Use case: Core isolation without ITR pipeline overhead
- Benefit: Noisy logger on one core doesn't affect others, fast backpressure

See [Configuration Examples](internal-telemetry-receiver-design.md#configuration-examples) for complete details and more variants.

## Key Components

### 1. StatefulOtlpEncoder

**Purpose**: Streaming OTLP encoding with buffer reuse

**Features**:
- Pre-allocated `Vec<u8>` buffer (configurable capacity)
- Encodes log records incrementally to OTLP bytes
- Buffer cleared (not reallocated) after flush
- Returns BufferFull error when capacity exceeded

**Status**: ✅ Complete

### 2. TracingLogRecord + TracingAnyValue

**Purpose**: Capture complete structure from tracing events

**Features**:
- Implements LogRecordView trait (zero-copy interface)
- TracingAnyValue mirrors OTLP's AnyValue (scalars, arrays, maps)
- Full fidelity preservation (no string formatting)
- Visit trait extracts typed data: `record_i64()`, `record_bool()`, etc.

**Key Insight**: No `fmt` layer needed - direct structural encoding to OTLP

**Status**: ✅ Complete with nested array/map support

### 3. OtlpTracingLayer (Global Subscriber - Slow Path)

**Purpose**: Capture tracing events from global subscriber

**Features**:
- Integrates with tracing_subscriber registry
- Constructs TracingLogRecord from tracing::Event
- Encodes to OTLP bytes using StatefulOtlpEncoder
- **Thread-local routing**:
  - Checks `CORE_LOCAL_ITR_SENDER` first (on-core ITR if configured)
  - Falls back to `GLOBAL_TELEMETRY_SENDER` (global collector)
  - Last resort: `OtlpBytesFormattingLayer` (raw logger - never fails)
- Synchronous flush per event (one OTLP bytes object per event)

**Status**: ✅ Complete and demonstrated in examples

### 4. OtlpBytesFormattingLayer (Raw Logger)

**Purpose**: Decode OTLP bytes and format for human consumption

**Features**:
- Decodes OTLP bytes back to LogRecordView
- Colorized console output with timestamps
- Level-based routing (INFO/DEBUG → stdout, WARN/ERROR → stderr)
- Thread name display
- Synchronous, never fails
- **Used as last-resort fallback** in thread-local routing chain

**Status**: ✅ Complete (used by slow path and as fallback)

### 5. InternalTelemetryReceiver (ITR)

**Purpose**: Receive telemetry from effect handlers and inject into pipeline

**Features**:
- Shared receiver (Send + Sync for multi-threaded operation)
- Receives OTLP bytes via `CORE_LOCAL_ITR_SENDER` thread-local
- Wraps as OtapPdata: `OtlpProtoBytes::ExportLogsRequest(bytes)`
- Periodic timer-based flushing
- Buffering with backpressure handling
- **Overflow handling**: When full → falls back to `GLOBAL_TELEMETRY_SENDER`
- Anti-recursion: ITR's own processing uses designated effect handler without telemetry

**Status**: ✅ Structurally complete, needs wire-up to effect handlers

### 6. TelemetryBuffer (Effect Handler)

**Purpose**: Per-thread telemetry buffer owned by effect handler

**Features**:
- Owns StatefulOtlpEncoder (reusable buffer)
- Streaming encoding: view → OTLP bytes → append
- Size-based threshold flushing
- **Thread-local routing** (same as slow path): `CORE_LOCAL_ITR_SENDER` → `GLOBAL_TELEMETRY_SENDER` → Raw logger
- Synchronous overflow handling (flush + retry)
- Shared or Local variants (Arc<Mutex<>> or Rc<RefCell<>>)

**Status**: ✅ Structure defined, encoding logic not yet wired to macros

### 7. Internal Telemetry Collection Thread

**Purpose**: Receives overflow telemetry from effect handlers on engine cores and routes to destinations

**Features**:
- Receives OTLP bytes from per-core overflow channels (INTERNAL_TELEMETRY_SENDER)
- Per-core isolated channels with small capacity (1-2 buffers ~64-128 KiB each)
- Routes based on configuration:
  - Mode 1 (SDK Only): Decodes OTLP bytes → SDK LogRecord → SDK exporter
  - Mode 2 (Hybrid): Overflow from on-core ITRs → SDK exporter
  - Mode 3 (OTAP Only): Wrap as OtapPdata → inject into dedicated OTAP pipeline
- **Anti-recursion**: Sets `IN_TELEMETRY_COLLECTION = true` during processing
- **Throttling isolation**: Noisy logger on one core gets backpressure without affecting other cores

**Status**: ✅ Structure defined, separate from metrics and tokio channels

### 8. Global Tokio Subscriber Thread

**Purpose**: Receives 3rd party library logs from global tracing subscriber

**Features**:
- Receives OTLP bytes from GLOBAL_TOKIO_SENDER
- Larger capacity (not overflow case - handles standard library logging)
- Routes to SDK or configured destination
- Sets `IN_TELEMETRY_COLLECTION = true` during processing
- Separate from internal telemetry overflow and metrics channels

**Status**: ✅ Structure defined, dedicated thread for 3rd party logs

### 9. Thread-Local Routing Infrastructure

**Purpose**: Provide core isolation and cascading fallbacks for telemetry delivery

**Thread-locals**:
- `CORE_LOCAL_ITR_SENDER: Option<Sender<Bytes>>` - On-core ITR (preferred, provides core isolation)
- `INTERNAL_TELEMETRY_SENDER: Sender<Bytes>` - Internal telemetry overflow thread (effect handler logs)
- `GLOBAL_TOKIO_SENDER: Option<Sender<Bytes>>` - Global tokio subscriber thread (3rd party logs)
- `IN_TELEMETRY_COLLECTION: bool` - Anti-recursion flag (prevents infinite loops)

**Separate Channels Architecture**: 
- **Internal telemetry overflow**: Per-core dedicated channels, small capacity (1-2 buffers ~64-128 KiB)
- **Global tokio subscriber**: Larger capacity channel (not overflow case)
- **Metrics reporting**: Continues using existing separate metrics channel
- **Benefit**: Isolation - noisy logger on one core doesn't affect other cores or metrics

**Routing logic for effect handlers** (internal telemetry):
```rust
if let Some(sender) = CORE_LOCAL_ITR_SENDER.get() {
    // Try on-core ITR first (core isolation, no contention)
    sender.send(otlp_bytes)?
} else if let Some(sender) = INTERNAL_TELEMETRY_SENDER.get() {
    // Overflow to dedicated internal telemetry thread (small channel: 1-2 buffers)
    sender.send(otlp_bytes)?
} else {
    // Last resort: raw logger (decode + format to console, never fails)
    raw_logger.format(otlp_bytes)
}
```

**Routing logic for global tokio subscriber** (3rd party logs):
```rust
if let Some(sender) = CORE_LOCAL_ITR_SENDER.get() {
    // Try on-core ITR first (if on engine core)
    sender.send(otlp_bytes)?
} else if let Some(sender) = GLOBAL_TOKIO_SENDER.get() {
    // Route to global tokio subscriber thread (larger capacity)
    sender.send(otlp_bytes)?
} else {
    // Last resort: raw logger
    raw_logger.format(otlp_bytes)
}
```

**Status**: 🚧 Design complete, awaiting Phase 1 implementation

## Structure Preservation (No String Formatting)

**Critical Design Decision**: We do NOT format values to strings. OTLP's rich type system preserves complete structure.

**OTLP AnyValue supports**:
- Scalars: String, Int, Double, Bool, Bytes
- Composite: Array (Vec<AnyValue>), KeyValueList (Map<String, AnyValue>)
- Equivalent to JSON: Can represent any JSON structure natively

**Data Flow**:
```
tracing::info!(count = 42, error = ?err, items = ?vec)
    ↓
Visit trait extracts typed values:
  - record_i64("count", 42)
  - record_debug("error", err)  → TracingAnyValue::String(format!("{:?}", err))
  - record_debug("items", vec)  → TracingAnyValue::Array(...)
    ↓
TracingLogRecord with nested TracingAnyValue
    ↓
Implements LogRecordView (zero-copy)
    ↓
StatefulOtlpEncoder.encode_log_record()
    ↓
OTLP bytes (protobuf) - full structural fidelity
    ↓
Can decode back to LogRecordView
    ↓
Format for console OR process in pipeline
```

**Result**: Complete fidelity from tracing events → OTLP structures → processing/export

## Implementation Status

### Phase 0: Foundation ✅ COMPLETED (December 18, 2025)

**What was built**:

1. **StatefulOtlpEncoder** - Core OTLP encoding with buffer reuse
2. **TracingLogRecord + TracingAnyValue** - Full structure capture via Visit trait
3. **OtlpTracingLayer** - Global subscriber for slow path with thread-local routing
4. **OtlpBytesFormattingLayer** - Console formatter (raw logger, fallback destination)
5. **InternalTelemetryReceiver** - ITR structure for fast path with overflow fallbacks
6. **TelemetryBuffer in EffectHandler** - Per-thread buffer ownership
7. **Thread-local routing infrastructure** - CORE_LOCAL_ITR_SENDER, GLOBAL_TELEMETRY_SENDER, IN_TELEMETRY_COLLECTION
8. **Global collector with anti-recursion** - TelemetryMessage enum (Metrics + Logs), IN_TELEMETRY_COLLECTION flag
9. **Examples** - Demonstrating both architectures

**Demonstrated in examples**:
- `otlp_bytes_formatting.rs` - Shows slow path working end-to-end
- `channel_based_tracing.rs` - Shows async channel pattern
- Others demonstrate OTLP round-tripping

**Test Results**: 249 tests passing, all compilation errors fixed

**Architecture refinements**:
- Thread-local routing provides core isolation (CORE_LOCAL_ITR_SENDER on-core delivery)
- Three-tier fallback: on-core ITR → global collector → raw logger (never fails)
- Optional SDK (Mode 1), optional ITR (Mode 2), or both (Mode 3)
- Global collector handles overflow with recursion prevention

### Phase 1: Macro Integration (NOT YET STARTED)

**Objective**: Update `otel_*!` macros to use effect handler (fast path)

**Remaining Work**:

1. **Update macro signatures** to require effect handler parameter:
   - `otel_info!(effect, "event", key=val)` - explicit, compile-time enforced
   - No fallback to global subscriber - forces architectural clarity

2. **Bypass global subscriber**:
   - Construct tracing::Event + Metadata directly
   - Pass to `effect.log_event(&log_record_view)`
   - No global dispatcher involvement

3. **Implement `log_event()` for effect handlers**:
   - Local: `Rc<RefCell<StatefulOtlpEncoder>>`
   - Shared: `Arc<Mutex<StatefulOtlpEncoder>>`
   - Streaming encoding to reusable buffer
   - Size-based threshold check
   - Overflow handling: flush → retry

4. **Wire up ITR channel**:
   - Effect handlers send OTLP bytes to ITR receiver
   - ITR wraps as OtapPdata and injects into pipeline
   - Timer-based periodic flushing

5. **Add bounded memory safeguards**:
   - Pre-allocated buffer with fixed capacity
   - BufferFull error → flush → retry
   - Channel full → raw logger fallback
   - Anti-recursion for ITR pipeline components

**Target**: < 100ns per log event (when not flushing)

### Future Phases (Beyond Scope)

- Phase 2: Performance optimization and tuning
- Phase 3: Distributed tracing correlation
- Phase 4: Metrics integration
- Phase 5: OpenTelemetry SDK direct integration

## Migration Strategy

**Philosophy**: Additive, not breaking. Two paths coexist permanently.

**Current State**:
- Bootstrap code uses `tracing::info!()` → slow path (working)
- Pipeline components don't log yet (or use slow path)

**Phase 1 Migration**:
- Update pipeline component logging to `otel_info!(effect, ...)`
- Requires effect handler parameter (compile-time enforced)
- Bootstrap code stays on slow path (not performance critical)
- Third-party libraries continue using `tracing::*!()` (unchanged)

**Clear Performance Model**:
- `otel_info!(effect, ...)` = fast path (< 100ns, requires effect handler)
- `tracing::info!()` = slow path (~5μs, global state)
- Intent is obvious at call site

**No Pressure to Migrate**:
- Cold paths can stay on slow path indefinitely
- Only hot paths need fast path
- Both paths fully supported

## Configuration Reference

### Bounded Memory Parameters

```yaml
service:
  telemetry:
    logs:
      level: info
      internal_collection:
        enabled: true
        
        # Per-thread buffer (fixed allocation)
        buffer_size_bytes: 65536      # 64 KiB (default)
        
        # Individual record size limit
        max_record_bytes: 16384       # 16 KiB (enables encoder optimization)
        
        # Bounded channel capacity
        max_record_count: 1000        # Records (default)
        
        # Timer-based flush interval
        flush_interval: "1s"
```

**Memory Guarantees**:
- Per thread: 64 KiB fixed
- Channel: 1000 × 64 KiB = 64 MB worst case
- Total bounded and predictable

### Routing Configuration

**Mode 1: SDK Only** (default)
```yaml
service:
  telemetry:
    logs:
      level: info
      # internal_collection disabled by default
      processors:
        - batch:
            exporter:
              console:
```

**Mode 2: Hybrid** (ITR for components, SDK for 3rd party)
```yaml
service:
  telemetry:
    logs:
      level: info
      internal_collection:
        enabled: true
      processors:
        - batch:
            exporter:
              otlp:
                endpoint: "http://localhost:4317"
```

**Mode 3: ITR Only** (no SDK)
```yaml
service:
  telemetry:
    logs:
      level: info
      internal_collection:
        enabled: true
      processors: []  # Empty = route 3rd party to ITR too
```

## Performance Targets

Based on design goals:

- **Fast path (buffered)**: < 100ns per event
- **Fast path (flush)**: < 10μs for typical batch (100 events)
- **Slow path**: ~5-10μs per event (acceptable for cold paths)
- **Overflow flush**: Synchronous, blocks caller (acceptable - rare)
- **Memory**: < 200KB per core overhead
- **Zero contention**: No locks on fast path (effect handler owned)

## Testing Strategy

### Unit Tests
- StatefulOtlpEncoder: encoding, buffer reuse, capacity limits
- TracingLogRecord: structure capture, nested data
- TelemetryBuffer: overflow handling, threshold logic

### Integration Tests
- Effect handler: log_event() with mock ITR
- ITR receiver: OTLP bytes → OtapPdata conversion
- End-to-end: Full pipeline with telemetry export

### Performance Tests
- Event recording latency benchmarks
- Throughput under load
- Memory usage profiling
- Contention measurement (should be zero)

### End-to-End Tests
- Full pipeline with OTLP export
- External collector integration
- Stress testing with backpressure

## Error Handling

**Principle**: Telemetry must never fail component operation.

**Strategies**:

1. **Buffer overflow**: Flush synchronously → Retry encoding
2. **Channel full**: Non-blocking try_send → Raw logger fallback
3. **Encoding errors**: Drop with counter → Raw logger
4. **Flush failures**: Raw logger fallback
5. **Record too large**: Drop with counter (> 16 KiB)

**Raw Logger**:
- Initialized early in main(): `LoggerProvider::init_default_console_tracing()`
- Synchronous console output (stderr)
- Never fails, never allocates
- No recursion risk

**Metrics** (when telemetry works):
- `telemetry.events_received`: Counter
- `telemetry.events_dropped`: Counter (by reason)
- `telemetry.bytes_flushed`: Counter
- `telemetry.overflow_flush_count`: Counter
- `telemetry.timer_flush_count`: Counter

## Open Questions

### 1. Resource and Scope Naming

**Question**: What Resource and scope name should components use?

**Proposed**:
- Resource: Pipeline-level (service.name, service.version)
- Scope: Component's node_id.to_string() (e.g., "receiver.otlp.0")

### 2. Cross-Core Tracing

**Question**: How to handle traces that span multiple cores?

**Proposed**: Accept eventual consistency, use trace IDs for correlation

### 3. Dynamic Reconfiguration

**Question**: Support runtime reconfiguration of telemetry?

**Proposed**: Future work - control messages for buffer size, flush interval

## References

1. [Custom Tracing Subscriber Plan](./custom-tracing-subscriber-plan.md) - Original fast-path design
2. [Internal Telemetry Receiver Design](./internal-telemetry-receiver-design.md) - Detailed ITR specification
3. [Stateful OTLP Encoder Phase 1 Summary](./stateful-encoder-phase1-summary.md) - Encoder implementation
4. [OpenTelemetry Logs Specification](https://opentelemetry.io/docs/specs/otel/logs/)
5. [Tokio Tracing Documentation](https://docs.rs/tracing)
