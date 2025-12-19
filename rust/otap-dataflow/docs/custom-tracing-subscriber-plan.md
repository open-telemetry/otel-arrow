# Custom Tokio Tracing Subscriber for OTAP-Dataflow Integration

**Status**: Phase 1 Complete - Planning Phase 2  
**Date**: December 17-18, 2025  
**Authors**: OTAP-Dataflow Team

**Phase 1 Complete**: ✅ [Stateful OTLP Encoder](./stateful-encoder-phase1-summary.md)

## Executive Summary

This document outlines a comprehensive plan to add a **fast-path logging mechanism** for OTAP-Dataflow pipeline components that bypasses the global tracing subscriber entirely. The `otel_*!` macros will require an effect handler as the first argument and **encode OTLP bytes directly into per-core reusable buffers** with zero contention. Each log event is encoded synchronously to OTLP bytes and appended to a `Vec<u8>` that grows until a size threshold is reached, then flushed as a single block. The global subscriber remains for bootstrap code and third-party libraries, creating a clear performance distinction: **pipeline instrumentation is fast** (< 100ns), **global instrumentation is slow** (~5-10μs).

## Background

### Current Implementation

The OTAP-Dataflow engine currently uses tokio tracing with a global subscriber initialized via:

```rust
tracing_subscriber::registry()
    .with(filter)
    .with(fmt_layer)
    .with(sdk_layer)
    .try_init();
```

This global approach has limitations for high-performance pipeline code:
1. **Global State Contention**: All cores/threads compete for the global subscriber lock
2. **Thread Safety Overhead**: Uses `Arc` internally for cross-thread synchronization
3. **Not Aligned with Engine Design**: Contradicts the shared-nothing, thread-per-core philosophy
4. **Limited Integration**: Cannot easily route internal logs through the dataflow pipeline

**However**, the global subscriber remains useful for:
- Bootstrap and initialization code (before pipelines start)
- Third-party library logging
- Test utilities and debugging code
- Standard `tracing::info!()` macros from dependencies

### OpenTelemetry Rust Recommendation

OpenTelemetry-Rust recommends tokio tracing as the logging solution. We have:
- A copy of tokio tracing in `rust/otap-dataflow/tokio-tracing-rs`
- A copy of OpenTelemetry Rust SDK in `rust/otap-dataflow/opentelemetry-rust-rs`
- Current integration using `opentelemetry-appender-tracing` layer

### Current Macro Implementation

The `otel_error!`, `otel_warn!`, `otel_info!`, and `otel_debug!` macros are defined in:
- `crates/telemetry/src/internal_events.rs`
- These wrap `tracing` crate's `error!`, `warn!`, `info!`, and `debug!` macros
- Include OpenTelemetry event name specification support
- Currently use global tracing dispatcher

## Goals

### Primary Objectives

1. **Fast Path for Pipelines**: Direct buffer writes bypass global subscriber entirely
2. **Explicit Performance Model**: Fast path requires effect handler (compile-time enforced)
3. **Thread-Per-Core Isolation**: Each core's buffer accessed via effect handler
4. **Pipeline Integration**: Route internal telemetry through dataflow components
5. **Backward Compatibility**: Global subscriber remains for non-pipeline code
6. **Zero-Copy Where Possible**: Minimize allocations and data copying

### Secondary Objectives

1. **Maintain Compatibility**: Keep existing `otel_*!` macro API surface
2. **Performance**: Achieve sub-microsecond latency for trace event recording
3. **Flexibility**: Support both in-process and external telemetry routing
4. **Testing**: Maintain isolated testability of components

## Architecture Overview

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Core 0 (Thread 0)                        │
├─────────────────────────────────────────────────────────────┤
│  Pipeline Components (FAST PATH)                            │
│  ┌────────────┐    ┌────────────┐    ┌────────────┐       │
│  │ Receiver   │───>│ Processor  │───>│ Exporter   │       │
│  └──────┬─────┘    └──────┬─────┘    └──────┬─────┘       │
│         │                 │                  │              │
│         │ otel_info!(effect, ...)  < 100ns   │              │
│         │                 │                  │              │
│         └─────────────────┴──────────────────┘              │
│                           │                                 │
│                           v                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │   Effect Handler with OTLP Buffer                   │   │
│  │   - Streaming OTLP encoding to Vec<u8>              │   │
│  │   - Reusable buffer, grows to threshold             │   │
│  │   - No locks, no global state                       │   │
│  │   - Zero indirection                                │   │
│  └─────────────────────────────────────────────────────┘   │
│                           │                                 │
│                           v                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │   Internal Telemetry Receiver                       │   │
│  │   - Drains effect.trace_buffer()                    │   │
│  │   - Converts to OpenTelemetry LogRecord             │   │
│  │   - Injects into dataflow pipeline                  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
                           v
             ┌──────────────────────────┐
             │  Telemetry Pipeline      │
             │  (Processes Internal     │
             │   Diagnostic Logs)       │
             └──────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Bootstrap / Third-Party Libraries (SLOW PATH)              │
│                                                             │
│  tracing::info!(...)  ~5μs                                  │
│         │                                                   │
│         v                                                   │
│  ┌──────────────────────┐                                  │
│  │ Global Subscriber    │                                  │
│  │ (tokio tracing)      │  ← Existing, unchanged          │
│  │ - Global locks       │                                  │
│  │ - Arc/Mutex overhead │                                  │
│  └──────────────────────┘                                  │
└─────────────────────────────────────────────────────────────┘
```

### Architecture Clarification: No Formatting, Full Structure Preservation

**Critical Design Decision**: We do NOT format values to strings. Instead, we leverage OTLP's rich type system:

- **OTLP AnyValue supports**: String, Int, Double, Bool, Bytes, Array, KeyValueList
- **Equivalent to JSON**: Can represent any JSON structure natively
- **tracing Visit provides typed data**: `record_i64()`, `record_bool()`, `record_str()`, etc.
- **Result**: Complete fidelity from tracing events → OTLP structures

**Implementation Flow**:
```
tracing::info!(count = 42, error = ?err, items = ?vec)
    ↓ Visit trait extracts typed values
TracingLogRecord (implements LogRecordView)
    ↓ Attributes as AnyValueView (with nested arrays/maps)
    ↓ Stateful OTLP Encoder
OTLP bytes (protobuf)
    ↓ Channel to internal telemetry receiver
OtapPdata::from_otlp_bytes()
    ↓ Injected into dataflow pipeline
```

**No `fmt` layer needed**: The stateful encoder directly serializes structured data. When we remove OpenTelemetry-appender-tracing layer, we replace it with our own `OtlpTracingLayer` that encodes to OTLP bytes with full type preservation.

### Key Components

#### 1. Per-Core OTLP Buffer with Streaming Encoding (Owned by Effect Handler)

**Key Insight**: Encode OTLP bytes synchronously as events occur, accumulate in reusable Vec<u8>!

The effect handler directly owns a **reusable byte buffer**:
- Single `Vec<u8>` for OTLP-encoded bytes
- Construct view synchronously on each log event
- Encode directly to OTLP bytes, append to buffer
- Buffer grows until size threshold reached
- Flush sends accumulated bytes, clears buffer (capacity retained)
- Threshold=1 enables synchronous logging

[SNIP]

**Advantages**:
- **Single Allocation**: One Vec<u8> per core, reused indefinitely
- **Streaming Encoding**: Encode each event as it happens, no batch delay
- **Memory Efficient**: No intermediate LogRecord structs, just bytes
- **Flexible Batching**: Threshold in bytes (e.g., 4KB, 64KB)
- **Synchronous Option**: threshold=1 = immediate flush per event
- **Simple**: Just a Vec<u8>, no complex ring buffer
- **Zero-Copy**: Buffer reused, capacity grows once then stable

#### 2. Streaming OTLP Encoding with Synchronous View Construction

**Key Insight**: Construct view and encode to OTLP bytes synchronously as each event occurs!

The `otel_*!` macros construct a view on-the-fly and encode directly to the reusable buffer:
- Build ephemeral view of single log event
- Encode OTLP bytes directly using view API
- Append encoded bytes to `Vec<u8>` buffer
- Check size threshold after each append
- No intermediate LogRecord storage needed

[SNIP]

**Advantages of Streaming Encoding**:
- **Zero Intermediate Storage**: LogRecord only lives on stack during encoding
- **Incremental**: Encode as events occur, not in batches
- **Reusable Buffer**: Vec<u8> grows once, then stable capacity
- **Flexible Batching**: Size-based threshold (bytes) not count-based
- **Synchronous Mode**: threshold=1 byte = flush every event
- **Memory Efficient**: Single allocation per core
- **Simple**: No ring buffer, no drain logic, just append + check size

#### 3. Size-Based Threshold Flush with Reusable Buffer

**Key Insight**: Append OTLP bytes to reusable Vec<u8>, flush when size threshold reached!

**Architecture**:
- Effect handler owns single `Vec<u8>` for OTLP bytes
- Each log event encoded synchronously, appended to buffer
- When buffer size (bytes) exceeds threshold → flush inline
- Send buffer as OTLP message via `send_message()`
- **Clear buffer** (not reallocate), capacity retained
- **Downstream batch processor** handles timing & further batching

[SNIP]

**Advantages**:
- **Reusable Buffer**: Vec<u8> cleared, not reallocated (capacity stable)
- **Size-Based Batching**: Threshold in bytes (e.g., 4KB, 64KB) not record count
- **Streaming**: Encode incrementally as events occur
- **Synchronous Option**: threshold=1 byte = flush every event immediately
- **Simple**: Just Vec<u8>, no complex data structures
- **Memory Efficient**: One allocation per core, grows once
- **Zero Intermediate Storage**: No LogRecord buffering, just bytes

**Flow**:
```
otel_info!(effect, ...) [50ns]
  ↓
Build LogRecord (stack) [30ns]
  ↓
Create view (ephemeral) [10ns]
  ↓
Encode OTLP → append to Vec<u8> [100-200ns]
  ↓
Check size threshold [5ns]
  ↓
(When buffer.len() >= threshold)
  Flush: send_message(buffer) + clear() [~5μs]
  ↓
→ Batch Processor (existing)
  ↓
→ Internal logs pipeline
```

**Configuration**:
- `flush_threshold_bytes`: Buffer size in bytes before flushing (default: 64KB)
- `flush_threshold_bytes: 1`: Synchronous mode (flush every event)
- Downstream batch processor handles timing (e.g., 1s max wait)
- Buffer capacity grows to working set, then stable

#### 4. Updated Macros

The `otel_*!` macros will be updated to **require** an effect handler reference:
- Effect handler parameter is mandatory - makes dependencies explicit
- Macro always uses thread-local context (zero overhead)
- Compile error if effect handler not provided - forces intentional design
- No silent fallback to global state - maintains architectural integrity

[SNIP]

**Two-Path Design**:

1. **Fast Path** (`otel_*!` macros with effect handler):
   - Requires effect handler as first argument
   - Direct buffer write: < 100ns
   - Bypasses tokio tracing, global subscriber, all locks
   - For pipeline components (receivers, processors, exporters)

2. **Slow Path** (standard `tracing::*!` macros, global subscriber):
   - Uses existing global subscriber
   - ~5-10μs latency (locks, synchronization)
   - For bootstrap, initialization, third-party libraries
   - Remains unchanged

**Rationale for Dual-Path Approach**:

1. **Explicit Performance**: Effect handler requirement = fast path, clear at call site
2. **Pragmatic Compatibility**: Don't break bootstrap/init code, it's not performance critical
3. **Third-Party Support**: Libraries using `tracing::` macros continue to work
4. **Clear Boundaries**: Pipeline runtime code vs. initialization code have different needs
5. **Maximum Performance Where It Matters**: Hot path is optimized, cold path is convenient
6. **No Migration Pressure**: Bootstrap code can stay on slow path indefinitely

**Usage Patterns**:

[SNIP]

**Key Insight**: By requiring effect handler as **argument 0**, the performance path is:
- **Obvious at call site** (you see the effect parameter)
- **Compile-time enforced** (no effect = won't compile with otel_*! macros)
- **Zero runtime overhead** (direct buffer access)

Bootstrap code simply uses `tracing::*!` instead of `otel_*!` - different macros for different needs.

## Implementation Status

### Phase 0: Internal Telemetry Receiver ✅ **COMPLETED** (December 18, 2025)

**Architecture**: The complete flow for internal telemetry capture:

```text
tracing::info!(count = 42, error = ?e)  [Application Code]
    ↓
OtlpTracingLayer (crates/telemetry/src/tracing_integration/)
    ↓ Visit::record_i64(), record_debug(), etc.
    ↓ Captures full structure via TracingAnyValue (scalars, arrays, maps)
    ↓ Builds TracingLogRecord with complete nested data
    ↓ Encodes to OTLP bytes using stateful encoder
    ↓ Sends Vec<u8> to channel
    ↓
InternalTelemetryReceiver (crates/otap/src/internal_telemetry_receiver/)
    ↓ Receives OTLP bytes from channel
    ↓ Wraps: OtapPdata::new(Context::default(), 
    │         OtlpProtoBytes::ExportLogsRequest(bytes).into())
    ↓ Injects into dataflow pipeline
    ↓
Dataflow Pipeline (processors, exporters, etc.)
```

**What was built**:
1. **Extended TracingAnyValue** (crates/telemetry/src/tracing_integration/log_record.rs):
   - Added `Array(Vec<TracingAnyValue>)` for nested lists
   - Added `KeyValueList(Vec<TracingAttribute>)` for maps/structs
   - Added `Bytes(Vec<u8>)` for binary data
   - Full AnyValueView implementation with nested iterators

2. **Internal Telemetry Receiver** (crates/otap/src/internal_telemetry_receiver/):
   - Receives pre-encoded OTLP bytes via channel
   - Wraps as OtapPdata and injects into pipeline
   - Buffering and periodic flushing
   - Backpressure handling

3. **Global Integration Point**:
   - `get_otlp_bytes_sender()` provides channel for OtlpTracingLayer
   - Initialized when receiver starts
   - Thread-safe static for global access

**Key Insight**: No `fmt` layer needed! The TracingLogRecord captures complete structure via the Visit trait, preserves it in TracingAnyValue (which mirrors OTLP's AnyValue perfectly), and encodes directly to OTLP bytes. Arrays, maps, nested structures - everything is preserved with full fidelity.

**Next Steps**: Wire up OtlpTracingLayer to use the channel (Phase 0.5), then Phase 1 for effect handler integration

## Implementation Phases

### Phase 1: Foundation (Weeks 1-2)

**Objective**: Build streaming OTLP encoder with reusable buffers

Tasks:
1. Add `Vec<u8>` buffer to `EffectHandlerCore` (reusable OTLP byte buffer)
2. Implement `SingleLogRecordView` (ephemeral view for one log event)
3. Create streaming OTLP encoder that appends to existing Vec<u8>
4. Implement inline LogRecord construction (stack-allocated, short-lived)
5. Write unit tests for streaming encoding and buffer reuse

Deliverables:
- `crates/telemetry/src/tracing/single_view.rs` (SingleLogRecordView)
- `crates/telemetry/src/tracing/streaming_otlp.rs` (append-based encoder)
- `crates/telemetry/src/tracing/logger.rs` (inline LogRecord builder)
- Update to `crates/engine/src/effect_handler.rs` (add Vec<u8> buffer)
- Comprehensive unit tests

**Key Innovation**: Synchronous view construction + streaming encoding to reusable buffer

### Phase 2: Effect Handler Integration (Weeks 3-4)

**Objective**: Wire up buffer ownership and access patterns

Tasks:
1. Update all effect handler constructors to create trace buffer
2. Implement buffer access methods (`trace_buffer()`, `drain_trace_events()`)
3. Add pipeline context to effect handler
4. Create initialization flow during pipeline startup
5. Add buffer cleanup on pipeline shutdown

Deliverables:
- Updated effect handler in `crates/engine/src/effect_handler.rs`
- Updated local/shared effect handlers
- Integration tests with mock pipelines
- Documentation updates

**Simpler**: Buffer is just another field on effect handler, like metrics_reporter

### Phase 3: Size-Based Flush & Direct Injection (Weeks 5-6)

**Objective**: Enable size-based threshold flushing with buffer reuse

Tasks:
1. Implement size-based flush logic (check `buffer.len()` after each append)
2. Add `flush_otlp_buffer()` method (send + clear, retain capacity)
3. Implement `send_message()` integration for OTLP bytes
4. Add configurable size threshold (bytes, default 64KB, 1=synchronous)
5. Handle backpressure (drop or block strategy)

Deliverables:
- Flush logic in `crates/engine/src/effect_handler.rs`
- Configuration schema updates (threshold in bytes)
- Integration tests with real pipeline
- Performance tests (streaming vs batch encoding)

**Simplified**: Inline threshold check, reusable buffer, no separate receiver

### Phase 4: Macro Updates (Week 7)

**Objective**: Update logging macros with strict requirements

Tasks:
1. **Change macro signatures** to require effect handler parameter
2. Implement thread-local-only logging (no fallback)
3. Add compile-time checks for proper usage
4. Create migration tooling/scripts to help with refactoring
5. Update all internal usages (will be compile errors otherwise)
6. Performance benchmarking

Deliverables:
- Updated macros in `crates/telemetry/src/internal_events.rs`
- Migration guide and automation tools
- All internal code migrated
- Performance comparison report

### Phase 5: Migration and Testing (Weeks 8-9)

**Objective**: Migrate existing components and validate

Tasks:
1. Update built-in receivers, processors, exporters
2. Add configuration examples
3. Comprehensive integration testing
4. Performance profiling and optimization
5. Documentation and examples

Deliverables:
- All built-in components using new system
- Example configurations
- Performance benchmarks
- Updated README and docs

### Phase 6: OpenTelemetry SDK Integration (Week 10)

**Objective**: Integrate with OpenTelemetry Rust SDK

Tasks:
1. Implement layer for OpenTelemetry SDK export
2. Support OTLP export of internal telemetry
3. Add resource attribute propagation
4. Test with external collectors
5. Documentation

Deliverables:
- OpenTelemetry SDK layer integration
- OTLP export examples
- End-to-end testing with real collectors

## Technical Design Details

### Reusable OTLP Buffer Design

Use a simple `Vec<u8>` for OTLP-encoded bytes:

[SNIP]

Buffer sizing:
- Default: 64KB per core (configurable)
- Set to 1 byte for synchronous logging
- Capacity grows to working set, then stabilizes
- Overflow strategy: flush immediately (inline)

### Event Representation

**Build ephemeral LogRecord, encode immediately**:

[SNIP]

**Implement single-event view for streaming**:

[SNIP]

**Streaming OTLP encoder**:

[SNIP]

**Benefits**:
- **Zero Heap Allocation**: LogRecord on stack, view borrows it
- **Streaming**: Encode incrementally, not in batches
- **Reusable Buffer**: Single Vec<u8> per core
- **Synchronous**: Construct view and encode as event occurs
- **Simple**: No buffering layer, just append bytes

### Buffer Lifecycle

```
┌─────────────────────────────────────────┐
│  Pipeline Engine Starts                 │
└────────────┬────────────────────────────┘
             │
             v
┌─────────────────────────────────────────┐
│  Create Effect Handlers                 │
│  - Allocate trace buffer in constructor │
│  - Buffer size from config              │
│  - No separate initialization needed    │
└────────────┬────────────────────────────┘
             │
             v
┌─────────────────────────────────────────┐
│  Start Pipeline Components              │
│  - Receivers, Processors, Exporters     │
│  - Each has effect handler with buffer  │
└────────────┬────────────────────────────┘
             │
             v
┌─────────────────────────────────────────┐
│  Start InternalTelemetryReceiver        │
│  - Set up periodic flush timer          │
│  - Access buffer via effect handler     │
└────────────┬────────────────────────────┘
             │
             v
┌─────────────────────────────────────────┐
│  Normal Operation                       │
│  - Components log via otel_*! macros    │
│  - Direct write to effect.trace_buffer  │
│  - Periodic flush to pipeline           │
└────────────┬────────────────────────────┘
             │
             v
┌─────────────────────────────────────────┐
│  Shutdown                               │
│  - Final flush via drain_trace_events() │
│  - Buffer dropped with effect handler   │
└─────────────────────────────────────────┘
```

**Simpler Lifecycle**:
- No thread-local initialization
- Buffer is just another field
- Automatic cleanup (RAII)

### Integration with Effect Handler

Effect handler owns reusable OTLP buffer and handles streaming encoding:

[SNIP]

**Performance Benefits**:
- **Single Allocation**: Vec<u8> grows once, capacity stable
- **Streaming Encoding**: Append bytes as events occur
- **Zero Intermediate Storage**: No LogRecord buffering
- **Size-Based Batching**: Flush when byte threshold reached
- **Synchronous Option**: threshold=1 = immediate flush
- **Direct Injection**: send_message() with OTLP bytes
- **Simple**: No complex buffer logic, just append + check size

### Configuration

Add configuration section for internal telemetry:

```yaml
service:
  telemetry:
    logs:
      level: info
      internal:
        enabled: true
        buffer_size: 1024
        flush_interval: 100ms
        overflow_strategy: drop_oldest  # or 'block'
        export:
          enabled: true
          pipeline: "telemetry/internal-logs"
```

## Performance Considerations

### Expected Improvements

1. **Zero Contention**: No locks, no synchronization
   - Current: O(cores) contention on global subscriber lock
   - New: O(1) direct buffer write via Rc<RefCell<_>>

2. **Ultra-Low Latency**: Direct buffer write
   - Target: < 100ns for buffered event write
   - Current: ~5-10μs with global subscriber
   - Eliminated: thread_local! lookup (~10-20ns)
   - Eliminated: subscriber trait dispatch overhead (~5-10ns)
   - Eliminated: Option unwrapping

3. **Better Cache Locality**: Buffer owned by effect handler (already hot)

4. **Predictable Performance**: No locks, no atomics, no contention

5. **Aggressive Inlining**: Direct access allows compiler optimizations

### Tradeoffs

1. **Memory**: Increased per-core memory usage
   - ~100KB per core for buffer (configurable)
   - Acceptable for thread-per-core model

2. **Complexity**: More complex initialization
   - Mitigated by clear lifecycle management

3. **Testing**: Need per-thread test setup
   - Addressed by test utilities

## Testing Strategy

### Unit Tests

1. `OtapSubscriber` basic functionality
2. Ring buffer operations (push, drain, overflow)
3. Event conversion to LogRecord
4. Macro expansion and compilation

### Integration Tests

1. Single pipeline with internal telemetry
2. Multiple pipelines on different cores
3. Graceful shutdown with buffer flush
4. Overflow handling under load
5. Effect handler integration

### Performance Tests

1. Event recording latency
2. Throughput benchmarks
3. Memory usage profiling
4. Contention measurement

### End-to-End Tests

1. Full pipeline with OTLP export
2. External collector integration
3. High-throughput scenarios
4. Stress testing

## Migration Path

### Additive Change Philosophy

This design is **additive, not breaking**:

1. **Global Subscriber Stays**: Existing code continues to work
2. **New Fast Path**: `otel_*!` macros with effect handler are opt-in
3. **Clear Performance Model**: Fast vs. slow paths are explicit at call site
4. **Progressive Migration**: Migrate hot paths first, leave cold paths alone

**Why This Approach Works**:

1. **Performance-Critical Code is Explicit**: Pipeline components already have effect handlers
2. **Bootstrap Code is Unchanged**: Initialization isn't performance-critical anyway
3. **Third-Party Libraries Work**: They use `tracing::*!`, hit global subscriber
4. **Clear at Call Site**: `otel_info!(effect, ...)` vs `tracing::info!()` shows intent

**Migration Strategy**:

[SNIP]

**The Key Insight**: 
- Use `otel_*!(effect, ...)` in hot paths (pipeline components)
- Use `tracing::*!()` in cold paths (bootstrap, tests)
- Performance model is **obvious from the macro name**

### Migration Steps for Components

1. **Identify hot paths** (performance-critical logging):
   - Receiver, processor, exporter implementations
   - Per-message or per-batch operations
   - Inner loops and high-frequency operations

2. **Update component logging** to use the effect handler:
   ```rust
   // Old - slow path (global subscriber)
   use tracing::info;
   info!("processing batch");  // ~5μs
   
   // New - fast path (direct buffer)
   use otap_df_telemetry::otel_info;
   otel_info!(effect, "processing.batch");  // < 100ns
   ```

3. **Leave cold paths alone**
   ```rust
   // Bootstrap/init code - no change needed
   fn main() {
       tracing::info!("Starting controller");  // Slow path is fine here
   }
   ```

4. **Validation**: Both paths work, choose based on needs
   - Fast path: Requires effect handler, < 100ns
   - Slow path: No requirements, ~5μs
   - No breaking changes

## Open Questions and Future Work

### Open Questions

1. **Span Context Propagation**: How to efficiently propagate span context in thread-local model?
   - Solution: Thread-local span stack in `ThreadLocalTracingContext`

2. **Cross-Core Tracing**: How to handle traces that span multiple cores?
   - Solution: Accept eventual consistency, use trace IDs for correlation

3. **Backpressure**: What happens when internal telemetry pipeline is slow?
   - Solution: Configurable overflow strategy (drop or block)

4. **Dynamic Configuration**: Support runtime reconfiguration?
   - Future: Add control messages for reconfiguration

### Future Enhancements

1. **Structured Logging**: Enhanced support for structured fields
2. **Sampling**: Intelligent sampling for high-throughput scenarios
3. **Compression**: Buffer compression for reduced memory
4. **External Storage**: Direct-to-disk buffering for critical logs
5. **Distributed Tracing**: Enhanced correlation across services
6. **Metrics Integration**: Automatic metrics from trace data

## SDK Integration Opportunity

This design opens a **major opportunity**: bolting OTel SDKs directly to the dataflow engine!

### Vision

[SNIP]

### Implementation Path

1. **SDK LogRecord implements LogsDataView** ✓ (Phase 1)
2. **OTLP encoder uses LogsDataView** ✓ (Phase 3)
3. **Custom SDK LogProcessor**:
   - Buffers SDK LogRecords
   - Uses view API to encode
   - Injects via effect handler
4. **Similar for Traces & Metrics**:
   - `TracesDataView` for SDK spans
   - `MetricsDataView` for SDK metrics

### Benefits

- **Standard SDKs**: Applications use official OTel SDKs
- **Dataflow Processing**: Telemetry flows through OTAP pipeline
- **View Abstraction**: Clean interface between SDK and engine
- **Performance**: Zero-copy encoding via views
- **Ecosystem**: Integrates with OTel exporters, processors, etc.

### Future Work

- Implement `TracesDataView` for SDK spans
- Implement `MetricsDataView` for SDK metrics  
- Create SDK processor adaptors
- Document SDK -> Dataflow integration patterns

## Dependencies

### Required Crates

- `tracing-core`: Core tracing abstractions
- `tracing-subscriber`: Layer and subscriber utilities
- `smallvec`: Stack-allocated vectors for fields
- `crossbeam-utils`: Atomic operations (if needed)

### Internal Dependencies

- `otap-df-engine`: Effect handler integration
- `otap-df-telemetry`: Existing telemetry infrastructure
- `otap-df-config`: Configuration schema

## Success Criteria

1. **Performance**: < 100ns per buffered trace event (direct access)
2. **Zero Contention**: No measurable lock contention (no locks!)
3. **Zero Indirection**: Direct buffer access, no thread_local! lookup
4. **Memory**: < 200KB per core overhead
5. **Compatibility**: Migration complete (breaking change)
6. **Coverage**: > 80% code coverage
7. **Documentation**: Complete API documentation and examples
8. **Simplicity**: Less code than current implementation

## References

1. [Tokio Tracing Documentation](https://docs.rs/tracing)
2. [OpenTelemetry Logs Specification](https://opentelemetry.io/docs/specs/otel/logs/)
3. [Thread-Per-Core Architecture Paper](https://www.usenix.org/conference/osdi21/presentation/ousterhout)
4. [OTAP-Dataflow Engine Design](../crates/engine/README.md)
5. [Shared-Nothing Architecture Principles](../../docs/design-principles.md)
