# Engine Crate Telemetry

This document lists telemetry emitted directly by the `crates/engine` crate.
It includes metric instruments registered by the crate and log events
emitted via `otel_*` log macros.

## Metrics

The admin server exposes the live metric registry at `GET /api/v1/metrics`
(an alias of `GET /api/v1/telemetry/metrics`). The default response uses
Prometheus text format. In that format, the metric-set name appears in the
`otel_scope_name` label and the instrument name becomes the sample name. For
example, `node.input.messages` appears as
`messages_total{otel_scope_name="node.input", ...}`. Use `?format=json` to see
the metric-set and instrument names separately.

A metric is visible only after all of these conditions are met:

- its configuration and platform conditions in the table below are satisfied;
- the event it measures has occurred (for counters and histograms); and
- the engine has completed a telemetry reporting interval.

The endpoint omits zero-valued metric sets by default. For JSON responses, add
`keep_all_zeroes=true` when investigating whether an instrument is registered
but has not yet recorded a value.

### Runtime metric levels

`policies.telemetry.runtime_metrics` controls channel, node, and shared
control-plane metrics. Its default value is `basic`.

| Level | Data-path metrics enabled |
| --- | --- |
| `none` | No channel or node input/output metrics. |
| `basic` | `channel.*` metrics. |
| `normal` | `basic`, plus `node.input.messages` and `node.output.messages`. A node can also opt into item and size metrics. |
| `detailed` | `normal`, plus node duration, item, and size metrics for every node. |

At `normal`, enable the optional measurements on an individual node with
`policies.telemetry.item_counts: true` and/or
`policies.telemetry.size: true`. These node options do not enable node metrics
when the pipeline level is `none` or `basic`.

Node kind determines which side exists:

| Node kind | `node.input.*` | `node.output.*` |
| --- | --- | --- |
| Receiver | Not produced; a receiver has no pipeline input channel. | Produced for messages sent into the pipeline. |
| Processor | Produced for messages consumed by the processor. | Produced only for messages the processor sends. |
| Exporter | Produced for messages consumed by the exporter. | Not produced; an exporter has no pipeline output channel. |

Flow metrics are independent of `runtime_metrics`. They are produced only for
ranges explicitly declared in `policies.telemetry.flow_metrics`. Omitting the
`metrics` list from a flow enables every flow metric; specifying the list
enables only the selected entries. If `flow_metrics` is omitted or empty, no
`flow.*` metrics are produced.

For configuration examples and guidance on interpreting flow attributes and
counts, see [Node and Flow Metrics](../../docs/node-and-flow-metrics.md#flow-metrics).

### Metric reference

| Metric name | Description | Produced when |
| --- | --- | --- |
| `engine.memory_rss` | Process resident memory (RSS) in bytes. | The engine is running; sampled once per engine about every five seconds. |
| `engine.cpu_utilization` | Process-wide CPU utilization as a ratio in `[0, 1]`, normalized across all logical CPU cores on the system. Aligned with the OTel semantic convention `process.cpu.utilization`. | The engine is running; sampled once per engine about every five seconds. |
| `engine.memory_pressure_state` | Memory limiter state: `0` for normal, `1` for soft pressure, and `2` for hard pressure. | The engine is running. The value remains `0` while no memory pressure is detected. |
| `engine.process_memory_usage_bytes` | Most recent process memory usage sample in bytes. | The engine is running and the memory limiter has sampled process memory. |
| `engine.process_memory_soft_limit_bytes` | Effective process-wide soft memory limit in bytes. | The engine is running. The value is `0` when no soft limit is configured. |
| `engine.process_memory_hard_limit_bytes` | Effective process-wide hard memory limit in bytes. | The engine is running. The value is `0` when no hard limit is configured. |
| `channel.sender.messages` | Number of immediate send attempts, grouped by `outcome` and, for PData channels, `signal`. | `runtime_metrics` is `basic` or higher and a data or control channel send is attempted. |
| `channel.sender.failures` | Number of unsuccessful send attempts, grouped by `error.type` and, for PData channels, `signal`. | `runtime_metrics` is `basic` or higher and a send finds a full or closed channel. Healthy pipelines may not produce this metric. |
| `channel.receiver.messages` | Number of messages successfully dequeued, grouped by `signal` for PData channels. | `runtime_metrics` is `basic` or higher and a data or control message is dequeued. Empty polls and channel closure are not counted. |
| `channel.receiver.queue.depth` | Current number of messages buffered in the channel. | `runtime_metrics` is `basic` or higher and the channel exists; sampled on the reporting interval. |
| `channel.receiver.capacity` | Configured channel buffer capacity. | `runtime_metrics` is `basic` or higher and the channel exists; sampled on the reporting interval. |
| `node.input.duration` | Duration from entry until the corresponding ack or nack is routed, in seconds (histogram). | `runtime_metrics` is `detailed`, a processor or exporter consumes PData, and its terminal ack or nack unwinds. |
| `node.input.messages` | Messages received by the node, grouped by the `signal` and `outcome` datapoint attributes. | `runtime_metrics` is `normal` or `detailed`, a processor or exporter consumes PData, and its terminal ack or nack unwinds. |
| `node.input.items` | Signal items received by an item-count-enabled node, grouped by the `signal` and `outcome` datapoint attributes. | Input message conditions are met and either the level is `detailed` or that processor/exporter sets `policies.telemetry.item_counts: true`. |
| `node.input.size` | Logical payload bytes received by a size-enabled node, grouped by the `signal` and `outcome` datapoint attributes. | Input message conditions are met and either the level is `detailed` or that processor/exporter sets `policies.telemetry.size: true`. |
| `node.output.duration` | Duration from output until the corresponding ack or nack is routed, in seconds (histogram). | `runtime_metrics` is `detailed`, a receiver emits PData, and its terminal ack or nack unwinds. Processor latency is recorded in `node.input.duration`. |
| `node.output.messages` | Messages emitted by the node, grouped by the `signal` and `outcome` datapoint attributes. | `runtime_metrics` is `normal` or `detailed`, a receiver or processor sends PData, and its terminal ack or nack unwinds. A processor that drops a whole message produces no output datapoint for it. |
| `node.output.items` | Signal items emitted by an item-count-enabled node, grouped by the `signal` and `outcome` datapoint attributes. | Output message conditions are met and either the level is `detailed` or that receiver/processor sets `policies.telemetry.item_counts: true`. |
| `node.output.size` | Logical payload bytes emitted by a size-enabled node, grouped by the `signal` and `outcome` datapoint attributes. | Output message conditions are met and either the level is `detailed` or that receiver/processor sets `policies.telemetry.size: true`. |
| `flow.input.messages` | PData messages entering an opted-in processor flow, grouped by the `signal` datapoint attribute. | A configured flow enables `input_messages` and a message enters its start processor. |
| `flow.input.items` | Signal items entering an opted-in processor flow, grouped by the `signal` datapoint attribute. | A configured flow enables `input_items` and a message enters its start processor. |
| `flow.input.size` | Logical payload bytes entering an opted-in processor flow, grouped by the `signal` datapoint attribute. | A configured flow enables `input_size` and a message enters its start processor. |
| `flow.output.messages` | PData messages leaving an opted-in processor flow, grouped by the `signal` datapoint attribute. | A configured flow enables `output_messages` and a message leaves its end processor. |
| `flow.output.items` | Signal items leaving an opted-in processor flow, grouped by the `signal` datapoint attribute. | A configured flow enables `output_items` and a message leaves its end processor. |
| `flow.output.size` | Logical payload bytes leaving an opted-in processor flow, grouped by the `signal` datapoint attribute. | A configured flow enables `output_size` and a message leaves its end processor. |
| `flow.compute.duration` | Histogram of processor compute duration within an opted-in flow, in seconds and grouped by the `signal` datapoint attribute. | A configured flow enables `compute_duration` and a message is processed within the declared processor range. |
| `flow.dropped.items` | Signal items dropped at a decision node in an opted-in flow, grouped by the `signal` datapoint attribute. | A configured flow enables `dropped_items` and a drop-capable decision processor inside the range drops one or more items. |
| `pipeline.uptime` | Time since pipeline instance start. | The pipeline sets `policies.telemetry.pipeline_metrics: true` (the default). |
| `pipeline.memory_usage` | Current heap memory in use by the pipeline thread. | `pipeline_metrics` is `true` and the non-Windows build uses jemalloc with working thread-local statistics; otherwise it remains zero and is normally omitted. |
| `pipeline.memory_allocated` | Cumulative bytes allocated by the pipeline thread. | Same conditions as `pipeline.memory_usage`. |
| `pipeline.memory_freed` | Cumulative bytes freed by the pipeline thread. | Same conditions as `pipeline.memory_usage`. |
| `pipeline.memory_allocated_delta` | Bytes allocated during the latest sampling interval. | Same conditions as `pipeline.memory_usage`. |
| `pipeline.memory_freed_delta` | Bytes freed during the latest sampling interval. | Same conditions as `pipeline.memory_usage`. |
| `pipeline.cpu_time` | Cumulative CPU seconds consumed by the pipeline thread. | The pipeline sets `pipeline_metrics: true`. |
| `pipeline.cpu_utilization` | Ratio of CPU time to wall time over the latest interval. | The pipeline sets `pipeline_metrics: true`. |
| `pipeline.context_switches_voluntary` | Cumulative voluntary thread context switches. | `pipeline_metrics` is `true`, the OS is Linux, FreeBSD, or OpenBSD, and per-thread `getrusage` succeeds. |
| `pipeline.context_switches_involuntary` | Cumulative involuntary thread context switches (preemption). | Same conditions as `pipeline.context_switches_voluntary`. |
| `pipeline.page_faults_minor` | Cumulative minor page faults for the pipeline thread. | Same conditions as `pipeline.context_switches_voluntary`. |
| `pipeline.page_faults_major` | Cumulative major page faults for the pipeline thread. | Same conditions as `pipeline.context_switches_voluntary`. |
| `tokio.runtime.worker_count` | Number of Tokio worker threads in the runtime. | The pipeline sets `policies.telemetry.tokio_metrics: true` (the default) and runs inside a Tokio runtime. |
| `tokio.runtime.task_active_count` | Current count of alive tasks in the runtime. | Same conditions as `tokio.runtime.worker_count`. |
| `tokio.runtime.global_task_queue_size` | Current count of tasks in Tokio global/injection queue. | Same conditions as `tokio.runtime.worker_count`. |
| `tokio.runtime.worker_busy_time` | Total worker busy time summed across workers. | Same conditions as `tokio.runtime.worker_count`, on a target with 64-bit atomics. |
| `tokio.runtime.worker_park_count` | Total worker park operations. | Same conditions as `tokio.runtime.worker_count`, on a target with 64-bit atomics. |
| `tokio.runtime.worker_park_unpark_count` | Total worker park/unpark transitions. | Same conditions as `tokio.runtime.worker_count`, on a target with 64-bit atomics. |
| `tokio.runtime.blocking_task_queue_size` | Current tasks pending in Tokio blocking queue. | Same conditions as `tokio.runtime.worker_count`, in a build compiled with `tokio_unstable`. |
| `tokio.runtime.blocking_thread_count` | Current number of Tokio blocking pool threads. | Same conditions as `tokio.runtime.worker_count`, in a build compiled with `tokio_unstable`. |
| `tokio.runtime.blocking_thread_idle_count` | Current number of idle Tokio blocking pool threads. | Same conditions as `tokio.runtime.worker_count`, in a build compiled with `tokio_unstable`. |
| `tokio.runtime.worker_local_queue_size` | Current tasks in all worker-local queues. | Same conditions as `tokio.runtime.worker_count`, in a build compiled with `tokio_unstable`. |
| `tokio.runtime.spawned_tasks_count` | Total tasks spawned since runtime creation. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.remote_schedule_count` | Total schedules from outside runtime. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.budget_forced_yield_count` | Total forced cooperative yields. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.worker_noop_count` | Total noop unpark events summed across workers. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.worker_steal_success_count` | Total successful worker steal operations. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.worker_steal_attempt_count` | Total worker steal attempts. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.worker_poll_count` | Total task poll operations across workers. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.worker_local_schedule_count` | Total schedules into worker-local queues. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.worker_overflow_count` | Total worker local-queue overflow events. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.io_driver_fd_registered_count` | Total file descriptors registered in Tokio I/O driver. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.io_driver_fd_deregistered_count` | Total file descriptors deregistered in Tokio I/O driver. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime.io_driver_ready_count` | Total ready events processed by Tokio I/O driver. | Same conditions as `tokio.runtime.worker_count`, with `tokio_unstable` and 64-bit atomics. |

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

Channel metrics and node input/output metrics observe different stages of
a message's lifecycle:

| Metric layer | Recorded when | Operational use |
| --- | --- | --- |
| `channel.sender` / `channel.receiver` | A forward-path send or receive operation completes. | Diagnose edge throughput, queue saturation, backpressure, and closed channels. |
| `node.input` / `node.output` | A terminal ACK or NACK unwinds through the node's route frame. | Attribute logical PData outcomes, durations, item counts, and sizes to nodes. |

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
| `node.metrics.reporting.fail` | `warn` | Reporting node input/output metrics failed (non-fatal). | `crates/engine/src/pipeline_ctrl.rs` |
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
