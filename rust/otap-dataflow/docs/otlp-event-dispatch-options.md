# Integrating OTLP Bytes with Existing Tracing Infrastructure

## Problem Statement

We have OTLP-encoded bytes (from internal telemetry, channels, etc.) and want to format them using the **existing** tracing-subscriber fmt layer configuration, rather than creating parallel formatting logic.

## Three Integration Options

### Option 1: Programmatic Event Dispatch (✅ Recommended)

**Concept**: Reconstruct `tracing::Event` objects from OTLP bytes and dispatch them through the global subscriber.

**Architecture**:
```
OTLP bytes → RawLogsData → Reconstruct Event → dispatch.event() → fmt layer → console
```

**Advantages**:
- ✅ Uses **exact same formatting** as existing code
- ✅ Respects **all filters** (EnvFilter, level, target)
- ✅ Works with **any fmt configuration** (colors, timestamps, format)
- ✅ **Zero code duplication**
- ✅ **Embedded-friendly** - uses whatever app configured

**Code Sketch**:
```rust
use tracing_core::{Event, Metadata, dispatch};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;

pub fn dispatch_otlp_as_events(otlp_bytes: &[u8]) {
    let logs_view = RawLogsData::new(otlp_bytes);
    
    for resource_logs in logs_view.resources() {
        for scope_logs in resource_logs.scopes() {
            for log_record in scope_logs.log_records() {
                // Create synthetic metadata
                let level = severity_to_level(log_record.severity_number());
                let target = extract_target(&log_record);
                let metadata = create_metadata(level, target);
                
                // Create fields from attributes
                let fields = extract_fields(&log_record);
                
                // Dispatch through global subscriber
                dispatch::get_default(|d| {
                    let event = Event::new(&metadata, &fields);
                    d.event(&event);
                });
            }
        }
    }
}

// Application code:
fn main() {
    // Configure tracing once
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_level(true)
        .init();
    
    // Later, dispatch OTLP bytes:
    let otlp_bytes = receive_from_channel();
    dispatch_otlp_as_events(&otlp_bytes);
    // ^ Uses the configured fmt layer!
}
```

**Challenges**:
- Need to reconstruct `Event` properly (metadata + fields)
- Need to handle `Field` and `ValueSet` construction
- Synthetic callsite creation

**Status**: Skeleton implementation in `otlp_event_dispatcher.rs`

---

### Option 2: Direct fmt Layer Call

**Concept**: Access the fmt layer directly and call its `on_event()` method.

**Architecture**:
```
OTLP bytes → RawLogsData → Create Event → fmt_layer.on_event() → console
```

**Advantages**:
- ✅ Direct access to formatting
- ✅ No global dispatch needed
- ✅ Can pass custom context

**Disadvantages**:
- ❌ Need reference to fmt_layer instance
- ❌ Need to construct `Context<S>` (requires registry)
- ❌ More complex setup
- ❌ Bypasses other layers/filters

**Code Sketch**:
```rust
use tracing_subscriber::fmt::Layer as FmtLayer;
use tracing_subscriber::layer::Context;

pub fn format_with_fmt_layer<S, N, E, W>(
    otlp_bytes: &[u8],
    fmt_layer: &FmtLayer<S, N, E, W>,
    registry: &S,
) where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let logs_view = RawLogsData::new(otlp_bytes);
    
    for log_record in iterate_logs(&logs_view) {
        // Create Event
        let event = create_event_from_log(&log_record);
        
        // Create Context (requires registry)
        let ctx = Context::new(&event, registry);
        
        // Call fmt layer directly
        fmt_layer.on_event(&event, ctx);
    }
}
```

**Challenges**:
- Need access to `FmtLayer` instance
- Must construct `Context` which needs the registry
- Doesn't work with other layers in stack

---

### Option 3: Bridge Layer (Most Flexible)

**Concept**: Create a layer that receives OTLP bytes and re-emits them as events.

**Architecture**:
```
OTLP bytes → channel → BridgeLayer → dispatch.event() → subscriber stack → fmt layer
```

**Advantages**:
- ✅ Clean separation of concerns
- ✅ Runs in subscriber stack (respects all layers)
- ✅ Can run in separate thread
- ✅ Preserves all filtering/routing

**Disadvantages**:
- ❌ Most code to write
- ❌ Lifecycle management (spawn/shutdown)
- ❌ More complex architecture

**Code Sketch**:
```rust
use tracing_subscriber::{Layer, layer::Context};
use std::sync::mpsc;

pub struct OtlpBridgeLayer {
    // Receives OTLP bytes from channel
}

impl<S> Layer<S> for OtlpBridgeLayer 
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // Forward regular events through
    }
}

// Background task that dispatches OTLP bytes
pub fn spawn_bridge_task(rx: mpsc::Receiver<Vec<u8>>) {
    tokio::spawn(async move {
        while let Some(otlp_bytes) = rx.recv().await {
            let logs_view = RawLogsData::new(&otlp_bytes);
            
            for log_record in iterate_logs(&logs_view) {
                // Create Event and dispatch
                let event = create_event(&log_record);
                dispatch::get_default(|d| d.event(&event));
            }
        }
    });
}

// Usage:
fn main() {
    let (tx, rx) = mpsc::channel();
    
    // Spawn bridge task
    spawn_bridge_task(rx);
    
    // Configure subscriber with regular layers
    tracing_subscriber::registry()
        .with(fmt::layer())
        .init();
    
    // Send OTLP bytes to bridge
    tx.send(otlp_bytes).unwrap();
}
```

**Status**: Conceptual design, not implemented

---

## Comparison Matrix

| Feature | Option 1: Dispatch | Option 2: Direct | Option 3: Bridge |
|---------|-------------------|------------------|------------------|
| Uses existing fmt config | ✅ | ✅ | ✅ |
| Respects filters | ✅ | ❌ | ✅ |
| Works with layer stack | ✅ | ❌ | ✅ |
| Complexity | Low | Medium | High |
| Code to write | ~100 lines | ~150 lines | ~300 lines |
| Embedded-friendly | ✅ | ⚠️ | ✅ |
| Async support | ⚠️ | ✅ | ✅ |
| Lifecycle management | Simple | Simple | Complex |

## Recommendation: **Option 1**

Start with **Option 1 (Programmatic Event Dispatch)** because:

1. **Minimal code** - Just reconstruct Events and dispatch
2. **Maximum compatibility** - Works with any subscriber configuration
3. **Respects all infrastructure** - Filters, layers, everything
4. **Embedded-friendly** - Uses whatever the application configured
5. **No parallel logic** - Zero duplication of formatting code

If you later need async formatting or more complex routing, you can migrate to Option 3 (Bridge Layer), but Option 1 solves the immediate need elegantly.

## Implementation Plan

### Phase 1: Basic Event Reconstruction
1. Implement proper `Metadata` creation from LogRecordView
2. Handle `Field` and `FieldSet` construction
3. Create `ValueSet` visitor that extracts attributes
4. Test with simple OTLP bytes

### Phase 2: Complete Integration
1. Handle all AnyValue types (scalars, arrays, maps)
2. Preserve attribute names and types
3. Handle body/message field
4. Test with complex nested structures

### Phase 3: Production Readiness
1. Error handling for malformed OTLP
2. Performance optimization (callsite caching)
3. Integration tests with various fmt configurations
4. Documentation and examples

## Key Technical Challenges

### Challenge 1: Field and FieldSet Construction

Tracing uses `Field` and `FieldSet` which are typically created by the `tracing!` macro at compile time. For synthetic events, we need to:

1. **Option A**: Use a global registry of field definitions
2. **Option B**: Create fields dynamically (requires unsafe or workaround)
3. **Option C**: Use `record_debug()` with formatted strings (simple but loses type info)

**Recommendation**: Start with Option C for proof-of-concept, migrate to Option A for production.

### Challenge 2: Callsite Identifiers

Each tracing event has a unique callsite identifier. For synthetic events:

1. Create a sentinel callsite for all OTLP events
2. Store it in a static
3. Use it for all reconstructed events

### Challenge 3: ValueSet Construction

The `ValueSet` type requires a `Visit` implementation. We need to:

1. Iterate through LogRecord attributes
2. Call appropriate `record_*()` methods based on AnyValue type
3. Handle nested structures (arrays, maps)

## Example Usage

```rust
// Application: Configure tracing once
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info,h2=warn,hyper=warn")
        .with_ansi(true)
        .with_thread_names(true)
        .init();
    
    // ... application code ...
}

// Internal telemetry receiver:
impl InternalTelemetryReceiver {
    async fn process_otlp_bytes(&mut self, otlp_bytes: Vec<u8>) {
        // Dispatch to configured subscriber
        if let Err(e) = dispatch_otlp_bytes_as_events(&otlp_bytes) {
            eprintln!("Failed to dispatch telemetry: {}", e);
        }
    }
}

// Output: Uses the configured fmt layer!
// [INFO  otap::processor] Processing batch count=42 size=1024
// [WARN  otap::exporter] Retrying export attempt=3
```

## Conclusion

**Option 1 (Programmatic Event Dispatch)** provides the best balance of:
- Simplicity
- Compatibility with existing configuration
- Zero code duplication
- Embedded software friendliness

The key insight is that we can **reconstruct Events from OTLP bytes** and dispatch them through the existing subscriber infrastructure, giving us all the benefits of the configured fmt layer "for free."
