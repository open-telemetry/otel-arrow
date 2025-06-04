# OTLP Nodes

## OTLP Batch Processor

### Overview

The OTLP Batch Processor is a flexible, type-safe batching component for
OpenTelemetry Protocol (OTLP) data, implemented in Rust. It efficiently batches
traces, metrics, and logs, preserving their hierarchical structure and
supporting configurable flush triggers for optimal performance and reliability.

### Key Features

- **Unified Batching**: Handles all OTLP data types (traces, metrics, logs)
  with a single, generic batching mechanism.
- **Type Safety**: Leverages Rust's type system to ensure compile-time safety
  and correctness.
- **Structure Preservation**: Maintains the original resource and scope
  hierarchy of telemetry data throughout batching.
- **Configurable Flush Triggers**:
  - **Batch Size**: Flushes when the number of items reaches a configurable
    threshold.
  - **Timeout**: Flushes after a specified duration to ensure timely delivery.
  - **Shutdown**: Ensures all buffered data is emitted during graceful
    shutdown.
- **Efficient Memory Use**: Uses move semantics to minimize allocations and
  copying.
- **Thread-Safe**: Designed for safe concurrent operation using message
  passing.

#### Sizer Modes (Planned)

Inspired by the Go OpenTelemetry Collector, future versions will support
configurable "Sizer" modes:

- **Requests**: Flush based on the number of top-level requests.
- **Items**: Flush based on the number of individual telemetry items (current
  behavior).
- **Bytes**: Flush based on the total serialized size of the batch.

### Use Cases

- High-throughput telemetry pipelines requiring efficient batching.
- Environments where data structure and integrity must be preserved.
- Systems needing configurable batching strategies for latency or throughput
  optimization.
- Mixed telemetry workloads (traces, metrics, logs) in a unified pipeline.

### Example Usage

```rust
use std::time::Duration;

let config = BatchConfig {
    send_batch_size: 100,  // Maximum number of items per batch
    timeout: Duration::from_secs(5),  // Maximum time to buffer before flushing
    // sizer: BatchSizer::Items, // (future: choose between Requests, Items, Bytes)
};
let batcher = GenericBatcher::new(config);
// Integrate `batcher` into your OTLP data pipeline
```

### Technical Highlights

- **Zero-Copy**: Moves telemetry items without unnecessary cloning.
- **Extensible**: Designed to support additional batching strategies and sizer
  modes.

---

For more details, see the source code and documentation comments.