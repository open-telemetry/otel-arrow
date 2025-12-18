# Tracing Integration Optimization

## Overview
Optimized the tokio-tracing integration to use tracing metadata's `target` field (typically the module path) as the OpenTelemetry InstrumentationScope name, simplifying the API from ID-based pre-encoded scopes to on-the-fly scope name encoding.

## Changes Made

### 1. StatefulOtlpEncoder API Refactoring

**Before:**
```rust
pub struct ScopeEncoding {
    pub id: ScopeId,
    pub encoded_bytes: OtlpBytes,
}

encoder.encode_log_record(&log_record, &resource_bytes, &scope_encoding)?;
```

**After:**
```rust
// Direct scope name API - much simpler!
encoder.encode_log_record(&log_record, &resource_bytes, "my_module::component")?;
```

**Key Changes:**
- Removed `ScopeEncoding` struct and `ScopeId` type alias
- Changed `encode_log_record()` signature to accept `scope_name: &str` instead of `scope_encoding: &ScopeEncoding`
- Added `encode_instrumentation_scope()` method that encodes scope on-the-fly
- Updated state tracking from `current_scope_id: Option<ScopeId>` to `current_scope_name: Option<String>`
- Scope comparison now uses string equality instead of ID equality
- Removed `encode_scope_bytes()` helper function

### 2. Tracing Integration Benefits

The tracing `Metadata` struct provides a `target()` method that returns the module path or custom target string. This is a perfect fit for OpenTelemetry's InstrumentationScope concept:

```rust
// Each tracing event has a target (module path by default)
tracing::info!(target: "app::server", "Server started");
// -> InstrumentationScope.name = "app::server"

tracing::warn!("Warning message");  
// -> InstrumentationScope.name = "my_crate::my_module" (automatic from callsite)
```

**Integration Pattern:**
```rust
let layer = OtlpTracingLayer::new(move |log_record| {
    let scope_name = log_record.target();  // Get module path from tracing
    encoder.encode_log_record(&log_record, &resource_bytes, scope_name)?;
});
```

### 3. Automatic Batching by Scope

The stateful encoder automatically batches log records by scope name:
- When scope names match: Appends log record to current ScopeLogs
- When scope names differ: Closes previous scope, opens new scope with the new name
- Encodes InstrumentationScope with just the `name` field (version and attributes omitted)

### 4. Files Modified

#### Core Changes:
- **crates/pdata/src/otlp/stateful_encoder.rs**
  - Refactored API from ScopeEncoding to scope_name
  - Added encode_instrumentation_scope() method
  - Updated all 3 encoder tests
  - Removed obsolete types and helpers
  - Updated documentation and examples

#### Integration Updates:
- **crates/telemetry/tests/tracing_integration_test.rs**
  - Removed ScopeEncoding import
  - Updated test_encode_tracing_log_record_mock to use scope_name

#### New Examples:
- **crates/telemetry/examples/tracing_to_otlp.rs**
  - Complete end-to-end example showing tracing -> OTLP encoding
  - Demonstrates multiple scopes and automatic batching
  - Shows proper use of Arc<Mutex<>> for shared encoder

## Benefits

1. **Simpler API**: No need to pre-encode scopes or manage scope IDs
2. **Natural Mapping**: Tracing targets map directly to InstrumentationScope names
3. **Less Boilerplate**: Removed ~100 lines of code (ScopeEncoding struct, helpers, ID management)
4. **Flexibility**: Scope names can be dynamically determined from tracing metadata
5. **Third-party Compatibility**: Works seamlessly with third-party libraries that use tracing

## Testing

All tests pass:
- **otap-df-pdata**: 3 encoder tests (state machine, batching, scope changes)
- **otap-df-telemetry**: 74 tests (69 unit + 5 integration)
- **Example**: tracing_to_otlp runs successfully, generates 430 bytes of OTLP data

## Usage Example

```rust
use otap_df_pdata::otlp::stateful_encoder::StatefulOtlpEncoder;
use otap_df_telemetry::tracing_integration::OtlpTracingLayer;
use std::sync::{Arc, Mutex};
use tracing_subscriber::prelude::*;

// Create shared encoder
let encoder = Arc::new(Mutex::new(StatefulOtlpEncoder::new(64 * 1024)));
let encoder_clone = encoder.clone();

// Pre-encode resource
let resource_bytes = vec![];

// Create layer that uses tracing target as scope name
let layer = OtlpTracingLayer::new(move |log_record| {
    let scope_name = log_record.target();  // "my_crate::my_module"
    if let Ok(mut enc) = encoder_clone.lock() {
        enc.encode_log_record(&log_record, &resource_bytes, scope_name)?;
    }
    Ok(())
});

// Install subscriber
tracing_subscriber::registry().with(layer).init();

// Emit events - automatically batched by target/scope
tracing::info!(target: "app::server", "Server started");
tracing::info!(target: "app::server", "Request received");  // Batched!
tracing::warn!(target: "app::cache", "Cache miss");  // New scope

// Flush to get OTLP bytes
let otlp_bytes = encoder.lock().unwrap().flush();
```

## Wire Format

The encoder generates OTLP ExportLogsServiceRequest with:
```protobuf
ExportLogsServiceRequest {
  resource_logs: [
    ResourceLogs {
      resource: Resource { ... }  // Pre-encoded, copied once
      scope_logs: [
        ScopeLogs {
          scope: InstrumentationScope {
            name: "app::server"  // From tracing target
          }
          log_records: [
            LogRecord { ... },  // All logs from app::server
            LogRecord { ... },
          ]
        },
        ScopeLogs {
          scope: InstrumentationScope {
            name: "app::cache"  // Different scope
          }
          log_records: [
            LogRecord { ... },  // Logs from app::cache
          ]
        }
      ]
    }
  ]
}
```

## Migration Guide

If you have existing code using the old API:

**Old Code:**
```rust
let scope_encoding = ScopeEncoding {
    id: 1,
    encoded_bytes: encode_scope_bytes(&scope),
};
encoder.encode_log_record(&log_record, &resource_bytes, &scope_encoding)?;
```

**New Code:**
```rust
let scope_name = "my_module::component";
encoder.encode_log_record(&log_record, &resource_bytes, scope_name)?;
```

No other changes needed - the encoder handles everything internally.
