# OTLP Nodes

## BatchProcessor

A generic, async batch processor for the OpenTelemetry Arrow pipeline engine. Buffers incoming data messages and emits them in batches based on batch size, timer ticks, timeout, or shutdown signals.

### Features
- Buffers messages of any type (`PData`).
- Emits batches based on batch size, timer ticks, timeout, or shutdown.
- Handles pipeline control messages (`TimerTick`, `Shutdown`, `Ack`, `Nack`, `Config`).
- Supports metadata-based batching.
- Conforms to the `Processor` trait in `otap_df_engine`.
- Uses `SendEffectHandler` for thread-safe message passing.

### Configuration Options
The batch processor can be configured with the following options:

- `send_batch_size`: The number of messages to buffer before sending a batch (default: 0).
- `send_batch_max_size`: The maximum allowed batch size (default: 0, no maximum).
- `timeout`: Time after which a batch will be sent regardless of size (default: 0).
- `metadata_keys`: List of metadata keys to use for distinct batching (default: empty).
- `metadata_cardinality_limit`: Maximum number of batcher instances (default: 1000).

### Usage

```rust
use otap_df_engine::message::{Message, ControlMsg};
use otap_df_engine::processor::{Processor, SendEffectHandler};
use otlp::batch_processor::{BatchProcessor, Config};
use tokio::sync::mpsc;
use std::time::Duration;

// Example instantiation with configuration:
let config = Config {
    send_batch_size: 3,
    send_batch_max_size: 10,
    timeout: Duration::from_secs(5),  // 5 second timeout
    metadata_keys: vec!["traceId".to_string()],
    metadata_cardinality_limit: 1000,
};

let mut processor = BatchProcessor::<u32>::new(config);

// Create an effect handler
let (tx, rx) = mpsc::channel(10);
let mut effect_handler = SendEffectHandler::new("test", tx);

// Simulate sending data messages:
let _ = processor.process(Message::PData(1), &mut effect_handler).await?;
let _ = processor.process(Message::PData(2), &mut effect_handler).await?;
let _ = processor.process(Message::PData(3), &mut effect_handler).await?;

// Batches will be sent when:
// - The buffer reaches batch size (3 messages)
// - The timeout (5 seconds) elapses
// - A TimerTick control message is received
// - A Shutdown control message is received
// - The metadata cardinality limit is reached

// You can also process control messages:
let _ = processor.process(
    Message::Control(ControlMsg::TimerTick {}),
    &mut effect_handler,
).await?;

let _ = processor.process(
    Message::Control(ControlMsg::Config { config: serde_json::json!({
        "send_batch_size": 5,
        "send_batch_max_size": 10,
        "timeout": 10,
        "metadata_keys": ["traceId"],
        "metadata_cardinality_limit": 1000
    }) }),
    &mut effect_handler,
).await?;
```

### When are batches emitted?
- When the buffer reaches the configured batch size
- When the configured timeout elapses
- When a `TimerTick` control message is received
- When a `Shutdown` control message is received
- When the metadata cardinality limit is reached

### License
Apache-2.0
