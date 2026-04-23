# Durable Buffer Processor Telemetry

This document lists telemetry emitted directly by the `durable_buffer_processor`
module (`crates/otap/src/durable_buffer_processor/`).
It includes metric instruments registered via `DurableBufferMetrics` and log
events emitted via `otel_*` log macros.

## Metrics

All metrics are emitted under the metric-set name `otap.processor.durable_buffer`.
The effective instrument name is `otap.processor.durable_buffer.<field_name>`.

### ACK / NACK Tracking

| Metric name | Unit | Instrument | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `otap.processor.durable_buffer.bundles_acked` | `{bundle}` | Counter | Number of bundles acknowledged by downstream. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.bundles_nacked_deferred` | `{bundle}` | Counter | Number of bundles deferred for retry after a transient downstream failure. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.bundles_nacked_permanent` | `{bundle}` | Counter | Number of bundles permanently rejected by downstream (not retried). Non-zero values may indicate malformed data. | `crates/otap/src/durable_buffer_processor/mod.rs` |

### Rejected Item Counts (per signal type)

| Metric name | Unit | Instrument | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `otap.processor.durable_buffer.rejected_log_records` | `{log_record}` | Counter | Number of log records in permanently rejected bundles. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.rejected_metric_points` | `{data_point}` | Counter | Number of metric data points in permanently rejected bundles. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.rejected_spans` | `{span}` | Counter | Number of spans in permanently rejected bundles. | `crates/otap/src/durable_buffer_processor/mod.rs` |

### Consumed Item Counts (per signal type)

| Metric name | Unit | Instrument | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `otap.processor.durable_buffer.consumed_log_records` | `{log_record}` | Counter | Number of log records ingested to durable storage. For OTLP bytes, counted via a protobuf wire-format scan without full deserialization. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.consumed_metric_points` | `{data_point}` | Counter | Number of metric data points ingested to durable storage. Same counting method as above. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.consumed_spans` | `{span}` | Counter | Number of spans ingested to durable storage. Same counting method as above. | `crates/otap/src/durable_buffer_processor/mod.rs` |

### Produced Item Counts (per signal type)

| Metric name | Unit | Instrument | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `otap.processor.durable_buffer.produced_log_records` | `{log_record}` | Counter | Number of log records sent downstream. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.produced_metric_points` | `{data_point}` | Counter | Number of metric data points sent downstream. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.produced_spans` | `{span}` | Counter | Number of spans sent downstream. | `crates/otap/src/durable_buffer_processor/mod.rs` |

### Error and Backpressure

| Metric name | Unit | Instrument | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `otap.processor.durable_buffer.ingest_errors` | `{error}` | Counter | Number of ingest errors (excludes storage-backpressure rejections). | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.ingest_backpressure` | `{rejection}` | Counter | Number of ingest rejections because the storage soft cap was exceeded. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.read_errors` | `{error}` | Counter | Number of read errors (poll or bundle-conversion failures). | `crates/otap/src/durable_buffer_processor/mod.rs` |

### Storage

| Metric name | Unit | Instrument | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `otap.processor.durable_buffer.storage_bytes_used` | `By` | Gauge | Current bytes used by persistent storage (durable storage + segments). Updated on each `CollectTelemetry` tick. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.storage_bytes_cap` | `By` | Gauge | Configured per-core storage capacity cap. Updated on each `CollectTelemetry` tick. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.dropped_segments` | `{segment}` | ObserveCounter | Cumulative segments force-dropped due to `DropOldest` retention policy. Non-zero values indicate data loss. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.dropped_bundles` | `{bundle}` | ObserveCounter | Cumulative bundles lost due to force-dropped segments. Non-zero values indicate data loss. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.dropped_items` | `{item}` | ObserveCounter | Cumulative individual items (log records, data points, spans) lost due to force-dropped segments. Non-zero values indicate data loss. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.expired_bundles` | `{bundle}` | ObserveCounter | Cumulative bundles lost due to `max_age` segment expiry. Non-zero values indicate data aged out before delivery. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.expired_items` | `{item}` | ObserveCounter | Cumulative individual items lost due to `max_age` segment expiry. Non-zero values indicate data aged out before delivery. | `crates/otap/src/durable_buffer_processor/mod.rs` |

### In-flight and Retry

| Metric name | Unit | Instrument | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `otap.processor.durable_buffer.in_flight` | `{bundle}` | Gauge | Current number of bundles sent downstream but not yet ACKed/NACKed. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.retries_scheduled` | `{retry}` | Counter | Cumulative number of retry attempts scheduled after a transient NACK. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.requeued_log_records` | `{log_record}` | Counter | Cumulative log records in bundles requeued for retry after NACK. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.requeued_metric_points` | `{data_point}` | Counter | Cumulative metric data points in bundles requeued for retry after NACK. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.requeued_spans` | `{span}` | Counter | Cumulative spans in bundles requeued for retry after NACK. | `crates/otap/src/durable_buffer_processor/mod.rs` |

### Queued Item Gauges (ingested but not yet ACKed)

| Metric name | Unit | Instrument | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `otap.processor.durable_buffer.queued_log_records` | `{log_record}` | Gauge | Current log records queued in durable storage/segments awaiting downstream ACK. Seeded from existing segments on restart. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.queued_metric_points` | `{data_point}` | Gauge | Current metric data points queued in durable storage/segments awaiting downstream ACK. Seeded from existing segments on restart. | `crates/otap/src/durable_buffer_processor/mod.rs` |
| `otap.processor.durable_buffer.queued_spans` | `{span}` | Gauge | Current spans queued in durable storage/segments awaiting downstream ACK. Seeded from existing segments on restart. | `crates/otap/src/durable_buffer_processor/mod.rs` |

## Logs

All events are emitted from `crates/otap/src/durable_buffer_processor/mod.rs`.

### Engine Lifecycle

| Event name | Level | Description |
| --- | --- | --- |
| `durable_buffer.engine.init` | `info` | Engine initialization started for this core instance; reports data path, size cap, retention policy, `max_segment_open_duration`, and `max_age`. |
| `durable_buffer.engine.ready` | `info` | Engine and subscriber successfully initialized and ready to process data. |
| `durable_buffer.engine.unavailable` | `error` | Engine not available when attempting to ingest data (e.g. initialization failure); upstream is NACKed. |
| `durable_buffer.timer.started` | `debug` | Periodic poll timer started on first message processed. |
| `durable_buffer.config.update` | `debug` | Config update received via `NodeControlMsg::Config`. |

### Startup Queued-Counter Seeding

| Event name | Level | Description |
| --- | --- | --- |
| `durable_buffer.queued.seeded` | `info` | Queued-item counters (logs/metrics/traces) successfully seeded from existing durable storage segments after restart. |
| `durable_buffer.queued.seed_error` | `warn` | Failed to read bundle metadata from a durable storage segment during seed; queued gauges may under-count for that segment. |

### Ingest (Upstream -> durable storage)

| Event name | Level | Description |
| --- | --- | --- |
| `durable_buffer.ingest.backpressure` | `warn` | Storage soft cap exceeded; the upstream bundle is NACKed. Rate-limited to at most once per `WARN_RATE_LIMIT` interval. |
| `durable_buffer.ingest.failed` | `error` | Non-backpressure ingest error; the upstream bundle is NACKed. |
| `durable_buffer.otlp.adapter_failed` | `error` | `OtlpBytesAdapter` creation failed in `PassThrough` mode; the upstream bundle is NACKed with the original bytes. |
| `durable_buffer.otlp.conversion_failed` | `error` | OTLP->Arrow conversion failed in `ConvertToArrow` mode; the upstream bundle is NACKed with the original bytes. |

### Flush and Maintenance

| Event name | Level | Description |
| --- | --- | --- |
| `durable_buffer.flush.failed` | `warn` | `engine.flush()` failed during a timer tick. Rate-limited to at most once per `WARN_RATE_LIMIT` interval. |
| `durable_buffer.maintenance.failed` | `warn` | `engine.maintain()` failed during a timer tick (subscriber progress flush or segment cleanup). |

### Drain Loop (Timer Tick)

| Event name | Level | Description |
| --- | --- | --- |
| `durable_buffer.drain.budget_exhausted` | `debug` | Drain loop consumed its time budget (50% of `poll_interval`); yields back to process incoming data. |
| `durable_buffer.drain.at_capacity` | `debug` | Drain loop halted because `max_in_flight` bundles are already in-flight. |
| `durable_buffer.drain.all_blocked` | `debug` | All available bundles are either in-flight or scheduled for retry; loop exits to avoid busy-spinning. |
| `durable_buffer.drain.backpressure` | `debug` | Downstream channel is full; drain loop halted and bundle is deferred for the next tick. |
| `durable_buffer.poll.failed` | `error` | `poll_next_bundle()` returned an error during the drain loop. |

### Bundle Lifecycle

| Event name | Level | Description |
| --- | --- | --- |
| `durable_buffer.bundle.forwarded` | `debug` | Bundle successfully sent downstream; reports segment sequence, bundle index, and retry count. |
| `durable_buffer.bundle.duplicate` | `warn` | `poll_next_bundle()` returned a bundle that is already tracked as in-flight (should not occur in normal operation). |
| `durable_buffer.bundle.conversion_failed` | `error` | Failed to convert a reconstructed Quiver bundle to `OtapPdata`; bundle is rejected and counted as a read error. |
| `durable_buffer.bundle.acked` | `debug` | Bundle ACKed by downstream and cleaned up from the in-flight map. |
| `durable_buffer.bundle.nacked` | `debug` | Bundle transiently NACKed by downstream; retry scheduled with exponential backoff. |
| `durable_buffer.bundle.rejected_permanent` | `warn` | Bundle permanently NACKed by downstream; items are counted as rejected and the bundle is not retried. |
| `durable_buffer.ack.unknown_bundle` | `warn` | ACK received for a bundle that is not in the in-flight map (unexpected). |
| `durable_buffer.nack.unknown_bundle` | `warn` | NACK received for a bundle that is not in the in-flight map (unexpected). |

### Retry

| Event name | Level | Description |
| --- | --- | --- |
| `durable_buffer.retry.sent` | `debug` | Retry attempt successfully forwarded downstream. |
| `durable_buffer.retry.deferred` | `debug` | Retry deferred because `max_in_flight` is at capacity; re-scheduled with `poll_interval` delay. |
| `durable_buffer.retry.backpressure` | `debug` | Retry halted due to downstream backpressure; bundle re-scheduled with `poll_interval` delay. |
| `durable_buffer.retry.claim_failed` | `debug` | Could not re-claim bundle from Quiver for retry (bundle may have been resolved or segment dropped). |
| `durable_buffer.retry.skipped` | `warn` | Just-claimed retry bundle was unexpectedly skipped (should not occur in normal operation). |
| `durable_buffer.retry.schedule_failed` | `warn` | Failed to schedule a retry for a transiently NACKed bundle via `delay_data()`. |
| `durable_buffer.retry.reschedule_failed` | `warn` | Failed to re-schedule a deferred retry; bundle removed from `retry_scheduled` to allow poll to pick it up. |
| `durable_buffer.retry.missing_calldata` | `warn` | `DelayedData` retry ticket has no source-route calldata; discarded. |
| `durable_buffer.retry.invalid_calldata` | `warn` | `DelayedData` retry ticket calldata cannot be decoded as a retry ticket; discarded. |
| `durable_buffer.delayed_data.unexpected` | `warn` | `DelayedData` message received that is not a recognized retry ticket; discarded. |

### Shutdown

| Event name | Level | Description |
| --- | --- | --- |
| `durable_buffer.shutdown.start` | `info` | Shutdown sequence started; reports the deadline. |
| `durable_buffer.shutdown.flushing` | `info` | About to call `engine.flush()` to finalize any open segment before draining. |
| `durable_buffer.shutdown.drained` | `info` | Reports the number of bundles drained to downstream during shutdown. |
| `durable_buffer.shutdown.complete` | `info` | Engine shutdown completed successfully. |
| `durable_buffer.shutdown.deadline_exceeded` | `warn` | Shutdown deadline already passed before the flush/drain sequence; flush and drain are skipped. |
| `durable_buffer.shutdown.drain_deadline` | `warn` | Shutdown drain loop exceeded its deadline; remaining bundles are not forwarded. |
| `durable_buffer.shutdown.backpressure` | `warn` | Downstream channel full during shutdown drain; drain halted. |
| `durable_buffer.shutdown.bundle_error` | `warn` | Bundle processing error during shutdown drain; drain continues. |
| `durable_buffer.shutdown.poll_error` | `warn` | `poll_next_bundle()` error during shutdown drain; drain halted. |
| `durable_buffer.shutdown.flush_failed` | `error` | `engine.flush()` failed during shutdown (data durability is still ensured by `engine.shutdown()`). |
| `durable_buffer.shutdown.engine_failed` | `error` | `engine.shutdown()` failed; open segment may not have been finalized. |

## Maintenance

When adding or changing telemetry in this module:

1. **Metrics**
     - If you add a field to `DurableBufferMetrics` in
         `crates/otap/src/durable_buffer_processor/mod.rs`, add/update the
         corresponding row in the **Metrics** table.
     - The effective emitted name is
         `otap.processor.durable_buffer.<field_name>` (or the `name` override
         in the `#[metric(...)]` attribute if present).
     - Note the instrument type (`Counter`, `Gauge`, or `ObserveCounter`) in
         the **Instrument** column.

2. **Logs**
     - If you add `otel_info!`, `otel_warn!`, `otel_error!`, `otel_debug!`, or
         `otel_trace!` calls in this module, add/update a row in the appropriate
         subsection of the **Logs** table.
     - Keep event names exact (first macro argument), include the log level,
         and describe the condition that triggers the event.

3. **Review checklist (quick)**
     - Search for new metric fields: `#[metric(` in
         `crates/otap/src/durable_buffer_processor/mod.rs`.
     - Search for new log events: `otel_(trace|debug|info|warn|error)!(` in
         `crates/otap/src/durable_buffer_processor/**`.
     - Confirm this document still matches current source files.
