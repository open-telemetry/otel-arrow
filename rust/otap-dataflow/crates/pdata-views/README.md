# pdata-views

`pdata-views` is a Rust crate within the `otap-dataflow` project. It provides 
view abstractions and utilities for working with OTLP pdata (protocol data) 
structures, enabling efficient data access in OpenTelemetry Arrow pipelines.

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
