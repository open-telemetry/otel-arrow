# Internal Telemetry Receiver Design

## Overview

This document describes the design for component-level telemetry logging using the Internal Telemetry Receiver (ITR). The system supports three distinct paths for telemetry:

1. **3rd Party Logs** (Global Tracing) - Admin component OtlpBytesChannel
2. **Shared ITR** - Multi-threaded components using Arc<Mutex<>>
3. **Local ITR** - Single-threaded components using Rc<RefCell<>>

## Bounded Memory Architecture

**Critical design principle:** All telemetry operations must have **bounded memory usage** and **never block component operations**.

### Memory Bounds

1. **Per-thread buffer**: Pre-allocated StatefulOtlpEncoder (default 64 KiB)
   - Fixed capacity, allocated at effect handler creation
   - Shared by all log events from that component/thread
   - Individual log records limited to 16 KiB (enables 2-byte length placeholders)
   - Returns BufferFull error when buffer full

2. **Bounded channel**: OtlpBytesChannel or ITR channel (default 1000 records)
   - Configurable capacity prevents unbounded growth
   - Non-blocking try_send used for backpressure handling
   - Worst case: 1000 × 64 KiB = 64 MB

3. **Total per component**: ~64 KiB buffer + share of channel capacity
   - Predictable, bounded memory footprint
   - No risk of OOM from telemetry
   - Graceful degradation under load

### Graceful Degradation

When limits are reached, the system degrades gracefully:

1. **Buffer full** → Flush buffer → Retry encoding in empty buffer
2. **Channel full** → Fall back to raw console logger (synchronous, never fails)
3. **Flush fails** → Fall back to raw console logger
4. **Never blocks** → Component operation continues regardless of telemetry state

### Raw Logger Fallback

- Initialized early via `LoggerProvider::init_default_console_tracing()`
- Synchronous console output (writes to stderr)
- Never fails, never allocates
- Used when telemetry path is unavailable
- No recursion risk (separate code path)

## Architecture Paths

### Path 1: 3rd Party Logs (Global Tracing Subscriber)

**Use case:** Logs from 3rd party libraries via `tokio::tracing!()` macros

```
tokio::info!("message")
  ↓ Global tracing subscriber
  ↓ OtlpTracingLayer
  ↓ TracingLogRecord (LogRecordView)
  ↓ StatefulOtlpEncoder.encode_log_record() 
  ↓ encoder.flush() → Bytes (one per event)
  ↓ OtlpBytesChannel.send()
  ↓ Admin runtime task
  ↓ Console formatter OR forward to internal receiver
```

**Key characteristics:**
- One OTLP bytes object per event (synchronous flush)
- No batching at encoding level
- Uses existing OtlpBytesChannel pattern
- Already implemented ✓

### Path 2: Shared ITR (Multi-threaded Components)

**Use case:** Component logging with `otel_info!(effect, "event", key=val)` in Send+Sync components

```
otel_info!(effect, "event", key=val)
  ↓ Create tracing::Event + Metadata (bypass global subscriber)
  ↓ TracingLogRecord (LogRecordView)
  ↓ effect.log_event(&log_record_view)
  ↓ Arc<Mutex<StatefulOtlpEncoder>>.lock()
  ↓ encoder.encode_log_record()
  ↓ Check size threshold
      ↙ Overflow                    ↘ Under threshold
  encoder.flush()                   Continue accumulating
  send_message(bytes)                     ↓
                                    Timer tick in receiver loop
                                          ↓
                                    encoder.flush()
                                    send_message(bytes)
```

**Key characteristics:**
- Shared encoder: `Arc<Mutex<StatefulOtlpEncoder>>`
- Batching with overflow protection
- Synchronous flush on overflow (caller context)
- Timer-based flush for normal case (receiver loop)
- Thread-safe via Mutex

### Path 3: Local ITR (Single-threaded Components)

**Use case:** Component logging with `otel_info!(effect, "event", key=val)` in !Send components

```
otel_info!(effect, "event", key=val)
  ↓ Create tracing::Event + Metadata (bypass global subscriber)
  ↓ TracingLogRecord (LogRecordView)
  ↓ effect.log_event(&log_record_view)
  ↓ Rc<RefCell<StatefulOtlpEncoder>>.borrow_mut()
  ↓ encoder.encode_log_record()
  ↓ Check size threshold
      ↙ Overflow                    ↘ Under threshold
  encoder.flush()                   Continue accumulating
  send_message(bytes)                     ↓
                                    Timer tick in receiver loop
                                          ↓
                                    encoder.flush()
                                    send_message(bytes)
```

**Key characteristics:**
- Local encoder: `Rc<RefCell<StatefulOtlpEncoder>>`
- Batching with overflow protection
- Synchronous flush on overflow (caller context)
- Timer-based flush for normal case (receiver loop)
- No synchronization overhead (!Send)

### Path 4: Component Logs to SDK (No ITR Configured)

**Use case:** Component logging when `internal_collection.enabled: false` (default)

This is the SDK fallback path - component logs share the same export path as 3rd party logs.

```
otel_info!(effect, "event", key=val)
  ↓ Create tracing::Event + Metadata (bypass global subscriber)
  ↓ TracingLogRecord (LogRecordView)
  ↓ effect.log_event(&log_record_view)
  ↓ Shared/Local StatefulOtlpEncoder
  ↓ encoder.encode_log_record()
  ↓ Check size threshold
      ↙ Overflow                    ↘ Under threshold
  encoder.flush() → Bytes           Continue accumulating
  OtlpBytesChannel.send()                 ↓
      ↓                              Timer in admin runtime
  Admin runtime task                      ↓
      ↓                              encoder.flush() → Bytes
  Decode OTLP bytes                 OtlpBytesChannel.send()
      ↓                                    ↓
  Create SDK LogRecord              Admin runtime task
      ↓                                    ↓
  Call SDK exporter                 (same as overflow)
```

**Key implementation details:**

1. **OtlpBytesChannel for component logs**: Same channel used by 3rd party logs
   - Effect handlers send batched OTLP bytes to channel
   - Overflow/timer flush both use same channel

2. **Admin runtime consumer task** (new requirement):
   - Receives OTLP bytes from OtlpBytesChannel
   - Decodes bytes back to LogRecordView (using OTLP decoder)
   - Constructs SDK LogRecord (emulates opentelemetry-tracing-appender bridge)
   - Calls SDK LogProcessor/Exporter directly
   
3. **SDK LogRecord construction**:
   ```rust
   // Decode OTLP bytes to access fields
   let decoded = decode_otlp_logs_request(&bytes)?;
   
   // For each log record in the decoded request:
   for log_record in decoded.resource_logs.scope_logs.log_records {
       // Construct SDK LogRecord (emulate tracing bridge)
       let sdk_record = opentelemetry_sdk::logs::LogRecord::builder()
           .with_timestamp(log_record.time_unix_nano)
           .with_severity_number(log_record.severity_number)
           .with_body(log_record.body)
           .with_attributes(log_record.attributes)
           // ... other fields
           .build();
       
       // Call SDK exporter
       sdk_logger_provider.log_processor().emit(sdk_record);
   }
   ```

**Key characteristics:**
- Default mode when `internal_collection.enabled: false`
- Shares OtlpBytesChannel with 3rd party logs
- Requires OTLP decode + SDK LogRecord construction
- Uses OpenTelemetry SDK exporters (console, OTLP, etc.)
- No ITR receiver needed in pipeline
- Anti-recursion: ITR pipeline components always use this path

## Implementation Components

### 1. Effect Handler Integration

#### EffectHandlerCore

Add telemetry buffer to the core:

[SNIP]

#### local::EffectHandler

[SNIP]

#### shared::EffectHandler

[SNIP]

### 2. Macro Modifications

Update `otel_info!` and family to accept effect handler:

[SNIP]

### 3. Internal Telemetry Receiver

The ITR is a standard OTAP receiver that:
- Has no inputs (generates data from effect handlers)
- Has one output port (forwards OTLP bytes as OtapPdata)
- Supports both shared and local modes
- Manages ONLY timer-based flushing (no overflow channel)
- **CRITICAL**: ITR and all downstream components MUST use raw console logger (no effect handler telemetry)


**Anti-recursion guarantee**: ITR receiver and all downstream components are configured with raw console logger only. They CANNOT log to the pipeline. Production configuration: errors only, INFO/WARN suppressed.

## Open Questions

### 1. Strict Anti-Recursion Requirements

**Problem:** Any logging in response to telemetry collection events with amplification factor ≥1.0 could create a feedback loop that destroys the system.

**Solution:** Complete isolation - no channels, synchronous flush only

**Component effect handlers (all modes)**:
- NO overflow channel - any channel could enable feedback loops
- Buffer fills up → flush synchronously in-place → retry encoding
- Flushed bytes are discarded (timer-based flush is primary mechanism)
- Never fails, never blocks, never amplifies

**Global subscriber for 3rd party logs (Mode 3 only)**:
- Bounded channel to send OTLP bytes to ITR: `logs_bridge_channel`
- Channel size: configurable (e.g., 1000 messages)
- Admin component locates ITR and registers bridge channel
- If channel full: fall back to raw console logger
- This is the ONLY channel in the system

**ITR pipeline components (all modes)**:
- ITR receiver and ALL downstream components MUST use raw console logger
- NO effect handler telemetry at all - complete isolation
- Production: raw logger configured for ERROR level only
- INFO/WARN logs suppressed for ITR pipelines
- Any logging from ITR path uses synchronous stderr output

**Key benefits**: 
- Zero amplification - telemetry operations never trigger more telemetry
- Synchronous flush discards data (no queueing, no blocking)
- Single bounded channel only for 3rd party logs in Mode 3
- Complete isolation prevents all feedback loops


### 2. ITR Pipeline Must Not Use ITR

**Problem:** **Critical recursion prevention** - The ITR receiver and its export path must NOT use ITR for their own telemetry, otherwise infinite recursion occurs.

**Solution:** Configuration-driven routing with three modes

1. **SDK only (default)**: `processors.len() > 0` and `internal_collection.enabled: false`
   - Both component logs and 3rd party logs use OpenTelemetry SDK
   - Admin task decodes OTLP bytes → constructs SDK LogRecord → calls SDK exporter
   - Uses OpenTelemetry SDK configuration (console, OTLP, etc.)
   - No ITR receiver needed in pipeline

2. **Hybrid mode**: `processors.len() > 0` and `internal_collection.enabled: true`
   - Component logs bypass SDK and go directly to ITR receiver
   - 3rd party logs still use OpenTelemetry SDK (separate path)
   - Admin component routes to OtlpBytesChannel for 3rd party, ITR for components
   - ITR pipeline components must use SDK fallback (anti-recursion)

3. **ITR only (no SDK)**: `processors.is_empty()` and `internal_collection.enabled: true`
   - Both component logs and 3rd party logs route to ITR receiver
   - No OpenTelemetry SDK used at all - "be our own SDK" fully
   - Admin component locates ITR receiver and registers it as global subscriber for 3rd party logs
   - Single unified path through OTAP pipeline
   - ITR pipeline components must use SDK fallback (anti-recursion)

**Configuration approach:**

**Mode 1: SDK only (default)** - Both 3rd party and component logs use SDK:
```yaml
service:
  telemetry:
    logs:
      level: info
      # internal_collection defaults to disabled
      
      # OpenTelemetry SDK configuration for all logs
      processors:
        - batch:
            exporter:
              console:
```

**Mode 2: Hybrid** - Component logs to ITR, 3rd party logs to SDK:
```yaml
service:
  telemetry:
    logs:
      level: info
      # Enable internal collection - component logs routed to ITR receiver
      internal_collection:
        enabled: true
        buffer_size_bytes: 65536       # 64 KiB per-thread buffer (pre-allocated)
        max_record_bytes: 16384        # 16 KiB max single record (enables encoder optimization)
        max_record_count: 1000         # Bounded channel (records)
        flush_interval: "1s"           # ITR periodic flush
      
      # OpenTelemetry SDK configuration for 3rd party logs
      # Component logs bypass this when internal_collection.enabled: true
      processors:
        - batch:
            exporter:
              otlp:
                endpoint: "http://localhost:4317"
```

**Mode 3: ITR only (no SDK)** - Both 3rd party and component logs to ITR:
```yaml
service:
  telemetry:
    logs:
      level: info
      # Enable internal collection with no SDK processors
      internal_collection:
        enabled: true
        buffer_size_bytes: 65536
        max_record_bytes: 16384
        max_record_count: 1000
        flush_interval: "1s"
      
      # Empty processors array = no SDK, route 3rd party logs to ITR too
      processors: []
```

Corresponding OTAP pipeline configuration (separate file, e.g., `configs/pipelines.yaml`):
```yaml
pipelines:
  # Main data pipeline
  - name: main
    receivers:
      - otlp
    processors:
      - batch:
          timeout: 1s
          send_batch_size: 1000
      - retry:
          max_attempts: 3
    exporters:
      - otlp:
          endpoint: "http://collector:4317"
  
  # ITR export pipeline (when internal_collection.enabled: true)
  # ITR and all downstream nodes automatically configured with SDK fallback
  - name: internal_telemetry
    receivers:
      - internal_telemetry  # References internal collection from service.telemetry
    processors:
      - batch:
          timeout: 5s
          send_batch_size: 100
    exporters:
      - otlp:
          endpoint: "http://observability-backend:4317"
```

**Routing logic in admin component:**

[SNIP]

**Note:** When ITR is configured, the pipeline engine automatically:
- Routes component logs to ITR receiver (bypasses SDK processors in modes 2 & 3)
- Routes 3rd party logs to ITR receiver in mode 3 (no SDK - admin locates and registers ITR as global subscriber)
- Marks ITR pipeline components as "anti-recursion zone" (they always use SDK fallback or console)
- Prevents infinite recursion by excluding ITR pipeline from internal_collection

**Implementation:** Effect handler creation receives destination channel based on configuration
- **Mode 1** (SDK only): All effect handlers get OtlpBytesChannel sender → SDK
- **Mode 2** (Hybrid): Component effect handlers get ITR sender, 3rd party uses SDK
- **Mode 3** (ITR only): All effect handlers get ITR sender, no SDK
- ITR pipeline components always get OtlpBytesChannel sender or raw console logger (anti-recursion)
- Both channels receive OTLP bytes with same encoding
- Pipeline engine passes appropriate sender during effect handler construction

**ITR location and registration (Mode 3):**
- Admin component scans pipeline graph for receiver named "internal_telemetry"
- Obtains overflow channel sender from ITR receiver instance
- Registers sender as destination for global tracing subscriber (3rd party logs)
- Global subscriber encodes to OTLP bytes and sends to ITR (same as component logs)
- Single unified path for all logging

### 3. Resource and Scope Name

**Problem:** What Resource and scope name should each component use?

**Options:**
- A) Shared Resource per pipeline, scope = node_id
- B) Per-component Resource, scope = component type
- C) Configurable per component

**Proposed:** 
- Resource: Pipeline-level (service.name, service.version)
- Scope: Component's node_id.to_string() (e.g., "receiver.otlp.0")

### 4. Buffer Size and Record Limits

**Problem:** What buffer size and individual record size limits should we use?

**Proposed:** 
- Buffer size: 64 KiB per thread (configurable)
- Max log record size: 16 KiB (configurable)
- Rationale: 
  - 16 KiB limit enables encoder optimization (2-byte length placeholders for 14-bit sizes)
  - 64 KiB buffer provides ~4× headroom for multiple records
  - Large enough to batch efficiently, small enough to avoid memory pressure
- When buffer is full: flush synchronously and retry encoding
- When record exceeds limit: drop with counter increment
- When channel is full: fall back to raw console logger

**Bounded Memory Guarantees:**
- StatefulOtlpEncoder: Pre-allocated 64 KiB per thread (fixed size)
- Channel capacity: 1000 records (configurable) × ~64 KiB = ~64 MB worst case
- Total per component: ~64 KiB buffer + share of channel capacity
- Graceful degradation when limits reached

### 5. Error Handling

**Problem:** What happens if `log_event()` fails?

**Solution:** Drop with metrics and occasional raw logging - **Critical for recursion prevention**

- `log_event()` returns void (never fails)
- **Buffer overflow**: Try to flush, then retry encoding; if flush fails, use raw logger
- **Channel full**: Try non-blocking send; if fails, use raw logger
- **Encoding errors**: Increment `telemetry.dropped_events` counter, use raw logger
- **Raw logger**: `LoggerProvider::init_default_console_tracing()` - synchronous console output
- **No per-event logging** - would cause recursion
- **No error returns** - caller must not handle telemetry failures

**Bounded Memory Guarantees:**

1. **Pre-allocated per-thread buffer**: StatefulOtlpEncoder with fixed capacity (e.g., 64 KiB)
2. **Max log record size**: 16 KiB limit (enables 2-byte length placeholders)
3. **Bounded channel**: OtlpBytesChannel/ITR channel with configurable depth (e.g., 1000 records)
4. **Graceful overflow**:
   - Buffer full → flush → retry encoding
   - Record too large (>16 KiB) → drop with counter
   - Channel full → raw logger (console)
   - Never blocks the component operation
   - Always bounded memory usage

**Implementation:**

[SNIP]

**Rationale:** Telemetry must never fail the component operation. Bounded resources ensure we never cause OOM. When buffers/channels are full, degrade gracefully to synchronous console logging.

### 6. Macro Bypass Global Subscriber

**Problem:** How do we create tracing::Event and bypass the global subscriber?

**Current approach:** Macros use `tracing::info!()` which goes through global dispatcher.

**Options:**
- A) Don't bypass - use global subscriber for everything
  - Con: Performance overhead, recursive logging risk
- B) Direct `Event::dispatch_to()` with custom collector
  - Pro: Full control, no global state
  - Con: Need to implement minimal collector
- C) Use `tracing::Span::new()` + `Event::new()` directly without dispatch
  - Pro: No subscriber overhead
  - Con: Must manually construct metadata

**Proposed:** Option C for now, reconsider if needed
- Construct Event and Metadata directly in macro
- Pass to effect.log_event() as TracingLogRecord
- No global subscriber involvement

### 7. Testing Strategy

**Problem:** How do we test ITR without full pipeline?

**Proposed:**
- Unit tests: Test LocalTelemetryState, SharedTelemetryState in isolation
- Integration tests: Mock effect handler, verify encoding
- End-to-end: Full pipeline with ITR, verify OTLP output

## Implementation Order

1. ✅ StatefulOtlpEncoder (already exists)
2. ✅ OtlpBytesChannel for 3rd party (already exists)
3. ✅ Add TelemetryBuffer to EffectHandlerCore
4. ✅ Implement log_event() for local::EffectHandler
5. ✅ Implement log_event() for shared::EffectHandler
6. ✅ Add InternalCollectionConfig (internal_collection.enabled flag + bounded memory params)
7. ❌ **Update StatefulOtlpEncoder with fixed capacity and BufferFull error**
   - Add max_capacity parameter to constructor
   - Return EncodingError::BufferFull when capacity exceeded
   - Pre-allocate buffer to max_capacity
8. ❌ **Update log_event() to handle BufferFull gracefully**
   - Try encode → BufferFull → flush_and_send → retry encode
   - Use try_send (non-blocking) on bounded channel
   - Fall back to raw logger on channel full or flush failure
9. ❌ **Implement raw logger fallback**
   - Add raw_log() function using console tracing
   - Initialize early in main() via LoggerProvider::init_default_console_tracing()
   - Never fails, synchronous, writes to stderr
10. ❌ **Update OtlpBytesChannel to use bounded channel**
    - Replace unbounded with bounded mpsc::channel
    - Add capacity parameter to constructor
    - Consumer handles try_recv for non-blocking
11. ❌ **Implement SDK path for component logs** (when internal_collection.enabled: false)
    - Create OTLP decoder: Bytes → LogRecordView
    - Create SDK LogRecord builder (emulate tracing bridge)
    - Update admin runtime task to decode + call SDK exporter
12. ❌ **Modify otel_* macros to accept effect handler**
13. ❌ **Implement ITR receiver with timer-based flush** (internal_collection.enabled: true path)
14. ❌ **Implement ITR detection and anti-recursion zone marking** in pipeline build
15. ❌ **Wire up during pipeline construction** (choose SDK vs ITR path, pass bounded channels)
16. ❌ **Add metrics for dropped events**
17. ❌ **Testing**

### SDK Path Implementation Details (Step 11)

When `internal_collection.enabled: false` (default), component logs must:

1. **Share OtlpBytesChannel with 3rd party logs**
   - Effect handlers get OtlpBytesChannel sender during creation
   - Same sender used by global tracing subscriber (Path 1)

2. **Admin runtime consumer** needs enhancement:
   - Current: Format 3rd party OTLP bytes to console
   - New: Detect source (3rd party vs component), route accordingly
   - Component logs: Decode → SDK LogRecord → SDK exporter

3. **Decoder implementation** (new):
[SNIP]

4. **SDK bridge** (new):
[SNIP]

**Current Status:** Configuration structure defined. Next step is implementing the SDK path for component logs.

## Configuration

### Default: SDK Only (Mode 1)

When no ITR is configured, all logging uses the OTel SDK:
- Effect handlers encode OTLP bytes using StatefulOtlpEncoder
- Send to OtlpBytesChannel shared with 3rd party logs (global tracing subscriber)
- Admin runtime task decodes and forwards to SDK exporters
- Benefits from all SDK features: stdout, OTLP exporters, declarative config
- Simple, no special pipeline setup required

```yaml
# configs/service.yaml - SDK only (default)
service:
  telemetry:
    logs:
      level: info
      # internal_collection defaults to disabled
      processors:
        - batch:
            exporter:
              console:

# configs/pipelines.yaml - Standard data pipelines only
pipelines:
  - name: main
    receivers:
      - otlp
    processors:
      - batch:
          timeout: 1s
    exporters:
      - otlp:
          endpoint: "http://collector:4317"
```

### Hybrid: ITR for Components, SDK for 3rd Party (Mode 2)

When ITR is configured alongside SDK processors:
- Component logs go to ITR receiver (OTAP pipeline)
- 3rd party logs still use SDK (separate path)
- Can use standard OTAP components: batch processor, retry processor, etc.
- Full control over component log batching, transformation, export
- Requires anti-recursive configuration (ITR pipeline must NOT use ITR)

```yaml
# configs/service.yaml - Hybrid mode
service:
  telemetry:
    logs:
      level: info
      internal_collection:
        enabled: true
        buffer_size_bytes: 65536
        max_record_bytes: 16384
        max_record_count: 1000
        flush_interval: "1s"
      # 3rd party logs still use SDK processors
      processors:
        - batch:
            exporter:
              otlp:
                endpoint: "http://localhost:4317"

# configs/pipelines.yaml - ITR pipeline for component logs
pipelines:
  - name: main
    receivers:
      - otlp
    processors:
      - batch:
          timeout: 1s
    exporters:
      - otlp:
          endpoint: "http://collector:4317"
  
  # ITR pipeline - processes component logs only
  # Components in this pipeline automatically use SDK fallback (anti-recursion)
  - name: internal_telemetry
    receivers:
      - internal_telemetry
    processors:
      - batch:
          timeout: 5s
          send_batch_size: 100
    exporters:
      - otlp:
          endpoint: "http://observability:4317"
```

### ITR Only: No SDK (Mode 3)

When ITR is configured with empty processors array:
- Both component logs and 3rd party logs route to ITR receiver
- No OpenTelemetry SDK used - "be our own SDK" completely
- Admin component locates ITR and registers as global subscriber
- Single unified path through OTAP pipeline
- Maximum control and consistency

```yaml
# configs/service.yaml - ITR only (no SDK)
service:
  telemetry:
    logs:
      level: info
      internal_collection:
        enabled: true
        buffer_size_bytes: 65536
        max_record_bytes: 16384
        max_record_count: 1000
        flush_interval: "1s"
      # Empty processors = route 3rd party logs to ITR too
      processors: []

# configs/pipelines.yaml - ITR pipeline for all logs
pipelines:
  - name: main
    receivers:
      - otlp
    processors:
      - batch:
          timeout: 1s
    exporters:
      - otlp:
          endpoint: "http://collector:4317"
  
  # ITR pipeline - processes both component and 3rd party logs
  # Components in this pipeline automatically use raw console logger (anti-recursion)
  - name: internal_telemetry
    receivers:
      - internal_telemetry
    processors:
      - batch:
          timeout: 5s
          send_batch_size: 100
    exporters:
      - otlp:
          endpoint: "http://observability:4317"
```

## Metrics

The ITR should expose:
- `telemetry.events_received`: Counter of log events
- `telemetry.events_dropped`: Counter of dropped events (overflow, errors)
- `telemetry.bytes_flushed`: Counter of bytes sent
- `telemetry.flush_count`: Counter of flush operations
- `telemetry.overflow_flush_count`: Counter of synchronous overflow flushes
- `telemetry.timer_flush_count`: Counter of timer-based flushes

## Performance Targets

Based on the original design doc:

- `otel_info!(effect, ...)`: < 100ns when not flushing
- Flush operation: < 10μs for typical batch (100 events)
- Overflow flush: Acceptable latency spike (blocking caller)
- No allocations in hot path (encoder reuses buffer)

## Migration Path

1. Phase 1: Implement ITR infrastructure (this design)
2. Phase 2: Migrate existing `otel_info!()` calls to use effect handler
3. Phase 3: Add configuration to enable/disable ITR per component
4. Phase 4: Performance tuning and optimization
