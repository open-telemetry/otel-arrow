# Engine Crate Telemetry

This document lists telemetry emitted directly by the `crates/engine` crate.
It includes metric instruments registered by the crate and log events
emitted via `otel_*` log macros.

## Metrics

| Metric name | Description | Produced in file |
| --- | --- | --- |
| `engine.memory_rss` | Process resident memory (RSS) in bytes. | `crates/engine/src/engine_metrics.rs` |
| `engine.cpu_utilization` | Process-wide CPU utilization as a ratio in `[0, 1]`, normalized across all logical CPU cores on the system. Aligned with the OTel semantic convention `process.cpu.utilization`. | `crates/engine/src/engine_metrics.rs` |
| `channel.sender.messages` | Number of immediate send attempts, grouped by `outcome` and, for PData channels, `signal`. | `crates/engine/src/channel_metrics.rs` |
| `channel.sender.failures` | Number of unsuccessful send attempts, grouped by `error.type` and, for PData channels, `signal`. | `crates/engine/src/channel_metrics.rs` |
| `channel.receiver.messages` | Number of messages successfully dequeued, grouped by `signal` for PData channels. | `crates/engine/src/channel_metrics.rs` |
| `channel.receiver.queue.depth` | Current number of messages buffered in the channel. | `crates/engine/src/channel_metrics.rs` |
| `channel.receiver.capacity` | Configured channel buffer capacity. | `crates/engine/src/channel_metrics.rs` |
| `node.consumer.consumed.duration` | Duration from entry until the corresponding ack or nack is routed, in nanoseconds (MMSC). | `crates/engine/src/channel_metrics.rs` |
| `node.consumer.consumed.messages` | Messages consumed by the node, grouped by the `signal` and `outcome` datapoint attributes. | `crates/engine/src/channel_metrics.rs` |
| `node.consumer.consumed.items` | Signal items consumed (received) by an item-count-enabled node, grouped by the `signal` and `outcome` datapoint attributes. | `crates/engine/src/channel_metrics.rs` |
| `node.producer.produced.duration` | Duration from production until the corresponding ack or nack is routed, in nanoseconds (MMSC). | `crates/engine/src/channel_metrics.rs` |
| `node.producer.produced.messages` | Messages produced by the node, grouped by the `signal` and `outcome` datapoint attributes. | `crates/engine/src/channel_metrics.rs` |
| `node.producer.produced.items` | Signal items produced (emitted) by an item-count-enabled node, grouped by the `signal` and `outcome` datapoint attributes. | `crates/engine/src/channel_metrics.rs` |
| `flow.consumed.items` | Signal items entering an opted-in processor flow, grouped by the `signal` datapoint attribute. | `crates/engine/src/flow_metrics.rs` |
| `flow.compute.duration` | Processor compute duration within an opted-in flow, grouped by the `signal` datapoint attribute. | `crates/engine/src/flow_metrics.rs` |
| `flow.produced.items` | Signal items leaving an opted-in processor flow, grouped by the `signal` datapoint attribute. | `crates/engine/src/flow_metrics.rs` |
| `flow.dropped.items` | Signal items dropped at a decision node in an opted-in flow, grouped by the `signal` datapoint attribute. | `crates/engine/src/flow_metrics.rs` |
| `pipeline.uptime` | Time since pipeline instance start. | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.memory_usage` | Current heap memory in use by the pipeline thread (jemalloc only). | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.memory_allocated` | Cumulative bytes allocated by the pipeline thread (jemalloc only). | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.memory_freed` | Cumulative bytes freed by the pipeline thread (jemalloc only). | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.memory_allocated_delta` | Bytes allocated during the latest sampling interval (jemalloc only). | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.memory_freed_delta` | Bytes freed during the latest sampling interval (jemalloc only). | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.cpu_time` | Cumulative CPU seconds consumed by the pipeline thread. | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.cpu_utilization` | Ratio of CPU time to wall time over the latest interval. | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.context_switches_voluntary` | Cumulative voluntary thread context switches. | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.context_switches_involuntary` | Cumulative involuntary thread context switches (preemption). | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.page_faults_minor` | Cumulative minor page faults for the pipeline thread. | `crates/engine/src/pipeline_metrics.rs` |
| `pipeline.page_faults_major` | Cumulative major page faults for the pipeline thread. | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_count` | Number of Tokio worker threads in the runtime. | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.task_active_count` | Current count of alive tasks in the runtime. | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.global_task_queue_size` | Current count of tasks in Tokio global/injection queue. | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_busy_time` | Total worker busy time summed across workers (`target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_park_count` | Total worker park operations (`target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_park_unpark_count` | Total worker park/unpark transitions (`target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.blocking_task_queue_size` | Current tasks pending in Tokio blocking queue (`tokio_unstable`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.blocking_thread_count` | Current number of Tokio blocking pool threads (`tokio_unstable`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.blocking_thread_idle_count` | Current number of idle Tokio blocking pool threads (`tokio_unstable`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_local_queue_size` | Current tasks in all worker-local queues (`tokio_unstable`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.spawned_tasks_count` | Total tasks spawned since runtime creation (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.remote_schedule_count` | Total schedules from outside runtime (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.budget_forced_yield_count` | Total forced cooperative yields (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_noop_count` | Total noop unpark events summed across workers (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_steal_success_count` | Total successful worker steal operations (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_steal_attempt_count` | Total worker steal attempts (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_poll_count` | Total task poll operations across workers (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_local_schedule_count` | Total schedules into worker-local queues (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.worker_overflow_count` | Total worker local-queue overflow events (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.io_driver_fd_registered_count` | Total file descriptors registered in Tokio I/O driver (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.io_driver_fd_deregistered_count` | Total file descriptors deregistered in Tokio I/O driver (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |
| `tokio.runtime.io_driver_ready_count` | Total ready events processed by Tokio I/O driver (`tokio_unstable`, `target_has_atomic = "64"`). | `crates/engine/src/pipeline_metrics.rs` |

### Channel metric semantics

`channel.sender.messages` records the terminal local result of each
instrumented send operation. Use `outcome=success` for messages enqueued on the
channel, `outcome=refused` for non-blocking sends rejected because the channel
is full, and `outcome=failure` for sends rejected because the receiver is
closed. The `channel.sender.failures` metric gives the actionable cause as
`error.type=full` or `error.type=closed`.

`channel.receiver.messages` records only successful dequeues. Empty polls are
normal for non-blocking consumers, and closure is channel lifecycle state, so
neither condition is counted as a receive error. Alert on sustained send
failures, or on `channel.receiver.queue.depth / channel.receiver.capacity`
remaining near one, rather than on empty receive attempts.

Channel metrics and node producer/consumer metrics observe different stages of
a message's lifecycle:

| Metric layer | Recorded when | Operational use |
| --- | --- | --- |
| `channel.sender` / `channel.receiver` | A forward-path send or receive operation completes. | Diagnose edge throughput, queue saturation, backpressure, and closed channels. |
| `node.producer` / `node.consumer` | A terminal ACK or NACK unwinds through the node's route frame. | Attribute logical PData outcomes, durations, and item counts to nodes. |

Do not aggregate `outcome` values across these layers as equivalent events. For
channel metrics, `refused` means local capacity backpressure and `failure`
means a closed receiver. For node metrics, `refused` means a permanent NACK and
`failure` means a retryable NACK.

On a healthy, drained, one-input/one-output path, successful channel send and
receive counts and terminal node message counts are often close. They can
differ while messages are queued or in flight, when a non-blocking send is
rejected, after a downstream NACK, or when an operation is retried. Use channel
metrics for transport health and node metrics for the eventual processing
result.

## Logs

| Event name | Level | Description | Produced in file |
| --- | --- | --- | --- |
| `pipeline.build.unconnected_node.removed` | `info` | Logs each unconnected node removed from pipeline config during build. | `crates/engine/src/lib.rs` |
| `pipeline.build.unconnected_nodes` | `warn` | Warns when one or more unconnected nodes were removed. | `crates/engine/src/lib.rs` |
| `receiver.create.start` | `debug` | Receiver node creation started. | `crates/engine/src/lib.rs` |
| `receiver.create.complete` | `debug` | Receiver node creation completed. | `crates/engine/src/lib.rs` |
| `processor.create.start` | `debug` | Processor node creation started. | `crates/engine/src/lib.rs` |
| `processor.create.complete` | `debug` | Processor node creation completed. | `crates/engine/src/lib.rs` |
| `exporter.create.start` | `debug` | Exporter node creation started. | `crates/engine/src/lib.rs` |
| `exporter.create.complete` | `debug` | Exporter node creation completed. | `crates/engine/src/lib.rs` |
| `pdata.sender.set` | `debug` | PData sender endpoint attached to a source node/port. | `crates/engine/src/lib.rs` |
| `pdata.receiver.set` | `debug` | PData receiver endpoint attached to a destination node. | `crates/engine/src/lib.rs` |
| `pipeline.draining.deadline_exceeded` | `warn` | Draining deadline exceeded; pipeline shutdown is forced. | `crates/engine/src/pipeline_ctrl.rs` |
| `pipeline.draining.ignored_start_timer` | `debug` | `StartTimer` ignored during shutdown draining. | `crates/engine/src/pipeline_ctrl.rs` |
| `pipeline.draining.ignored_start_telemetry_timer` | `debug` | `StartTelemetryTimer` ignored during shutdown draining. | `crates/engine/src/pipeline_ctrl.rs` |
| `pipeline.metrics.reporting.fail` | `warn` | Reporting pipeline internal metrics failed (non-fatal). | `crates/engine/src/pipeline_ctrl.rs` |
| `tokio.metrics.reporting.fail` | `warn` | Reporting Tokio runtime metrics failed (non-fatal). | `crates/engine/src/pipeline_ctrl.rs` |
| `channel.metrics.reporting.fail` | `warn` | Reporting channel metrics failed (non-fatal). | `crates/engine/src/pipeline_ctrl.rs` |
| `node.metrics.reporting.fail` | `warn` | Reporting node (consumer/producer) metrics failed (non-fatal). | `crates/engine/src/pipeline_ctrl.rs` |
| `pipeline.ctrl.pending_sends.high` | `warn` | Pending sends buffer reached the warning threshold; a node's control channel may be persistently full. | `crates/engine/src/pipeline_ctrl.rs` |
| `topic.tracked_publish.duplicate_message_id` | `warn` | Tracked publish tracker registered a duplicate message id and overwrote the previous entry. | `crates/engine/src/topic/types.rs` |

## Maintenance

When adding or changing telemetry in this crate:

1. **Metrics**
     - If you add a field under a `#[metric_set(...)]` struct in
         `crates/engine/src/*metrics*.rs`, add/update the corresponding row in
         the **Metrics** table.
     - Use the effective emitted name as
         `<metric_set_name>.<metric_field_name_or_metric_name_override>`.
     - If the metric is feature/target gated (for example `tokio_unstable`,
         `target_has_atomic = "64"`, or jemalloc-specific), note that in the
         description.

2. **Logs**
     - If you add `otel_trace!`, `otel_debug!`, `otel_info!`, `otel_warn!`, or
         `otel_error!` calls in `crates/engine/src/**`, add/update a row in the
         **Logs** table.
     - Keep event names exact (first macro argument), include the explicit
         log level, and reference the file where it is emitted.

3. **Review checklist (quick)**
     - Search for new metric sets: `#[metric_set(` in `crates/engine/src/**`.
     - Search for new log events: `otel_(trace|debug|info|warn|error)!(` in
         `crates/engine/src/**`.
     - Confirm this document still matches current source files.
