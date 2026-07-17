# Azure Monitor Exporter Telemetry

This document lists telemetry emitted directly by the
`azure_monitor_exporter` crate. It includes metric instruments registered
by the crate and log events emitted via `otel_*` log macros.

## Metrics

| Metric name | Description | Produced in file |
| --- | --- | --- |
| `exporter.azure_monitor.exports.items` | Number of log items (Azure Monitor rows) resolved by `signal="logs"` and `outcome` (`success` or `failure`). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `exporter.azure_monitor.exports.batches` | Number of log batches resolved by `signal="logs"` and `outcome` (`success` or `failure`). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `exporter.azure_monitor.exports.messages` | Number of log messages resolved by `signal="logs"` and `outcome` (`success` or `failure`). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `exporter.azure_monitor.http.responses` | Number of HTTP export attempts by `response` (`http_2xx`, `http_400`, `http_401`, `http_403`, `http_404`, `http_413`, `http_429`, `http_5xx`, `network_error`, or `other`). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `exporter.azure_monitor.http.latency` | HTTP export attempt latency in milliseconds (min/max/sum/count), partitioned by `response`. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `exporter.azure_monitor.batch_size` | Compressed batch size in bytes (min/max/sum/count). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `exporter.azure_monitor.batch_uncompressed_size` | Uncompressed batch size in bytes (min/max/sum/count). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `exporter.azure_monitor.in_flight_exports` | Current number of in-flight export requests. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `exporter.azure_monitor.in_flight_log_records` | Current number of log records in-flight at the exporter (enqueued export requests awaiting completion, including records being retried). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `exporter.azure_monitor.state.mappings` | Current number of exporter state-map entries, partitioned by `mapping` (`batch_to_message`, `message_to_batch`, or `message_to_data`). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `exporter.azure_monitor.log_entries_too_large` | Number of log entries rejected for exceeding batch size limit. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `exporter.azure_monitor.heartbeats.sends` | Number of heartbeat sends resolved by `outcome` (`success` or `failure`). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |

## Logs

| Event name | Level | Description | Produced in file |
| --- | --- | --- | --- |
| `azure_monitor_exporter.start` | `info` | Exporter startup with endpoint/stream/DCR context. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.export.success` | `debug` | Export completed successfully for a batch. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.export.failed` | `error` | Export failed for a batch and messages are nacked. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.export.periodic_flush` | `debug` | Periodic flush triggered for pending batch data. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.export.retry_delay` | `warn` | Retry/backoff delay selected after retryable export failure. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.client.error` | `warn` | Non-success HTTP response from Azure ingestion endpoint. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.message.log_entry_too_large` | `warn` | A transformed log entry exceeds batcher size limit. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.message.batch_push_failed` | `error` | Failed to push transformed log entry into gzip batcher. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.message.no_valid_entries` | `debug` | Incoming message produced no valid transformed log entries. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.message.unsupported_signal` | `warn` | Unsupported metrics/traces signal received by logs-only exporter. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.shutdown.orphaned_message` | `warn` | Message remained unresolved during shutdown and was nacked. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.exporter.shutdown` | `info` | Exporter shutdown completed. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.auth.credential_type` | `info` | Selected Azure authentication credential type. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/auth.rs` |
| `azure_monitor_exporter.auth.get_token_succeeded` | `debug` | Token acquisition succeeded. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/auth.rs` |
| `azure_monitor_exporter.auth.get_token_failed` | `warn` | Token acquisition attempt failed. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/auth.rs` |
| `azure_monitor_exporter.auth.retry_scheduled` | `warn` | Next token acquisition retry scheduled with backoff+jitter. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/auth.rs` |
| `azure_monitor_exporter.auth.token_refresh` | `info` | Token refresh succeeded and next refresh window was scheduled. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.auth.header_creation_failed` | `error` | Failed to build authorization header from refreshed token. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.auth.token_refresh_failed` | `error` | Token refresh operation failed. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.heartbeat.sent` | `debug` | Heartbeat request sent successfully. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.heartbeat.send_failed` | `warn` | Heartbeat request failed. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.collect` | `debug` | Debug snapshot emitted when telemetry collection is requested. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.transform.serialize_failed` | `warn` | Failed to serialize transformed log record JSON; record skipped. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/transformer.rs` |

## Maintenance

When adding or changing telemetry in this crate:

1. **Metrics**
   - If you add a field under
     `#[metric_set(name = "exporter.azure_monitor")]`, add or
     update its row in the **Metrics** table.
   - Use metric names in the form
     `exporter.azure_monitor.<field_name>` unless the field has
     an explicit metric-name override.

2. **Logs**
   - If you add `otel_trace!`, `otel_debug!`, `otel_info!`, `otel_warn!`,
     or `otel_error!`, add or update the corresponding row in the
     **Logs** table.
   - Keep the event name exact (first macro argument), include level, and
     file path.

3. **Quick review checklist**
   - Search metric sets: `#[metric_set(` in `crates/contrib-nodes/src/exporters/azure_monitor_exporter/*.rs`.
   - Search log events: `otel_(trace|debug|info|warn|error)!(` in `crates/contrib-nodes/src/exporters/azure_monitor_exporter/*.rs`.
