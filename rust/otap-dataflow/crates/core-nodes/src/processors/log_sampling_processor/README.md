# Log Sampling Processor

URN: `urn:otel:processor:log_sampling`

## Overview

The Log Sampling processor reduces log volume by discarding a portion of
incoming log records according to a configurable sampling strategy. Non-log
signals (metrics and traces) pass through unchanged.

## Architecture

Sampling logic is encapsulated behind the `Sampler` trait (defined in
`sample/mod.rs`). Each sampler produces a `BooleanArray` selection vector via
`sample_arrow_records()`, where `true` = keep and `false` = drop.

The processor applies the selection vector to the full OTAP batch and records
metrics accordingly.

Currently implemented samplers:

- **Zip** (`sample/zip.rs`) -- emit at most N records per time window.
- **Ratio** (`sample/ratio.rs`) -- emit a fixed fraction of records.

See the module-level documentation in each sampler file for algorithm details.

## Signal Handling

| Signal  | Behavior               |
|---------|------------------------|
| Logs    | Apply sampling         |
| Metrics | Pass through unchanged |
| Traces  | Pass through unchanged |

## Configuration

The processor configuration uses an externally tagged enum on the `policy`
field. Exactly one policy must be specified.

### Zip Sampling

```yaml
config:
  policy:
    zip:
      interval: 60s
      max_items: 100
```

### Ratio Sampling

```yaml
config:
  policy:
    ratio:
      emit: 1
      out_of: 10
```

## Ack/Nack Behavior

The processor does not subscribe to downstream ack/nack interests and does
not maintain correlation state between inbound and outbound messages.

When records are kept, the processor constructs a new `OtapPdata` with the
filtered Arrow records and the original `Context`, then sends it downstream
via `send_message_with_source_node`. The original context is preserved, so
downstream acks and nacks propagate transparently to the original sender.

When all records are dropped (`to_keep == 0`) or the incoming batch is
empty, the processor immediately acks the inbound request via
`notify_ack(AckMsg::new(pdata))`.

## Telemetry

| Metric                  | Unit      | Description                        |
|-------------------------|-----------|------------------------------------|
| `log_signals_consumed`  | `{log}`   | Total log records received         |
| `log_signals_dropped`   | `{log}`   | Log records dropped by sampling    |
| `batches_fully_dropped` | `{batch}` | Batches where all records dropped  |
