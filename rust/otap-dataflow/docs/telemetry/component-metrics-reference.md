# Component metrics reference

<!-- markdownlint-disable MD013 -->

This document lists component-level metric instruments declared across receiver,
processor, exporter, and extension crates in `rust/otap-dataflow/crates/`.
Core engine, runtime, channel, node, flow, and shared boundary metrics are
documented in [`crates/engine/telemetry.md`](../../crates/engine/telemetry.md)
and [`metrics-guide.md`](metrics-guide.md).

The table below is synchronized from Rust `#[metric_set]` and `#[attribute_set]`
declarations via `cargo xtask telemetry-schema --update` and verified in CI by
`cargo xtask telemetry-schema --check`.

<!-- BEGIN GENERATED: telemetry-metrics scope=components -->
| Metric name | Instrument | Unit | Attributes | Description | Source |
| --- | --- | --- | --- | --- | --- |
| `exporter.azure_data_explorer.batches` | Counter | `{batch}` | `signal`, `outcome` | Number of logical ADX batches reaching a terminal outcome. | `crates/contrib-nodes/src/exporters/azure_data_explorer_exporter/metrics.rs` |
| `exporter.azure_data_explorer.http.responses` | Counter | `{response}` | `response` | Number of HTTP attempts by response category. | `crates/contrib-nodes/src/exporters/azure_data_explorer_exporter/metrics.rs` |
| `exporter.azure_monitor.batch_size` | Histogram | `By` | `signal`, `outcome` | Compressed batch size in bytes. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.azure_monitor.batch_uncompressed_size` | Histogram | `By` | `signal`, `outcome` | Uncompressed batch size in bytes. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.azure_monitor.batches` | Counter | `{batch}` | `signal`, `outcome` | Number of completed batches. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.azure_monitor.heartbeats.sends` | Counter | `{heartbeat}` | `outcome` | Number of completed heartbeat send attempts. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.azure_monitor.http.latency` | Histogram | `ms` | `response` | HTTP export attempt latency in milliseconds (min/max/sum/count). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.azure_monitor.http.responses` | Counter | `{response}` | `response` | Number of HTTP export attempts by response category. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.azure_monitor.in_flight_exports` | Gauge | `{export}` | None | Current number of in-flight export requests. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.azure_monitor.in_flight_log_records` | Gauge | `{log}` | None | Current number of log records in-flight at the exporter (enqueued export requests awaiting completion, including records being retried). | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.azure_monitor.log_entries_too_large` | Counter | `{entry}` | None | Number of log entries rejected for exceeding the batch size limit. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.azure_monitor.state.mappings` | Gauge | `{entry}` | `mapping` | Current number of entries in the exporter state mapping. | `crates/contrib-nodes/src/exporters/azure_monitor_exporter/metrics.rs` |
| `exporter.clickhouse.batches.transformed` | Counter | `{batch}` | `path` | Total number of log batches transformed by the given path. | `crates/contrib-nodes/src/exporters/clickhouse_exporter/metrics.rs` |
| `exporter.clickhouse.rows.written` | Counter | `{row}` | `signal` | Total number of rows written successfully into clickhouse | `crates/contrib-nodes/src/exporters/clickhouse_exporter/metrics.rs` |
| `exporter.console.failures.messages` | Counter | `{message}` | `format`, `signal`, `error.type` | Number of PData messages that failed for the classified reason. | `crates/core-nodes/src/exporters/console_exporter/metrics.rs` |
| `exporter.file.bytes` | Counter | `By` | `signal` | Successfully written frame bytes, including newline delimiters. | `crates/core-nodes/src/exporters/file_exporter/metrics.rs` |
| `exporter.file.exports.messages` | Counter | `{message}` | `signal`, `outcome` | Telemetry messages whose export reached a terminal outcome. | `crates/core-nodes/src/exporters/file_exporter/metrics.rs` |
| `exporter.file.failures` | Counter | `{failure}` | `signal`, `operation` | Open, write, sync, or rollback failures. | `crates/core-nodes/src/exporters/file_exporter/metrics.rs` |
| `exporter.file.items` | Counter | `{item}` | `signal` | Signal items in frames successfully written before ACK routing. | `crates/core-nodes/src/exporters/file_exporter/metrics.rs` |
| `exporter.file.tail.recovered.bytes` | Counter | `By` | `signal` | Bytes removed by successful append-tail recovery. | `crates/core-nodes/src/exporters/file_exporter/metrics.rs` |
| `exporter.file.tail.recoveries` | Counter | `{recovery}` | `signal` | Append-mode partial tails successfully repaired. | `crates/core-nodes/src/exporters/file_exporter/metrics.rs` |
| `exporter.geneva.encoding.duration` | Histogram | `s` | `signal`, `outcome` | Time spent encoding and compressing signal data. | `crates/contrib-nodes/src/exporters/geneva_exporter/metrics.rs` |
| `exporter.geneva.failures.messages` | Counter | `{message}` | `signal`, `error.type` | Number of failed Geneva processing operations. | `crates/contrib-nodes/src/exporters/geneva_exporter/metrics.rs` |
| `exporter.geneva.skipped.messages` | Counter | `{message}` | `signal`, `reason` | Number of skipped PData messages. | `crates/contrib-nodes/src/exporters/geneva_exporter/metrics.rs` |
| `exporter.kafka.exports.bytes` | Histogram | `By` | `signal`, `outcome` | Encoded Kafka payload bytes for attempts that reached the producer. | `crates/contrib-nodes/src/exporters/kafka_exporter/metrics.rs` |
| `exporter.kafka.failures.messages` | Counter | `{message}` | `signal`, `error.type` | Number of failed export attempts. | `crates/contrib-nodes/src/exporters/kafka_exporter/metrics.rs` |
| `exporter.kafka.operations.duration` | Histogram | `s` | `signal`, `operation`, `outcome` | Time spent in the selected exporter phase. | `crates/contrib-nodes/src/exporters/kafka_exporter/metrics.rs` |
| `exporter.kafka.routing.messages` | Counter | `{message}` | `signal`, `topic.source` | Number of messages routed using the selected bounded source. | `crates/contrib-nodes/src/exporters/kafka_exporter/metrics.rs` |
| `exporter.otap.failures.messages` | Counter | `{message}` | `signal`, `error.type` | Number of PData messages that failed for the classified error type. | `crates/core-nodes/src/exporters/otap_exporter/metrics.rs` |
| `exporter.otap.streams.correlation.depth` | Histogram | `{batch}` | `signal` | Occupancy of the response correlation queue before enqueueing a yielded batch. | `crates/core-nodes/src/exporters/otap_exporter/mod.rs` |
| `exporter.otap.streams.correlation.enqueue.duration` | Histogram | `s` | `signal` | Time spent enqueueing a yielded batch into the response correlation queue. | `crates/core-nodes/src/exporters/otap_exporter/mod.rs` |
| `exporter.otap.streams.encode.duration` | Histogram | `s` | `signal` | Time spent encoding an OTAP batch into outbound Arrow batch records. | `crates/core-nodes/src/exporters/otap_exporter/mod.rs` |
| `exporter.otap.streams.enqueue.depth` | Histogram | `{batch}` | `signal` | Occupancy of the per-signal stream task queue before enqueueing a batch. | `crates/core-nodes/src/exporters/otap_exporter/mod.rs` |
| `exporter.otap.streams.enqueue.duration` | Histogram | `s` | `signal` | Time spent waiting to enqueue a batch into the per-signal stream task. | `crates/core-nodes/src/exporters/otap_exporter/mod.rs` |
| `exporter.otap.streams.response.active` | Histogram | `{batch}` | `signal` | Number of yielded batches actively awaiting a matching server response. | `crates/core-nodes/src/exporters/otap_exporter/mod.rs` |
| `exporter.otap.streams.response.wait.duration` | Histogram | `s` | `signal` | Time spent waiting for the next server response on an OTAP stream. | `crates/core-nodes/src/exporters/otap_exporter/mod.rs` |
| `exporter.otlp_grpc.authentication.ready` | UpDownCounter | `{1}` | `source` | Whether authenticated progress is currently possible (0=false, 1=true). | `crates/core-nodes/src/exporters/otlp_grpc_exporter/metrics.rs` |
| `exporter.otlp_grpc.failures.messages` | Counter | `{message}` | `signal`, `error.type` | Number of PData messages that failed for the classified error type. | `crates/core-nodes/src/exporters/otlp_grpc_exporter/metrics.rs` |
| `exporter.otlp_http.authentication.ready` | UpDownCounter | `{1}` | `source` | Whether authenticated progress is currently possible (0=false, 1=true). | `crates/core-nodes/src/exporters/otlp_http_exporter/metrics.rs` |
| `exporter.otlp_http.failures.messages` | Counter | `{message}` | `signal`, `error.type` | Number of PData messages that failed for the classified error type. | `crates/core-nodes/src/exporters/otlp_http_exporter/metrics.rs` |
| `exporter.parquet.file.operations` | Counter | `{operation}` | `operation` | Number of Parquet file lifecycle and flush operations. | `crates/core-nodes/src/exporters/parquet_exporter/metrics.rs` |
| `exporter.parquet.rows.written` | Counter | `{row}` | None | Total number of rows written into Parquet writers (appended, not necessarily flushed yet). | `crates/core-nodes/src/exporters/parquet_exporter/metrics.rs` |
| `exporter.topic.rejections.messages` | Counter | `{message}` | `signal`, `reason` | Number of topic publishes refused for the classified reason. | `crates/core-nodes/src/exporters/topic_exporter/metrics.rs` |
| `exporter.topic.tracked.in.flight` | Gauge | `{message}` | None | Number of admitted tracked publishes waiting for a terminal result. | `crates/core-nodes/src/exporters/topic_exporter/metrics.rs` |
| `exporter.topic.tracked.messages` | Counter | `{message}` | `signal`, `result` | Number of admitted tracked publishes reaching the classified result. | `crates/core-nodes/src/exporters/topic_exporter/metrics.rs` |
| `exporter.validation.check.failed` | Counter | `{check}` | None | Number of validation checks that did not match expectation | `crates/validation/src/validation_exporter.rs` |
| `exporter.validation.check.passed` | Counter | `{check}` | None | Number of validation checks that did match expectation | `crates/validation/src/validation_exporter.rs` |
| `exporter.validation.finished` | Gauge | `{state}` | None | Whether the exporter has finished processing 0 -> still receiving / processing 1 -> idle timeout reached, final validation performed | `crates/validation/src/validation_exporter.rs` |
| `exporter.validation.valid` | Gauge | `{input}` | None | The value of the last comparison result 0 -> not valid 1 -> valid | `crates/validation/src/validation_exporter.rs` |
| `extension.azure_identity_auth.auth.failures` | Counter | `{acquisition}` | None | Number of failed credential acquisitions. | `crates/contrib-extensions/src/azure_identity_auth/metrics.rs` |
| `extension.azure_identity_auth.auth.success.latency` | Histogram | `ms` | None | Latency of successful acquisitions in milliseconds (min/max/sum/count). | `crates/contrib-extensions/src/azure_identity_auth/metrics.rs` |
| `extension.azure_identity_auth.auth.successes` | Counter | `{acquisition}` | None | Number of successful credential acquisitions. | `crates/contrib-extensions/src/azure_identity_auth/metrics.rs` |
| `extension.azure_identity_auth.auth.token.publish` | Counter | `{token}` | None | Number of tokens published to consumers via the watch channel. | `crates/contrib-extensions/src/azure_identity_auth/metrics.rs` |
| `extension.azure_key_vault_sasl_auth.acquisition.failures` | Counter | `{acquisition}` | None | Failed complete acquisitions, with no partial credential publication. | `crates/contrib-extensions/src/azure_key_vault_sasl_auth/metrics.rs` |
| `extension.azure_key_vault_sasl_auth.acquisition.success.latency` | Histogram | `ms` | None | Successful complete acquisition duration. | `crates/contrib-extensions/src/azure_key_vault_sasl_auth/metrics.rs` |
| `extension.azure_key_vault_sasl_auth.acquisition.successes` | Counter | `{acquisition}` | None | Successful complete username/password acquisitions. | `crates/contrib-extensions/src/azure_key_vault_sasl_auth/metrics.rs` |
| `extension.azure_key_vault_sasl_auth.credential.publishes` | Counter | `{credential}` | None | Complete credentials published to consumers. | `crates/contrib-extensions/src/azure_key_vault_sasl_auth/metrics.rs` |
| `extension.azure_key_vault_user_pass_auth.acquisition.failures` | Counter | `{acquisition}` | None | Failed acquisitions, without partial credential publication. | `crates/contrib-extensions/src/azure_key_vault_user_pass_auth/metrics.rs` |
| `extension.azure_key_vault_user_pass_auth.acquisition.success.latency` | Histogram | `ms` | None | Successful complete acquisition duration. | `crates/contrib-extensions/src/azure_key_vault_user_pass_auth/metrics.rs` |
| `extension.azure_key_vault_user_pass_auth.acquisition.successes` | Counter | `{acquisition}` | None | Successful complete username/password acquisitions. | `crates/contrib-extensions/src/azure_key_vault_user_pass_auth/metrics.rs` |
| `extension.azure_key_vault_user_pass_auth.credential.publishes` | Counter | `{credential}` | None | Complete credentials published to consumers. | `crates/contrib-extensions/src/azure_key_vault_user_pass_auth/metrics.rs` |
| `extension.flat_file_api_key_auth.auth.failures` | Counter | `{acquisition}` | None | Number of failed API key acquisitions. | `crates/contrib-extensions/src/flat_file_api_key_auth/metrics.rs` |
| `extension.flat_file_api_key_auth.auth.publishes` | Counter | `{credential}` | None | Number of API keys published to consumers. | `crates/contrib-extensions/src/flat_file_api_key_auth/metrics.rs` |
| `extension.flat_file_api_key_auth.auth.success.latency` | Histogram | `ms` | None | Latency of successful acquisitions in milliseconds. | `crates/contrib-extensions/src/flat_file_api_key_auth/metrics.rs` |
| `extension.flat_file_api_key_auth.auth.successes` | Counter | `{acquisition}` | None | Number of successful API key acquisitions. | `crates/contrib-extensions/src/flat_file_api_key_auth/metrics.rs` |
| `extension.flat_file_sasl_auth.auth.failures` | Counter | `{acquisition}` | None | Number of failed credential acquisitions. | `crates/contrib-extensions/src/flat_file_sasl_auth/metrics.rs` |
| `extension.flat_file_sasl_auth.auth.publishes` | Counter | `{credential}` | None | Number of credentials published to consumers via the watch channel. | `crates/contrib-extensions/src/flat_file_sasl_auth/metrics.rs` |
| `extension.flat_file_sasl_auth.auth.success.latency` | Histogram | `ms` | None | Latency of successful acquisitions in milliseconds (min/max/sum/count). | `crates/contrib-extensions/src/flat_file_sasl_auth/metrics.rs` |
| `extension.flat_file_sasl_auth.auth.successes` | Counter | `{acquisition}` | None | Number of successful credential acquisitions. | `crates/contrib-extensions/src/flat_file_sasl_auth/metrics.rs` |
| `extension.flat_file_user_pass_auth.auth.failures` | Counter | `{acquisition}` | None | Number of failed credential acquisitions. | `crates/contrib-extensions/src/flat_file_user_pass_auth/metrics.rs` |
| `extension.flat_file_user_pass_auth.auth.publishes` | Counter | `{credential}` | None | Number of credentials published to consumers via the watch channel. | `crates/contrib-extensions/src/flat_file_user_pass_auth/metrics.rs` |
| `extension.flat_file_user_pass_auth.auth.success.latency` | Histogram | `ms` | None | Latency of successful acquisitions in milliseconds (min/max/sum/count). | `crates/contrib-extensions/src/flat_file_user_pass_auth/metrics.rs` |
| `extension.flat_file_user_pass_auth.auth.successes` | Counter | `{acquisition}` | None | Number of successful credential acquisitions. | `crates/contrib-extensions/src/flat_file_user_pass_auth/metrics.rs` |
| `extension.gcp_auth.auth.failures` | Counter | `{acquisition}` | None | Number of failed token acquisitions. | `crates/contrib-extensions/src/gcp_auth/metrics.rs` |
| `extension.gcp_auth.auth.success.latency` | Histogram | `ms` | None | Latency of successful acquisitions in milliseconds (min/max/sum/count). | `crates/contrib-extensions/src/gcp_auth/metrics.rs` |
| `extension.gcp_auth.auth.successes` | Counter | `{acquisition}` | None | Number of successful token acquisitions. | `crates/contrib-extensions/src/gcp_auth/metrics.rs` |
| `extension.gcp_auth.auth.token.publish` | Counter | `{token}` | None | Number of tokens published to consumers via the watch channel. | `crates/contrib-extensions/src/gcp_auth/metrics.rs` |
| `extension.oauth2_client_auth.auth.failures` | Counter | `{acquisition}` | None | Number of failed token acquisitions. | `crates/contrib-extensions/src/oauth2_client_auth/metrics.rs` |
| `extension.oauth2_client_auth.auth.success.latency` | Histogram | `ms` | None | Latency of successful acquisitions in milliseconds (min/max/sum/count). | `crates/contrib-extensions/src/oauth2_client_auth/metrics.rs` |
| `extension.oauth2_client_auth.auth.successes` | Counter | `{acquisition}` | None | Number of successful token acquisitions. | `crates/contrib-extensions/src/oauth2_client_auth/metrics.rs` |
| `extension.oauth2_client_auth.auth.token.publish` | Counter | `{token}` | None | Number of tokens published to consumers via the watch channel. | `crates/contrib-extensions/src/oauth2_client_auth/metrics.rs` |
| `otap.processor.batch.batching.errors` | Counter | `{error}` | None | Number of batches for which errors encountered | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.dropped.conversion` | Counter | `{msg}` | None | Number of messages dropped due to conversion failures | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.flush.age.duration` | Histogram | `ns` | None | Time from first pending input arrival to actual flush start | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.flush.output.batches` | Histogram | `{batch}` | None | Number of output batches successfully handed downstream by each flush | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.flush.output.bytes` | Histogram | `By` | None | Final-representation bytes successfully handed downstream by each flush, when all output sizes are known; excludes transport compression and framing | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.flush.pending.bytes` | Histogram | `By` | None | Number of bytes pending at flush time when byte size is known | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.flush.pending.requests` | Histogram | `{request}` | None | Number of input requests pending at flush time | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.flush.timer.lateness.duration` | Histogram | `ns` | None | Delay between scheduled timer wakeup and actual timer flush start | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.flushes.size` | Counter | `{flush}` | None | Number of flushes triggered by size threshold (all signals) | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.flushes.timer` | Counter | `{flush}` | None | Number of flushes triggered by timer (all signals) | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.nacked.inbound.slots` | Counter | `{msg}` | None | Number of requests nacked due to inbound slot exhaustion | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.nacked.outbound.slots` | Counter | `{msg}` | None | Number of requests nacked due to inbound slot exhaustion | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.passthrough.batches` | Counter | `{batch}` | None | Number of inputs forwarded as-is, without re-batching or completion tracking, because their size was already within the configured [min_size, max_size] range on arrival. | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `otap.processor.batch.split.budget.fallbacks` | Counter | `{entry}` | None | Number of oversize resource entries emitted whole (best-effort) because splitting them would have exceeded the configured fragment budget. | `crates/core-nodes/src/processors/batch_processor/mod.rs` |
| `processor.attributes.modified.entries` | Counter | `{attr}` | `action`, `domain` | Metric `processor.attributes.modified.entries`. | `crates/core-nodes/src/processors/attributes_processor/metrics.rs` |
| `processor.attributes.transforms` | Counter | `{transform}` | `outcome` | Metric `processor.attributes.transforms`. | `crates/core-nodes/src/processors/attributes_processor/metrics.rs` |
| `processor.content_router.messages` | Counter | `{message}` | `outcome`, `reason` | Number of messages with this outcome. | `crates/core-nodes/src/processors/content_router/mod.rs` |
| `processor.debug.consumed.events` | Counter | `{event}` | `signal` | Named log events for logs, or span events for traces | `crates/core-nodes/src/processors/debug_processor/metrics.rs` |
| `processor.debug.consumed.links` | Counter | `{link}` | `signal` | Span links | `crates/core-nodes/src/processors/debug_processor/metrics.rs` |
| `processor.durable_buffer.bundles.resolved` | Counter | `{bundle}` | `outcome` | Number of bundles resolved by downstream outcome. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.flush.failures` | Counter | `{error}` | None | Number of segment finalization failures. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.in.flight` | Gauge | `{bundle}` | None | Current number of bundles in flight to downstream. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.ingest.failures` | Counter | `{failure}` | `failure` | Number of failed ingest attempts. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.items.consumed` | Counter | `{item}` | `signal` | Number of items ingested to durable storage. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.items.produced` | Counter | `{item}` | `signal` | Number of items sent downstream. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.items.queued` | Gauge | `{item}` | `signal` | Current number of items queued in durable storage. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.items.rejected` | Counter | `{item}` | `signal` | Number of items in permanently rejected bundles. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.items.requeued` | Counter | `{item}` | `signal` | Number of items requeued for retry after a NACK. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.loss.bundles` | Counter | `{bundle}` | `reason` | Number of bundles attributed to retention loss. This may be only the unresolved subset of a removed segment. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.loss.bytes` | Counter | `By` | `reason` | Logical payload bytes attributed to retention loss. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.loss.items` | Counter | `{item}` | `signal`, `reason` | Number of items in bundles attributed to retention loss. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.read.errors` | Counter | `{error}` | None | Number of read errors. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.reclaimed.bytes` | Counter | `By` | `reason` | Full persisted bytes reclaimed from removed segment files. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.reclaimed.segments` | Counter | `{segment}` | `reason` | Number of physical segment files removed. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.retries.scheduled` | Counter | `{retry}` | None | Number of retry attempts scheduled. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.storage.bytes.cap` | Gauge | `By` | None | Configured storage capacity cap. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.storage.bytes.used` | Gauge | `By` | None | Current bytes used by persistent storage. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.durable_buffer.storage.utilization` | Gauge | `{1}` | None | Current storage utilization ratio. | `crates/core-nodes/src/processors/durable_buffer_processor/metrics.rs` |
| `processor.fanout.active` | Gauge | `{message}` | None | Current number of active pipeline messages tracked by the processor. | `crates/core-nodes/src/processors/fanout_processor/metrics.rs` |
| `processor.fanout.max.inflight.config` | Gauge | `{message}` | None | Configured max_inflight value (0 means unlimited). | `crates/core-nodes/src/processors/fanout_processor/metrics.rs` |
| `processor.fanout.throttle.episodes` | Counter | `{episode}` | None | Increments on transition from not-throttled to throttled. | `crates/core-nodes/src/processors/fanout_processor/metrics.rs` |
| `processor.fanout.throttled` | Gauge | `1` | None | 1 when fanout is currently refusing new pdata via accept_pdata(), else 0. | `crates/core-nodes/src/processors/fanout_processor/metrics.rs` |
| `processor.fanout.timed.out` | Counter | `{message}` | `destination`, `signal` | Destination message attempts that timed out. | `crates/core-nodes/src/processors/fanout_processor/metrics.rs` |
| `processor.filter.dropped.items` | Counter | `{item}` | `signal` | Number of signal items (log records, spans, or metric data points) a decision node chose to drop. | `crates/core-nodes/src/processors/filter_processor/metrics.rs` |
| `processor.log_parser.failures` | Counter | `{operation}` | `error.type` | Failed local parsing operations. | `crates/core-nodes/src/processors/log_parser_processor/metrics.rs` |
| `processor.log_parser.operations` | Counter | `{operation}` | `signal`, `outcome` | Matching input messages whose local parsing operation terminated. | `crates/core-nodes/src/processors/log_parser_processor/metrics.rs` |
| `processor.log_parser.records` | Counter | `{record}` | `format`, `reason` | Preserved malformed records and committed observed-time fallbacks. | `crates/core-nodes/src/processors/log_parser_processor/metrics.rs` |
| `processor.log_sampling.dropped.items` | Counter | `{item}` | `signal` | Log records dropped by sampling. | `crates/core-nodes/src/processors/log_sampling_processor/metrics.rs` |
| `processor.log_sampling.filter.buffer.reclamation.failures` | Counter | `{error}` | `signal` | How many times we fail to reclaim the underlying filter buffer. | `crates/core-nodes/src/processors/log_sampling_processor/metrics.rs` |
| `processor.log_sampling.filtering.errors` | Counter | `{error}` | `signal` | Errors encountered while filtering OTAP batches. | `crates/core-nodes/src/processors/log_sampling_processor/metrics.rs` |
| `processor.partition.operations` | Counter | `{operation}` | `outcome` | Number of partitioning operations attempted by this processor. | `crates/core-nodes/src/processors/partition_processor/metrics.rs` |
| `processor.resource_validator.failures` | Counter | `{batch}` | `error.type` | Number of batches that failed resource validation. | `crates/contrib-nodes/src/processors/resource_validator_processor/metrics.rs` |
| `processor.retry.messages.recovered` | Counter | `{message}` | `signal` | Number of PData messages accepted downstream after at least one retry. | `crates/core-nodes/src/processors/retry_processor/mod.rs` |
| `processor.retry.messages.terminated` | Counter | `{message}` | `signal`, `reason` | Number of PData messages the retry processor stopped retrying. | `crates/core-nodes/src/processors/retry_processor/mod.rs` |
| `processor.retry.retries.scheduled` | Counter | `{retry}` | `signal` | Number of retries successfully scheduled after a downstream refusal. | `crates/core-nodes/src/processors/retry_processor/mod.rs` |
| `processor.temporal_reaggregation.failures` | Counter | `{operation}` | `error.type` | Metric `processor.temporal_reaggregation.failures`. | `crates/core-nodes/src/processors/temporal_reaggregation_processor/telemetry.rs` |
| `processor.temporal_reaggregation.flushes` | Counter | `{flush}` | `outcome`, `reason` | Metric `processor.temporal_reaggregation.flushes`. | `crates/core-nodes/src/processors/temporal_reaggregation_processor/telemetry.rs` |
| `processor.temporal_reaggregation.metrics` | Counter | `{metric}` | `type`, `aggregable`, `temporality` | Number of input metric records containing metric data. | `crates/core-nodes/src/processors/temporal_reaggregation_processor/telemetry.rs` |
| `processor.temporal_reaggregation.operations` | Counter | `{operation}` | `outcome` | Metric `processor.temporal_reaggregation.operations`. | `crates/core-nodes/src/processors/temporal_reaggregation_processor/telemetry.rs` |
| `processor.transform.failures` | Counter | `{operation}` | `language`, `signal`, `error.type` | Number of failed transform operations. | `crates/core-nodes/src/processors/transform_processor/metrics.rs` |
| `processor.transform.operations` | Counter | `{operation}` | `language`, `signal`, `outcome` | Number of matching input messages whose local transform operation terminated. | `crates/core-nodes/src/processors/transform_processor/metrics.rs` |
| `processor.type_router.signals.decision` | Counter | `{message}` | `outcome`, `reason`, `signal` | Number of messages with this outcome. | `crates/core-nodes/src/processors/signal_type_router/mod.rs` |
| `processor.wasm_processor.pdata.guest.counter.add.calls` | Counter | `{item}` | None | Number of guest `counter-add` calls accepted (name within both the length and cardinality bounds). | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.guest.counter.add.rejected.cardinality` | Counter | `{item}` | None | Number of guest `counter-add` calls rejected (silently no-op'd) because accepting the name would exceed the host's distinct-counter cardinality bound. Means the plugin is tracking more counters than the host allows, and some of its counts are being lost. | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.guest.counter.add.rejected.name.len` | Counter | `{item}` | None | Number of guest `counter-add` calls rejected (silently no-op'd) because the counter name exceeded the host's maximum name length. Distinct from the cardinality rejection below: this one means the plugin is emitting a malformed name. | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.guest.counter.add.value` | Counter | `{item}` | None | Cumulative sum of values accepted through guest `counter-add` calls. Saturates at `u64::MAX` for the lifetime of the plugin instance. | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.guest.host.service.calls.rejected` | Counter | `{item}` | None | Number of guest `log`/`counter-add`/`get-config` calls rejected (silently no-op'd) because the shared token-bucket rate limiter was empty. The limiter is scoped to the plugin instance's whole lifetime, not reset per `initialize`/`process` call. A non-zero value is how an operator tells a throttled plugin apart from a quiet one. | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.guest.log.message.truncated` | Counter | `{item}` | None | Number of guest `log` messages truncated to the host's maximum retained length before being forwarded to telemetry. | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.guest.process.calls` | Counter | `{item}` | None | Number of guest `process` calls attempted. | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.guest.process.errors` | Counter | `{item}` | None | Number of guest `process` calls that failed or trapped. | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.kernel.calls` | Counter | `{item}` | None | Total host kernel invocations dispatched by the guest. | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.pdata.dropped` | Counter | `{item}` | None | Number of pdata messages intentionally dropped by guest `process` returning `none`. | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.records.records.in` | Counter | `{item}` | `signal` | Records entering the guest (root batch row count). | `crates/wasm-host/src/metrics.rs` |
| `processor.wasm_processor.pdata.records.records.out` | Counter | `{item}` | `signal` | Records leaving the guest after filtering. | `crates/wasm-host/src/metrics.rs` |
| `receiver.database.acks` | Counter | `{ack}` | None | Downstream acknowledgements matching an in-flight page. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.batches.sent` | Counter | `{batch}` | None | Pages sent downstream. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.cancellations` | Counter | `{cancellation}` | None | Active database operations cancelled by a control message. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.checkpoint.cleanup.failures` | Counter | `{failure}` | None | Stale checkpoint revisions that could not be removed. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.checkpoint.commits` | Counter | `{commit}` | None | Durable checkpoint commits. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.checkpoint.failures` | Counter | `{failure}` | None | Durable checkpoint write failures. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.drains` | Counter | `{drain}` | None | Drain requests observed, including during checkpoint writes and retries. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.encoded.bytes.sent` | Counter | `By` | None | Encoded OTLP bytes sent downstream. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.event.time.fallbacks` | Counter | `{record}` | None | Records whose source event time cannot fit OTLP's timestamp range. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.nacks` | Counter | `{nack}` | None | Downstream negative acknowledgements matching an in-flight page. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.polls` | Counter | `{poll}` | None | Bounded page polls attempted against the database. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.query.failures` | Counter | `{failure}` | None | Query-page attempts that failed, including operation setup. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.reconnects` | Counter | `{attempt}` | None | Reconnect operations started after transient database failures. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.rejection.paused` | Gauge | `1` | None | One while polling is paused by permanent rejection, otherwise zero. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.replays` | Counter | `{replay}` | None | Replay schedules created after a NACK, not completed query attempts. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.rows.sent` | Counter | `{row}` | None | Database rows sent downstream. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.shutdowns` | Counter | `{shutdown}` | None | Shutdown requests observed, including during checkpoint writes and retries. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.stale.feedback` | Counter | `{feedback}` | None | ACK/NACK feedback discarded as malformed, wrong-generation, stale, or duplicate. | `crates/scraper/src/telemetry.rs` |
| `receiver.database.starts` | Counter | `{start}` | None | Receiver starts. | `crates/scraper/src/telemetry.rs` |
| `receiver.etw.decode.errors` | Counter | `{event}` | `signal` | Number of ETW events that failed TDH decode/parsing. Note that these events remain eligible for forwarding downstream with empty decoded fields. | `crates/contrib-nodes/src/receivers/etw_receiver/metrics.rs` |
| `receiver.etw.drops.events` | Counter | `{event}` | `signal`, `reason` | Number of dropped ETW events. | `crates/contrib-nodes/src/receivers/etw_receiver/metrics.rs` |
| `receiver.etw.events.forwarded` | Counter | `{event}` | `signal` | Events successfully forwarded downstream in batches. | `crates/contrib-nodes/src/receivers/etw_receiver/metrics.rs` |
| `receiver.etw.events.received` | Counter | `{event}` | `signal` | Total events produced/received by the session before channel enqueue. This is the ingress denominator and authoritative total. | `crates/contrib-nodes/src/receivers/etw_receiver/metrics.rs` |
| `receiver.etw.session.buffers.lost` | Counter | `{buffer}` | `signal`, `delivery` | Number of kernel ETW buffers lost. | `crates/contrib-nodes/src/receivers/etw_receiver/metrics.rs` |
| `receiver.etw.session.buffers.written` | Counter | `{buffer}` | `signal` | Number of kernel ETW buffers written by the session. | `crates/contrib-nodes/src/receivers/etw_receiver/metrics.rs` |
| `receiver.etw.session.events.lost` | Counter | `{event}` | `signal` | Number of events lost inside kernel ETW buffers before delivery (buffer overrun). Distinct from receiver-level drops and not included in received total. | `crates/contrib-nodes/src/receivers/etw_receiver/metrics.rs` |
| `receiver.host_metrics.batches.sent` | Counter | `{batch}` | None | Number of batches sent downstream. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.families.scraped` | Counter | `{family}` | None | Number of due metric families processed. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.partial.errors` | Counter | `{error}` | None | Number of source read errors skipped because other families succeeded. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.scrape.duration.ns` | Histogram | `ns` | None | Wall-clock scrape duration. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.scrape.lag.ns` | Histogram | `ns` | None | Delay between scheduled and actual scrape start. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.scraper.failures` | Counter | `{error}` | None | Number of scraper worker failures. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.scrapes.completed` | Counter | `{scrape}` | None | Number of scrape ticks that built and sent a metrics batch. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.scrapes.failed` | Counter | `{scrape}` | None | Number of fatal scrape failures. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.scrapes.overrun` | Counter | `{scrape}` | None | Number of scrape ticks skipped or timed out before completion. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.scrapes.started` | Counter | `{scrape}` | None | Number of scrape ticks started. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.send.failures` | Counter | `{error}` | None | Number of downstream send failures. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.host_metrics.source.read.errors` | Counter | `{error}` | None | Number of source read errors seen during scrapes. | `crates/core-nodes/src/receivers/host_metrics_receiver/mod.rs` |
| `receiver.journald.acknowledgements.responses` | Counter | `{response}` | `outcome` | Number of downstream responses. | `crates/core-nodes/src/receivers/journald_receiver/metrics.rs` |
| `receiver.journald.acknowledgements.rewinds` | Counter | `{rewind}` | `outcome` | Number of rewinds triggered by NACKs. | `crates/core-nodes/src/receivers/journald_receiver/metrics.rs` |
| `receiver.journald.checkpoints.commits` | Counter | `{commit}` | `outcome` | Number of durable cursor commits. | `crates/core-nodes/src/receivers/journald_receiver/metrics.rs` |
| `receiver.journald.lifecycle.transitions` | Counter | `{transition}` | `transition.type` | Number of lifecycle transitions. | `crates/core-nodes/src/receivers/journald_receiver/metrics.rs` |
| `receiver.journald.output.batches` | Counter | `{batch}` | None | Number of log batches emitted downstream. | `crates/core-nodes/src/receivers/journald_receiver/metrics.rs` |
| `receiver.journald.output.dropped.fields` | Counter | `{field}` | None | Number of journald fields dropped by extraction limits. | `crates/core-nodes/src/receivers/journald_receiver/metrics.rs` |
| `receiver.journald.output.records` | Counter | `{record}` | None | Number of log records emitted downstream. | `crates/core-nodes/src/receivers/journald_receiver/metrics.rs` |
| `receiver.journald.source.events` | Counter | `{event}` | `error.type` | Number of source read failures. | `crates/core-nodes/src/receivers/journald_receiver/metrics.rs` |
| `receiver.kafka.acknowledgements.responses` | Counter | `{response}` | `signal`, `outcome` | Number of downstream acknowledgement responses. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.group.feedback.after_revocation` | Counter | `{response}` | None | Ack or nack responses ignored because their partition ownership was stale. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.group.lag` | Gauge | `{message}` | None | Mean broker-committed consumer-group lag across every owned partition. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.group.partition.assignments` | Counter | `{partition}` | None | Partitions newly acquired by this consumer across rebalances. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.group.partition.revocations` | Counter | `{partition}` | None | Owned partitions revoked from this consumer across rebalances. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.group.partitions` | UpDownCounter | `{partition}` | None | Current number of partitions owned by this consumer. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.group.rebalance.commit_enqueue_failures` | Counter | `{error}` | None | Async commit-before-revoke calls that failed to enqueue locally during partition revocation. A broker rejection of the commit is not counted here; it is reported by `offset_commits` with `outcome="failure"`. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.group.rebalance.resume_failures` | Counter | `{error}` | None | Partition resume operations that failed while clearing rebalance pause state. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.group.rebalances` | Counter | `{rebalance}` | None | Consumer-group assignment events observed by this consumer. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.records.bytes` | Counter | `By` | None | Kafka record payload bytes delivered before filtering or decoding. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.records.duplicates` | Counter | `{message}` | None | Messages skipped because their offsets were already tracked in this ownership generation. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.records.inflight` | UpDownCounter | `{message}` | None | Current number of delivered records awaiting acknowledgement and commit progress. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.records.received` | Counter | `{message}` | None | Kafka records delivered by the consumer before filtering or decoding. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.retry.feedback.stale` | Counter | `{response}` | None | Feedback ignored because it belongs to an obsolete replay generation. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.retry.partitions_paused` | UpDownCounter | `{partition}` | None | Current number of partitions paused during transient-NACK recovery. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.retry.pause_failures` | Counter | `{error}` | None | Partition pause operations that failed during transient-NACK recovery. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.retry.replay_attempts` | Counter | `{attempt}` | None | Kafka replay attempts started after transient NACK backoff. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.retry.resume_failures` | Counter | `{error}` | None | Partition resume operations that failed during transient-NACK recovery. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.retry.seek_failures` | Counter | `{error}` | None | Partition seek operations that failed during transient-NACK recovery. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.consumer.retry.transient_nacks` | Counter | `{response}` | None | Non-permanent NACK responses received from downstream. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.messages.completed` | Counter | `{message}` | `signal` | Number of admitted messages whose receiver work terminated. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.messages.payload.size` | Counter | `By` | `signal` | Encoded Kafka payload bytes admitted to the pipeline send path. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.messages.started` | Counter | `{message}` | `signal` | Number of decoded messages admitted to the pipeline send path. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.offset_commits.commits` | Counter | `{commit}` | `outcome` | Number of offset commit results reported by librdkafka. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.rejections.messages` | Counter | `{message}` | `signal`, `error.type`, `reason` | Number of rejected Kafka messages. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.kafka.transport.errors` | Counter | `{error}` | `error.type` | Number of non-EOF errors returned by the Kafka consumer. | `crates/contrib-nodes/src/receivers/kafka_receiver/metrics.rs` |
| `receiver.microsoft_json.compressed.requests` | Counter | `{request}` | None | Number of HTTP requests received with an encoded body. | `crates/contrib-nodes/src/receivers/microsoft_json_receiver/config.rs` |
| `receiver.microsoft_json.config.endpoints.failed` | Counter | `{error}` | None | Number of endpoint configuration files that failed startup loading. | `crates/contrib-nodes/src/receivers/microsoft_json_receiver/config.rs` |
| `receiver.microsoft_json.config.endpoints.loaded` | Counter | `{item}` | None | Number of endpoint configuration files loaded at startup. | `crates/contrib-nodes/src/receivers/microsoft_json_receiver/config.rs` |
| `receiver.microsoft_json.cors.preflight.requests` | Counter | `{request}` | None | Number of CORS preflight (`OPTIONS`) requests handled. | `crates/contrib-nodes/src/receivers/microsoft_json_receiver/config.rs` |
| `receiver.microsoft_json.events.oversized` | Counter | `{item}` | None | Number of events rejected for exceeding the endpoint event-size limit. | `crates/contrib-nodes/src/receivers/microsoft_json_receiver/config.rs` |
| `receiver.microsoft_json.ikey.ignored` | Counter | `{item}` | None | Number of events silently dropped by the iKey filter. | `crates/contrib-nodes/src/receivers/microsoft_json_receiver/config.rs` |
| `receiver.microsoft_json.ikey.rejected` | Counter | `{item}` | None | Number of events rejected by the iKey filter. | `crates/contrib-nodes/src/receivers/microsoft_json_receiver/config.rs` |
| `receiver.microsoft_json.ikey.unauthorized` | Counter | `{item}` | None | Number of events rejected because their origin is not allowed. | `crates/contrib-nodes/src/receivers/microsoft_json_receiver/config.rs` |
| `receiver.microsoft_json.received.logs.invalid` | Counter | `{item}` | None | Number of JSON events that failed parsing or endpoint validation. | `crates/contrib-nodes/src/receivers/microsoft_json_receiver/config.rs` |
| `receiver.otap.acknowledgements.responses` | Counter | `{response}` | `signal`, `outcome` | Number of routed or invalid acknowledgement responses. | `crates/core-nodes/src/receivers/otap_receiver/mod.rs` |
| `receiver.otap.batches.completed` | Counter | `{batch}` | `signal` | Number of admitted batches whose receiver work terminated. | `crates/core-nodes/src/receivers/otap_receiver/mod.rs` |
| `receiver.otap.batches.payload.size` | Counter | `By` | `signal` | Protobuf-encoded batch bytes after gRPC transport decompression. | `crates/core-nodes/src/receivers/otap_receiver/mod.rs` |
| `receiver.otap.batches.started` | Counter | `{batch}` | `signal` | Number of batches admitted to the pipeline send path. | `crates/core-nodes/src/receivers/otap_receiver/mod.rs` |
| `receiver.otap.rejections.batches` | Counter | `{batch}` | `error.type` | Number of rejected OTAP batches within admitted streams. | `crates/core-nodes/src/receivers/otap_receiver/mod.rs` |
| `receiver.otap.rejections.streams` | Counter | `{stream}` | `error.type` | Number of rejected OTAP streaming RPCs. | `crates/core-nodes/src/receivers/otap_receiver/mod.rs` |
| `receiver.otlp.acknowledgements.responses` | Counter | `{response}` | `signal`, `outcome` | Number of routed or invalid acknowledgement responses. | `crates/otap/src/otlp/metrics.rs` |
| `receiver.otlp.requests.accepted` | Counter | `{request}` | `signal`, `protocol` | Number of OTLP requests admitted to the pipeline send path. | `crates/otap/src/otlp/metrics.rs` |
| `receiver.otlp.requests.rejected` | Counter | `{request}` | `protocol`, `error.type` | Number of rejected requests. | `crates/otap/src/otlp/metrics.rs` |
| `receiver.otlp.transport.errors` | Counter | `{error}` | `protocol` | Number of transport-level server errors. | `crates/otap/src/otlp/metrics.rs` |
| `receiver.prometheus.batches.sent` | Counter | `{batch}` | None | Number of OTAP batches sent downstream. | `crates/core-nodes/src/receivers/prometheus_receiver/mod.rs` |
| `receiver.prometheus.data.points.decoded` | Counter | `{data_point}` | None | Total number of metric data points decoded across all scrapes. | `crates/core-nodes/src/receivers/prometheus_receiver/mod.rs` |
| `receiver.prometheus.scrape.duration.ns` | Histogram | `ns` | None | Per-target wall-clock scrape + decode duration. | `crates/core-nodes/src/receivers/prometheus_receiver/mod.rs` |
| `receiver.prometheus.scrapes.completed` | Counter | `{scrape}` | None | Number of target scrapes that succeeded. | `crates/core-nodes/src/receivers/prometheus_receiver/mod.rs` |
| `receiver.prometheus.scrapes.failed` | Counter | `{scrape}` | None | Number of target scrapes that failed HTTP fetch or exposition decoding. | `crates/core-nodes/src/receivers/prometheus_receiver/mod.rs` |
| `receiver.prometheus.scrapes.started` | Counter | `{scrape}` | None | Number of target scrapes started. | `crates/core-nodes/src/receivers/prometheus_receiver/mod.rs` |
| `receiver.prometheus.send.failures` | Counter | `{error}` | None | Number of downstream send failures. | `crates/core-nodes/src/receivers/prometheus_receiver/mod.rs` |
| `receiver.syslog_cef.connections.active` | UpDownCounter | `{connection}` | None | Active connections | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.syslog_cef.connections.rejected` | Counter | `{connection}` | None | Rejected connections | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.syslog_cef.rejections.items` | Counter | `{item}` | `signal`, `protocol`, `error.type` | Number of items rejected | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.syslog_cef.transport.errors` | Counter | `{error}` | `protocol` | Number of transport-level server errors. | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.syslog_cef.truncations.items` | Counter | `{item}` | `signal` | Truncated payloads | `crates/core-nodes/src/receivers/syslog_cef_receiver/mod.rs` |
| `receiver.topic.bridge.controls` | Counter | `{control}` | `control`, `result` | Number of bridge controls. | `crates/core-nodes/src/receivers/topic_receiver/metrics.rs` |
| `receiver.topic.downstream.backpressure.events` | Counter | `{event}` | None | Number of downstream backpressure events (>= 500ms blocked). | `crates/core-nodes/src/receivers/topic_receiver/metrics.rs` |
| `receiver.topic.downstream.blocked.duration` | Histogram | `s` | None | Time blocked while forwarding one message to downstream. | `crates/core-nodes/src/receivers/topic_receiver/metrics.rs` |
| `receiver.topic.lag.events` | Counter | `{event}` | `event.type` | Number of lag events. | `crates/core-nodes/src/receivers/topic_receiver/metrics.rs` |
| `receiver.topic.lagged.messages` | Counter | `{message}` | None | Total messages missed across lag notifications. | `crates/core-nodes/src/receivers/topic_receiver/metrics.rs` |
| `receiver.traffic_generator.completion.drain_deadline_forced` | Counter | `{drain}` | None | Number of drains forced to finish with unresolved batches at the deadline. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.completion.pending` | Gauge | `{batch}` | None | Number of subscribed batches waiting for Ack/Nack completion. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.batch_interval` | Gauge | `ns` | None | Smooth-mode configured interval between batches. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.batch_tick_lateness_duration` | Histogram | `s` | None | Lateness of smooth-mode batch ticks relative to their scheduled instant. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.late_remaining_batches` | Histogram | `{batch}` | None | Number of batches remaining when smooth mode detects that a run is late. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.late_remaining_items` | Histogram | `{item}` | None | Number of signal items remaining when smooth mode detects that a run is late. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.payload.sends.attempts` | Counter | `{attempt}` | `attempt`, `outcome` | Number of smooth-mode payload send attempts. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.payload_generate_duration` | Histogram | `s` | None | Wall-clock time spent generating or cloning one smooth-mode payload. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.payload_send_duration` | Histogram | `s` | None | Wall-clock time spent sending one smooth-mode payload into the downstream channel. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.run_batches` | Gauge | `{batch}` | None | Smooth-mode configured batches per one-second run. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.runs.terminations` | Counter | `{run}` | `outcome` | Number of smooth-mode production runs terminated. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.runs_started` | Counter | `{run}` | None | Number of smooth-mode production runs started. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.traffic_generator.smooth.schedule_deadline_misses` | Counter | `{miss}` | None | Number of times smooth mode detects a missed scheduling boundary. | `crates/dev-nodes/src/receivers/traffic_generator/metrics.rs` |
| `receiver.user_events.downstream.send.blocked.ns` | Counter | `ns` | None | Total time spent waiting for downstream channel capacity. | `crates/contrib-nodes/src/receivers/user_events_receiver/metrics.rs` |
| `receiver.user_events.drops.samples` | Counter | `{item}` | `reason` | Number of dropped samples. | `crates/contrib-nodes/src/receivers/user_events_receiver/metrics.rs` |
| `receiver.user_events.flushed.batches` | Counter | `{event}` | None | Number of Arrow batches flushed downstream. | `crates/contrib-nodes/src/receivers/user_events_receiver/metrics.rs` |
| `receiver.user_events.forwarded.samples` | Counter | `{item}` | None | Number of perf samples forwarded downstream. | `crates/contrib-nodes/src/receivers/user_events_receiver/metrics.rs` |
| `receiver.user_events.late.registration.retries` | Counter | `{event}` | None | Number of late-registration retries attempted while waiting for tracepoints. | `crates/contrib-nodes/src/receivers/user_events_receiver/metrics.rs` |
| `receiver.user_events.lost.perf.samples` | Counter | `{item}` | None | Number of lost samples reported by the perf ring. | `crates/contrib-nodes/src/receivers/user_events_receiver/metrics.rs` |
| `receiver.user_events.received.samples` | Counter | `{item}` | None | Number of perf samples received from the kernel ring. | `crates/contrib-nodes/src/receivers/user_events_receiver/metrics.rs` |
| `receiver.user_events.sessions.started` | Counter | `{event}` | None | Number of receiver sessions successfully started. | `crates/contrib-nodes/src/receivers/user_events_receiver/metrics.rs` |
| `receiver.windows_perf_counters.active.counters` | Gauge | `{counter}` | None | Counters currently added to a query. | `crates/contrib-nodes/src/receivers/windows_perf_counters_receiver/metrics.rs` |
| `receiver.windows_perf_counters.batches.dropped` | Counter | `{batch}` | None | Batches dropped because downstream was full or closed. | `crates/contrib-nodes/src/receivers/windows_perf_counters_receiver/metrics.rs` |
| `receiver.windows_perf_counters.counter.add.failures` | Counter | `{failure}` | None | Failed counter adds, including retries. | `crates/contrib-nodes/src/receivers/windows_perf_counters_receiver/metrics.rs` |
| `receiver.windows_perf_counters.scrape.duration` | Histogram | `s` | None | Time from scrape request to report. | `crates/contrib-nodes/src/receivers/windows_perf_counters_receiver/metrics.rs` |
| `receiver.windows_perf_counters.scrape.failures` | Counter | `{scrape}` | None | Scrapes in which at least one counter's query failed to open or collect, plus startup if a query failed to open or prime. | `crates/contrib-nodes/src/receivers/windows_perf_counters_receiver/metrics.rs` |
| `receiver.windows_perf_counters.scrape.overruns` | Counter | `{tick}` | None | Ticks skipped because the previous scrape was still running. | `crates/contrib-nodes/src/receivers/windows_perf_counters_receiver/metrics.rs` |
| `receiver.windows_perf_counters.scrapes` | Counter | `{scrape}` | None | Scrapes in which at least one counter's query collected. | `crates/contrib-nodes/src/receivers/windows_perf_counters_receiver/metrics.rs` |
<!-- END GENERATED: telemetry-metrics -->
