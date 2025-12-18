# View-Based OTLP Encoder Design

**Status**: Design Study  
**Date**: December 17, 2025  
**Context**: Custom Tracing Subscriber - Riskiest Component Analysis

## Executive Summary

This document analyzes the approach for building a **bespoke OTLP bytes encoder** that uses the existing `LogsDataView` trait to encode OTLP protobuf messages in a single pass. The key innovation is **1-pass encoding with padded length-delimited fields**, allowing us to write OTLP bytes in traversal order without buffering or backtracking.

## Background: Current OTLP Encoding

### Existing Implementation Pattern

The current OTAP-to-OTLP encoder (in [crates/pdata/src/otlp/logs.rs](../crates/pdata/src/otlp/logs.rs)) uses a clever 1-pass encoding strategy:

```rust
// From proto_encode_len_delimited_unknown_size! macro
proto_encode_len_delimited_unknown_size!(
    FIELD_TAG,
    encode_child_content(&mut buf)?,  // Don't know child size yet!
    buf
);
```

**How it works:**

1. **Reserve Fixed Padding**: Always reserve 4 bytes for length (even if length fits in 1-2 bytes)
2. **Write Length Placeholder**: Write `[0x80, 0x80, 0x80, 0x00]` (zero-padded varint with continuation bits set)
3. **Encode Child Content**: Write child message bytes inline
4. **Patch Length Field**: After child complete, compute actual length and patch the 4-byte placeholder

**Example:** If child is 150 bytes:
```
Before patch:  [tag] [0x80, 0x80, 0x80, 0x00] [... 150 bytes of child ...]
After patch:   [tag] [0x96, 0x81, 0x80, 0x00] [... 150 bytes of child ...]
               Varint 150 = 0b10010110, 0b00000001 (with continuation bits)
```

### Key Insight: Size Constraints Enable Padding

From the macro comments in [common.rs:482](../crates/pdata/src/otlp/common.rs#L482):

> Our proto encoding algorithm tries to encode in a single pass over the OTAP data, but it will not know the size of the nested child messages a priori. Because the length fields are encoded in a varint, we don't know how many bytes we need to set aside for the length before we start appending the encoded child.
>
> The workaround is that we set aside a fixed length number of bytes, and create a zero-padded varint.

**Current choice:** Always use 4 bytes (supports lengths up to ~2^28 = 268MB per message)

**User's proposed optimization:** If max log event is 16KiB, then:
- Max size = 2^14 bytes
- Varint encoding: 2^14 requires 2 bytes (14 bits = 7 bits + 7 bits)
- Therefore: **use 2-byte padding instead of 4-byte padding**

## Proposed Design: View-Based OTLP Encoder

### Architecture Overview

```
LogsDataView trait (existing)
       │
       v
SingleLogRecordView (new)
  - Wraps single LogRecord
  - Implements LogsDataView
  - Ephemeral (stack-allocated)
       │
       v
BespokeOtlpEncoder (new)
  - Uses LogsDataView API
  - 1-pass encoding with 2-byte padding
  - Accumulates multiple records per resource
  - Outputs OTLP bytes to Vec<u8>
```

### Component 1: SingleLogRecordView

Create an ephemeral view wrapper for a single log record that implements `LogsDataView`:

```rust
/// Ephemeral view for encoding a single LogRecord with its Resource and Scope context
pub struct SingleLogRecordView<'a> {
    log_record: &'a LogRecord,
    resource: &'a Resource,
    scope: &'a InstrumentationScope,
}

impl<'a> LogsDataView for SingleLogRecordView<'a> {
    type ResourceLogs<'res> = SingleResourceLogsView<'res> where Self: 'res;
    type ResourcesIter<'res> = std::iter::Once<Self::ResourceLogs<'res>> where Self: 'res;

    fn resources(&self) -> Self::ResourcesIter<'_> {
        std::iter::once(SingleResourceLogsView {
            resource: self.resource,
            scope: self.scope,
            log_record: self.log_record,
        })
    }
}

// Similar implementations for ResourceLogsView, ScopeLogsView, LogRecordView
```

**Benefits:**
- Zero heap allocation (borrows existing data)
- Stack-only structure
- Short-lived (only during encoding)
- Reuses existing `LogsDataView` trait

### Component 2: BespokeOtlpEncoder

A specialized encoder optimized for streaming single log events:

```rust
/// Bespoke OTLP encoder optimized for single-record encoding with 2-byte padding
pub struct BespokeOtlpEncoder {
    /// Reusable buffer for encoding
    buffer: Vec<u8>,
    
    /// Size hint for padding (2 bytes for 16KiB max, 4 bytes for larger)
    padding_bytes: usize,
    
    /// Context for the current batch (Resource + Scope)
    current_resource: Option<Resource>,
    current_scope: Option<InstrumentationScope>,
    
    /// Position tracking for batch assembly
    resource_start_pos: Option<usize>,
    scope_start_pos: Option<usize>,
}

impl BespokeOtlpEncoder {
    /// Create encoder with specified padding size
    pub fn new(max_event_size_bytes: usize) -> Self {
        // Calculate required padding based on max size
        let padding_bytes = if max_event_size_bytes <= (1 << 14) {
            2  // 16KiB max = 2 bytes
        } else if max_event_size_bytes <= (1 << 21) {
            3  // 2MB max = 3 bytes
        } else {
            4  // 268MB max = 4 bytes
        };
        
        Self {
            buffer: Vec::with_capacity(max_event_size_bytes * 4),
            padding_bytes,
            current_resource: None,
            current_scope: None,
            resource_start_pos: None,
            scope_start_pos: None,
        }
    }
    
    /// Encode a single log record using the view API
    pub fn encode_log_record(
        &mut self,
        view: &impl LogsDataView,
    ) -> Result<(), Error> {
        // Use the LogsDataView API to traverse and encode
        for resource_logs in view.resources() {
            self.encode_resource_logs(&resource_logs)?;
        }
        Ok(())
    }
    
    /// Encode with resource/scope batching optimization
    pub fn encode_log_record_batched(
        &mut self,
        log_record: &LogRecord,
        resource: &Resource,
        scope: &InstrumentationScope,
    ) -> Result<(), Error> {
        // Check if we can append to existing ResourceLogs
        if self.can_batch_with_current_resource(resource) {
            // Same resource - check scope
            if self.can_batch_with_current_scope(scope) {
                // Same scope - just append LogRecord
                self.append_log_record(log_record)?;
            } else {
                // New scope - close current ScopeLogs, start new one
                self.close_current_scope()?;
                self.start_new_scope(scope)?;
                self.append_log_record(log_record)?;
            }
        } else {
            // New resource - close everything, start fresh
            self.close_current_scope()?;
            self.close_current_resource()?;
            self.start_new_resource(resource)?;
            self.start_new_scope(scope)?;
            self.append_log_record(log_record)?;
        }
        
        Ok(())
    }
    
    /// Take accumulated bytes and reset encoder
    pub fn take_bytes(&mut self) -> Vec<u8> {
        self.close_all();
        std::mem::take(&mut self.buffer)
    }
    
    // Private methods for 1-pass encoding
    
    fn encode_len_delimited<F>(
        &mut self,
        field_tag: u32,
        encode_fn: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(&mut Vec<u8>) -> Result<(), Error>,
    {
        // Encode field tag
        self.encode_field_tag(field_tag, WIRE_TYPE_LEN);
        
        // Reserve padding for length
        let len_start_pos = self.buffer.len();
        self.encode_len_placeholder();
        
        // Encode child content
        let content_start = self.buffer.len();
        encode_fn(&mut self.buffer)?;
        let content_len = self.buffer.len() - content_start;
        
        // Patch length field
        self.patch_len_placeholder(len_start_pos, content_len);
        
        Ok(())
    }
    
    fn encode_len_placeholder(&mut self) {
        match self.padding_bytes {
            2 => self.buffer.extend_from_slice(&[0x80, 0x00]),
            3 => self.buffer.extend_from_slice(&[0x80, 0x80, 0x00]),
            4 => self.buffer.extend_from_slice(&[0x80, 0x80, 0x80, 0x00]),
            _ => unreachable!("Invalid padding size"),
        }
    }
    
    fn patch_len_placeholder(&mut self, start_pos: usize, len: usize) {
        for i in 0..self.padding_bytes {
            self.buffer[start_pos + i] += ((len >> (i * 7)) & 0x7f) as u8;
        }
    }
}
```

### Component 3: Batching Optimization

**Key insight:** Most consecutive log events share the same Resource and often the same InstrumentationScope. We can optimize by keeping the current `ResourceLogs` and `ScopeLogs` "open" and appending new `LogRecord`s to them.

```rust
impl BespokeOtlpEncoder {
    fn start_new_resource(&mut self, resource: &Resource) -> Result<(), Error> {
        // Start a new ResourceLogs message
        let pos = self.buffer.len();
        
        // Write ResourceLogs field tag + placeholder length
        self.encode_field_tag(LOGS_DATA_RESOURCE, WIRE_TYPE_LEN);
        self.resource_start_pos = Some(self.buffer.len());
        self.encode_len_placeholder();
        
        // Encode Resource message
        self.encode_resource(resource)?;
        
        // Save resource for comparison
        self.current_resource = Some(resource.clone());
        
        Ok(())
    }
    
    fn append_log_record(&mut self, log_record: &LogRecord) -> Result<(), Error> {
        // Append LogRecord to current ScopeLogs
        self.encode_len_delimited(SCOPE_LOGS_LOG_RECORDS, |buf| {
            encode_log_record_fields(log_record, buf)
        })
    }
    
    fn close_current_scope(&mut self) -> Result<(), Error> {
        if let Some(start_pos) = self.scope_start_pos.take() {
            // Patch the ScopeLogs length field
            let total_len = self.buffer.len() - start_pos - self.padding_bytes;
            self.patch_len_placeholder(start_pos, total_len);
        }
        self.current_scope = None;
        Ok(())
    }
    
    fn close_current_resource(&mut self) -> Result<(), Error> {
        if let Some(start_pos) = self.resource_start_pos.take() {
            // Patch the ResourceLogs length field
            let total_len = self.buffer.len() - start_pos - self.padding_bytes;
            self.patch_len_placeholder(start_pos, total_len);
        }
        self.current_resource = None;
        Ok(())
    }
}
```

**Batching benefits:**
1. **Space efficiency**: One `Resource` + `InstrumentationScope` per batch instead of per record
2. **Spec compliance**: Matches OTLP's hierarchical structure naturally
3. **Optimization opportunity**: Check for repeated Resource/Scope and batch automatically

## Integration with Effect Handler

### Usage Pattern

```rust
// In the otel_info! macro expansion
pub fn log_event_fast(
    effect: &impl EffectHandlerTraceBuffer,
    level: Level,
    name: &str,
    fields: &[(&str, &dyn Debug)],
    location: &'static Location<'static>,
) {
    // 1. Build LogRecord inline (stack allocation)
    let log_record = build_log_record_inline(
        level,
        name,
        fields,
        location,
        effect.pipeline_context(),
    );
    
    // 2. Get encoder from effect handler
    let mut encoder = effect.otlp_encoder_mut();
    
    // 3. Encode with batching (checks Resource/Scope)
    encoder.encode_log_record_batched(
        &log_record,
        effect.pipeline_context().resource(),
        effect.pipeline_context().scope(),
    ).expect("encoding failed");
    
    // 4. Check size threshold
    if encoder.buffer_len() >= effect.otlp_buffer_threshold() {
        effect.flush_otlp_buffer().ok();
    }
}
```

### Effect Handler Storage

```rust
pub struct EffectHandlerCore<PData> {
    // ... existing fields ...
    
    /// Bespoke OTLP encoder (reusable)
    otlp_encoder: RefCell<BespokeOtlpEncoder>,
    
    /// Flush threshold in bytes
    otlp_buffer_flush_threshold: usize,
}

impl<PData> EffectHandlerCore<PData> {
    pub fn otlp_encoder_mut(&self) -> RefMut<BespokeOtlpEncoder> {
        self.otlp_encoder.borrow_mut()
    }
    
    pub fn flush_otlp_buffer(&self) -> Result<(), Error> {
        let mut encoder = self.otlp_encoder.borrow_mut();
        
        // Take accumulated bytes
        let otlp_bytes = encoder.take_bytes();
        
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

## Performance Analysis

### Memory Efficiency

**Per-core overhead:**
- `BespokeOtlpEncoder`: ~100 bytes struct
- Buffer capacity: Grows to working set (e.g., 64KB)
- Total: ~64KB per core (configurable)

**Encoding overhead per event:**
- `LogRecord` struct: ~200 bytes (stack-allocated, short-lived)
- `SingleLogRecordView`: ~24 bytes (stack-allocated, short-lived)
- Zero heap allocations for encoding

### CPU Efficiency

**Single-event encoding (no batching):**
1. Build `LogRecord`: ~30ns
2. Create `SingleLogRecordView`: ~10ns
3. Encode OTLP (with padding): ~100-150ns
4. Total: **~200ns per event**

**Batched encoding (same Resource/Scope):**
1. Build `LogRecord`: ~30ns
2. Check Resource/Scope match: ~20ns
3. Append `LogRecord` only: ~80ns
4. Total: **~130ns per event** (35% faster)

**Flush overhead:**
- Close current Scope: ~50ns
- Close current Resource: ~50ns
- Patch length fields: ~20ns
- Total: **~120ns per flush**

### Space Efficiency

**Padding overhead:**
- 2-byte padding: 2 bytes per length field
- Fields per LogRecord: ~5-10 length-delimited fields
- Overhead: ~10-20 bytes per record (1-2%)

**Batching savings:**
- Without batching: Resource (100B) + Scope (50B) per record = 150B overhead
- With batching (10 records/batch): 150B / 10 = 15B per record
- **Savings: ~135B per record (batched) = 90% reduction**

### Threshold Trade-offs

| Threshold | Behavior | Latency | Batch Size | Memory |
|-----------|----------|---------|------------|--------|
| 1 byte | Synchronous (flush every event) | ~5μs | 1 record | Minimal |
| 4KB | Small batches | ~5μs / 20 events | 10-20 records | 4KB |
| 64KB | Large batches | ~5μs / 300 events | 100-300 records | 64KB |

**Recommended:** 64KB (balances throughput and latency)

## Implementation Phases

### Phase 1: Core Encoder (Week 1)

**Tasks:**
1. Implement `BespokeOtlpEncoder` with 1-pass encoding
2. Add configurable padding (2/3/4 bytes)
3. Write unit tests for encoding correctness
4. Validate against existing OTLP encoder output

**Deliverables:**
- `crates/pdata/src/otlp/bespoke_encoder.rs`
- Unit tests with known OTLP messages
- Benchmark comparing to existing encoder

### Phase 2: View Integration (Week 2)

**Tasks:**
1. Implement `SingleLogRecordView`
2. Add `LogsDataView`-based encoding path
3. Test view-based encoding
4. Add batching optimization (Resource/Scope checking)

**Deliverables:**
- `crates/pdata/src/views/single_record.rs`
- Integration tests with `LogsDataView`
- Batching benchmarks

### Phase 3: Effect Handler Integration (Week 3)

**Tasks:**
1. Add `BespokeOtlpEncoder` to effect handler
2. Update `otel_*!` macros to use encoder
3. Implement flush logic
4. Add configuration for threshold and padding

**Deliverables:**
- Updated `crates/engine/src/effect_handler.rs`
- Updated `crates/telemetry/src/internal_events.rs`
- Configuration schema
- End-to-end tests

## Risk Analysis

### Risk 1: Incorrect Length Patching

**Risk:** Length calculation or patching logic has bugs, resulting in invalid OTLP messages.

**Mitigation:**
- Extensive unit tests with known OTLP messages
- Fuzz testing with random log records
- Validation against prost-generated code
- Cross-check with existing encoder output

### Risk 2: Padding Overflow

**Risk:** Message exceeds padding size (e.g., > 16KiB with 2-byte padding).

**Mitigation:**
- Add size checks before encoding
- Return error if message too large
- Option: Dynamically switch to larger padding if needed
- Document size limits clearly

### Risk 3: Batching Bugs

**Risk:** Resource/Scope checking logic fails, resulting in incorrect batching.

**Mitigation:**
- Comprehensive tests for all batching scenarios:
  - Same Resource, same Scope
  - Same Resource, different Scope
  - Different Resource
  - Interleaved resources
- Add debug assertions

### Risk 4: Performance Regression

**Risk:** Custom encoder is slower than expected.

**Mitigation:**
- Benchmark against existing encoder
- Profile hot paths
- Optimize critical sections
- Fall back to simpler approach if needed

## Alternative Approaches Considered

### Alternative 1: Two-Pass Encoding

**Description:** Calculate all sizes first, then encode with exact lengths.

**Pros:**
- No wasted space from padding
- Simpler length handling

**Cons:**
- Requires full traversal before encoding
- More complex code (two passes)
- Higher CPU cost
- Doesn't match existing pattern

**Decision:** Rejected - 1-pass is more efficient and matches existing code

### Alternative 2: Use Existing OTAP Encoder

**Description:** Convert `LogRecord` → OTAP batch → OTLP bytes.

**Pros:**
- Reuses existing code
- Less new code to maintain

**Cons:**
- Significant overhead (Arrow batch creation)
- Memory allocations
- Doesn't leverage view API
- Overkill for single records

**Decision:** Rejected - Too much overhead for single records

### Alternative 3: Use prost Directly

**Description:** Use prost's `Message::encode()` for each record.

**Pros:**
- Simple and proven
- Handles all edge cases
- Maintained by prost team

**Cons:**
- Requires full `LogsData` struct in memory
- Can't do streaming encoding
- Can't optimize batching
- Allocates for each encoding

**Decision:** Rejected - Doesn't support streaming or batching

## Open Questions

### Q1: Should we dynamically adjust padding?

**Question:** If a message is too large for current padding, should we:
- A) Return error (fail fast)
- B) Reallocate with larger padding (fallback)
- C) Start new batch (split)

**Recommendation:** Start with (A) for simplicity, consider (C) for robustness.

### Q2: How to handle Resource/Scope changes efficiently?

**Question:** Should we:
- A) Always batch (compare every time)
- B) Never batch (always new Resource/Scope)
- C) Configurable batching strategy

**Recommendation:** (A) - Comparison is cheap (~20ns), batching savings are large.

### Q3: Should we support TracesDataView too?

**Question:** This design focuses on logs, but the same pattern applies to traces.

**Recommendation:** Yes, but start with logs. Traces can follow the same pattern later.

### Q4: Buffer pre-allocation strategy?

**Question:** How should we grow the buffer?
- A) Start small, grow exponentially
- B) Pre-allocate to threshold size
- C) Fixed capacity, error if exceeded

**Recommendation:** (B) - Pre-allocate to threshold, avoids reallocations.

## Success Criteria

### Correctness

- [ ] All encoded OTLP messages validate with protobuf decoder
- [ ] Output matches existing encoder (byte-for-byte, except padding)
- [ ] Handles all LogRecord field types
- [ ] Batching produces valid hierarchical structure

### Performance

- [ ] Single-event encoding: < 200ns
- [ ] Batched encoding: < 150ns per event
- [ ] Flush overhead: < 150ns
- [ ] Memory overhead: < 100KB per core

### Maintainability

- [ ] Clear separation of concerns
- [ ] Reusable for TracesDataView
- [ ] Well-documented padding logic
- [ ] Comprehensive tests

## Related Documents

- [Custom Tracing Subscriber Plan](./custom-tracing-subscriber-plan.md) - Overall architecture
- [LogsDataView trait](../crates/pdata/src/views/logs.rs) - View API
- [Existing OTLP encoder](../crates/pdata/src/otlp/logs.rs) - Current implementation
- [OTAP encoding](../crates/pdata/src/encode/mod.rs) - OTAP batch encoding

## Appendix A: Wire Format Examples

### Example 1: Single LogRecord (2-byte padding)

```
LogsData {
  resource_logs: [
    ResourceLogs {
      resource: { ... }
      scope_logs: [
        ScopeLogs {
          scope: { ... }
          log_records: [
            LogRecord { time_unix_nano: 123, body: "hello", ... }
          ]
        }
      ]
    }
  ]
}
```

**Encoded bytes (with 2-byte padding):**
```
0x0A           # Tag for LogsData.resource_logs (field 1, LEN)
0x80 0x00      # Length placeholder (2 bytes, will be patched)
  0x0A         # Tag for ResourceLogs.resource (field 1, LEN)
  0x80 0x00    # Length placeholder (2 bytes)
    ...        # Resource fields
  0x12         # Tag for ResourceLogs.scope_logs (field 2, LEN)
  0x80 0x00    # Length placeholder (2 bytes)
    0x0A       # Tag for ScopeLogs.scope (field 1, LEN)
    0x80 0x00  # Length placeholder (2 bytes)
      ...      # Scope fields
    0x12       # Tag for ScopeLogs.log_records (field 2, LEN)
    0x80 0x00  # Length placeholder (2 bytes)
      0x08     # Tag for LogRecord.time_unix_nano (field 1, VARINT)
      0x7B     # Value: 123
      0x52     # Tag for LogRecord.body (field 10, LEN)
      0x05     # Length: 5
      "hello"  # Body content
      ...      # Other fields
```

**After patching lengths:**
```
0x0A           # Tag for LogsData.resource_logs
0xA5 0x01      # Length: 165 bytes (patched)
  0x0A         # Tag for ResourceLogs.resource
  0x93 0x00    # Length: 19 bytes (patched)
    ...
  0x12         # Tag for ResourceLogs.scope_logs
  0x8C 0x01    # Length: 140 bytes (patched)
    ...
```

### Example 2: Batched LogRecords

```
LogsData {
  resource_logs: [
    ResourceLogs {
      resource: { service.name: "my-service" }
      scope_logs: [
        ScopeLogs {
          scope: { name: "my-tracer" }
          log_records: [
            LogRecord { time: 100, body: "event1" },
            LogRecord { time: 101, body: "event2" },
            LogRecord { time: 102, body: "event3" },
          ]
        }
      ]
    }
  ]
}
```

**Key observation:** Resource and Scope encoded **once**, but three LogRecords appended sequentially. Total size: ~200 bytes instead of ~450 bytes (if each had its own Resource/Scope).

## Appendix B: Varint Encoding Reference

### Varint encoding rules:

- 7 bits of data per byte
- MSB = continuation bit (1 = more bytes follow, 0 = last byte)
- Little-endian (LSB first)

### Padding examples:

**1-byte actual (value 5) in 2-byte padded:**
```
Actual:  0x05                    # 5 in 1 byte
Padded:  0x85 0x00               # 5 in 2 bytes with continuation
         ^
         continuation bit set
```

**2-byte actual (value 150) in 2-byte padded:**
```
Actual:  0x96 0x01               # 150 in 2 bytes
Padded:  0x96 0x01               # Same (already 2 bytes)
```

**2-byte actual (value 16383 = 2^14 - 1) in 2-byte padded:**
```
Actual:  0xFF 0x7F               # 16383 in 2 bytes (max for 2 bytes)
Padded:  0xFF 0x7F               # Same
```

**3-byte actual (value 16384 = 2^14) requires 3-byte padding:**
```
Value:   16384 = 0b100000000000000
Encoded: 0x80 0x80 0x01          # Requires 3 bytes
```

### Size limits by padding:

| Padding Bytes | Max Value | Max Size | Use Case |
|---------------|-----------|----------|----------|
| 1 | 127 | 127 B | Tiny messages only |
| 2 | 16383 | ~16 KiB | Small log events |
| 3 | 2097151 | ~2 MiB | Large logs/spans |
| 4 | 268435455 | ~256 MiB | Huge batches |

**Recommendation:** Use 2-byte padding with 16KiB limit for log events.
