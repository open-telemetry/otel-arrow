# Stateful OTLP Encoder Implementation

**Status**: Implementation Design  
**Date**: December 17, 2025  
**Focus**: Practical implementation of streaming OTLP encoder with state management

## Executive Summary

This document describes a **stateful OTLP bytes encoder** that maintains open `ResourceLogs` and `ScopeLogs` messages, appending individual `LogRecord`s as they arrive. The encoder reuses the existing 4-byte padding logic from the OTAP-to-OTLP encoder and provides automatic batching when Resource/Scope repeat.

## Architecture

### High-Level Flow

```
┌─────────────────────────────────────────────────────┐
│  otel_info!(effect, "event", key = value)          │
│                                                     │
│  1. Build LogRecord (stack-allocated)              │
│  2. Get precomputed Resource & Scope from context  │
│  3. Call encoder.encode_log_record()               │
└──────────────────┬──────────────────────────────────┘
                   │
                   v
┌─────────────────────────────────────────────────────┐
│  StatefulOtlpEncoder                                │
│                                                     │
│  State Check:                                       │
│  - Same Scope?              → Append LogRecord only │
│  - Different Scope?         → Close Scope, new one  │
│                                                     │
│  Buffer State:                                      │
│  ┌─────────────────────────────────────────────┐    │
│  │ ResourceLogs (OPEN)                         │    │
│  │   Resource: { service.name: "my-svc" }      │    │
│  │   ├─ ScopeLogs (OPEN)                       │    │
│  │   │   Scope: { name: "tracer" }             │    │
│  │   │   ├─ LogRecord { time: 100, ... }       │    │
│  │   │   ├─ LogRecord { time: 101, ... }       │    │
│  │   │   └─ LogRecord { time: 102, ... }   ←─ APPEND│
│  │   │   [length fields not yet patched]       │    │
│  └─────────────────────────────────────────────┘    │
└──────────────────┬──────────────────────────────────┘
                   │
                   v (when threshold reached)
┌─────────────────────────────────────────────────────┐
│  flush()                                            │
│                                                     │
│  1. Close current ScopeLogs (patch length)         │
│  2. Close current ResourceLogs (patch length)      │
│  3. Send Vec<u8> as OTLP bytes                     │
│  4. Clear buffer, reset state                      │
└─────────────────────────────────────────────────────┘
```

### Key Simplifications

1. **Use 4-byte padding**: Reuse existing `proto_encode_len_delimited_unknown_size!` macro
2. **Precompute Resource/Scope**: Store in `PipelineContext`, no runtime allocation
3. **Singleton encoding**: Each call encodes one `LogRecord`, encoder manages batching state
4. **No view abstraction needed**: Direct encoding from SDK types

## Core Data Structure

### StatefulOtlpEncoder

```rust
/// Stateful OTLP encoder that maintains open ResourceLogs and ScopeLogs messages
pub struct StatefulOtlpEncoder {
    /// Output buffer for OTLP bytes
    buffer: Vec<u8>,
    
    /// Current state of the encoder
    state: EncoderState,
    
    /// Position of the ResourceLogs length placeholder (for patching)
    resource_logs_len_pos: Option<usize>,
    
    /// Position of the ScopeLogs length placeholder (for patching)
    scope_logs_len_pos: Option<usize>,
    
    /// Hash of the current Resource (for comparison)
    current_resource_hash: Option<u64>,
    
    /// Hash of the current InstrumentationScope (for comparison)
    current_scope_hash: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncoderState {
    /// No messages open, buffer is empty or contains complete messages
    Idle,
    
    /// ResourceLogs is open, no ScopeLogs yet
    ResourceOpen,
    
    /// ResourceLogs and ScopeLogs are both open, ready to append LogRecords
    ScopeOpen,
}

impl StatefulOtlpEncoder {
    /// Create a new encoder with pre-allocated buffer
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity_bytes),
            state: EncoderState::Idle,
            resource_logs_len_pos: None,
            scope_logs_len_pos: None,
            current_resource_hash: None,
            current_scope_hash: None,
        }
    }
    
    /// Encode a single log record with context
    pub fn encode_log_record(
        &mut self,
        log_record: &LogRecord,
        resource: &Resource,
        scope: &InstrumentationScope,
    ) -> Result<(), Error> {
        let resource_hash = compute_resource_hash(resource);
        let scope_hash = compute_scope_hash(scope);
        
        match self.state {
            EncoderState::Idle => {
                // Start new ResourceLogs and ScopeLogs
                self.start_resource_logs(resource, resource_hash)?;
                self.start_scope_logs(scope, scope_hash)?;
                self.append_log_record(log_record)?;
            }
            
            EncoderState::ResourceOpen => {
                // Resource is open but no scope yet
                if Some(resource_hash) == self.current_resource_hash {
                    // Same resource - start scope
                    self.start_scope_logs(scope, scope_hash)?;
                    self.append_log_record(log_record)?;
                } else {
                    // Different resource - close and restart
                    self.close_resource_logs()?;
                    self.start_resource_logs(resource, resource_hash)?;
                    self.start_scope_logs(scope, scope_hash)?;
                    self.append_log_record(log_record)?;
                }
            }
            
            EncoderState::ScopeOpen => {
                // Both ResourceLogs and ScopeLogs are open
                if Some(resource_hash) == self.current_resource_hash
                    && Some(scope_hash) == self.current_scope_hash
                {
                    // Same resource and scope - just append LogRecord
                    self.append_log_record(log_record)?;
                } else if Some(resource_hash) == self.current_resource_hash {
                    // Same resource, different scope
                    self.close_scope_logs()?;
                    self.start_scope_logs(scope, scope_hash)?;
                    self.append_log_record(log_record)?;
                } else {
                    // Different resource - close everything and restart
                    self.close_scope_logs()?;
                    self.close_resource_logs()?;
                    self.start_resource_logs(resource, resource_hash)?;
                    self.start_scope_logs(scope, scope_hash)?;
                    self.append_log_record(log_record)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get current buffer size
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
    
    /// Flush the encoder, returning accumulated OTLP bytes
    pub fn flush(&mut self) -> Vec<u8> {
        // Close any open messages
        if self.state == EncoderState::ScopeOpen {
            self.close_scope_logs().ok();
        }
        if self.state == EncoderState::ResourceOpen || self.state == EncoderState::ScopeOpen {
            self.close_resource_logs().ok();
        }
        
        // Take the buffer
        let bytes = std::mem::take(&mut self.buffer);
        
        // Reset state
        self.state = EncoderState::Idle;
        self.resource_logs_len_pos = None;
        self.scope_logs_len_pos = None;
        self.current_resource_hash = None;
        self.current_scope_hash = None;
        
        bytes
    }
    
    // Private implementation methods...
}
```

## Implementation Details

### 1. Starting a ResourceLogs Message

```rust
impl StatefulOtlpEncoder {
    fn start_resource_logs(
        &mut self,
        resource: &Resource,
        resource_hash: u64,
    ) -> Result<(), Error> {
        // Encode LogsData.resource_logs field (tag 1, length-delimited)
        self.encode_field_tag(LOGS_DATA_RESOURCE_LOGS, WIRE_TYPE_LEN);
        
        // Save position of length placeholder
        self.resource_logs_len_pos = Some(self.buffer.len());
        
        // Write 4-byte length placeholder (reuse existing pattern)
        self.buffer.extend_from_slice(&[0x80, 0x80, 0x80, 0x00]);
        
        // Encode the Resource message
        self.encode_resource(resource)?;
        
        // Update state
        self.state = EncoderState::ResourceOpen;
        self.current_resource_hash = Some(resource_hash);
        
        Ok(())
    }
}
```

### 2. Starting a ScopeLogs Message

```rust
impl StatefulOtlpEncoder {
    fn start_scope_logs(
        &mut self,
        scope: &InstrumentationScope,
        scope_hash: u64,
    ) -> Result<(), Error> {
        // Encode ResourceLogs.scope_logs field (tag 2, length-delimited)
        self.encode_field_tag(RESOURCE_LOGS_SCOPE_LOGS, WIRE_TYPE_LEN);
        
        // Save position of length placeholder
        self.scope_logs_len_pos = Some(self.buffer.len());
        
        // Write 4-byte length placeholder
        self.buffer.extend_from_slice(&[0x80, 0x80, 0x80, 0x00]);
        
        // Encode the InstrumentationScope message
        self.encode_instrumentation_scope(scope)?;
        
        // Update state
        self.state = EncoderState::ScopeOpen;
        self.current_scope_hash = Some(scope_hash);
        
        Ok(())
    }
}
```

### 3. Appending a LogRecord

```rust
impl StatefulOtlpEncoder {
    fn append_log_record(&mut self, log_record: &LogRecord) -> Result<(), Error> {
        // Encode ScopeLogs.log_records field (tag 2, length-delimited)
        self.encode_field_tag(SCOPE_LOGS_LOG_RECORDS, WIRE_TYPE_LEN);
        
        // Use 4-byte padding for LogRecord
        let len_pos = self.buffer.len();
        self.buffer.extend_from_slice(&[0x80, 0x80, 0x80, 0x00]);
        
        let content_start = self.buffer.len();
        
        // Encode LogRecord fields
        if let Some(time) = log_record.time_unix_nano {
            self.encode_uint64(LOG_RECORD_TIME_UNIX_NANO, time);
        }
        
        if let Some(observed_time) = log_record.observed_time_unix_nano {
            self.encode_uint64(LOG_RECORD_OBSERVED_TIME_UNIX_NANO, observed_time);
        }
        
        if let Some(severity) = log_record.severity_number {
            self.encode_int32(LOG_RECORD_SEVERITY_NUMBER, severity);
        }
        
        if let Some(severity_text) = &log_record.severity_text {
            self.encode_string(LOG_RECORD_SEVERITY_TEXT, severity_text);
        }
        
        if let Some(body) = &log_record.body {
            self.encode_any_value(LOG_RECORD_BODY, body)?;
        }
        
        // Encode attributes
        for attr in &log_record.attributes {
            self.encode_key_value(LOG_RECORD_ATTRIBUTES, attr)?;
        }
        
        if log_record.dropped_attributes_count > 0 {
            self.encode_uint32(
                LOG_RECORD_DROPPED_ATTRIBUTES_COUNT,
                log_record.dropped_attributes_count,
            );
        }
        
        if let Some(flags) = log_record.flags {
            self.encode_uint32(LOG_RECORD_FLAGS, flags);
        }
        
        if let Some(trace_id) = &log_record.trace_id {
            self.encode_bytes(LOG_RECORD_TRACE_ID, trace_id);
        }
        
        if let Some(span_id) = &log_record.span_id {
            self.encode_bytes(LOG_RECORD_SPAN_ID, span_id);
        }
        
        // Patch the length
        let content_len = self.buffer.len() - content_start;
        self.patch_length_4byte(len_pos, content_len);
        
        Ok(())
    }
}
```

### 4. Closing Messages

```rust
impl StatefulOtlpEncoder {
    fn close_scope_logs(&mut self) -> Result<(), Error> {
        if let Some(len_pos) = self.scope_logs_len_pos.take() {
            // Calculate total length of ScopeLogs content
            let total_len = self.buffer.len() - len_pos - 4; // Subtract 4-byte placeholder
            
            // Patch the length field
            self.patch_length_4byte(len_pos, total_len);
            
            // Update state
            self.state = EncoderState::ResourceOpen;
            self.current_scope_hash = None;
        }
        
        Ok(())
    }
    
    fn close_resource_logs(&mut self) -> Result<(), Error> {
        if let Some(len_pos) = self.resource_logs_len_pos.take() {
            // Calculate total length of ResourceLogs content
            let total_len = self.buffer.len() - len_pos - 4;
            
            // Patch the length field
            self.patch_length_4byte(len_pos, total_len);
            
            // Update state
            self.state = EncoderState::Idle;
            self.current_resource_hash = None;
        }
        
        Ok(())
    }
}
```

### 5. Reusing Existing Length Patching Logic

```rust
impl StatefulOtlpEncoder {
    /// Patch a 4-byte length placeholder with the actual length
    /// This matches the existing logic from proto_encode_len_delimited_unknown_size!
    fn patch_length_4byte(&mut self, start_pos: usize, len: usize) {
        for i in 0..4 {
            self.buffer[start_pos + i] += ((len >> (i * 7)) & 0x7f) as u8;
        }
    }
    
    /// Encode field tag and wire type
    fn encode_field_tag(&mut self, field_num: u32, wire_type: u8) {
        let tag = (field_num << 3) | (wire_type as u32);
        self.encode_varint(tag as u64);
    }
    
    /// Encode a varint (shared with existing code)
    fn encode_varint(&mut self, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80; // Set continuation bit
            }
            self.buffer.push(byte);
            if value == 0 {
                break;
            }
        }
    }
    
    // Other encoding helpers (can reuse from existing otlp/common.rs)
    fn encode_uint64(&mut self, field_num: u32, value: u64) { /* ... */ }
    fn encode_int32(&mut self, field_num: u32, value: i32) { /* ... */ }
    fn encode_string(&mut self, field_num: u32, value: &str) { /* ... */ }
    fn encode_bytes(&mut self, field_num: u32, value: &[u8]) { /* ... */ }
    fn encode_any_value(&mut self, field_num: u32, value: &AnyValue) -> Result<(), Error> { /* ... */ }
    fn encode_key_value(&mut self, field_num: u32, attr: &KeyValue) -> Result<(), Error> { /* ... */ }
}
```

## Precomputed Context

### PipelineContext Extension

```rust
/// Pipeline context with precomputed Resource and InstrumentationScope
pub struct PipelineContext {
    /// Precomputed Resource for this pipeline
    resource: Arc<Resource>,
    
    /// Precomputed InstrumentationScope for this pipeline
    scope: Arc<InstrumentationScope>,
    
    /// Cached hash of the resource (for fast comparison)
    resource_hash: u64,
    
    /// Cached hash of the scope (for fast comparison)
    scope_hash: u64,
}

impl PipelineContext {
    pub fn new(resource: Resource, scope: InstrumentationScope) -> Self {
        let resource_hash = compute_resource_hash(&resource);
        let scope_hash = compute_scope_hash(&scope);
        
        Self {
            resource: Arc::new(resource),
            scope: Arc::new(scope),
            resource_hash,
            scope_hash,
        }
    }
    
    pub fn resource(&self) -> &Resource {
        &self.resource
    }
    
    pub fn scope(&self) -> &InstrumentationScope {
        &self.scope
    }
    
    pub fn resource_hash(&self) -> u64 {
        self.resource_hash
    }
    
    pub fn scope_hash(&self) -> u64 {
        self.scope_hash
    }
}
```

### Fast Hashing for Comparison

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Compute a fast hash of a Resource for comparison
fn compute_resource_hash(resource: &Resource) -> u64 {
    let mut hasher = DefaultHasher::new();
    
    // Hash attributes in sorted order for consistency
    let mut attrs: Vec<_> = resource.attributes.iter().collect();
    attrs.sort_by_key(|kv| &kv.key);
    
    for kv in attrs {
        kv.key.hash(&mut hasher);
        // Hash the AnyValue (implement Hash for AnyValue)
        hash_any_value(&kv.value, &mut hasher);
    }
    
    hasher.finish()
}

/// Compute a fast hash of an InstrumentationScope for comparison
fn compute_scope_hash(scope: &InstrumentationScope) -> u64 {
    let mut hasher = DefaultHasher::new();
    
    scope.name.hash(&mut hasher);
    scope.version.hash(&mut hasher);
    
    // Hash attributes
    let mut attrs: Vec<_> = scope.attributes.iter().collect();
    attrs.sort_by_key(|kv| &kv.key);
    
    for kv in attrs {
        kv.key.hash(&mut hasher);
        hash_any_value(&kv.value, &mut hasher);
    }
    
    hasher.finish()
}

fn hash_any_value(value: &AnyValue, hasher: &mut DefaultHasher) {
    // Implement hashing for AnyValue variants
    match value {
        AnyValue::String(s) => {
            0u8.hash(hasher);
            s.hash(hasher);
        }
        AnyValue::Int(i) => {
            1u8.hash(hasher);
            i.hash(hasher);
        }
        AnyValue::Bool(b) => {
            2u8.hash(hasher);
            b.hash(hasher);
        }
        // ... other variants
    }
}
```

## Integration with Effect Handler

### Effect Handler Storage

```rust
pub struct EffectHandlerCore<PData> {
    node_id: NodeId,
    pipeline_ctrl_msg_sender: Option<PipelineCtrlMsgSender<PData>>,
    metrics_reporter: MetricsReporter,
    
    /// Precomputed pipeline context (Resource + Scope)
    pipeline_context: Arc<PipelineContext>,
    
    /// Stateful OTLP encoder
    otlp_encoder: RefCell<StatefulOtlpEncoder>,
    
    /// Flush threshold in bytes
    otlp_buffer_flush_threshold: usize,
}

impl<PData> EffectHandlerCore<PData> {
    pub fn new(
        node_id: NodeId,
        metrics_reporter: MetricsReporter,
        pipeline_context: Arc<PipelineContext>,
        flush_threshold_bytes: usize,
    ) -> Self {
        Self {
            node_id,
            pipeline_ctrl_msg_sender: None,
            metrics_reporter,
            pipeline_context,
            otlp_encoder: RefCell::new(StatefulOtlpEncoder::new(flush_threshold_bytes)),
            otlp_buffer_flush_threshold: flush_threshold_bytes,
        }
    }
    
    pub fn pipeline_context(&self) -> &PipelineContext {
        &self.pipeline_context
    }
    
    pub fn otlp_encoder_mut(&self) -> RefMut<StatefulOtlpEncoder> {
        self.otlp_encoder.borrow_mut()
    }
    
    pub fn flush_otlp_buffer(&self) -> Result<(), Error> {
        let mut encoder = self.otlp_encoder.borrow_mut();
        
        let otlp_bytes = encoder.flush();
        
        if otlp_bytes.is_empty() {
            return Ok(());
        }
        
        // Convert to OtapPdata and send
        let pdata = OtapPdata::from_otlp_bytes(otlp_bytes)?;
        self.send_message_internal(pdata)?;
        
        Ok(())
    }
}
```

### Macro Integration

```rust
// In the otel_info! macro expansion
pub fn log_event_fast(
    effect: &impl EffectHandlerTraceBuffer,
    level: Level,
    name: &str,
    fields: &[(&str, &dyn Debug)],
    location: &'static Location<'static>,
) {
    // 1. Build LogRecord inline (stack allocation, ~30ns)
    let log_record = build_log_record_inline(
        level,
        name,
        fields,
        location,
        effect.pipeline_context(),
    );
    
    // 2. Get encoder and context (~10ns)
    let mut encoder = effect.otlp_encoder_mut();
    let context = effect.pipeline_context();
    
    // 3. Encode with automatic batching (~100ns)
    encoder.encode_log_record(
        &log_record,
        context.resource(),
        context.scope(),
    ).expect("encoding failed");
    
    // 4. Check size threshold (~5ns)
    if encoder.buffer_len() >= effect.otlp_buffer_threshold() {
        drop(encoder); // Release borrow
        effect.flush_otlp_buffer().ok();
    }
}

fn build_log_record_inline(
    level: Level,
    name: &str,
    fields: &[(&str, &dyn Debug)],
    location: &'static Location<'static>,
    context: &PipelineContext,
) -> LogRecord {
    let mut log_record = LogRecord::default();
    
    log_record.time_unix_nano = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    );
    
    log_record.observed_time_unix_nano = log_record.time_unix_nano;
    log_record.severity_number = Some(level_to_severity(level));
    
    // Add standard attributes
    log_record.attributes.push(KeyValue {
        key: "event.name".into(),
        value: AnyValue::String(name.into()),
    });
    
    log_record.attributes.push(KeyValue {
        key: "code.filepath".into(),
        value: AnyValue::String(location.file().into()),
    });
    
    log_record.attributes.push(KeyValue {
        key: "code.lineno".into(),
        value: AnyValue::Int(location.line() as i64),
    });
    
    // Add user fields
    for (key, value) in fields {
        log_record.attributes.push(KeyValue {
            key: (*key).into(),
            value: AnyValue::String(format!("{:?}", value).into()),
        });
    }
    
    log_record
}
```

## Code Reuse Strategy

### Reuse from Existing OTLP Encoder

The following can be directly reused from `crates/pdata/src/otlp/`:

1. **Length patching logic**: `patch_len_placeholder()` from `common.rs:495`
2. **Field encoding helpers**: `ProtoBuffer` methods from `common.rs:376-450`
3. **Resource encoding**: `proto_encode_resource()` from `common.rs:52-87`
4. **Scope encoding**: `proto_encode_instrumentation_scope()` from `common.rs:143-178`
5. **Attribute encoding**: `encode_key_value()` and `encode_any_value()` from `attributes.rs`

### New Components Needed

1. **StatefulOtlpEncoder** struct - manages state and batching
2. **PipelineContext** extension - precomputed Resource/Scope with hashes
3. **Hash functions** - for fast Resource/Scope comparison
4. **Integration glue** - connect encoder to effect handler

## Performance Analysis

### Encoding Cost per Event

| Operation | Time | Notes |
|-----------|------|-------|
| Build LogRecord | ~30ns | Stack allocation |
| Get encoder + context | ~10ns | RefCell borrow |
| Check Resource/Scope match | ~20ns | Hash comparison |
| Encode LogRecord (append) | ~100ns | Protobuf encoding |
| Check threshold | ~5ns | Simple comparison |
| **Total (batched)** | **~165ns** | Same Resource/Scope |
| **Total (new scope)** | **~250ns** | Different Scope |
| **Total (new resource)** | **~350ns** | Different Resource |

### Flush Cost

| Operation | Time | Notes |
|-----------|------|-------|
| Close ScopeLogs | ~50ns | Patch one length field |
| Close ResourceLogs | ~50ns | Patch one length field |
| Send message | ~5μs | Pipeline message overhead |
| **Total** | **~5.1μs** | Amortized over batch |

### Space Overhead

**4-byte padding overhead:**
- ResourceLogs message: 4 bytes
- ScopeLogs message: 4 bytes
- Each LogRecord: 4 bytes
- Total per record: 4 bytes (Resource/Scope shared)

**Comparison to non-batched:**
- Non-batched: Resource (100B) + Scope (50B) + LogRecord (200B) = 350B per event
- Batched (10 events): 100B + 50B + (200B * 10) = 2150B / 10 = 215B per event
- **Savings: ~38% space reduction**

## Implementation Plan

### Phase 1: Core Encoder (Week 1)

**Files to create:**
- `crates/pdata/src/otlp/stateful_encoder.rs` - StatefulOtlpEncoder implementation

**Tasks:**
1. Implement `StatefulOtlpEncoder` struct with state machine
2. Add `start_resource_logs()`, `start_scope_logs()`, `append_log_record()` methods
3. Add `close_scope_logs()`, `close_resource_logs()`, `flush()` methods
4. Extract and reuse encoding helpers from existing code
5. Write unit tests for state transitions

**Deliverables:**
- Working encoder with state management
- Unit tests for all state transitions
- Tests for encoding correctness

### Phase 2: Context and Hashing (Week 2)

**Files to modify:**
- `crates/engine/src/pipeline_context.rs` (new or extend existing)
- `crates/pdata/src/otlp/stateful_encoder.rs` (add hashing)

**Tasks:**
1. Create/extend `PipelineContext` with precomputed Resource/Scope
2. Implement hash functions for Resource and InstrumentationScope
3. Add hash-based comparison to encoder
4. Add tests for hash consistency and collision handling

**Deliverables:**
- PipelineContext with cached hashes
- Hash functions with tests
- Benchmark showing comparison speedup

### Phase 3: Effect Handler Integration (Week 3)

**Files to modify:**
- `crates/engine/src/effect_handler.rs` - add encoder field
- `crates/telemetry/src/internal_events.rs` - update macros

**Tasks:**
1. Add `StatefulOtlpEncoder` field to `EffectHandlerCore`
2. Update constructor to initialize encoder
3. Implement `log_event_fast()` function
4. Update `otel_*!` macros to use new function
5. Add flush logic with threshold checking

**Deliverables:**
- Effect handler with integrated encoder
- Working macros
- End-to-end tests

### Phase 4: Testing and Optimization (Week 4)

**Tasks:**
1. Comprehensive integration tests
2. Performance benchmarks
3. Memory profiling
4. Edge case testing (large messages, many attributes, etc.)
5. Documentation

**Deliverables:**
- Full test coverage
- Performance report
- Documentation and examples

## Example Usage

### Initialization

```rust
// During pipeline startup
let resource = Resource {
    attributes: vec![
        KeyValue {
            key: "service.name".into(),
            value: AnyValue::String("my-service".into()),
        },
        KeyValue {
            key: "service.version".into(),
            value: AnyValue::String("1.0.0".into()),
        },
    ],
    dropped_attributes_count: 0,
};

let scope = InstrumentationScope {
    name: "otap-dataflow".into(),
    version: Some("0.1.0".into()),
    attributes: vec![],
    dropped_attributes_count: 0,
};

let context = Arc::new(PipelineContext::new(resource, scope));

let effect = EffectHandler::new(
    node_id,
    metrics_reporter,
    context,
    64 * 1024, // 64KB threshold
);
```

### Runtime Usage

```rust
// In a processor
impl Processor {
    async fn process(&mut self, msg: Message, effect: &mut EffectHandler) {
        // Fast path - automatic batching
        otel_info!(effect, "processor.batch_received", 
            batch_size = msg.batch_size,
            records = msg.record_count,
        );
        
        // ... process message ...
        
        // Another event - will be batched with previous if same Resource/Scope
        otel_debug!(effect, "processor.batch_processed",
            duration_us = elapsed.as_micros(),
        );
        
        // Flush happens automatically when threshold reached
        // Or manually via effect.flush_otlp_buffer()
    }
}
```

### Batching Behavior

```rust
// Example sequence with automatic batching

// Event 1: Start new ResourceLogs and ScopeLogs
otel_info!(effect, "event1"); 
// Buffer: ResourceLogs(open) -> ScopeLogs(open) -> LogRecord1

// Event 2: Same Resource/Scope - append to existing
otel_info!(effect, "event2");
// Buffer: ResourceLogs(open) -> ScopeLogs(open) -> LogRecord1, LogRecord2

// Event 3: Different Scope (e.g., different logger) - close and start new
otel_info!(effect_with_different_scope, "event3");
// Buffer: ResourceLogs(closed), ResourceLogs(open) -> ScopeLogs(open) -> LogRecord3

// Flush: Close everything and send
effect.flush_otlp_buffer();
// Buffer: Empty, all messages sent as OTLP bytes
```

## Validation Strategy

### Unit Tests

1. **State machine tests**: Verify all state transitions
2. **Encoding correctness**: Compare output with prost-generated code
3. **Hash consistency**: Verify same Resource/Scope produces same hash
4. **Length patching**: Verify lengths are correctly calculated and patched

### Integration Tests

1. **Single record**: Verify non-batched case works
2. **Batched records**: Verify multiple records in same Resource/Scope
3. **Scope changes**: Verify Scope closure and new Scope start
4. **Resource changes**: Verify full closure and restart
5. **Flush behavior**: Verify threshold-based and manual flush

### Property-Based Tests

1. **Roundtrip**: Encode then decode, verify data matches
2. **Fuzz testing**: Random sequences of encode/flush operations
3. **Invariants**: Verify buffer state is always valid

## Success Criteria

- [ ] Encoder correctly implements OTLP protobuf format
- [ ] All state transitions work correctly
- [ ] Encoding performance: < 200ns per event (batched)
- [ ] Memory overhead: < 100KB per core
- [ ] Batching works automatically and correctly
- [ ] Hash-based comparison is faster than full comparison
- [ ] Integration with effect handler is seamless
- [ ] All tests pass
- [ ] Documentation is complete

## Next Steps

1. Review this design with team
2. Create implementation branch
3. Start with Phase 1 (core encoder)
4. Iterate based on testing results
5. Integrate with custom tracing subscriber plan

## References

- [Custom Tracing Subscriber Plan](./custom-tracing-subscriber-plan.md)
- [View-Based OTLP Encoder Design](./view-based-otlp-encoder-design.md)
- [Existing OTLP encoder](../crates/pdata/src/otlp/logs.rs)
- [OTLP length padding macro](../crates/pdata/src/otlp/common.rs#L482)
- [OTLP Logs Specification](https://opentelemetry.io/docs/specs/otel/protocol/file-exporter/)
