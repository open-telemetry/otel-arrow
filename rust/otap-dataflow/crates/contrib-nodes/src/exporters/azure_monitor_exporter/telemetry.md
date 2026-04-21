# Azure Monitor Exporter Telemetry

This document lists telemetry emitted directly by the
`azure_monitor_exporter` crate. It includes metric instruments registered
by the crate and log events emitted via `otel_*` log macros.

## Metrics

| Metric name | Description | Produced in file |
| --- | --- | --- |
| `azure_monitor_exporter.metrics.successful_rows` | Number of rows successfully exported. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.successful_batches` | Number of batches successfully exported. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.successful_messages` | Number of messages successfully exported. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.failed_rows` | Number of rows that failed to export. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.failed_batches` | Number of batches that failed to export. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.failed_messages` | Number of messages that failed to export. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.laclient_http_success_latency` | HTTP client success latency in milliseconds (min/max/sum/count). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.metrics.laclient_http_2xx` | Number of HTTP 2xx responses. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.metrics.laclient_http_401` | Number of HTTP 401 responses. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.metrics.laclient_http_403` | Number of HTTP 403 responses. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.metrics.laclient_http_413` | Number of HTTP 413 responses. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.metrics.laclient_http_429` | Number of HTTP 429 responses. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.metrics.laclient_http_5xx` | Number of HTTP 5xx responses. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.metrics.auth_failures` | Number of failed authentication attempts. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/auth.rs` |
| `azure_monitor_exporter.metrics.auth_success_latency` | Authentication success latency in milliseconds (min/max/sum/count). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/auth.rs` |
| `azure_monitor_exporter.metrics.batch_size` | Compressed batch size in bytes (min/max/sum/count). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/client.rs` |
| `azure_monitor_exporter.metrics.batch_uncompressed_size` | Uncompressed batch size in bytes (min/max/sum/count). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.in_flight_exports` | Current number of in-flight export requests. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.batch_to_msg_count` | Current number of batch-to-message mappings. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.msg_to_batch_count` | Current number of message-to-batch mappings. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.msg_to_data_count` | Current number of message-to-data mappings. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.log_entries_too_large` | Number of log entries rejected for exceeding batch size limit. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |
| `azure_monitor_exporter.metrics.heartbeats` | Number of heartbeat sends attempted/successful. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs` |

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
     `#[metric_set(name = "azure_monitor_exporter.metrics")]`, add or
     update its row in the **Metrics** table.
   - Use metric names in the form
     `azure_monitor_exporter.metrics.<field_name>` unless the field has
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
