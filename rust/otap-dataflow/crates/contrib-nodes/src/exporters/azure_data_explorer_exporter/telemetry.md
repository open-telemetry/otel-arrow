<!-- markdownlint-disable MD013 -->

# Azure Data Explorer Exporter Telemetry

This document specifies telemetry planned for the
 `azure_data_explorer_exporter` module. The non-operational skeleton does not
 currently register these instruments or emit these log events.

## Metrics

### Shared exporter boundary metrics

The exporter uses the shared node-boundary metric contract. One observation is
recorded for each physical ADX submission, including every retry. A failure
during compression or dispatch setup is recorded as a preparation-only
attempt. Attempt duration excludes retry backoff.

All shared metrics have bounded `signal` and `outcome` attributes. The
`duration`, `payload.size`, and `items` measurements are enabled by the
pipeline's telemetry-interest policy.

| Metric name | Description | Type | Produced in file |
| --- | --- | --- | --- |
| `exporter.attempted.messages` | Number of ADX attempts by signal and outcome. | Counter | `client.rs`, `exporter.rs` |
| `exporter.attempted.duration` | Duration in seconds of each ADX attempt. Initial attempts include compression and HTTP work; retries exclude backoff and reuse the encoded payload. Coalescing wait and pre-batch transformation are excluded. | Exponential histogram | `client.rs`, `exporter.rs` |
| `exporter.attempted.payload.size` | Encoded payload bytes produced or submitted by each attempt. | Counter | `client.rs`, `exporter.rs` |
| `exporter.attempted.items` | Source OTLP signal items represented by each attempt. Expanded ADX rows do not increase this count. | Counter | `client.rs`, `exporter.rs` |

These metrics are not input-message counters. `node.input.messages` describes
PData entering the exporter, while `exporter.attempted.messages` describes
physical submissions and therefore counts retries separately.
An accepted input that produces no ADX rows records one successful no-op
attempt before ACK.

HTTP error bodies are retained only up to 64 KiB. The `client.http_failure`
event records whether the body was truncated. Retry backoff honors
server-provided `Retry-After` values within the operation deadline.
An HTTP request canceled by the operation deadline is classified as
`response=network_error` so HTTP-attempt totals remain aligned with shared
exporter-attempt metrics.

Malformed histograms and missing or non-finite numeric metric values are
permanently refused before ADX row emission. When a valid input contains no
signal records, the successful no-op attempt remains observable through
`exporter.attempted.*`; no payload content is logged.
Generated histogram bounds and summary quantiles use string dimensions that
match the Go exporter, including `+Inf` and `-Inf`.

### ADX-specific diagnostic metrics

Only diagnostics not represented by the shared attempt contract remain
ADX-specific. Logical batches are counted once at their terminal outcome, so
retries do not duplicate them. HTTP attempts use one bounded response
dimension instead of separate instruments for each status category.

| Metric name | Description | Type | Produced in file |
| --- | --- | --- | --- |
| `exporter.azure_data_explorer.batches` | Terminal logical batches by `signal` and `outcome`. | Counter | `exporter.rs` |
| `exporter.azure_data_explorer.http.responses` | HTTP attempts by bounded `response` category. | Counter | `client.rs` |

Token acquisition and refresh telemetry is emitted by the bound authentication
provider. With `azure_identity_auth`, these instruments use scope
`extension.azure_identity_auth`. The Microsoft JSON performance profile
continues to rename its `auth.failures` instrument to
`exporter_azure_data_explorer_auth_failures` for schema continuity.

## Logs

| Event name | Level | Description | Produced in file |
| --- | --- | --- | --- |
| `azure_data_explorer_exporter.start` | `info` | Exporter startup with cluster/database/table context. | `exporter.rs` |
| `azure_data_explorer_exporter.shutdown` | `info` | Exporter shutdown completed. | `exporter.rs` |
| `azure_data_explorer_exporter.export.failed` | `warn` | Export failed for a batch and messages are nacked; sensitive payload and response content is never included. | `exporter.rs` |
| `azure_data_explorer_exporter.export.retained_after_unauthorized` | `warn` | A coalesced request is retained until a replacement token is available. | `exporter.rs` |
| `azure_data_explorer_exporter.export.sample_row` | `debug` | A bounded sample row is logged only when `log_failed_payload` is enabled for debugging. | `exporter.rs` |
| `azure_data_explorer_exporter.export.failed_payload` | `debug` | The complete failed payload is logged only when `log_failed_payload` is enabled for debugging. | `exporter.rs` |
| `azure_data_explorer_exporter.message.no_records` | `debug` | An incoming message produced no rows. | `exporter.rs` |
| `azure_data_explorer_exporter.extraction_failed` | `warn` | The exporter could not construct a signal view from the input payload. | `exporter.rs` |
| `azure_data_explorer_exporter.payload_refused` | `warn` | A malformed signal payload was permanently refused. | `exporter.rs` |
| `azure_data_explorer_exporter.invalid_metric_data` | `warn` | Invalid numeric or histogram metric data was permanently refused. | `exporter.rs` |
| `azure_data_explorer_exporter.row_too_large` | `warn` | A serialized row exceeded `max_row_bytes`; records `actual_row_bytes` and `max_row_bytes`. | `exporter.rs` |
| `azure_data_explorer_exporter.request_too_large` | `warn` | A request exceeded its row or byte budget; records the corresponding actual and maximum fields. | `exporter.rs` |
| `azure_data_explorer_exporter.transformation_limit_exceeded` | `warn` | OTLP value nesting or traversal work exceeded a transformation limit; records `limit_kind`, `actual`, and `limit`. | `exporter.rs` |
| `azure_data_explorer_exporter.batch.timer_flush` | `debug` | A partial coalesced request reached its flush interval. | `exporter.rs` |
| `azure_data_explorer_exporter.client_pool_exhausted` | `error` | The bounded client pool was unexpectedly empty at dispatch. | `exporter.rs` |
| `azure_data_explorer_exporter.compression_failed` | `error` | Gzip compression failed for a batch. | `exporter.rs` |
| `azure_data_explorer_exporter.auth.invalid_bearer_token` | `warn` | A provider token could not be converted to an HTTP authorization header. | `exporter.rs` |
| `azure_data_explorer_exporter.auth.token_stream_closed` | `warn` | The provider closed its token stream. If no usable token remains, pending and subsequent data receive retryable NACKs. | `exporter.rs` |
| `azure_data_explorer_exporter.client.network_error` | `warn` | Network error during export; will retry. | `client.rs` |
| `azure_data_explorer_exporter.client.rate_limited` | `warn` | HTTP 429 received; will retry with backoff. | `client.rs` |
| `azure_data_explorer_exporter.client.server_error` | `warn` | HTTP 5xx received; will retry with backoff. | `client.rs` |
| `azure_data_explorer_exporter.client.http_failure` | `warn` | ADX returned a non-success response; records status, bounded response-size information, and truncation without response content. | `client.rs` |
| `azure_data_explorer_exporter.client.response_detail` | `debug` | A bounded chunk of the ADX error response is logged only when `log_response_body` is enabled for debugging. | `client.rs` |
| `azure_data_explorer_exporter.client.conflict` | `warn` | ADX returned HTTP 409; this event never includes the response body. | `client.rs` |
| `azure_data_explorer_exporter.client.backoff` | `debug` | Retry backoff delay selected. | `client.rs` |
| `azure_data_explorer_exporter.shutdown.deadline_elapsed` | `warn` | The shutdown deadline elapsed; unresolved requests were canceled, recorded as failures, and abandoned when completion routing could not finish by the deadline. | `exporter.rs` |

## Sensitive debugging logs

`log_failed_payload` and `log_response_body` are independent, disabled by
default, and intended only for temporarily diagnosing failed or rejected ADX
exports:

- `log_failed_payload` enables the debug-level `export.sample_row` and
  `export.failed_payload` events containing outbound telemetry.
- `log_response_body` enables the debug-level `client.response_detail` events
  containing inbound ADX error response chunks.

Neither option is recommended in production unless strictly necessary.
Payloads and service responses may contain PII, schema values, telemetry
attributes, or other customer-sensitive data. Successful exports never emit
these events. Warning-level failure events remain payload- and response-free,
even when either option is enabled. Disable the options immediately after
diagnosis and protect any collected debug logs appropriately.

## Maintenance

When adding or changing telemetry in this module:

1. **Metrics** - If you add a field under
   `#[metric_set(name = "exporter.azure_data_explorer")]`, add or update its
   row in the **Metrics** table.

2. **Logs** - If you add `otel_trace!`, `otel_debug!`, `otel_info!`,
   `otel_warn!`, or `otel_error!`, add or update the corresponding row
   in the **Logs** table.

3. **Quick review checklist**
   - Search metric sets: `#[metric_set(` in `azure_data_explorer_exporter/*.rs`
   - Search log events: `otel_(trace|debug|info|warn|error)!(` in
     `azure_data_explorer_exporter/*.rs`
