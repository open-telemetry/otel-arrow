# Internal Telemetry Receiver Design

## Overview

This document describes the design for component-level telemetry logging using the Internal Telemetry Receiver (ITR). The system supports three distinct paths for telemetry:

1. **3rd Party Logs** (Global Tracing) - Admin component OtlpBytesChannel
2. **Shared ITR** - Multi-threaded components using Arc<Mutex<>>
3. **Local ITR** - Single-threaded components using Rc<RefCell<>>

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

## Implementation Components

### 1. Effect Handler Integration

#### EffectHandlerCore

Add telemetry buffer to the core:

```rust
pub(crate) struct EffectHandlerCore<PData> {
    pub(crate) node_id: NodeId,
    pub(crate) pipeline_ctrl_msg_sender: Option<PipelineCtrlMsgSender<PData>>,
    pub(crate) metrics_reporter: MetricsReporter,
    
    // New: Telemetry buffer (None if ITR not configured)
    // IMPORTANT: Only stores Shared variant to maintain Send bounds
    pub(crate) telemetry_buffer: Option<TelemetryBuffer>,
}

// Telemetry buffer for shared (Send) effect handlers
// Wraps SharedTelemetryState in Arc<Mutex<>>
pub struct TelemetryBuffer(Arc<Mutex<SharedTelemetryState>>);

// Local effect handlers store LocalTelemetryState directly in their struct
// This avoids breaking Send bounds for EffectHandlerCore
pub struct EffectHandler<PData> {  // local::EffectHandler
    pub(crate) core: EffectHandlerCore<PData>,
    msg_senders: HashMap<PortName, LocalSender<PData>>,
    default_sender: Option<LocalSender<PData>>,
    
    // Local telemetry state (not in core to preserve Send)
    telemetry_state: Option<Rc<RefCell<LocalTelemetryState>>>,
}

struct LocalTelemetryState {
    encoder: StatefulOtlpEncoder,
    resource_bytes: OtlpProtoBytes,
    scope_name: String,  // From node_id or component name
    flush_threshold_bytes: usize,
    overflow_sender: mpsc::UnboundedSender<Bytes>,  // Destination for overflow
}

struct SharedTelemetryState {
    encoder: StatefulOtlpEncoder,
    resource_bytes: OtlpProtoBytes,
    scope_name: String,
    flush_threshold_bytes: usize,
    overflow_sender: mpsc::UnboundedSender<Bytes>,  // Destination for overflow
}

// Note: overflow_sender can point to either:
// 1. ITR receiver (when configured) - enters OTAP pipeline
// 2. OtlpBytesChannel (SDK fallback) - same as 3rd party logs
// Both receive OTLP bytes, same encoding path

```

#### local::EffectHandler

```rust
impl<PData> EffectHandler<PData> {
    pub fn log_event(&self, log_record: &impl LogRecordView) {
        if let Some(TelemetryBuffer::Local(buffer)) = &self.core.telemetry_buffer {
            let mut state = buffer.borrow_mut();
            
            // Encode the log record (silent drop on error to prevent recursion)
            if state.encoder.encode_log_record(
                log_record,
                &state.resource_bytes,
                &state.scope_name,
            ).is_err() {
                return; // Drop event, increment metrics
            }
            
            // Check overflow - non-blocking send to break recursion
            if state.encoder.len() >= state.flush_threshold_bytes {
                let bytes = state.encoder.flush();
                let _ = state.overflow_sender.send(bytes); // Non-blocking
            }
        }
    }
}
```

#### shared::EffectHandler

```rust
impl<PData> EffectHandler<PData> {
    pub fn log_event(&self, log_record: &impl LogRecordView) -> Result<(), Error> {
        if let Some(TelemetryBuffer::Shared(buffer)) = &self.{
        if let Some(TelemetryBuffer::Shared(buffer)) = &self.core.telemetry_buffer {
            let mut state = buffer.lock().unwrap();
            
            // Encode the log record (silent drop on error to prevent recursion)
            if state.encoder.encode_log_record(
                log_record,
                &state.resource_bytes,
                &state.scope_name,
            ).is_err() {
                return; // Drop event, increment metrics
            }
            
            // Check overflow - non-blocking send to break recursion
            if state.encoder.len() >= state.flush_threshold_bytes {
                let bytes = state.encoder.flush();
                let _ = state.overflow_sender.send(bytes); // Non-blocking
            }
        }
}
```

### 2. Macro Modifications

Update `otel_info!` and family to accept effect handler:

```rust
#[macro_export]
macro_rules! otel_info {
    // New signature: otel_info!(effect, "event.name", key = val, ...)
    ($effect:expr, $name:expr $(,)?) => {{
        // Create tracing::Event with metadata
        let metadata = tracing::Metadata::new(
            $name,
            env!("CARGO_PKG_NAME"),
            tracing::Level::INFO,
            // ... other fields
        );
        
        // Create event
        let event = tracing::Event::new(&metadata, &tracing::field::ValueSet::new(...));
        
        // Create TracingLogRecord view
        let log_record = $crate::tracing_integration::TracingLogRecord::new(&event);
        
        // Log directly to effect handler (bypass global subscriber)
        $effect.log_event(&log_record)?;
    }};
    
    ($effect:expr, $name:expr, $($key:ident = $value:expr),+ $(,)?) => {{
        // Similar with attributes
    }};
}
```

### 3. Internal Telemetry Receiver

The ITR is a standard OTAP receiver that:
- Has no inputs (generates data from effect handlers)
- Has one output port (forwards OTLP bytes as OtapPdata)
- Supports both shared and local modes
- Manages timer-based flushing AND overflow channel

```rust
pub struct InternalTelemetryReceiver {
    config: Config,
    mode: ReceiverMode,
    overflow_receiver: mpsc::UnboundedReceiver<Bytes>,  // Receives overflow flushes
}

enum ReceiverMode {
    Local(Rc<RefCell<LocalTelemetryState>>),
    Shared(Arc<Mutex<SharedTelemetryState>>),
}

impl Receiver<OtapPdata> for InternalTelemetryReceiver {
    async fn start(&mut self, effect_handler: &mut EffectHandler<OtapPdata>) {
        // Start timer for periodic flush
        let timer = effect_handler.start_periodic_timer(self.config.flush_interval).await?;
        
        // Main loop handles both timer ticks AND overflow channel
        loop {
            tokio::select! {
                // Overflow flush from effect handlers (non-blocking send)
                Some(bytes) = self.overflow_receiver.recv() => {
                    let pdata = OtapPdata::from_otlp_bytes(bytes);
                    effect_handler.send_message(pdata, None).await?;
                }
                
                // Timer tick - periodic flush
                _ = timer_tick => {
                    let bytes = match &self.mode {
                        ReceiverMode::Local(buffer) => {
                            buffer.borrow_mut().encoder.flush()
                        }
                        ReceiverMode::Shared(buffer) => {
                            buffer.lock().unwrap().encoder.flush()
                        }
                    };
                    
                    if !bytes.is_empty() {
                        let pdata = OtapPdata::from_otlp_bytes(bytes);
                        effect_handler.send_message(pdata, None).await?;
                    }
                }
            }
        }
    }
}
```

## Open Questions

### 1. Send Message from Effect Handler

**Problem:** When overflow occurs during `log_event()`, we need to send data to ITR but:
- Cannot call `send_message()` directly - risk of **recursive self-dependency/lockup**
- If telemetry APIs trigger more telemetry (directly or through send_message), we could deadlock
- Need non-blocking approach that breaks recursion

**Solution:** Non-blocking channel send (Option A)

- Store `mpsc::UnboundedSender<Bytes>` in telemetry state
- On overflow: flush encoder, send bytes to channel (non-blocking)
- ITR receiver consumes channel in its event loop
- ITR receiver calls `send_message()` on its output port

**Key benefit:** Breaks recursion chain - flush just queues data, doesn't execute more telemetry code

```rust
// Note: LocalTelemetryState stored directly in local::EffectHandler
// to avoid breaking Send bounds for shared::EffectHandler.
// Only SharedTelemetryState lives in EffectHandlerCore via TelemetryBuffer.

struct LocalTelemetryState {
    encoder: StatefulOtlpEncoder,
    resource_bytes: OtlpProtoBytes,  // Changed from OtlpBytes
    scope_name: String,
    flush_threshold_bytes: usize,
    overflow_sender: mpsc::UnboundedSender<Bytes>,  // Non-blocking send on overflow
}

pub fn log_event(&self, log_record: &impl LogRecordView) {
    if let Some(TelemetryBuffer::Local(buffer)) = &self.core.telemetry_buffer {
        let mut state = buffer.borrow_mut();
        
        // Encode (can fail silently - increment dropped counter)
        if state.encoder.encode_log_record(
            log_record,
            &state.resource_bytes,
            &state.scope_name,
        ).is_err() {
            return; // Silent drop, no recursion risk
        }
        
        // Check overflow
        if state.encoder.len() >= state.flush_threshold_bytes {
            let bytes = state.encoder.flush();
            // Non-blocking send - no async, no recursion
            let _ = state.overflow_sender.send(bytes);
        }
    }
}
```

### 2. ITR Pipeline Must Not Use ITR

**Problem:** **Critical recursion prevention** - The ITR receiver and its export path must NOT use ITR for their own telemetry, otherwise infinite recursion occurs.

**Solution:** Pipeline-level logger configuration with three options

1. **No logger configured (default)**: SDK fallback via OtlpBytesChannel
   - Same channel used by 3rd party logs
   - Shares admin component's OTel SDK configuration
   - Gets all SDK exporters: stdout, OTLP, etc.

2. **ITR configured**: Component logs enter OTAP pipeline
   - "Be our own SDK" using OTAP components
   - Can batch, transform, export using standard processors
   - Must configure ITR pipeline to NOT use itself (use SDK fallback)

3. **Console configured**: Direct console output
   - Uses default tracing subscriber from main()
   - Fallback when ITR send fails

**Configuration approach:**
```yaml
pipelines:
  # Main data pipeline - uses ITR for component telemetry
  - name: main
    receivers: [otlp]
    processors: [batch, retry]
    exporters: [otlp_grpc]
    telemetry:
      logger: internal_telemetry_receiver  # Use ITR
  
  # ITR export pipeline - uses SDK fallback (NO recursion)
  - name: internal_telemetry
    receivers: [internal_telemetry]
    processors: [batch]  # Standard OTAP processor!
    exporters: [otlp_grpc]
    telemetry:
      logger: sdk  # SDK fallback (OtlpBytesChannel), not ITR
      # This breaks recursion: ITR pipeline logs go to admin SDK, not back to ITR
```

**Implementation:** Effect handler creation receives destination channel
- Default: OtlpBytesChannel sender (SDK fallback)
- ITR configured: ITR overflow channel sender
- Both receive OTLP bytes, same encoding
- Pipeline engine passes appropriate sender during effect handler construction

### 3. Resource and Scope Name

**Problem:** What Resource and scope name should each component use?

**Options:**
- A) Shared Resource per pipeline, scope = node_id
- B) Per-component Resource, scope = component type
- C) Configurable per component

**Proposed:** 
- Resource: Pipeline-level (service.name, service.version)
- Scope: Component's node_id.to_string() (e.g., "receiver.otlp.0")

### 4. Flush Threshold

**Problem:** What size threshold triggers overflow flush?

**Options:**
- Bytes: 16 KiB, 64 KiB, 256 KiB?
- Record count: 100 records, 1000 records?
- Time: Not applicable for overflow case

**Proposed:** 
- Default: 64 KiB bytes
- Configurable per ITR instance
- Rationale: Large enough to batch, small enough to avoid memory pressure

### 5. Error Handling

**Problem:** What happens if `log_event()` fails?

**Solution:** Drop with metrics and occasional raw logging - **Critical for recursion prevention**

- `log_event()` returns void (never fails)
- Encoding errors: Drop event, increment `telemetry.dropped_events` counter
- Channel send errors: Drop event, increment counter (shouldn't happen with unbounded)
- **Periodic raw logging**: When drop count exceeds threshold, log to stdout/stderr using "Raw" logger
- **No per-event logging** - would cause recursion
- **No error returns** - caller must not handle telemetry failures

**Fallback to console logging:** The same console logger installed early in main()
- When ITR send_message() fails, fall back to console output
- Uses the default tokio-tracing-rs provider already configured
- No risk of recursion since it's a separate code path (not using ITR)
- Already exists - same system used before ITR is initialized

**Rationale:** Telemetry must never fail the component operation. When ITR path fails, degrade gracefully to console.

```rust
pub fn log_event(&self, log_record: &impl LogRecordView) {
    if let Some(buffer) = &self.core.telemetry_buffer {
        // Try to encode and send via ITR
        if let Err(_) = self.try_log_event_internal(log_record) {
            // Fall back to console logging (default provider from main)
            // This is the same logger used before ITR initialization
            tracing::info!(
                target: self.node_id,
                name = log_record.event_name(),
                "Telemetry overflow, falling back to console"
            );
            
            // TODO: Count dropped events and log statistics periodically
        }
    }
}
```

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
3. **Add TelemetryBuffer to EffectHandlerCore**
4. **Implement log_event() for local::EffectHandler**
5. **Implement log_event() for shared::EffectHandler**
6. **Modify otel_* macros to accept effect handler**
7. **Implement ITR receiver with timer-based flush**
8. **Wire up ITR during pipeline construction**
9. **Add configuration for ITR (flush interval, threshold)**
10. **Add metrics for dropped events**

## Configuration

### Default: No ITR (Fallback to OTel SDK)

When no ITR is configured, component logging falls back to the OTel SDK configured for the admin component:
- Effect handlers encode OTLP bytes using StatefulOtlpEncoder
- Send to same OtlpBytesChannel used by 3rd party (global tracing subscriber)
- Benefits from all SDK features: stdout, OTLP exporters, declarative config
- Simple, no special pipeline setup required

```yaml
# No ITR receiver configured
pipelines:
  - name: main
    receivers: [otlp]
    processors: [batch]
    exporters: [otlp_grpc]
    # telemetry: (not configured - defaults to SDK fallback)
```

### With ITR: "Be Our Own SDK"

When ITR is configured, component logging goes directly into an OTAP pipeline:
- Effect handlers send to ITR receiver (dedicated pipeline)
- Can use standard OTAP components: batch processor, retry processor, etc.
- Full control over batching, transformation, export
- Requires anti-recursive configuration (ITR pipeline must NOT use ITR)

```yaml
receivers:
  - name: internal_telemetry
    type: urn:otap:receiver:internal-telemetry:v1
    config:
      flush_interval: 1s
      flush_threshold_bytes: 65536
      mode: shared  # or 'local'

pipelines:
  # Main data pipeline - uses ITR for component telemetry
  - name: main
    receivers: [otlp]
    processors: [batch, retry]
    exporters: [otlp_grpc]
    telemetry:
      logger: internal_telemetry_receiver  # Enable ITR
  
  # ITR export pipeline - uses SDK fallback (NO recursion)
  - name: internal_telemetry
    receivers: [internal_telemetry]
    processors: [batch]  # Can use standard OTAP processors
    exporters: [otlp_grpc]
    telemetry:
      logger: sdk  # Use SDK fallback, not ITR
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
