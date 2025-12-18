# Internal Telemetry Receiver

The internal telemetry receiver collects self-diagnostic telemetry from the OTAP-Dataflow engine itself, enabling observability of the observability pipeline.

## Overview

This receiver supports two operational modes to align with the dataflow engine's architecture:

### 1. Global Tracing Subscriber Mode (Shared Receiver) ✅ **Implemented**
- Registers as a global `tracing` subscriber to collect logs from all components
- Operates as a **shared** receiver (Send + Sync) to handle multithreaded collection
- Suitable for collecting telemetry from bootstrap code, initialization, and third-party libraries
- Current implementation phase

### 2. Effect Handler Mode (Local Receiver) 🚧 **Future Work**
- Will directly integrate with the effect handler's OTLP buffer
- Will operate as a **local** receiver (!Send) for per-core efficiency
- Enables fast-path logging with sub-microsecond overhead
- Part of Phase 1-6 of the custom tracing subscriber plan

## Configuration

```yaml
receivers:
  internal_telemetry:
    urn: "urn:otap:receiver:internal-telemetry:v1"
    # Flush interval for sending batched telemetry (default: 1s)
    flush_interval: 1s
    
    # Maximum log level to capture (trace, debug, info, warn, error)
    max_level: info
    
    # Whether to register as the global tracing subscriber (default: true)
    register_global: true
    
    # Maximum number of events to buffer before dropping (default: 10000)
    max_buffer_size: 10000
```

## Usage Example

Add the receiver to a pipeline to route internal telemetry through the same dataflow as user data:

```yaml
pipelines:
  # Main data pipeline
  traces/production:
    receivers: [otlp]
    processors: [batch]
    exporters: [otlp]
  
  # Internal diagnostics pipeline
  logs/diagnostics:
    receivers: [internal_telemetry]
    processors: [batch, attributes]
    exporters: [otlp/internal]
```

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Application Code (including OTAP components)       │
│                                                      │
│  tracing::info!("message")  ~5μs (slow path)        │
│         │                                            │
│         v                                            │
│  ┌──────────────────────┐                           │
│  │ Global Subscriber    │                           │
│  │ (tokio tracing)      │                           │
│  └──────────┬───────────┘                           │
│             │                                        │
└─────────────┼────────────────────────────────────────┘
              │
              v
┌─────────────────────────────────────────────────────┐
│  Internal Telemetry Receiver (Shared)               │
│                                                      │
│  - Captures TracingEvent from global subscriber     │
│  - Buffers events between flushes                   │
│  - Converts to OTLP format (TODO)                   │
│  - Injects into dataflow pipeline                   │
└─────────────┬───────────────────────────────────────┘
              │
              v
┌─────────────────────────────────────────────────────┐
│  Dataflow Pipeline                                  │
│  (Processors → Exporters)                           │
└─────────────────────────────────────────────────────┘
```

## Implementation Status

### Current (Phase 0)
- ✅ Global tracing subscriber integration
- ✅ Event buffering and periodic flushing
- ✅ Shared receiver implementation
- ✅ Configuration schema
- ✅ Backpressure handling (drop oldest events)
- ⏸️ OTLP encoding (returns error - needs Phase 1)

### Future Phases (from custom-tracing-subscriber-plan.md)

#### Phase 1: Foundation
- [ ] Streaming OTLP encoder with reusable buffers
- [ ] SingleLogRecordView for ephemeral log events
- [ ] Inline LogRecord construction

#### Phase 2: Effect Handler Integration
- [ ] Add Vec<u8> OTLP buffer to EffectHandlerCore
- [ ] Buffer ownership and access patterns
- [ ] PipelineContext integration

#### Phase 3: Direct Injection
- [ ] Size-based threshold flushing
- [ ] Direct OTLP bytes injection via send_message()
- [ ] Configurable flush thresholds

#### Phase 4: Macro Updates
- [ ] Update otel_*! macros to require effect handler
- [ ] Thread-local-only logging (no fallback)
- [ ] Compile-time enforcement

#### Phase 5: Migration
- [ ] Update built-in components
- [ ] Integration testing
- [ ] Performance profiling

#### Phase 6: OpenTelemetry SDK Integration
- [ ] Layer for OpenTelemetry SDK export
- [ ] OTLP export support
- [ ] Resource attribute propagation

## Performance Considerations

### Current Implementation (Global Subscriber)
- **Latency**: ~5-10μs per log event (due to global locks)
- **Thread Safety**: Uses locks for synchronization
- **Memory**: Buffers up to `max_buffer_size` events (default 10,000)
- **Contention**: O(cores) on global subscriber lock

### Future Implementation (Effect Handler Mode)
- **Latency**: < 100ns for buffered event write
- **Thread Safety**: Zero locks, zero contention
- **Memory**: ~100KB per core (configurable)
- **Contention**: None (thread-local buffers)

## Testing

Run the tests:
```bash
cargo test -p otap-df-otap internal_telemetry
```

## Related Documentation

- [Custom Tracing Subscriber Plan](../../../docs/custom-tracing-subscriber-plan.md) - Full implementation roadmap
- [Stateful Encoder Phase 1 Summary](../../../docs/stateful-encoder-phase1-summary.md) - OTLP encoding foundation
- [OTAP Basics](../../../docs/otap_basics.md) - Data model overview

## Future Enhancements

1. **Structured Logging**: Enhanced support for structured fields
2. **Sampling**: Intelligent sampling for high-throughput scenarios
3. **Compression**: Buffer compression for reduced memory
4. **Distributed Tracing**: Enhanced correlation across services
5. **Span Tracking**: Full span context propagation
6. **Metrics Integration**: Automatic metrics from trace data

## Notes

- The receiver currently returns an error when attempting to convert events to OTLP format. This is expected and will be resolved in Phase 1 of the implementation plan.
- The global subscriber approach is intentionally chosen as a pragmatic starting point that works with existing code patterns.
- Future phases will introduce the fast-path effect handler integration for performance-critical pipeline code.
