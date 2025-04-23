# OTLP Nodes

## BatchProcessor

A generic, async batch processor for the OpenTelemetry Arrow pipeline engine. 

Buffers incoming data messages and emits them in batches based on batch size, timer ticks, or shutdown signals.

### Features

- Buffers messages of any type (`PData`).
- Emits batches when the buffer is full, on timer ticks, or on shutdown.
- Handles pipeline control messages (`TimerTick`, `Shutdown`).
- Conforms to the `Processor` trait in `engine/src/processor.rs`.

### Usage

```rust
use otlp::batch_processor::BatchProcessor;
use engine::message::{Message, ControlMsg};
use engine::processor::{Processor, EffectHandler};

// Example instantiation with batch size 3 for a u32 payload:
let mut processor = BatchProcessor::<u32>::new(3);

// Simulate sending data messages:
let mut effect_handler = ...; // obtain from pipeline context
let _ = processor.process(Message::PData(1), &mut effect_handler).await;
let _ = processor.process(Message::PData(2), &mut effect_handler).await;
let batch = processor.process(Message::PData(3), &mut effect_handler).await;
assert_eq!(batch.unwrap().unwrap(), vec![1, 2, 3]);
```

### When are batches emitted?

- When the buffer reaches the configured batch size
- When a `TimerTick` control message is received
- When a `Shutdown` control message is received

### License

Apache-2.0
