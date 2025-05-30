# OTLP Nodes

## GenericBatchProcessor

A flexible, type-safe batch processor for OpenTelemetry Protocol (OTLP) data that leverages Rust's type system for efficient and safe batching of telemetry data.

### Key Features

- **Generic Data Handling**: Uses Rust's type system to handle all OTLP data types (traces, metrics, logs) with a single, unified implementation
- **Type-Safe Batching**: Ensures type safety at compile time while maintaining efficient batching operations
- **Preservation of Data Structure**: Maintains the original structure of metrics (Gauges, Sums, Histograms) during batching

### Configurable Flush Triggers

- **Batch Size**: Flushes when the number of items reaches a configurable threshold
- **Timeout**: Flushes after a specified duration to ensure timely processing
- **Shutdown**: Guarantees all buffered data is emitted during graceful shutdown

### Technical Highlights

- **Efficient Data Structures**: Uses `VecDeque` for optimal front/back operations during batching
- **Zero-Copy Operations**: Minimizes data copying during batching operations
- **Thread-Safe Design**: Implements safe concurrent operation through message passing

### Use Cases

- Production environments requiring type safety and performance
- Scenarios with mixed telemetry types needing consistent batching
- Systems where data integrity and structure preservation are critical
- High-performance telemetry pipelines with strict latency requirements

### Example Usage

```rust
let config = BatchConfig {
    send_batch_size: 100,  // Batch size threshold
    timeout: Duration::from_secs(5),  // Maximum time to hold a batch
};
let batcher = GenericBatcher::new(config);
// Use with your OTLP data pipeline