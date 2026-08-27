# Syslog CEF Receiver Telemetry

<!-- markdownlint-disable MD013 -->

This document lists telemetry emitted directly by the
`syslog_cef_receiver` component. It includes metric instruments registered
by the component and log events emitted via `otel_*` log macros.

## Metrics

The shared metrics describe the complete receiver-local lifecycle of each
classified external message. Component metrics retain protocol-specific
diagnostics.

| Metric name | Type | Unit | Description | Produced in file |
| --- | --- | --- | --- | --- |
| `receiver.received.messages` | Counter | `{message}` | Classified external messages grouped by `signal=logs` and terminal `outcome`. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.received.payload.size` | Counter | `By` | Encoded payload bytes visible before parsing, excluding the TCP newline delimiter. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.received.duration` | Histogram | `s` | Time from classified payload observation through handoff, refusal, or failure, including batch buffering. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.syslog_cef.rejections.items` | Counter | `{item}` | Rejected messages grouped by bounded `protocol` and `error.type`. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.syslog_cef.truncations.items` | Counter | `{item}` | Payloads that reached the fixed `MAX_MESSAGE_SIZE` receive limit. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.syslog_cef.transport.errors` | Counter | `{error}` | Transport-level errors grouped by `protocol`. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.syslog_cef.connections.active` | UpDownCounter | `{connection}` | Currently active TCP connections. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.syslog_cef.connections.rejected` | Counter | `{connection}` | TCP connections rejected or closed under admission pressure. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |

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
   - Add or update the corresponding row in the **Metrics** table.
   - Use shared receiver metric sets for external-boundary behavior and
     `receiver.syslog_cef.*` only for richer component diagnostics.

2. **Logs**
   - If you add `otel_trace!`, `otel_debug!`, `otel_info!`, `otel_warn!`,
     or `otel_error!`, add or update the corresponding row in the
     **Logs** table.
   - Keep the event name exact (first macro argument), include level, and
     file path.

3. **Quick review checklist**
   - Search metric sets: `#[metric_set(` in `crates/core-nodes/src/receivers/syslog_cef_receiver/*.rs`.
   - Search log events: `otel_(trace|debug|info|warn|error)!(` in `crates/core-nodes/src/receivers/syslog_cef_receiver/*.rs`.
