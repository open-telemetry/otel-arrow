# OTLP Nodes

## SimpleBatchProcessor

A lightweight, synchronous batch processor designed for OpenTelemetry Protocol (OTLP) data within the Arrow pipeline. This processor efficiently collects and batches telemetry data before forwarding it for further processing.

### Key Features

- Multiple Data Type Support: Handles all three OTLP data types (traces, metrics, and logs) independently
- Automatic Batching: Collects data points and combines them into batches based on configurable criteria

### Configurable Flush Triggers

- **Batch size**: Flushes when the number of items reaches a configured threshold
- **Timeout**: Flushes after a specified duration, ensuring data doesn't stay buffered too long
- **Shutdown**: Ensures all buffered data is emitted during graceful shutdown

### Additional Features

- **Thread-Safe**: Uses message passing for safe concurrent operation
- **Lightweight**: Minimal overhead implementation suitable for basic batching needs

### Use Cases

- Simple telemetry collection and forwarding
- Development and testing environments
- Scenarios where minimal processing overhead is desired
- When you need a straightforward, no-frills batching solution

### When to Consider Alternatives

- If you need advanced batching features like metadata-based routing
- When you require dynamic configuration changes at runtime
- For high-cardinality scenarios that might need more sophisticated batching logic