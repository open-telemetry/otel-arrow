# Self-observability

The OTAP Dataflow Engine records internal metrics and structured events in an
in-memory telemetry registry and exposes them through two paths:

- Pull and live inspection on the admin HTTP server configured by
  `engine.http_admin.bind_address`, which defaults to `127.0.0.1:8080`. The
  server provides an embedded browser UI, Prometheus and JSON metric endpoints,
  live semantic convention schema output, an optional log tap, health probes,
  and CPU and heap profiling routes.
- Push export through the internal observability pipeline under
  `engine.observability.pipeline`. The `receiver:internal_telemetry` node reads
  metrics and log events from the registry on an interval and forwards them as
  OTAP records to exporters such as `exporter:otlp_grpc`, `exporter:otlp_http`,
  or `exporter:console`.

## Embedded debugging UI

When `engine.http_admin` is set, the admin server serves the browser UI at
`GET /` and `GET /dashboard` with static assets under `/static/*`.

- The UI polls `/api/v1/telemetry/metrics?format=json&reset=false` every 2
  seconds without resetting counters. You can toggle `keep_all_zeroes`, pause
  polling to freeze the time window, and scrub across buffered samples.
- Engine summary cards read `memory.rss`, `memory.commit`, and `cpu.utilization`
  from the `engine` metric set, derive displayed uptime from the maximum
  observed `pipeline.uptime`, and count active pipeline groups, pipelines, and
  worker cores from observed metric-set attributes.
- Pipeline and runtime cards read per-core memory, CPU, context switches, and
  Tokio thread statistics from the `pipeline` and `tokio.runtime` metric sets.
  Scope selectors filter by pipeline group, pipeline, and worker core ID, or
  aggregate across all cores when set to `ALL`.
- The DAG topology view renders live edges from `channel.sender` and
  `channel.receiver` metric sets, showing PData throughput, queue depth,
  capacity, and send errors.
- The log viewer queries `GET /api/v1/telemetry/logs` and streams events over
  `GET /api/v1/telemetry/logs/stream` when `engine.telemetry.logs.tap.enabled`
  is `true`.

See [Admin UI Architecture](admin/architecture.md) for UI state and derivation
details.

## Admin HTTP endpoints

The admin HTTP listener has no built-in authentication or TLS and also exposes
control endpoints for [live reconfiguration](admin/live-reconfiguration.md),
config reconciliation, and pipeline shutdown. Keep it bound to loopback or put
it behind an authenticated reverse proxy. See the
[Security and privacy guide](telemetry/security-privacy-guide.md) and
[Admin security notes](../crates/admin/README.md#security).

| Method and path | Description |
| --- | --- |
| `GET /`, `GET /dashboard` | Embedded browser debugging dashboard. |
| `GET /api/v1/metrics`, `GET /api/v1/telemetry/metrics` | Per-entity metrics snapshot, defaulting to Prometheus text format. |
| `GET /api/v1/telemetry/metrics/aggregate` | Cross-entity metrics grouped by `otel_scope_name` and optional `attrs`. |
| `GET /api/v1/telemetry/live-schema` | Live semantic convention registry JSON for registered entities, metric sets, and events. |
| `GET /api/v1/telemetry/logs` | Paginated query over the in-memory log tap buffer. Accepts `after` and `limit`. Returns `404` when the log tap is disabled. |
| `GET /api/v1/telemetry/logs/stream` | WebSocket live stream of internal log tap records. |
| `GET /api/v1/status` | Engine status snapshot, observed state store report, and build info. |
| `GET /api/v1/livez`, `GET /api/v1/readyz` | Engine-wide liveness and readiness probes across deployed pipelines. |
| `GET /api/v1/groups/status` | Pipeline status grouped by pipeline group ID. |
| `GET /api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/status` | Status and conditions for a single logical pipeline. |
| `GET /api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/livez` | Per-pipeline liveness probe. |
| `GET /api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/readyz` | Per-pipeline readiness probe. |
| `GET /api/v1/debug/pprof/heap` | Jemalloc heap profile in pprof protobuf format, `application/x-protobuf`. Requires the `jemalloc` feature, default on Linux and macOS, and runtime activation with `MALLOC_CONF="prof:true,prof_active:true,lg_prof_sample:19"` on Linux or `_RJEM_MALLOC_CONF` on macOS. See [Profiling](../PROFILING.md#live-heap-profiling-jemalloc-pprof). |
| `GET /api/v1/debug/pprof/profile` | Sampled CPU profile in pprof protobuf format, `application/x-protobuf`, on non-Windows targets. Accepts `seconds`, default `30`, and `frequency` in Hz, default `100`. |

### Metrics query parameters

| Endpoint | Parameter | Values | Default | Behavior |
| --- | --- | --- | --- | --- |
| `/metrics`, `/telemetry/metrics`, `/telemetry/metrics/aggregate` | `format` | `prometheus`, `json`, `json_compact`, `line_protocol` | `prometheus` | Response serialization format. |
| `/metrics`, `/telemetry/metrics`, `/telemetry/metrics/aggregate` | `reset` | `true`, `false` | `false` | Resets the shared admin accumulator for synchronous counters and distributions after reading, which affects all admin clients including Prometheus scrapers and the browser UI. Keep `false` for scraping and UI polling. |
| `/metrics`, `/telemetry/metrics` | `keep_all_zeroes` | `true`, `false` | `false` | Retains all-zero metric sets in `json` and `json_compact` outputs. Ignored by `prometheus` and `line_protocol`. |
| `/telemetry/metrics/aggregate` | `attrs` | Comma-separated entity attribute keys | empty | Groups by the listed entity attribute keys, such as `pipeline.id,node.id`. Wildcards like `*` and datapoint attributes like `signal` are not supported. When omitted, each `otel_scope_name` collapses into one group. |

### Scraping with Prometheus

```yaml
scrape_configs:
  - job_name: otap-dataflow-engine
    metrics_path: /api/v1/metrics
    static_configs:
      - targets: ["127.0.0.1:8080"]
```

## Internal observability pipeline and telemetry policies

### Exporting internal telemetry

Configure `engine.observability.pipeline` with `receiver:internal_telemetry` to
push internal metrics and logs to an OTLP backend or console sink:

```yaml
version: otel_dataflow/v1

engine:
  http_admin:
    bind_address: 127.0.0.1:8080
  telemetry:
    resource:
      service.name: otap-dataflow
    logs:
      level: info
      providers:
        engine: its
        admin: console_direct
      tap:
        enabled: true
        max_entries: 10000
        max_bytes: 8388608
  observability:
    pipeline:
      nodes:
        internal:
          type: receiver:internal_telemetry
          config:
            signals: [logs, metrics]
            metrics:
              interval: 10s
        otlp:
          type: exporter:otlp_grpc
          config:
            grpc_endpoint: http://127.0.0.1:4317
      connections:
        - from: internal
          to: otlp
```

When `engine.observability.pipeline` is omitted, the controller installs a
default pipeline that routes `its` logs to `exporter:console` and metrics to
`exporter:noop`.

### Controlling metric verbosity

Set `policies.telemetry` at the top level, pipeline group, or pipeline:

```yaml
policies:
  telemetry:
    pipeline_metrics: true
    tokio_metrics: true
    runtime_metrics: basic
```

`runtime_metrics` defaults to `basic` and controls channel transport, node
boundary outcomes, and shared control-plane metrics:

| Level | Collected runtime instrumentation |
| --- | --- |
| `none` | Disables channel transport metrics, node boundary metrics, and control-plane metrics under `pipeline.completion` and `pipeline.runtime_control`. Component-specific metrics and engine, pipeline, or Tokio metrics remain active. |
| `basic` | Default. Enables `channel.sender` and `channel.receiver` metrics and control-plane state gauges: `pending_sends.buffered`, `drain.active`, `drain.pending_receivers`, `timers.active`, and `telemetry_timers.active`. Enables no node boundary metrics by default. |
| `normal` | Adds node boundary message counters `node.input.messages`, `node.output.messages`, `receiver.received.messages`, and `exporter.attempted.messages`, plus `node.completion_emission` counters and control-plane message and phase counters. |
| `detailed` | Adds per-signal item counts `items`, logical and payload byte sizes `size` and `payload.size`, node completion and local durations `node.completion.duration`, `receiver.processing.duration`, `processor.compute.duration`, and `exporter.attempted.duration`, drain duration summaries, and completion `unwind.depth`. |

Per-node policies `messages`, `item_counts`, `size`, `completion_duration`, and
`duration_distribution` defaults to `normal` and selects aggregation fidelity
for supported node-local duration instruments, including the shared
`receiver.processing.duration` and `exporter.attempted.duration`
instrumentation. `processor.compute.duration` and `node.completion.duration`
currently always use `HistogramNormal`.

For instruments that honor this setting, `basic` uses bucketless `Mmsc`
summaries, while `normal` and `detailed` use `HistogramNormal` and
`HistogramDetailed`, which are exponential histograms with adaptive resolution
whose scale adjusts to fit the observed range. Check `details.relative_error`
in the JSON metrics output to see the resulting precision.

Configure processor range `flow_metrics` under
`policies.telemetry.flow_metrics`. Each flow entry has its own
`duration_distribution` setting for `compute_duration`. See
[Node and flow metrics](node-and-flow-metrics.md).

## Prometheus and OTLP mapping rules

- Resource attributes from `engine.telemetry.resource` and
  `engine.telemetry.detectors` appear once per Prometheus scrape on a
  `target_info` gauge with value `1`. The default detectors are
  `service_instance`, `env`, and `service_name`; `host`, `os`, `process`,
  `container`, and `k8s` require explicit opt-in under
  `engine.telemetry.detectors`.
- Every sample includes `otel_scope_name="<scope>"`. Entity attributes carry the
  `otel_scope_` prefix with dots and dashes converted to underscores, so
  `pipeline.id` becomes `otel_scope_pipeline_id`.
- Datapoint attributes declared on a `#[metric_set]`, such as `signal`,
  `outcome`, `error.type`, or `protocol`, render as Prometheus labels without
  the `otel_scope_` prefix, converting dots to underscores, so `error.type`
  becomes `error_type`.
- Metric names derive from the instrument field name, UCUM unit, and type
  without an `otel_scope_name` prefix. Filter by `otel_scope_name` in PromQL
  when scopes share an instrument name, such as
  `messages_total{otel_scope_name="node.input"}`.
- UCUM units expand to Prometheus unit words: `By` to `bytes`, `s` to `seconds`,
  `ms` to `milliseconds`, `ns` to `nanoseconds`, and `%` to `percent`, emitting
  a `# UNIT` line. Count annotations in braces such as `{item}`, `{message}`,
  or `{batch}` do not add a unit suffix.
- `Counter` and `ObserveCounter` instruments append `_total` and emit
  `# TYPE <metric> counter`. `UpDownCounter`, `ObserveUpDownCounter`, and
  `Gauge` emit `# TYPE <metric> gauge` without `_total`.
- `Mmsc`, `HistogramNormal`, and `HistogramDetailed` distributions emit four
  Prometheus series when observation count is non-zero: `<name>_min` and
  `<name>_max` as `gauge`, and `<name>_sum` and `<name>_count` as `counter`.
  With `?format=json` or `?format=json_compact`, the bucketed tiers
  `HistogramNormal` and `HistogramDetailed` include a `details` object with
  `zero_count`, `relative_error`, and estimated `p50`, `p90`, and `p99`
  quantiles. Raw histogram bucket counts are emitted only through OTLP export.

### Entity attribute sets

| Entity level | Attribute set | Entity attributes with `otel_scope_<key>` in Prometheus |
| --- | --- | --- |
| Process / engine | `engine` | none; process identity is on `target_info` |
| Controller monitor | `controller.monitor.attrs` | `extension.id` |
| Worker core | `controller.attrs` | `core.id`, `numa.node.id` |
| Pipeline instance | `pipeline.attrs` | `pipeline.group.id`, `pipeline.id`, `deployment.generation`, `core.id`, `numa.node.id` |
| Node instance | `node.attrs` | `node.id`, `node.urn`, `node.type`, plus `pipeline.attrs` and optional `topic`, `protocol`, or `custom` attributes |
| Node channel endpoint | `node.channel.attrs` | `channel.id`, `node.port`, `channel.kind`, `channel.mode`, `channel.type`, `channel.impl`, plus `node.attrs` |
| Extension instance | `extension.attrs` | `extension.id`, `extension.variant`, `scope.kind`, plus `pipeline.attrs` |
| Extension channel endpoint | `extension.channel.attrs` | `channel.id`, `channel.mode`, `channel.impl`, plus `extension.attrs` |
| Processor flow range | `flow.attrs` | `flow.id`, `flow.node.start`, `flow.node.end`, `flow.purpose`, `flow.node.decision`, plus `pipeline.attrs` |

## Metrics reference

### Engine, pipeline, and runtime

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `admission.rate_limiter` | `refusals` | `refusals_total` | `Counter<u64>` | `{decision}` | `dimension`, `refusal` | Admission attempts declined or observed as over limit. |
| `controller.monitor` | `observed_pipelines.live` | `observed_pipelines_live` | `Gauge<u64>` | `{pipeline}` | none | Logical pipelines currently marked live. |
| `controller.monitor` | `observed_pipelines.ready` | `observed_pipelines_ready` | `Gauge<u64>` | `{pipeline}` | none | Logical pipelines currently marked ready. |
| `controller.monitor` | `observed_pipelines.total` | `observed_pipelines_total` | `Gauge<u64>` | `{pipeline}` | none | Logical pipelines tracked by the controller. |
| `controller.monitor` | `telemetry.entities` | `telemetry_entities` | `Gauge<u64>` | `{entity}` | none | Registered telemetry entities. |
| `controller.monitor` | `telemetry.metric_sets` | `telemetry_metric_sets` | `Gauge<u64>` | `{metric_set}` | none | Registered telemetry metric sets. |
| `engine` | `cpu.utilization` | `cpu_utilization` | `Gauge<f64>` | `{1}` | none | Process CPU utilization in `[0, 1]`, normalized across all logical system cores. |
| `engine` | `memory.commit` | `memory_commit_bytes` | `ObserveUpDownCounter<u64>` | `By` | none | Process private committed memory, including paged-out memory. |
| `engine` | `memory.pressure.state` | `memory_pressure_state` | `Gauge<u64>` | `{state}` | none | Process memory limiter state: `0=normal`, `1=soft`, `2=hard`. |
| `engine` | `memory.rss` | `memory_rss` | `ObserveUpDownCounter<u64>` | `{By}` | none | Process resident set size in physical RAM. |
| `engine` | `process.memory.hard.limit.bytes` | `process_memory_hard_limit_bytes` | `Gauge<u64>` | `{By}` | none | Process memory limiter hard limit in bytes. |
| `engine` | `process.memory.soft.limit.bytes` | `process_memory_soft_limit_bytes` | `Gauge<u64>` | `{By}` | none | Process memory limiter soft limit in bytes. |
| `engine` | `process.memory.usage.bytes` | `process_memory_usage_bytes` | `Gauge<u64>` | `{By}` | none | Most recent process memory limiter sample in bytes. |
| `engine.console_output` | `bytes.written` | `bytes_written_bytes_total` | `ObserveCounter<u64>` | `By` | `console.stream` | Bytes written to the console stream. |
| `engine.console_output` | `diagnostics.dropped` | `diagnostics_dropped_total` | `ObserveCounter<u64>` | `{frame}` | `console.stream` | Diagnostics dropped due to full queue or byte budget limit. |
| `engine.console_output` | `frames.enqueue.failed` | `frames_enqueue_failed_total` | `ObserveCounter<u64>` | `{frame}` | `console.stream` | Frames rejected due to closed/full queue, oversize frame, or failed writer. |
| `engine.console_output` | `frames.submitted` | `frames_submitted_total` | `ObserveCounter<u64>` | `{frame}` | `console.stream` | Frames accepted into the stream queue. |
| `engine.console_output` | `frames.written` | `frames_written_total` | `ObserveCounter<u64>` | `{frame}` | `console.stream` | Frames written to the stream. |
| `engine.console_output` | `write.errors` | `write_errors_total` | `ObserveCounter<u64>` | `{error}` | `console.stream` | Failed stream writes or flushes. |
| `pipeline` | `context.switches.involuntary` | `context_switches_involuntary_total` | `ObserveCounter<u64>` | `{1}` | none | Involuntary scheduler preemptions of the pipeline thread. |
| `pipeline` | `context.switches.voluntary` | `context_switches_voluntary_total` | `ObserveCounter<u64>` | `{1}` | none | Voluntary CPU yields by the pipeline thread. |
| `pipeline` | `cpu.time` | `cpu_time_total` | `Counter<f64>` | `{s}` | none | CPU seconds consumed by the pipeline thread. |
| `pipeline` | `cpu.utilization` | `cpu_utilization` | `Gauge<f64>` | `{1}` | none | Pipeline thread CPU utilization ratio in `[0, 1]`. |
| `pipeline` | `memory.allocated` | `memory_allocated_total` | `ObserveCounter<u64>` | `{By}` | none | Cumulative heap bytes allocated by the pipeline instance. |
| `pipeline` | `memory.allocated.delta` | `memory_allocated_delta_total` | `Counter<u64>` | `{By}` | none | Heap bytes allocated during the last measurement window. |
| `pipeline` | `memory.freed` | `memory_freed_total` | `ObserveCounter<u64>` | `{By}` | none | Cumulative heap bytes freed by the pipeline instance. |
| `pipeline` | `memory.freed.delta` | `memory_freed_delta_total` | `Counter<u64>` | `{By}` | none | Heap bytes freed during the last measurement window. |
| `pipeline` | `memory.usage` | `memory_usage` | `ObserveUpDownCounter<u64>` | `{By}` | none | Current heap bytes in use by the pipeline instance on this core. |
| `pipeline` | `page.faults.major` | `page_faults_major_total` | `ObserveCounter<u64>` | `{1}` | none | Major page faults requiring disk I/O. |
| `pipeline` | `page.faults.minor` | `page_faults_minor_total` | `ObserveCounter<u64>` | `{1}` | none | Minor page faults served without disk I/O. |
| `pipeline` | `uptime` | `uptime` | `Gauge<f64>` | `{s}` | none | Pipeline instance uptime in seconds. |
| `pipeline.completion` | `ack.attempted` | `ack_attempted_total` | `Counter<u64>` | `{message}` | none | Ack completions routed toward an interested upstream node. |
| `pipeline.completion` | `ack.delivered` | `ack_delivered_total` | `Counter<u64>` | `{message}` | none | Ack control messages delivered to upstream nodes. |
| `pipeline.completion` | `ack.dropped_no_interest` | `ack_dropped_no_interest_total` | `Counter<u64>` | `{message}` | none | Ack completions dropped because no upstream frame subscribed. |
| `pipeline.completion` | `deliver_ack.received` | `deliver_ack_received_total` | `Counter<u64>` | `{message}` | none | `DeliverAck` messages received by the completion dispatcher. |
| `pipeline.completion` | `deliver_nack.received` | `deliver_nack_received_total` | `Counter<u64>` | `{message}` | none | `DeliverNack` messages received by the completion dispatcher. |
| `pipeline.completion` | `nack.attempted` | `nack_attempted_total` | `Counter<u64>` | `{message}` | none | Nack completions routed toward an interested upstream node. |
| `pipeline.completion` | `nack.delivered` | `nack_delivered_total` | `Counter<u64>` | `{message}` | none | Nack control messages delivered to upstream nodes. |
| `pipeline.completion` | `nack.dropped_no_interest` | `nack_dropped_no_interest_total` | `Counter<u64>` | `{message}` | none | Nack completions dropped because no upstream frame subscribed. |
| `pipeline.completion` | `pending_sends.buffered` | `pending_sends_buffered` | `Gauge<u64>` | `{message}` | none | Completion deliveries buffered for retry on full node-control channels. |
| `pipeline.completion` | `unwind.depth` | `unwind_depth_{min,max,sum,count}` | `Mmsc` | `{frame}` | none | Unwind stack frames traversed per completion outcome. |
| `pipeline.runtime_control` | `cancel_telemetry_timer.received` | `cancel_telemetry_timer_received_total` | `Counter<u64>` | `{message}` | none | Telemetry timer cancel requests received by the runtime manager. |
| `pipeline.runtime_control` | `cancel_timer.received` | `cancel_timer_received_total` | `Counter<u64>` | `{message}` | none | `CancelTimer` control messages received by the runtime manager. |
| `pipeline.runtime_control` | `collect_telemetry.sent` | `collect_telemetry_sent_total` | `Counter<u64>` | `{message}` | none | `CollectTelemetry` control messages sent to nodes. |
| `pipeline.runtime_control` | `downstream_shutdown.sent` | `downstream_shutdown_sent_total` | `Counter<u64>` | `{message}` | none | Downstream `Shutdown` fan-out operations sent after receiver drain. |
| `pipeline.runtime_control` | `drain.active` | `drain_active` | `Gauge<u64>` | `{1}` | none | `1` while the runtime is in graceful drain mode, else `0`. |
| `pipeline.runtime_control` | `drain.pending_receivers` | `drain_pending_receivers` | `Gauge<u64>` | `{node}` | none | Receivers that have not yet reported `ReceiverDrained`. |
| `pipeline.runtime_control` | `drain.receiver_phase_duration_ns` | `drain_receiver_phase_duration_ns_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Time spent waiting for receivers to finish draining. |
| `pipeline.runtime_control` | `drain.total_duration_ns` | `drain_total_duration_ns_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Total graceful shutdown duration for the runtime. |
| `pipeline.runtime_control` | `drain_ingress.sent` | `drain_ingress_sent_total` | `Counter<u64>` | `{message}` | none | `DrainIngress` operations sent to receivers. |
| `pipeline.runtime_control` | `pending_sends.buffered` | `pending_sends_buffered` | `Gauge<u64>` | `{message}` | none | Node-control deliveries buffered for retry by the runtime manager. |
| `pipeline.runtime_control` | `receiver_drained.received` | `receiver_drained_received_total` | `Counter<u64>` | `{message}` | none | `ReceiverDrained` notifications received from receivers. |
| `pipeline.runtime_control` | `shutdown.deadline_forced` | `shutdown_deadline_forced_total` | `Counter<u64>` | `{message}` | none | Shutdowns forced at the deadline before natural drain completion. |
| `pipeline.runtime_control` | `shutdown.received` | `shutdown_received_total` | `Counter<u64>` | `{message}` | none | `Shutdown` requests accepted by the runtime manager. |
| `pipeline.runtime_control` | `start_telemetry_timer.received` | `start_telemetry_timer_received_total` | `Counter<u64>` | `{message}` | none | Telemetry timer start requests received by the runtime manager. |
| `pipeline.runtime_control` | `start_timer.received` | `start_timer_received_total` | `Counter<u64>` | `{message}` | none | `StartTimer` control messages received by the runtime manager. |
| `pipeline.runtime_control` | `telemetry_timers.active` | `telemetry_timers_active` | `Gauge<u64>` | `{timer}` | none | Active telemetry collection timers. |
| `pipeline.runtime_control` | `timer_tick.sent` | `timer_tick_sent_total` | `Counter<u64>` | `{message}` | none | `TimerTick` control messages sent to nodes. |
| `pipeline.runtime_control` | `timers.active` | `timers_active` | `Gauge<u64>` | `{timer}` | none | Active per-node timers. |
| `tokio.runtime` | `blocking.task.queue.size` | `blocking_task_queue_size` | `ObserveUpDownCounter<u64>` | `{task}` | none | Tasks queued in the blocking thread pool. |
| `tokio.runtime` | `blocking.thread.count` | `blocking_thread_count` | `ObserveUpDownCounter<u64>` | `{thread}` | none | Threads created for the blocking thread pool. |
| `tokio.runtime` | `blocking.thread.idle.count` | `blocking_thread_idle_count` | `ObserveUpDownCounter<u64>` | `{thread}` | none | Idle threads in the blocking thread pool. |
| `tokio.runtime` | `budget.forced.yield.count` | `budget_forced_yield_count_total` | `ObserveCounter<u64>` | `{yield}` | none | Tasks forced to yield after exhausting their cooperative budget. |
| `tokio.runtime` | `global.task.queue.size` | `global_task_queue_size` | `ObserveUpDownCounter<u64>` | `{task}` | none | Tasks pending in the runtime global injection queue. |
| `tokio.runtime` | `io.driver.fd.deregistered.count` | `io_driver_fd_deregistered_count_total` | `ObserveCounter<u64>` | `{fd}` | none | File descriptors deregistered from the I/O driver. |
| `tokio.runtime` | `io.driver.fd.registered.count` | `io_driver_fd_registered_count_total` | `ObserveCounter<u64>` | `{fd}` | none | File descriptors registered with the I/O driver. |
| `tokio.runtime` | `io.driver.ready.count` | `io_driver_ready_count_total` | `ObserveCounter<u64>` | `{ready}` | none | Ready events processed by the I/O driver. |
| `tokio.runtime` | `remote.schedule.count` | `remote_schedule_count_total` | `ObserveCounter<u64>` | `{remote}` | none | Tasks scheduled from outside the runtime. |
| `tokio.runtime` | `spawned.tasks.count` | `spawned_tasks_count_total` | `ObserveCounter<u64>` | `{spawn}` | none | Tasks spawned in the runtime since creation. |
| `tokio.runtime` | `task.active.count` | `task_active_count` | `ObserveUpDownCounter<u64>` | `{task}` | none | Alive tasks in the runtime. |
| `tokio.runtime` | `worker.busy.time` | `worker_busy_time_total` | `ObserveCounter<f64>` | `{s}` | none | Cumulative worker busy time in seconds across runtime workers. |
| `tokio.runtime` | `worker.count` | `worker_count` | `ObserveUpDownCounter<u64>` | `{thread}` | none | Worker threads used by the Tokio runtime. |
| `tokio.runtime` | `worker.local.queue.size` | `worker_local_queue_size` | `ObserveUpDownCounter<u64>` | `{task}` | none | Tasks pending in worker-local queues. |
| `tokio.runtime` | `worker.local.schedule.count` | `worker_local_schedule_count_total` | `ObserveCounter<u64>` | `{schedule}` | none | Tasks scheduled onto worker-local queues. |
| `tokio.runtime` | `worker.noop.count` | `worker_noop_count_total` | `ObserveCounter<u64>` | `{noop}` | none | Worker unpark events that found no work before parking again. |
| `tokio.runtime` | `worker.overflow.count` | `worker_overflow_count_total` | `ObserveCounter<u64>` | `{overflow}` | none | Worker local-queue overflows in the multi-threaded scheduler. |
| `tokio.runtime` | `worker.park.count` | `worker_park_count_total` | `ObserveCounter<u64>` | `{park}` | none | Times runtime workers parked while idle. |
| `tokio.runtime` | `worker.park.unpark.count` | `worker_park_unpark_count_total` | `ObserveCounter<u64>` | `{unpark}` | none | Worker park and unpark transitions. |
| `tokio.runtime` | `worker.poll.count` | `worker_poll_count_total` | `ObserveCounter<u64>` | `{poll}` | none | Task polls across runtime workers. |
| `tokio.runtime` | `worker.steal.attempt.count` | `worker_steal_attempt_count_total` | `ObserveCounter<u64>` | `{steal}` | none | Work-stealing attempts across workers. |
| `tokio.runtime` | `worker.steal.success.count` | `worker_steal_success_count_total` | `ObserveCounter<u64>` | `{steal}` | none | Successful work-stealing operations across workers. |

Within `tokio.runtime`, `worker.count`, `task.active.count`,
`global.task.queue.size`, `worker.busy.time`, `worker.park.count`, and
`worker.park.unpark.count` are available without `tokio_unstable`. All other
`tokio.runtime` instruments listed above require a build with
`RUSTFLAGS="--cfg tokio_unstable" cargo build --release`, run from
`rust/otap-dataflow`. The cumulative Tokio runtime counters also require
64-bit atomics. Enabling `tokio_metrics` or increasing `runtime_metrics`
does not enable instruments omitted at compile time.

### Channels, nodes, and flows

On `channel.sender` and `channel.receiver`, the `channel.kind` entity attribute
distinguishes `pdata` channels, which carry the `signal` datapoint attribute,
from `control` channels. See [Node and flow metrics](node-and-flow-metrics.md)
for node boundary and flow semantics.

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `channel.receiver` | `capacity` | `capacity` | `Gauge<u64>` | `{message}` | none | Configured channel buffer capacity. |
| `channel.receiver` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | none | Control messages dequeued from the channel. |
| `channel.receiver` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal` | PData messages dequeued from the channel. |
| `channel.receiver` | `queue.depth` | `queue_depth` | `Gauge<u64>` | `{message}` | none | Messages currently buffered in the channel. |
| `channel.sender` | `failures` | `failures_total` | `Counter<u64>` | `{message}` | `error.type` | Failed or refused control-message send attempts. |
| `channel.sender` | `failures` | `failures_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | Failed or refused PData send attempts. |
| `channel.sender` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `outcome` | Completed immediate control-message send attempts. |
| `channel.sender` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Completed immediate PData send attempts. |
| `flow.compute` | `duration` | `duration_seconds_{min,max,sum,count}` | `Mmsc` / `HistogramNormal` / `HistogramDetailed` | `s` | `signal` | Aggregate processor compute duration across the flow range. |
| `flow.dropped` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Signal items dropped by a decision processor in the flow range. |
| `flow.input` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Signal items entering the flow range. |
| `flow.input` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal` | PData messages entering the flow range. |
| `flow.input` | `size` | `size_bytes_total` | `Counter<u64>` | `By` | `signal` | Logical payload bytes entering the flow range. |
| `flow.output` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Signal items leaving the flow range. |
| `flow.output` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal` | PData messages leaving the flow range. |
| `flow.output` | `size` | `size_bytes_total` | `Counter<u64>` | `By` | `signal` | Logical payload bytes leaving the flow range. |
| `node.completion` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal`, `outcome` | Duration from tracked node boundary until terminal Ack or Nack routing. |
| `node.completion_emission` | `notify_ack.routed` | `notify_ack_routed_total` | `Counter<u64>` | `{message}` | none | `notify_ack` calls routed into the completion lane. |
| `node.completion_emission` | `notify_nack.routed` | `notify_nack_routed_total` | `Counter<u64>` | `{message}` | none | `notify_nack` calls routed into the completion lane. |
| `node.input` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal`, `outcome` | Input signal items at the node boundary. |
| `node.input` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Input PData messages at the node boundary. |
| `node.input` | `size` | `size_bytes_total` | `Counter<u64>` | `By` | `signal`, `outcome` | Input logical payload bytes at the node boundary. |
| `node.local_wakeup.scheduler` | `cancel.ignored_shutdown` | `cancel_ignored_shutdown_total` | `Counter<u64>` | `{attempt}` | none | Wakeup cancel attempts ignored after shutdown was latched. |
| `node.local_wakeup.scheduler` | `cancel.missed` | `cancel_missed_total` | `Counter<u64>` | `{attempt}` | none | Wakeup cancel attempts that found no live slot. |
| `node.local_wakeup.scheduler` | `cancel.removed` | `cancel_removed_total` | `Counter<u64>` | `{wakeup}` | none | Wakeup cancel attempts that removed a live wakeup. |
| `node.local_wakeup.scheduler` | `capacity` | `capacity` | `Gauge<u64>` | `{wakeup}` | none | Configured live wakeup slot capacity. |
| `node.local_wakeup.scheduler` | `live` | `live` | `Gauge<u64>` | `{wakeup}` | none | Current live wakeups. |
| `node.local_wakeup.scheduler` | `pop.due` | `pop_due_total` | `Counter<u64>` | `{wakeup}` | none | Due wakeups popped for processor delivery. |
| `node.local_wakeup.scheduler` | `set.error_capacity` | `set_error_capacity_total` | `Counter<u64>` | `{attempt}` | none | Wakeup set attempts rejected due to full slot capacity. |
| `node.local_wakeup.scheduler` | `set.error_shutdown` | `set_error_shutdown_total` | `Counter<u64>` | `{attempt}` | none | Wakeup set attempts rejected after shutdown was latched. |
| `node.local_wakeup.scheduler` | `set.inserted` | `set_inserted_total` | `Counter<u64>` | `{wakeup}` | none | Newly inserted wakeups. |
| `node.local_wakeup.scheduler` | `set.replaced` | `set_replaced_total` | `Counter<u64>` | `{wakeup}` | none | Wakeups that replaced an existing live slot. |
| `node.local_wakeup.scheduler` | `shutdown.cleared` | `shutdown_cleared_total` | `Counter<u64>` | `{wakeup}` | none | Live wakeups cleared when shutdown was latched. |
| `node.output` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal`, `outcome` | Output signal items at the node boundary. |
| `node.output` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Output PData messages at the node boundary. |
| `node.output` | `size` | `size_bytes_total` | `Counter<u64>` | `By` | `signal`, `outcome` | Output logical payload bytes at the node boundary. |
| `processor.compute` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `outcome` | Wall-clock duration of processor-local synchronous compute. |

### Component-specific and boundary metrics

Receiver, processor, exporter, and extension metric sets are documented in the
`Telemetry` section of each component's README under the
[core-node catalog](../crates/core-nodes/README.md),
[contrib-node catalog](../crates/contrib-nodes/README.md),
[contrib-extension catalog](../crates/contrib-extensions/README.md), and
[dev-node catalog](../crates/dev-nodes/README.md). For shared receiver and
exporter boundary instruments, see
[Node and flow metrics](node-and-flow-metrics.md) and the
[System metrics guide](telemetry/metrics-guide.md#shared-receiver-and-exporter-boundary-metrics).
Query `GET /api/v1/telemetry/live-schema` on a running engine to inspect the
exact metric sets registered by that build.
