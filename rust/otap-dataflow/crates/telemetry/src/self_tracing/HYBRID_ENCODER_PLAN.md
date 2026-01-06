# Hybrid Log Record Encoder: Planning Document

## Overview

This document plans a hybrid approach for encoding tracing events that:

1. **Keeps cheap, useful data in structural form** (for sorting, filtering, indexing)
2. **Encodes borrowed/expensive data to OTLP bytes** (body, attributes)
3. **Caches static callsite details** as pre-encoded LogRecord.event_name bytes

## Current vs. Proposed Architecture

### Current: Full OTLP Encoding

```
Event → StatefulDirectEncoder → Complete OTLP bytes
         ├── ResourceLogs envelope
         ├── ScopeLogs envelope (scope name string)
         └── LogRecord (all fields as protobuf)
```

**Issues:**
- All fields encoded immediately at event time
- Callsite info (target/name/file/line) re-encoded for every event
- Can't sort/filter without decoding

### Proposed: Hybrid Structural + Partial OTLP

```
Event → CompactLogRecord → Accumulate → Batch encode
         ├── callsite_id: Identifier (for cached event_name lookup)
         ├── timestamp_ns: u64 (structural, cheap copy)
         ├── severity: u8 (structural, cheap copy)
         └── body_attrs_bytes: Bytes (OTLP body+attributes only)
```

**Benefits:**
- Callsite details (target/name/file/line) encoded once per unique log statement
- Cached event_name bytes appended to each LogRecord at flush time
- Structural fields available for filtering/indexing
- Body+attributes already OTLP-encoded (common output path)

## Tokio Tracing Event Anatomy

```rust
// From tracing crate
pub struct Event<'a> {
    fields: ValueSet<'a>,      // Borrowed from callsite + formatted values
    metadata: &'static Metadata<'static>,  // Static callsite metadata
}

pub struct Metadata<'static> {
    name: &'static str,        // Static
    target: &'static str,      // Static (module path)
    level: Level,              // Static
    file: Option<&'static str>,
    line: Option<u32>,
    callsite: Identifier,      // &'static dyn Callsite
    // ...
}
```

**Key insight:** `Metadata` is `'static` (owned by callsite). Only the formatted field *values* are borrowed from the event.

## What to Keep Structural vs. Encode as OTLP

| Field | Lifetime | Keep Structural? | Rationale |
|-------|----------|------------------|-----------|
| `callsite.Identifier` | `'static` | ✓ | Key for cached event_name lookup |
| `metadata.level` | `'static` | ✓ | Cheap u8, useful for filtering |
| `timestamp` | Generated | ✓ | Cheap u64, useful for sorting |
| `metadata.target` | `'static` | Cache → event_name | Static, encode once per callsite |
| `metadata.name` | `'static` | Cache → event_name | Static, encode once per callsite |
| `metadata.file/line` | `'static` | Cache → event_name | Static, encode once per callsite |
| `body` (message) | `'a` | **Encode** | Borrowed, must capture |
| `attributes` | `'a` | **Encode** | Borrowed values, must capture |

## Proposed Data Structures

### Core Insight

Since tracing provides lazy callsite registration via `register_callsite`, we can:

1. **Cache encoded event_name bytes** per callsite at the subscriber level
2. **Store minimal event structs** with just `Identifier` + structural fields + pre-encoded body/attrs bytes
3. **Append event_name on flush** - look up cached bytes from Identifier when encoding each LogRecord

### `CompactLogRecord`

```rust
/// A compact log record with structural metadata and pre-encoded body/attributes.
/// 
/// Cheap-to-copy fields are kept in structural form for sorting/filtering.
/// Only borrowed data (body, attributes) is encoded to OTLP bytes.
/// Callsite details (target/name/file/line) are cached and appended at flush time.
pub struct CompactLogRecord {
    /// Callsite identifier - used to look up cached event_name encoding
    pub callsite_id: Identifier,
    
    /// Timestamp in nanoseconds since Unix epoch (cheap u64 copy)
    pub timestamp_ns: u64,
    
    /// Severity number: 1=TRACE, 5=DEBUG, 9=INFO, 13=WARN, 17=ERROR (cheap u8 copy)
    pub severity_number: u8,
    
    /// Severity text - &'static str from Level::as_str() (no allocation)
    pub severity_text: &'static str,
    
    /// Pre-encoded OTLP bytes for body (field 5) and attributes (field 6) only
    /// These are the only fields with borrowed lifetimes that must be captured
    pub body_attrs_bytes: Bytes,
}
```

**Why this split?**

| Field | Size | Keep Structural | Rationale |
|-------|------|-----------------|-----------|
| `callsite_id` | 8 bytes | ✓ | Pointer to static callsite, for event_name lookup |
| `timestamp_ns` | 8 bytes | ✓ | Useful for time-based sorting/filtering |
| `severity_number` | 1 byte | ✓ | Useful for level filtering |
| `severity_text` | 16 bytes | ✓ | `&'static str`, just a pointer+len |
| `body` | variable | **Encode** | Borrowed `&str` or formatted, lifetime ends |
| `attributes` | variable | **Encode** | Borrowed values, lifetime ends |
| `event_name` | variable | **Cache** | Static callsite info, encode once per callsite |

Total structural overhead per event: ~33 bytes + `Bytes` (Arc pointer)

### Subscriber-Level Callsite Cache

The key insight: callsite metadata (target, module, file, line) are **static properties of the log statement**, not the scope. We encode them once per callsite and include them in each LogRecord's `event_name` field.

```rust
/// Cache of pre-encoded callsite details, keyed by callsite Identifier.
/// 
/// Populated lazily via `register_callsite` hook.
pub struct CallsiteCache {
    /// Map from Identifier to pre-encoded callsite details
    callsites: HashMap<Identifier, CachedCallsite>,
}

pub struct CachedCallsite {
    /// Target module path - &'static from Metadata
    pub target: &'static str,
    
    /// Event name - &'static from Metadata
    pub name: &'static str,
    
    /// Source file - &'static from Metadata
    pub file: Option<&'static str>,
    
    /// Source line
    pub line: Option<u32>,
    
    /// Pre-encoded LogRecord.event_name OTLP bytes (lazily computed on first flush)
    /// Format: "target::name" or "target::name (file:line)"
    pub event_name_bytes: OnceCell<Bytes>,
}

impl CallsiteCache {
    /// Called from register_callsite hook
    pub fn register(&mut self, metadata: &'static Metadata<'static>) {
        let id = metadata.callsite();
        self.callsites.entry(id).or_insert_with(|| CachedCallsite {
            target: metadata.target(),
            name: metadata.name(),
            file: metadata.file(),
            line: metadata.line(),
            event_name_bytes: OnceCell::new(),
        });
    }
    
    /// Get or lazily encode event_name bytes for an Identifier
    pub fn get_event_name_bytes(&self, id: &Identifier) -> &Bytes {
        let cached = self.callsites.get(id).expect("callsite not registered");
        cached.event_name_bytes.get_or_init(|| {
            encode_event_name(cached.target, cached.name, cached.file, cached.line)
        })
    }
}

/// Encode callsite details as LogRecord.event_name field bytes.
/// 
/// Format options:
/// - "module::path::event_name"
/// - "module::path::event_name (file.rs:42)"
fn encode_event_name(
    target: &str, 
    name: &str, 
    file: Option<&str>, 
    line: Option<u32>
) -> Bytes {
    let mut buf = ProtoBuffer::with_capacity(128);
    
    // LogRecord.event_name (field 12, string)
    // Build the string: "target::name" or "target::name (file:line)"
    if let (Some(file), Some(line)) = (file, line) {
        let event_name = format!("{}::{} ({}:{})", target, name, file, line);
        buf.encode_string(LOG_RECORD_EVENT_NAME, &event_name);
    } else {
        let event_name = format!("{}::{}", target, name);
        buf.encode_string(LOG_RECORD_EVENT_NAME, &event_name);
    }
    
    buf.into_bytes()
}
```

### Design Evolution

```rust
// Original full-OTLP design:
pub struct StatefulDirectEncoder {
    // Encodes complete LogsData with ResourceLogs/ScopeLogs/LogRecord
    // All fields encoded immediately, scope batching only for consecutive
}

// New compact design:
pub struct CompactLogRecord {
    pub callsite_id: Identifier,        // For cached event_name lookup
    pub timestamp_ns: u64,              // Structural: for sorting/filtering
    pub severity_number: u8,            // Structural: for level filtering
    pub severity_text: &'static str,    // Structural: static, no alloc
    pub body_attrs_bytes: Bytes,        // Encoded: borrowed data captured
}
```

**Encoding strategy:**
- Structural fields encoded to OTLP at flush time (trivial: 9 + 2 + ~6 bytes)
- Body/attrs bytes appended directly (already OTLP encoded)
- event_name looked up from callsite cache, appended to each LogRecord

### `CallsiteRegistry`

The tracing crate maintains a global callsite registry internally, but it is **not exposed** for enumeration. The `Callsites::for_each` method is private.

However, we can build our own registry lazily via the `Subscriber::register_callsite` hook, which is called **once per callsite** before any events from that callsite are emitted:

```rust
impl<S: Subscriber> Layer<S> for HybridEncoderLayer {
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        // Called once per callsite, with static metadata we can store
        self.callsite_cache.register(metadata);
        Interest::always()
    }
}
```

**Key insight**: `Metadata<'static>` gives us `&'static str` references that we can store without lifetime issues. No allocation needed for callsite names.

Note: The `CallsiteRegistry` struct defined above is essentially the same as `CallsiteCache`, just with a different focus. We can consolidate these into a single `CallsiteCache` struct.

### `CompactLogRecord` Formatter

Instead of accumulating records, we format and write immediately. This is a minimal `fmt::layer()` alternative:

```rust
/// Formats a CompactLogRecord as a human-readable string.
/// 
/// This is our minimal fmt::layer() replacement.
pub fn format_log_record(record: &CompactLogRecord, callsite_cache: &CallsiteCache) -> String {
    let cached = callsite_cache.get(record.callsite_id);
    
    // Format: "2026-01-06T10:30:45.123Z  INFO target::name: body [attr1=val1, attr2=val2]"
    format!(
        "{}  {:5} {}::{}: {}",
        format_timestamp(record.timestamp_ns),
        record.severity_text,
        cached.target,
        cached.name,
        format_body_attrs(&record.body_attrs_bytes),
    )
}

/// Format nanosecond timestamp as ISO 8601
fn format_timestamp(nanos: u64) -> String {
    // TODO: Use a more efficient formatter
    let secs = nanos / 1_000_000_000;
    let subsec_nanos = (nanos % 1_000_000_000) as u32;
    // ... format as "2026-01-06T10:30:45.123Z"
}

/// Decode and format body+attrs bytes as readable string
fn format_body_attrs(bytes: &Bytes) -> String {
    // Decode the pre-encoded OTLP bytes back to readable form
    // Body becomes the main message, attrs become "[key=value, ...]"
}
```

### Simple Writer

```rust
use std::io::{self, Write};

pub enum OutputTarget {
    Stdout,
    Stderr,
}

pub struct SimpleWriter {
    target: OutputTarget,
}

impl SimpleWriter {
    pub fn stdout() -> Self {
        Self { target: OutputTarget::Stdout }
    }
    
    pub fn stderr() -> Self {
        Self { target: OutputTarget::Stderr }
    }
    
    pub fn write_line(&self, line: &str) {
        match self.target {
            OutputTarget::Stdout => {
                let _ = writeln!(io::stdout(), "{}", line);
            }
            OutputTarget::Stderr => {
                let _ = writeln!(io::stderr(), "{}", line);
            }
        }
    }
}
```

### Minimal Layer Implementation

```rust
pub struct CompactFormatterLayer {
    callsite_cache: RwLock<CallsiteCache>,
    writer: SimpleWriter,
}

impl<S: Subscriber> Layer<S> for CompactFormatterLayer {
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        self.callsite_cache.write().unwrap().register(metadata);
        Interest::always()
    }
    
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        
        // Encode body+attrs (borrowed data)
        let body_attrs_bytes = encode_body_and_attrs(event);
        
        // Build compact record
        let record = CompactLogRecord {
            callsite_id: metadata.callsite(),
            timestamp_ns: current_time_nanos(),
            severity_number: level_to_severity(metadata.level()),
            severity_text: metadata.level().as_str(),
            body_attrs_bytes,
        };
        
        // Format and write immediately
        let line = format_log_record(&record, &self.callsite_cache.read().unwrap());
        self.writer.write_line(&line);
    }
}
```

## Encoding Flow (Simplified)

### 1. Callsite Registration (once per callsite, lazy)

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
    self.callsite_cache.write().unwrap().register(metadata);
    Interest::always()
}
```

### 2. Event Capture → Format → Write (per event)

```rust
fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
    let metadata = event.metadata();
    
    // Encode body+attrs (borrowed data that won't survive)
    let body_attrs_bytes = encode_body_and_attrs(event);
    
    // Build compact record with structural fields
    let record = CompactLogRecord {
        callsite_id: metadata.callsite(),
        timestamp_ns: current_time_nanos(),
        severity_number: level_to_severity(metadata.level()),
        severity_text: metadata.level().as_str(),
        body_attrs_bytes,
    };
    
    // Format and write immediately (no accumulation)
    let line = format_log_record(&record, &self.callsite_cache.read().unwrap());
    self.writer.write_line(&line);
}
```

### Key Benefits (MVP)

1. **Simple**: No accumulator, no batching, no deferred encoding
2. **Immediate output**: Events written as they occur
3. **Composable**: Accumulator/batching can be layered on later
4. **Testable**: `format_log_record()` returns String, easy to test
5. **Familiar**: Similar mental model to `fmt::layer()`

## Implementation Plan

### Phase 1: Core Data Structures ✅ COMPLETE
- [x] 1.1 Create `CompactLogRecord` struct (callsite_id + structural fields + body_attrs_bytes)
- [x] 1.2 Create `CallsiteCache` with `register()` and `get()`
- [x] 1.3 Create `CachedCallsite` struct storing static metadata refs

### Phase 2: Formatting ✅ COMPLETE
- [x] 2.1 Implement `format_log_record()` → String
- [x] 2.2 Implement `format_timestamp()` for ISO 8601 output
- [x] 2.3 Implement `format_body_attrs()` using pdata View types (`RawAnyValue`, `RawKeyValue`)
- [x] 2.4 Create `SimpleWriter` for stdout/stderr output
- [x] 2.5 Implement `format_any_value()` consistent with `otlp_bytes_formatter.rs`

### Phase 3: Body+Attrs Encoder ✅ COMPLETE
- [x] 3.1 Reuse `DirectFieldVisitor` for body+attrs encoding
- [x] 3.2 Create `encode_body_and_attrs(event) -> Bytes` function

### Phase 4: Layer Integration ✅ COMPLETE
- [x] 4.1 Create `CompactFormatterLayer` implementing tracing Layer
- [x] 4.2 Implement `register_callsite()` to populate CallsiteCache
- [x] 4.3 Implement `on_event()` to encode, format, and write immediately
- [x] 4.4 Add basic tests with mock subscriber

### Phase 5: Future Extensions (deferred)
- [ ] 5.1 Add `LogAccumulator` for batching
- [ ] 5.2 Add OTLP encoding path (flush to bytes)
- [ ] 5.3 Add configurable output formats (JSON, compact, etc.)

## Open Questions (Resolved)

1. **Body+attrs encoding**: ✅ We encode to OTLP bytes using `DirectFieldVisitor`, then decode for formatting using pdata View types (`RawAnyValue`, `RawKeyValue`). This keeps the data path consistent with future OTLP batching.

2. **Timestamp format**: ✅ ISO 8601 with milliseconds: `2026-01-06T10:30:45.123Z`

3. **Output format**: ✅ Single compact format for MVP: `timestamp LEVEL target::name: body [attr=value, ...]`

4. **Thread safety**: ✅ `RwLock<CallsiteCache>` - readers don't block each other, writes are rare (only during callsite registration)

5. **Color support**: Deferred to future work (can be added to `SimpleWriter`)

## Resolved Design Decisions

1. **pdata View integration**: Instead of writing custom OTLP decoders, we reuse the existing `RawAnyValue`, `RawKeyValue` types from `otap_df_pdata::views::otlp::bytes::common`. Made `RawKeyValue::new()` public to enable this.

2. **format_any_value consistency**: The `format_any_value()` function in `compact_formatter.rs` matches the implementation in `otlp_bytes_formatter.rs`, ensuring consistent formatting across the crate.

## Success Metrics

1. **Simplicity**: MVP should be <300 lines of code
2. **Correctness**: Output matches expected format for all log levels
3. **Performance**: Comparable to `fmt::layer()` for immediate writes
4. **Extensibility**: Easy to add accumulator/batching layer later

---

## Next Steps

Please review this plan and let me know:

1. Do the proposed data structures align with your vision?
2. Any changes to what should be structural vs. encoded?
3. Which phase should we start with?
4. Answers to any of the open questions?
