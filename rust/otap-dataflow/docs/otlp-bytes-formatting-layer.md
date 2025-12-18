# OTLP Bytes Formatting Layer

## Overview

The `OtlpBytesFormattingLayer` provides a way to decode OTLP-encoded telemetry back into human-readable console output. This enables a flexible architecture where:

1. **All tracing events are encoded to OTLP bytes first** (preserving complete structural fidelity)
2. **OTLP bytes can be formatted for human consumption** (colorized, customizable output)
3. **No dependency on opentelemetry SDK for formatting** (uses only our LogsDataView traits)
4. **Future-ready for async formatting** (in separate thread/process)

## Architecture

```text
tracing::info!(count = 42, error = ?e)
    ↓
OtlpTracingLayer
    ↓ Visit::record_*() - captures full structure via TracingAnyValue
    ↓ Builds TracingLogRecord (implements LogRecordView)
    ↓ Encodes to OTLP bytes using StatefulOtlpEncoder
    ↓ Sends Vec<u8> to channel
    ↓
OtlpBytesFormattingLayer
    ↓ Constructs RawLogsData (LogsDataView) from OTLP bytes (zero-copy)
    ↓ Iterates through resources → scopes → log records
    ↓ Formats each field (timestamp, level, target, message, attributes)
    ↓ Writes to console with ANSI colors
```

## Benefits

### 1. Complete Structural Fidelity
- TracingAnyValue supports: scalars, arrays, maps (nested structures)
- OTLP encoding preserves everything
- No information loss through the round-trip

### 2. Remove OpenTelemetry SDK Dependency
- No need for `opentelemetry` crate for formatting
- Uses only our pdata views (LogsDataView, LogRecordView)
- Lighter dependency footprint

### 3. Flexible Output Configuration
- ANSI colors (on/off)
- Timestamp (on/off)
- Log level (on/off)
- Target/module path (on/off)
- Custom formatters possible

### 4. Async Formatting Ready
- OTLP bytes can be sent to separate thread
- Decoding and formatting don't block event capture
- Can buffer, batch, or rate-limit output

## Usage Example

```rust
use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;
use otap_df_telemetry::tracing_integration::{OtlpTracingLayer, OtlpBytesFormattingLayer};
use std::sync::mpsc;
use std::io;

// Create channel for OTLP bytes
let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(100);

// Create OTLP encoding layer
let resource_bytes = Vec::new();
let tx_clone = tx.clone();
let otlp_layer = OtlpTracingLayer::new(move |log_record| {
    let mut enc = StatefulOtlpEncoder::new(4096);
    let target = log_record.target();
    
    enc.encode_log_record(&log_record, &resource_bytes, target).unwrap();
    let bytes = enc.flush();
    tx_clone.send(bytes.to_vec()).unwrap();
});

// Create formatting layer for human output
let fmt_layer = OtlpBytesFormattingLayer::new(io::stdout)
    .with_ansi(true)
    .with_timestamp(true)
    .with_target(true);

// Format OTLP bytes in separate thread
std::thread::spawn(move || {
    for otlp_bytes in rx {
        fmt_layer.format_otlp_bytes(&otlp_bytes).unwrap();
    }
});

// Install tracing subscriber
use tracing_subscriber::prelude::*;
tracing_subscriber::registry()
    .with(otlp_layer)
    .init();

// Emit events - they'll be encoded to OTLP and formatted
tracing::info!(port = 8080, "Server starting");
```

## Implementation Details

### OtlpBytesFormattingLayer

**Location**: `crates/telemetry/src/tracing_integration/otlp_bytes_formatter.rs`

**Key Features**:
- Decodes OTLP bytes using `RawLogsData` (zero-copy view)
- Traverses logs data structure: resources → scopes → log records
- Formats each log record with configurable options
- Supports nested data structures (arrays, maps)
- ANSI color support for log levels

**Configuration Options**:
```rust
OtlpBytesFormattingLayer::new(writer)
    .with_ansi(bool)       // ANSI colors on/off
    .with_timestamp(bool)  // Include timestamp
    .with_level(bool)      // Include log level
    .with_target(bool)     // Include target (module path)
```

### format_any_value

Recursively formats OTLP AnyValue types:
- **Scalars**: String, Int64, Bool, Double, Bytes
- **Array**: `[item1, item2, ...]`
- **KeyValueList**: `{key1=val1, key2=val2, ...}`

Preserves full structure through nested iteration.

## Example Output

```
INFO app::server: Server starting port=8080, host=localhost
INFO app::server: Server ready status=ready, uptime_seconds=5
WARN app::database: Connection retry retry_count=3, max_retries=5
ERROR app::database: Connection failed error=connection timeout, timeout_ms=5000
DEBUG app::cache: Cache statistics cache_size=1024, hit_rate=0.950000
```

(With ANSI colors: INFO=green, WARN=yellow, ERROR=red, DEBUG=blue, TRACE=magenta)

## Future Enhancements

### 1. Async Formatting Thread
- Spawn dedicated formatter thread
- Use bounded channel with backpressure
- Rate limit console output

### 2. Custom Formatters
- JSON output formatter
- Compact formatter (single line)
- Detailed formatter (multi-line attributes)

### 3. Filtering
- Filter by log level
- Filter by target pattern
- Filter by attribute values

### 4. Performance
- Batch multiple log records before formatting
- Reuse string buffers
- Profile and optimize hot paths

## Comparison with Previous Approach

### Before (OpenTelemetry SDK)
```text
tracing::info!() → tracing-appender-opentelemetry
                   → opentelemetry SDK
                   → opentelemetry-stdout exporter
                   → format and print
```

**Issues**:
- Heavy dependency on opentelemetry SDK
- Limited control over formatting
- Coupled to OpenTelemetry's formatting logic

### After (OTLP Bytes Formatting)
```text
tracing::info!() → OtlpTracingLayer
                   → encode to OTLP bytes
                   → OtlpBytesFormattingLayer
                   → decode and format
                   → print
```

**Benefits**:
- No opentelemetry SDK dependency
- Full control over formatting
- Can format in separate thread
- OTLP bytes can go to multiple destinations (pipeline + console)

## Testing

Example located at: `crates/telemetry/examples/otlp_bytes_formatting.rs`

Run with:
```bash
cargo run -p otap-df-telemetry --example otlp_bytes_formatting
```

## Integration Points

### With Internal Telemetry Receiver
The internal telemetry receiver (Phase 0) receives OTLP bytes via channel and injects them into the dataflow pipeline. The same OTLP bytes can also be sent to `OtlpBytesFormattingLayer` for console output:

```text
OtlpTracingLayer → OTLP bytes → ┬→ InternalTelemetryReceiver → Pipeline
                                 └→ OtlpBytesFormattingLayer → Console
```

This enables both structured telemetry (in pipeline) and human-readable output (on console) from the same encoded bytes.

### With Effect Handler Telemetry (Phase 1)
Future integration will allow effect handlers to emit OTLP bytes directly. These can be formatted for debugging without requiring the full pipeline.

## Summary

The `OtlpBytesFormattingLayer` completes the round-trip:
1. **Encode**: Capture structure → OTLP bytes (lossless)
2. **Decode**: OTLP bytes → LogsDataView (zero-copy)
3. **Format**: LogsDataView → human-readable output (colorized)

This architecture provides:
- ✅ Complete structural fidelity
- ✅ No OpenTelemetry SDK dependency
- ✅ Flexible, customizable output
- ✅ Ready for async formatting
- ✅ Multiple output destinations from single encoding
