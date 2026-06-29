# ETW Receiver Telemetry

<!-- markdownlint-disable MD013 -->

This document lists telemetry emitted directly by the `etw_receiver`
component. It includes metric instruments registered by the component and log
events emitted via `otel_*` log macros.

## Metrics

Metrics are registered under the metric set name `receiver.etw`.

| Metric name | Type | Unit | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `receiver.etw.received_events_observed` | Counter | `{event}` | Total number of ETW events the session produced, counted on the `ProcessTrace` producer side before attempting the per-core channel send. Includes events that are immediately dropped because the internal queue is full. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |
| `receiver.etw.received_events_total` | Counter | `{event}` | Number of ETW events the async receiver loop actually pulled off the channel. This is the consumer-side denominator. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |
| `receiver.etw.received_events_forwarded` | Counter | `{event}` | Number of ETW events included in batches successfully forwarded downstream. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |
| `receiver.etw.received_events_invalid` | Counter | `{event}` | Number of ETW events whose TDH decode failed. The event is still forwarded with empty decoded fields, so this is a quality signal rather than a drop bucket. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |
| `receiver.etw.received_events_forward_failed` | Counter | `{event}` | Number of ETW events lost because their batch failed to build or could not be sent downstream. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |
| `receiver.etw.received_events_rejected_memory_pressure` | Counter | `{event}` | Number of ETW events dropped due to process-wide memory pressure. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |
| `receiver.etw.received_events_dropped_queue_full` | Counter | `{event}` | Number of ETW events dropped because the internal per-core queue was full before the async receiver loop could accept them. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |

## Counter algebra

There are two distinct denominators, and they are not the same number:

```text
received_events_observed
    = received_events_total                 (events pulled off the channel)
    + received_events_dropped_queue_full    (events the producer dropped, never enqueued)

received_events_total
    = received_events_forwarded             (events in successfully-sent batches)
    + received_events_forward_failed        (events in batches that failed to build/send)
    + events still buffered in the in-flight builder at snapshot time
```

Derived rates:

- queue-full drop rate =
  `received_events_dropped_queue_full / received_events_observed`
- forward-failure rate =
  `received_events_forward_failed / received_events_total`

`received_events_invalid` is orthogonal to the delivery accounting: a decode
failure does not drop the event, so it is not subtracted from
`received_events_total`.

## Aggregation notes

- `received_events_observed`, `received_events_dropped_queue_full`,
  and `received_events_invalid` are session-scoped across all per-core
  receivers sharing a `session_name`.
- On each telemetry collection tick, the first core to drain the shared
  atomics claims the whole delta via `swap(0)`, preventing double-counting.
- The telemetry registry then sums the per-core snapshots into one exact
  session-wide series.
- These counters intentionally do not carry a per-core attribute. Per-core
  detail lives in the rate-limited `etw.event.dropped` log, which carries a
  `core` field.
- A residual delta produced after the last surviving core takes its terminal
  snapshot is acceptably dropped at teardown.

## Logs

| Event name | Level | Description | Produced in file |
| --- | --- | --- | --- |
| `etw_receiver.start` | `info` | Receiver startup with session name, provider count, and batching configuration. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |
| `etw_receiver.forward_failed` | `warn` | Failed to forward an ETW Arrow batch downstream; the batch is dropped and counted as a forward failure. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |
| `etw_receiver.shutdown` | `info` | Receiver shutdown requested through the control plane. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |
| `etw_receiver.session_ended` | `info` | The ETW session event channel closed and the receiver finished draining any remaining buffered batch. | `crates/contrib-nodes/src/receivers/etw_receiver/mod.rs` |

## Maintenance

When adding or changing telemetry in this component:

1. **Metrics**
   - If you add a field under `#[metric_set(name = "receiver.etw")]`, add or
     update its row in the **Metrics** table.
   - Use metric names in the form `receiver.etw.<field_name>` unless the field
     has an explicit metric-name override.

2. **Logs**
   - If you add `otel_trace!`, `otel_debug!`, `otel_info!`, `otel_warn!`, or
     `otel_error!`, add or update the corresponding row in the **Logs** table.
   - Keep the event name exact, including punctuation and underscores.

3. **Quick review checklist**
   - Search metric sets: `#[metric_set(` in
     `crates/contrib-nodes/src/receivers/etw_receiver/*.rs`.
   - Search log events: `otel_(trace|debug|info|warn|error)!(` in
     `crates/contrib-nodes/src/receivers/etw_receiver/*.rs`.

<!-- markdownlint-enable MD013 -->
