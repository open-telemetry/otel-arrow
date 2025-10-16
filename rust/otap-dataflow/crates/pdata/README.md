# pdata

PData refers generally to the pipeline data type used within an
OpenTelemetry pipeline.  This package is the location of our core
utilities for converting between several representations.

The `otap_df_pdata::views` provides view abstractions and utilities
for working with OTLP pdata (protocol data) structures, enabling
efficient data access in OpenTelemetry Arrow pipelines.

## Key Features

- **Zero-cost abstractions**: No runtime overhead for abstraction
- **Lazy evaluation**: Data is parsed/accessed only when needed
- **Memory efficient**: Direct access to underlying data without copying
- **Type safety**: Strong typing with compile-time guarantees
- **Lifetime-aware**: Proper memory management across different storage
  patterns

## Supported Backends Roadmap

[ ] **Struct Backend**: Native Rust structs with owned data
[ ] **OTLP Bytes Backend**: serialized otlp bytes representation
[ ] **JSON Backend**: serde_json::Value for dynamic JSON processing
[ ] **SYSLOG Backend**: Zero-allocation parsing of syslog/CEF strings
