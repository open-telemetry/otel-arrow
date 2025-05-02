# OTLP Nodes

## BatchProcessor

A generic, async batch processor for the OpenTelemetry Arrow pipeline engine. Buffers incoming data messages and emits them in batches based on batch size, timer ticks, or shutdown signals.

### Features
- Buffers messages of any type (`PData`).
- Emits batches when the buffer is full, on timer ticks, or on shutdown.
- Handles pipeline control messages (`TimerTick`, `Shutdown`, `Ack`, `Nack`, `Config`).
- Conforms to the `Processor` trait in `otap_df_engine`.
- Uses `SendEffectHandler` for thread-safe message passing.

### Usage

```rust
use otap_df_engine::message::{Message, ControlMsg};
use otap_df_engine::processor::{Processor, SendEffectHandler};
use otlp::batch_processor::BatchProcessor;
use tokio::sync::mpsc;

// Example instantiation with batch size 3 for a u32 payload:
let mut processor = BatchProcessor::<u32>::new(3);

// Create an effect handler
let (tx, rx) = mpsc::channel(10);
let mut effect_handler = SendEffectHandler::new("test", tx);

// Simulate sending data messages:
let _ = processor.process(Message::PData(1), &mut effect_handler).await?;
let _ = processor.process(Message::PData(2), &mut effect_handler).await?;
let _ = processor.process(Message::PData(3), &mut effect_handler).await?;

// The batch will be automatically sent through the effect handler when it reaches size 3
// No need to manually retrieve the batch

// You can also process control messages:
let _ = processor.process(
    Message::Control(ControlMsg::TimerTick {}),
    &mut effect_handler,
).await?;

let _ = processor.process(
    Message::Control(ControlMsg::Config { config: serde_json::json!({"batch_size": 5}) }),
    &mut effect_handler,
).await?;
```

### When are batches emitted?
- When the buffer reaches the configured batch size
- When a `TimerTick` control message is received
- When a `Shutdown` control message is received

### License
Apache-2.0
