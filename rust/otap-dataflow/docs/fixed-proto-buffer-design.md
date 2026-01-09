# Fixed-Size Proto Buffer Design

## Problem Statement

The OTAP dataflow pipeline requires an internal logging path for self-diagnostics that feeds OTLP protocol bytes directly into the pipeline. This internal instrumentation has specific constraints:

1. **Safety**: Internal logging must not cause heap allocations that could interfere with the main data path or cause memory pressure during high-load scenarios.

2. **Low Impact**: The encoding path must be lightweight and predictable, suitable for use in hot paths like `tracing::info!` statements.

3. **Fixed-Size Buffers**: For stack-allocated buffers with a predetermined capacity, the encoder must handle out-of-space conditions gracefully rather than panicking or reallocating.

4. **Truncation Support**: When encoding attributes into a fixed buffer, if space runs out mid-encoding (e.g., while looping through event variables), the encoder should:
   - Return a "truncated" error result
   - Allow callers to use the partially-encoded contents
   - Enable tracking of dropped attributes

5. **Code Reuse**: We explicitly want to avoid maintaining two separate encoder implementations—one for growable buffers (normal telemetry path) and one for fixed-size buffers (internal instrumentation).

6. **Graceful Degradation**: Even for growable buffers, we want configurable limits to prevent unbounded growth from malformed or malicious data. Large attribute values and log bodies should be truncated gracefully with informative markers.

### OTLP Protocol Support

The OpenTelemetry LogRecord proto already provides mechanisms for handling truncation:

```protobuf
message LogRecord {
    // ... other fields ...
    uint32 dropped_attributes_count = 7;  // Track dropped attributes
    fixed32 flags = 8;                    // 5 bytes total (tag + fixed32)
}
```

This means we can:
- Reserve 5 bytes at the end of our encoding buffer for `dropped_attributes_count`
- Encode as many attributes as fit
- On truncation, count remaining attributes and encode the count in the reserved space

### Example Use Case

```rust
// During a tracing::info! statement, encode log attributes into a fixed buffer
let mut buf = FixedProtoBuffer::<1024>::new();

// Reserve space for dropped_attributes_count (tag=7 varint + uint32 varint = ~5 bytes)
buf.reserve_tail(5);

let mut encoded_count = 0;
for attr in event_attributes {
    if encode_key_value(&mut buf, attr).is_err() {
        // Truncation occurred - use partial contents
        break;
    }
    encoded_count += 1;
}

// Release reserved space and encode dropped count
let dropped_count = event_attributes.len() - encoded_count;
buf.release_tail(5);
if dropped_count > 0 {
    buf.encode_field_tag(7, WIRE_TYPE_VARINT);
    buf.encode_varint(dropped_count as u64);
}
```

## Solution

### Design Approach

The solution introduces a `ProtoWrite` trait that abstracts over buffer implementations, allowing encoding logic to work with both growable (`ProtoBuffer`) and fixed-size (`FixedProtoBuffer`) buffers through the same code path.

### Core Concepts

#### Buffer Space Model

```
|-------- written --------|----- remaining -----|---- reserved ----|
                          ^                     ^
                          len                   limit - reserved_tail
                          
effective_remaining = limit - len - reserved_tail
```

- **limit**: Maximum bytes that can be written (may be less than capacity)
- **reserved_tail**: Bytes reserved at the end for fields like `dropped_attributes_count`
- **effective_remaining**: Actual bytes available for the next write operation

#### Length Placeholder Optimization

When encoding nested messages, we don't know the size upfront, so we reserve placeholder bytes for the length varint and patch them afterward. The number of bytes needed depends on the maximum possible message size:

| Buffer Limit | Max Length | Varint Bytes | Savings vs 4-byte |
|-------------|------------|--------------|-------------------|
| ≤ 127 B     | 127        | 1 byte       | 75%              |
| ≤ 16 KiB    | 16383      | 2 bytes      | 50%              |
| ≤ 2 MiB     | 2097151    | 3 bytes      | 25%              |
| > 2 MiB     | 2^28-1     | 4 bytes      | 0%               |

For internal instrumentation with small fixed buffers (e.g., 1-4 KiB), using 2-byte placeholders instead of 4-byte saves significant space, especially in deeply nested structures like attributes within log records within scope logs within resource logs.

**Example savings for a LogRecord with 10 nested messages:**
- 4-byte placeholders: 40 bytes overhead
- 2-byte placeholders: 20 bytes overhead
- Savings: 20 bytes (could fit another small attribute!)

#### `LengthPlaceholderSize` Enum

```rust
/// Determines how many bytes to reserve for length placeholders in nested messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LengthPlaceholderSize {
    /// 1 byte - for buffers ≤ 127 bytes (max length = 127)
    OneByte,
    /// 2 bytes - for buffers ≤ 16 KiB (max length = 16383)
    TwoBytes,
    /// 3 bytes - for buffers ≤ 2 MiB (max length = 2097151)
    ThreeBytes,
    /// 4 bytes - for larger buffers (max length = 268435455)
    #[default]
    FourBytes,
}

impl LengthPlaceholderSize {
    /// Choose the optimal placeholder size for a given buffer limit.
    pub const fn for_limit(limit: usize) -> Self {
        if limit <= 127 {
            Self::OneByte
        } else if limit <= 16383 {
            Self::TwoBytes
        } else if limit <= 2097151 {
            Self::ThreeBytes
        } else {
            Self::FourBytes
        }
    }
    
    /// Number of bytes this placeholder uses.
    pub const fn num_bytes(self) -> usize {
        match self {
            Self::OneByte => 1,
            Self::TwoBytes => 2,
            Self::ThreeBytes => 3,
            Self::FourBytes => 4,
        }
    }
    
    /// Maximum length that can be encoded with this placeholder size.
    pub const fn max_length(self) -> usize {
        match self {
            Self::OneByte => 127,
            Self::TwoBytes => 16383,
            Self::ThreeBytes => 2097151,
            Self::FourBytes => 268435455,
        }
    }
    
    /// Encode a zero-padded length placeholder.
    /// Returns the bytes to write.
    pub const fn placeholder_bytes(self) -> &'static [u8] {
        match self {
            Self::OneByte => &[0x00],
            Self::TwoBytes => &[0x80, 0x00],
            Self::ThreeBytes => &[0x80, 0x80, 0x00],
            Self::FourBytes => &[0x80, 0x80, 0x80, 0x00],
        }
    }
}
```

#### Dispatch Mechanism

**Dynamic sizing based on remaining capacity:**

The placeholder size only needs to accommodate the *remaining buffer space*. When writing a length placeholder, we check how much space is left and choose the smallest sufficient placeholder:

```rust
/// Returned from write_length_placeholder, used to patch the length later.
#[derive(Clone, Copy)]
pub struct LengthPlaceholder {
    pub offset: usize,
    pub size: LengthPlaceholderSize,
}

fn write_length_placeholder(&mut self) -> Result<LengthPlaceholder, Truncated> {
    let offset = self.len();
    let remaining = self.capacity() - offset - self.reserved_tail;
    let size = LengthPlaceholderSize::for_limit(remaining);
    self.write_bytes(size.placeholder_bytes())?;
    Ok(LengthPlaceholder { offset, size })
}

fn patch_length_placeholder(&mut self, placeholder: LengthPlaceholder, length: usize) {
    let slice = self.as_mut_slice();
    for i in 0..placeholder.size.num_bytes() {
        slice[placeholder.offset + i] += ((length >> (i * 7)) & 0x7f) as u8;
    }
}
```

**Usage in macro:**

```rust
macro_rules! proto_encode_len_delimited_try {
    ($buf:expr, $tag:expr, $encode_fn:expr) => {{
        proto_encode_varint($buf, $tag);
        let placeholder = $buf.write_length_placeholder()?;  // returns LengthPlaceholder
        let start = $buf.len();
        $encode_fn;
        let length = $buf.len() - start;
        $buf.patch_length_placeholder(placeholder, length);  // uses stored offset + size
    }};
}
```

**Benefits:**

- **No configuration needed**: The encoder automatically chooses optimal sizes
- **Simple**: The placeholder struct is just 2 usizes on the stack
- **Optimal**: Uses smallest sufficient placeholder for remaining space

**Example progression in a 4 KiB buffer:**

| Write # | Position | Remaining | Placeholder Size | Overhead |
|---------|----------|-----------|------------------|----------|
| 1 | 0 | 4096 | 2 bytes | 2 |
| 2 | 100 | 3996 | 2 bytes | 2 |
| 3 | 3900 | 196 | 2 bytes | 2 |
| 4 | 4000 | 96 | 1 byte | 1 |

### New Types

#### `Truncated` Error

A simple, lightweight error type indicating a fixed-size buffer ran out of space:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Truncated;
```

This error is:
- Zero-sized (no runtime overhead)
- Copyable (can be returned by value)
- Convertible to the main `Error` type via `From`

#### `StringTruncation` Result

Information about how a string was truncated:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringTruncation {
    /// Number of bytes actually written (including any marker)
    pub bytes_written: usize,
    /// Number of bytes from the original string that were truncated
    pub bytes_truncated: usize,
}

impl StringTruncation {
    pub fn none() -> Self {
        Self { bytes_written: 0, bytes_truncated: 0 }
    }
    
    pub fn was_truncated(&self) -> bool {
        self.bytes_truncated > 0
    }
}
```

#### `ProtoWrite` Trait

The trait defines the core buffer operations with fallible semantics:

```rust
pub trait ProtoWrite {
    // === Core required methods ===
    
    /// Append bytes to the buffer. Returns Err(Truncated) if insufficient capacity.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Truncated>;
    
    /// Current length of encoded data.
    fn len(&self) -> usize;
    
    /// Get a reference to the encoded bytes.
    fn as_slice(&self) -> &[u8];
    
    /// Get a mutable reference for patching length placeholders.
    fn as_mut_slice(&mut self) -> &mut [u8];
    
    /// Clear the buffer contents (does not clear reserved_tail or limit).
    fn clear(&mut self);
    
    /// Physical capacity of the buffer.
    fn capacity(&self) -> usize;
    
    // === Limit and reservation management ===
    
    /// Set a soft limit on buffer size. For fixed buffers, clamped to capacity.
    /// For growable buffers, prevents growth beyond this point.
    /// Also updates the length placeholder size to match the new limit.
    fn set_limit(&mut self, limit: usize);
    
    /// Get current limit (defaults to capacity for fixed, usize::MAX for growable).
    fn limit(&self) -> usize;
    
    /// Reserve bytes at the end of the buffer for later use.
    /// Returns the new effective remaining space.
    /// This space is protected from writes until released.
    fn reserve_tail(&mut self, bytes: usize) -> usize;
    
    /// Release previously reserved tail bytes, making them available for writing.
    fn release_tail(&mut self, bytes: usize);
    
    /// Get current tail reservation.
    fn reserved_tail(&self) -> usize;
    
    /// Bytes available for writing: limit - len - reserved_tail
    fn remaining(&self) -> usize {
        self.limit()
            .saturating_sub(self.len())
            .saturating_sub(self.reserved_tail())
    }
    
    // === Length placeholder configuration ===
    
    /// Get the length placeholder size for this buffer.
    /// Determined by the buffer's limit.
    fn length_placeholder_size(&self) -> LengthPlaceholderSize {
        LengthPlaceholderSize::for_limit(self.limit())
    }
    
    /// Override the length placeholder size.
    /// Useful when you know nested messages will be small even in a large buffer.
    fn set_length_placeholder_size(&mut self, size: LengthPlaceholderSize);
    
    /// Write the length placeholder bytes and return the position where length starts.
    fn write_length_placeholder(&mut self) -> Result<usize, Truncated> {
        let pos = self.len();
        let placeholder = self.length_placeholder_size().placeholder_bytes();
        self.write_bytes(placeholder)?;
        Ok(pos)
    }
    
    /// Patch a previously written length placeholder with the actual length.
    fn patch_length_placeholder(&mut self, len_start_pos: usize, length: usize) {
        let num_bytes = self.length_placeholder_size().num_bytes();
        let slice = self.as_mut_slice();
        for i in 0..num_bytes {
            slice[len_start_pos + i] += ((length >> (i * 7)) & 0x7f) as u8;
        }
    }
    
    // === Encoding methods with default implementations ===
    
    fn encode_varint(&mut self, value: u64) -> Result<(), Truncated>;
    fn encode_field_tag(&mut self, field_number: u64, wire_type: u64) -> Result<(), Truncated>;
    fn encode_sint32(&mut self, value: i32) -> Result<(), Truncated>;
    fn encode_string(&mut self, field_tag: u64, val: &str) -> Result<(), Truncated>;
    fn encode_bytes_field(&mut self, field_tag: u64, val: &[u8]) -> Result<(), Truncated>;
    
    // === Truncating string encoder ===
    
    /// Encode a string field, truncating if necessary to fit in available space.
    /// 
    /// If the full string doesn't fit, truncates and appends the marker.
    /// The marker should be a short fixed string like "..." or "[TRUNCATED]".
    /// 
    /// Returns information about what was written and truncated.
    /// Returns Err(Truncated) only if even the field tag + minimal content won't fit.
    fn encode_string_truncated(
        &mut self,
        field_tag: u64,
        val: &str,
        marker: &str,
    ) -> Result<StringTruncation, Truncated>;
}
```

#### `FixedProtoBuffer<const N: usize>`

A stack-allocatable, fixed-size buffer:

```rust
pub struct FixedProtoBuffer<const N: usize> {
    buffer: [u8; N],
    len: usize,
    reserved_tail: usize,
    placeholder_size: LengthPlaceholderSize,
}

impl<const N: usize> FixedProtoBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; N],
            len: 0,
            reserved_tail: 0,
            // Computed at compile time based on N
            placeholder_size: LengthPlaceholderSize::for_limit(N),
        }
    }
}
```

Key properties:
- **No heap allocation**: The buffer is a fixed-size array
- **Automatic placeholder sizing**: `LengthPlaceholderSize` is determined from `N` at compile time
- **Atomic writes where possible**: `write_bytes` checks capacity before writing
- **Truncation-safe**: Returns `Err(Truncated)` instead of panicking
- **Limit equals capacity**: `set_limit` is a no-op (or clamps to capacity)

#### Updated `ProtoBuffer`

The growable buffer gains limit, reservation, and placeholder size support:

```rust
pub struct ProtoBuffer {
    buffer: Vec<u8>,
    limit: usize,                           // Default: usize::MAX (unlimited)
    reserved_tail: usize,                   // Default: 0
    placeholder_size: LengthPlaceholderSize, // Default: FourBytes
}
```

Key properties:
- **Configurable limit**: Prevents unbounded growth
- **Configurable placeholder size**: Can use 2-byte placeholders when limit is set appropriately
- **Truncation on limit**: Returns `Err(Truncated)` when limit reached (no realloc)
- **Backward compatible**: Default limit is unlimited, default placeholder is 4 bytes

### String Truncation Behavior

The `encode_string_truncated` method implements graceful truncation:

```rust
fn encode_string_truncated(
    &mut self,
    field_tag: u64,
    val: &str,
    marker: &str,  // e.g., "..." or "[TRUNCATED]"
) -> Result<StringTruncation, Truncated> {
    let tag_len = varint_len((field_tag << 3) | WIRE_TYPE_LEN);
    let full_len = tag_len + varint_len(val.len()) + val.len();
    
    // Check if full string fits
    if full_len <= self.remaining() {
        self.encode_string(field_tag, val)?;
        return Ok(StringTruncation::none());
    }
    
    // Calculate how much of the string we can fit with marker
    let marker_bytes = marker.as_bytes();
    let available = self.remaining();
    
    // Need at least: tag + length(1 byte min) + marker
    let min_needed = tag_len + 1 + marker_bytes.len();
    if available < min_needed {
        return Err(Truncated);
    }
    
    // Calculate truncated string length
    let max_content = available - tag_len - 1; // Assuming 1-byte length varint
    let truncated_str_len = max_content.saturating_sub(marker_bytes.len());
    
    // Find UTF-8 safe truncation point
    let truncated_str = truncate_utf8_safe(val, truncated_str_len);
    let bytes_truncated = val.len() - truncated_str.len();
    
    // Build the truncated content: truncated_str + marker
    let total_content_len = truncated_str.len() + marker_bytes.len();
    
    self.encode_field_tag(field_tag, WIRE_TYPE_LEN)?;
    self.encode_varint(total_content_len as u64)?;
    self.write_bytes(truncated_str.as_bytes())?;
    self.write_bytes(marker_bytes)?;
    
    Ok(StringTruncation {
        bytes_written: tag_len + varint_len(total_content_len) + total_content_len,
        bytes_truncated,
    })
}

/// Truncate a string at a UTF-8 safe boundary
fn truncate_utf8_safe(s: &str, max_bytes: usize) -> &str {
    if max_bytes >= s.len() {
        return s;
    }
    // Find the last valid UTF-8 char boundary at or before max_bytes
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}
```

### Backward Compatibility

The existing `ProtoBuffer` type retains its infallible inherent methods:

```rust
impl ProtoBuffer {
    // These remain infallible (no Result return type) when limit is unlimited
    pub fn encode_varint(&mut self, value: u64) { ... }
    pub fn encode_string(&mut self, field_tag: u64, val: &str) { ... }
    // ... etc
}

// Additionally implements ProtoWrite (may return Err if limit set)
impl ProtoWrite for ProtoBuffer { ... }
```

This means:
- All existing code using `ProtoBuffer` continues to work unchanged
- New generic code can use `impl ProtoWrite` to work with either buffer type
- Setting a limit on `ProtoBuffer` enables truncation behavior

### Macro Support

The macros now use the buffer's configured `LengthPlaceholderSize`:

1. **`proto_encode_len_delimited_unknown_size!`** (existing, updated)
   - Uses infallible helpers for `ProtoBuffer`
   - Uses the buffer's `length_placeholder_size()` instead of hardcoded 4 bytes
   - No error propagation needed

2. **`proto_encode_len_delimited_try!`** (new)
   - For use with generic `ProtoWrite` code
   - Uses the buffer's `length_placeholder_size()` 
   - Propagates `Truncated` errors via `?`
   - Returns `Result<(), Truncated>`

```rust
/// Updated macro using configurable placeholder size
#[macro_export]
macro_rules! proto_encode_len_delimited_try {
    ($field_tag: expr, $encode_fn:expr, $buf:expr) => {{
        use $crate::otlp::ProtoWrite;
        $buf.encode_field_tag($field_tag, $crate::proto::consts::wire_types::LEN)?;
        let len_start_pos = $buf.write_length_placeholder()?;
        $encode_fn;
        let num_bytes = $buf.length_placeholder_size().num_bytes();
        let len = $buf.len() - len_start_pos - num_bytes;
        $buf.patch_length_placeholder(len_start_pos, len);
        Ok::<(), $crate::error::Truncated>(())
    }};
}
```

## Usage Examples

### Generic Encoding Function

```rust
use otap_df_pdata::otlp::{ProtoWrite, Truncated};

fn encode_attributes<W: ProtoWrite>(
    buf: &mut W,
    attrs: &[KeyValue],
) -> Result<usize, Truncated> {
    let mut count = 0;
    for attr in attrs {
        buf.encode_string(KEY_TAG, &attr.key)?;
        buf.encode_string(VALUE_TAG, &attr.value)?;
        count += 1;
    }
    Ok(count)
}
```

### Fixed Buffer with Reserved Space for Dropped Count

```rust
use otap_df_pdata::otlp::{FixedProtoBuffer, ProtoWrite, Truncated};

fn encode_log_record(attrs: &[KeyValue], body: &str) -> Vec<u8> {
    let mut buf = FixedProtoBuffer::<2048>::new();
    
    // Reserve 5 bytes for dropped_attributes_count (tag + varint)
    buf.reserve_tail(5);
    
    // Encode body with truncation marker
    let body_result = buf.encode_string_truncated(
        LOG_RECORD_BODY, 
        body, 
        "...[truncated]"
    );
    
    // Encode attributes until we run out of space
    let mut encoded = 0;
    for attr in attrs {
        match encode_key_value(&mut buf, attr) {
            Ok(()) => encoded += 1,
            Err(Truncated) => break,
        }
    }
    
    // Release reserved space and encode dropped count
    let dropped = attrs.len() - encoded;
    buf.release_tail(5);
    
    if dropped > 0 {
        let _ = buf.encode_field_tag(DROPPED_ATTRIBUTES_COUNT, WIRE_TYPE_VARINT);
        let _ = buf.encode_varint(dropped as u64);
    }
    
    buf.as_slice().to_vec()
}
```

### Limiting Growable Buffer

```rust
use otap_df_pdata::otlp::{ProtoBuffer, ProtoWrite, LengthPlaceholderSize};

fn encode_with_limit(data: &LargeData) -> Result<Bytes, Truncated> {
    let mut buf = ProtoBuffer::new();
    
    // Prevent unbounded growth - limit to 16KB
    // This automatically sets placeholder size to TwoBytes
    buf.set_limit(16 * 1024);
    
    // Or explicitly use small placeholders for even smaller limits
    // buf.set_limit(4 * 1024);
    // buf.set_length_placeholder_size(LengthPlaceholderSize::TwoBytes);
    
    // Reserve space for metadata at the end
    buf.reserve_tail(64);
    
    // Encode potentially large content with truncation
    for item in &data.items {
        buf.encode_string_truncated(ITEM_TAG, &item.value, "...")?;
    }
    
    // Add metadata in reserved space
    buf.release_tail(64);
    encode_metadata(&mut buf, data)?;
    
    Ok(buf.into_bytes())
}
```

### Space-Efficient Small Buffer

```rust
use otap_df_pdata::otlp::FixedProtoBuffer;

fn encode_compact_log() {
    // 4KB buffer automatically uses 2-byte length placeholders
    let mut buf = FixedProtoBuffer::<4096>::new();
    
    assert_eq!(buf.length_placeholder_size().num_bytes(), 2);
    
    // Each nested message saves 2 bytes compared to 4-byte placeholders!
    // In a LogRecord with 10 nested structures, that's 20 bytes saved.
}
```

### Body Truncation with Byte Count

For cases where you want to include the byte count in the truncation marker:

```rust
fn encode_body_with_count<W: ProtoWrite>(buf: &mut W, body: &str) -> StringTruncation {
    // First attempt with simple marker
    match buf.encode_string_truncated(LOG_RECORD_BODY, body, "...") {
        Ok(info) => {
            if info.was_truncated() {
                // Log the truncation details for observability
                // The bytes_truncated field tells us exactly how much was lost
                tracing::debug!(
                    truncated_bytes = info.bytes_truncated,
                    "Log body truncated"
                );
            }
            info
        }
        Err(Truncated) => {
            // Couldn't fit even minimal content
            StringTruncation { bytes_written: 0, bytes_truncated: body.len() }
        }
    }
}
```

## Design Rationale

### Why Configurable Length Placeholder Size?

The protobuf wire format uses varints for length-delimited field lengths. Since we encode nested messages without knowing their size upfront, we reserve placeholder bytes and patch them later.

The problem: varints are variable-length! A length of 127 needs 1 byte, but 128 needs 2 bytes. Our solution uses zero-padded varints where each byte has its continuation bit set until the final byte.

For a 4 KiB buffer, no nested message can exceed 4096 bytes, which fits in a 2-byte varint. Using 4-byte placeholders wastes 2 bytes per nested message. In a typical LogRecord with its nested structure:

```
ResourceLogs         [4 bytes wasted]
  └─ ScopeLogs       [4 bytes wasted if 4-byte, 2 bytes if 2-byte]
       └─ LogRecord  [...]
            ├─ Body (AnyValue)
            └─ Attributes (repeated KeyValue)
                 └─ Value (AnyValue)
```

With 10 attributes, that's potentially 20+ extra bytes wasted—space that could hold another attribute!

### Why Reserve Tail Space?

The `reserve_tail` mechanism ensures that critical fields like `dropped_attributes_count` can always be encoded, even when the buffer is nearly full. Without this:

1. We might fill the buffer completely with attributes
2. Then have no room to record that we dropped some
3. The receiver would have no indication of data loss

### Why Truncate Strings vs. Drop Entirely?

Truncated data with a marker is often more useful than no data:
- A truncated log message still conveys intent
- A truncated attribute value may still be useful for filtering/grouping
- The marker makes it clear that truncation occurred

### Why UTF-8 Safe Truncation?

Truncating in the middle of a multi-byte UTF-8 character would produce invalid UTF-8, which could cause issues downstream. The `truncate_utf8_safe` function ensures we always produce valid UTF-8.

### Why Configurable Limits for Growable Buffers?

Even in the "normal" path, we want protection against:
- Malformed data causing unbounded memory growth
- DoS attacks via large payloads
- Accidental memory exhaustion from unexpectedly large telemetry

## File Changes

| File | Changes |
|------|---------|
| `crates/pdata/src/error.rs` | Added `Truncated` error type with `Display` and `Error` impls |
| `crates/pdata/src/otlp/common.rs` | Added `ProtoWrite` trait, `FixedProtoBuffer`, `StringTruncation`, `LengthPlaceholderSize`, updated `ProtoBuffer` with limit/reservation/placeholder fields, helper functions, updated macros |
| `crates/pdata/src/otlp/mod.rs` | Export `ProtoWrite`, `FixedProtoBuffer`, `StringTruncation`, `LengthPlaceholderSize`, `Truncated` |

## Testing

The implementation includes comprehensive tests covering:

- Basic `FixedProtoBuffer` operations
- Truncation behavior for various encoding operations
- Varint encoding with partial writes
- Generic function usage with both buffer types
- Backward compatibility of `ProtoBuffer` inherent methods
- Partial content availability after truncation
- String truncation with UTF-8 safety
- Reserved tail space behavior
- Limit enforcement for growable buffers

All existing tests continue to pass, plus new tests for the added functionality.
