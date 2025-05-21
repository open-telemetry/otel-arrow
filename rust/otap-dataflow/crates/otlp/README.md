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
