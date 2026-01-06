# Tracing Integration Design: Zero-Copy Exploration

This document captures the design exploration for integrating `tokio-tracing` events with our OTLP-bytes-first encoding architecture.

## Goal

Convert tracing events to OTLP bytes with minimal allocations, following the principle that **encoding to bytes happens before crossing thread boundaries**. The ideal is true zero-copy: borrow data directly from the tracing event and encode it in-place.

## Architecture Context

From `ARCHITECTURE.md`: The system uses a thread-per-core design where components are local to each thread. OTLP bytes are the interchange format that crosses thread boundaries, not structured data.

## What We Achieved

### 1. `TracingAnyValue<'a>` is `Copy`

```rust
#[derive(Debug, Clone, Copy)]
pub enum TracingAnyValue<'a> {
    Str(&'a str),
    Int(i64),
    Bool(bool),
    Double(f64),
    Bytes(&'a [u8]),
    Array(&'a [TracingAnyValue<'a>]),
    KeyValueList(&'a [TracingAttribute<'a>]),
}
```

The enum only contains borrowed references or primitive values. "Copying" this type just copies the pointer+length, not the underlying data. The lifetime `'a` is preserved in the copy.

### 2. `TracingAttribute<'a>` is `Copy`

```rust
#[derive(Debug, Clone, Copy)]
pub struct TracingAttribute<'a> {
    pub key: &'a str,
    pub value: TracingAnyValue<'a>,
}
```

### 3. `TracingLogRecord<'a>` Borrows from Metadata

```rust
pub struct TracingLogRecord<'a> {
    event_name: Option<&'static str>,  // metadata.name() is always static
    target: &'a str,                    // borrowed from Metadata<'a>
    attributes: Vec<TracingAttribute<'a>>,
    body: Option<&'a str>,
    // ...
}
```

The lifetime `'a` ties the log record to the tracing event callback scope.

### 4. Direct Trait Implementations (No Wrappers)

`TracingAnyValue<'a>` implements `AnyValueView<'a>` directly.
`TracingAttribute<'a>` implements `AttributeView` directly.
No wrapper types needed because the underlying types are `Copy`.

### 5. GAT Lifetime Handling

The `LogRecordView` trait uses Generic Associated Types:

```rust
type Attribute<'att>: AttributeView where Self: 'att;
type Body<'bod>: AnyValueView<'bod> where Self: 'bod;
```

For `TracingLogRecord<'a>`:
- `type Attribute<'att> = TracingAttribute<'a>` — uses data lifetime `'a`, not GAT lifetime
- `type Body<'bod> = TracingAnyValue<'bod>` — constructs on demand from stored `&'a str`

The key insight: when `Self: 'bod`, it implies `'a: 'bod`, so we can shorten the lifetime.

## The Barrier: The `Visit` Trait

The tracing crate's `Visit` trait erases lifetime information:

```rust
pub trait Visit {
    fn record_str(&mut self, field: &Field, value: &str);
    fn record_debug(&mut self, field: &Field, value: &dyn Debug);
    // ...
}
```

The `value: &str` has an anonymous lifetime. Even though in practice the data is borrowed from the `Event<'_>` which exists for the entire callback, **the trait boundary prevents expressing this relationship**.

### What This Means

1. **Field names (`field.name()`)**: Always `&'static str` — zero-copy ✓
2. **Primitive values (i64, bool, f64)**: No allocation needed — zero-copy ✓
3. **String values**: The borrow lifetime is erased by the trait, so we must either:
   - Allocate (copy to `String`)
   - Use `unsafe` to assert the lifetime relationship

### Current Implementation

We use owned storage (`OwnedValue`) in the visitor:

```rust
enum OwnedValue {
    Str(String),  // Allocated copy
    Int(i64),
    Bool(bool),
    Double(f64),
}
```

This is the safe approach at the cost of one allocation per string attribute.

## The `Send + Sync` Barrier

The tracing ecosystem requires subscribers to be `Send + Sync`:

```rust
impl Dispatch {
    pub fn new<S>(subscriber: S) -> Self
    where
        S: Subscriber + Send + Sync + 'static
}
```

Our layer uses `RefCell<HashMap<...>>` for span storage (single-threaded design), which is `!Sync`. This prevents using standard tracing test utilities like `with_default`.

### Workaround

Tests must use thread-local storage or other patterns that don't require the subscriber to be `Sync`.

## Alternatives Not Taken

### 1. Unsafe Lifetime Extension

```rust
fn record_str(&mut self, field: &Field, value: &str) {
    // UNSAFE: Assert that value lives as long as the event
    let extended: &'static str = unsafe { std::mem::transmute(value) };
    self.attr_values.push(TracingAnyValue::Str(extended));
}
```

Rejected because:
- Requires proving the invariant holds for all tracing macros
- Third-party libraries might violate the assumption
- The allocation cost is minimal compared to encoding

### 2. Arc/Rc for Cheap Cloning

Earlier iterations used `Rc<str>` and `Rc<[u8]>` to make cloning cheap. Rejected because:
- Still requires initial allocation
- Adds reference counting overhead
- The goal is zero-copy, not cheap-copy

### 3. String Arena

Could store formatted strings in a pre-allocated arena that lives for the callback scope. Not implemented because:
- Adds complexity
- Still requires copying data into the arena
- The simple `String` approach is clear and correct

## Summary

| Data Type | Zero-Copy? | Notes |
|-----------|------------|-------|
| Field names | ✓ | `&'static str` from tracing |
| `metadata.name()` | ✓ | `&'static str` |
| `metadata.target()` | ✓ | `&'a str` borrowed from metadata |
| Primitive values | ✓ | Copied by value (cheap) |
| String values | ✗ | `Visit` trait erases lifetime |
| Debug-formatted values | ✗ | Requires formatting to String |

The current implementation achieves zero-copy for everything except string attribute values, where the `Visit` trait's lifetime erasure forces allocation. This is a fundamental limitation of the tracing crate's design, not something we can work around without `unsafe`.

## Future Considerations

If the tracing crate ever adds a lifetime-aware visitor pattern, or if we're willing to use `unsafe` with careful auditing, we could achieve true zero-copy for all data types.
