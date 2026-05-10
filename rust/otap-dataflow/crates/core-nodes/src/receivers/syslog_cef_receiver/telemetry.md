# Syslog CEF Receiver Telemetry

<!-- markdownlint-disable MD013 -->

This document lists telemetry emitted directly by the
`syslog_cef_receiver` component. It includes metric instruments registered
by the component and log events emitted via `otel_*` log macros.

## Metrics

Metrics are registered under the metric set name `syslog_cef.receiver`.

| Metric name | Type | Unit | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `syslog_cef.receiver.received_logs_total` | Counter | `{item}` | Total number of log records observed at the socket before parsing. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef.receiver.received_logs_forwarded` | Counter | `{item}` | Number of log records successfully forwarded downstream. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef.receiver.received_logs_invalid` | Counter | `{item}` | Number of log records rejected because their payload is zero-length. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef.receiver.received_logs_truncated` | Counter | `{item}` | Number of log records whose raw message exceeded the maximum message size and were truncated before parsing. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef.receiver.received_logs_forward_failed` | Counter | `{item}` | Number of log records refused by downstream (backpressure/unavailable). | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef.receiver.tcp_connections_active` | UpDownCounter | `{conn}` | Number of currently active TCP connections. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef.receiver.tls_handshake_failures` | Counter | `{error}` | Number of TLS handshake failures. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |

## Logs

| Event name | Level | Description | Produced in file |
| --- | --- | --- | --- |
| `syslog_cef_receiver.start` | `info` | Receiver startup with protocol (TCP or UDP) and listening address. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef_receiver.tls_enabled` | `info` | TLS has been enabled for the TCP receiver. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef_receiver.tls.handshake.success` | `debug` | TLS handshake completed successfully for an incoming connection. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef_receiver.tls.handshake.failed` | `warn` | TLS handshake failed; the connection is closed. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef_receiver.drain_ingress.timeout` | `warn` | Ingress drain timeout expired with connection tasks still active during shutdown. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `syslog_cef_receiver.arrow_records.build_failed` | `warn` | Failed to build Arrow records from a parsed batch; the batch is dropped. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |

## Maintenance

When adding or changing telemetry in this component:

1. **Metrics**
   - If you add a field under
     `#[metric_set(name = "syslog_cef.receiver")]`, add or
     update its row in the **Metrics** table.
   - Use metric names in the form
     `syslog_cef.receiver.<field_name>` unless the field has
     an explicit metric-name override.

2. **Logs**
   - If you add `otel_trace!`, `otel_debug!`, `otel_info!`, `otel_warn!`,
     or `otel_error!`, add or update the corresponding row in the
     **Logs** table.
   - Keep the event name exact (first macro argument), include level, and
     file path.

3. **Quick review checklist**
   - Search metric sets: `#[metric_set(` in `crates/core-nodes/src/receivers/syslog_cef_receiver/*.rs`.
   - Search log events: `otel_(trace|debug|info|warn|error)!(` in `crates/core-nodes/src/receivers/syslog_cef_receiver/*.rs`.
