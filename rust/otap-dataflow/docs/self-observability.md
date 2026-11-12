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

Set `policies.telemetry` at the top level, pipeline group, pipeline, or node:

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
`duration` enable those measurements at any `runtime_metrics` level.
`duration_distribution` selects histogram fidelity separately and defaults to
`normal`, which uses `HistogramNormal` at scale 3 with 32 buckets; `basic` uses
bucketless `Mmsc` summaries and `detailed` uses `HistogramDetailed` at scale 5
with 160 buckets. Configure processor range `flow_metrics` under
`policies.telemetry.flow_metrics`. See
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

### Receivers

Includes shared receiver boundary instruments, core receivers, contrib
receivers, database scrapers, and traffic generator receivers.

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `receiver.database` | `acks` | `acks_total` | `Counter<u64>` | `{ack}` | none | Downstream acknowledgements matching an in-flight page. |
| `receiver.database` | `batches.sent` | `batches_sent_total` | `Counter<u64>` | `{batch}` | none | Pages sent downstream. |
| `receiver.database` | `cancellations` | `cancellations_total` | `Counter<u64>` | `{cancellation}` | none | Active database operations cancelled by a control message. |
| `receiver.database` | `checkpoint.cleanup.failures` | `checkpoint_cleanup_failures_total` | `Counter<u64>` | `{failure}` | none | Stale checkpoint revisions that could not be removed. |
| `receiver.database` | `checkpoint.commits` | `checkpoint_commits_total` | `Counter<u64>` | `{commit}` | none | Durable checkpoint commits. |
| `receiver.database` | `checkpoint.failures` | `checkpoint_failures_total` | `Counter<u64>` | `{failure}` | none | Durable checkpoint write failures. |
| `receiver.database` | `drains` | `drains_total` | `Counter<u64>` | `{drain}` | none | Drain requests observed. |
| `receiver.database` | `encoded.bytes.sent` | `encoded_bytes_sent_bytes_total` | `Counter<u64>` | `By` | none | Encoded OTLP bytes sent downstream. |
| `receiver.database` | `event.time.fallbacks` | `event_time_fallbacks_total` | `Counter<u64>` | `{record}` | none | Records whose source event time exceeded OTLP timestamp bounds. |
| `receiver.database` | `nacks` | `nacks_total` | `Counter<u64>` | `{nack}` | none | Downstream NACKs matching an in-flight page. |
| `receiver.database` | `polls` | `polls_total` | `Counter<u64>` | `{poll}` | none | Bounded page polls attempted against the database. |
| `receiver.database` | `query.failures` | `query_failures_total` | `Counter<u64>` | `{failure}` | none | Failed query-page attempts. |
| `receiver.database` | `reconnects` | `reconnects_total` | `Counter<u64>` | `{attempt}` | none | Reconnect operations started after transient database failures. |
| `receiver.database` | `rejection.paused` | `rejection_paused` | `Gauge<u64>` | `1` | none | `1` while polling is paused by permanent rejection, else `0`. |
| `receiver.database` | `replays` | `replays_total` | `Counter<u64>` | `{replay}` | none | Replay schedules created after a NACK. |
| `receiver.database` | `rows.sent` | `rows_sent_total` | `Counter<u64>` | `{row}` | none | Database rows sent downstream. |
| `receiver.database` | `shutdowns` | `shutdowns_total` | `Counter<u64>` | `{shutdown}` | none | Shutdown requests observed. |
| `receiver.database` | `stale.feedback` | `stale_feedback_total` | `Counter<u64>` | `{feedback}` | none | Discarded malformed, wrong-generation, stale, or duplicate feedback. |
| `receiver.database` | `starts` | `starts_total` | `Counter<u64>` | `{start}` | none | Receiver starts. |
| `receiver.etw.decode` | `errors` | `errors_total` | `Counter<u64>` | `{event}` | `signal` | ETW events that failed TDH decoding. |
| `receiver.etw.drops` | `events` | `events_total` | `Counter<u64>` | `{event}` | `reason`, `signal` | Dropped ETW events. |
| `receiver.etw.events` | `forwarded` | `forwarded_total` | `Counter<u64>` | `{event}` | `signal` | ETW events forwarded downstream in batches. |
| `receiver.etw.events` | `received` | `received_total` | `Counter<u64>` | `{event}` | `signal` | ETW events received by the session before channel enqueue. |
| `receiver.etw.session` | `buffers.lost` | `buffers_lost_total` | `Counter<u64>` | `{buffer}` | `delivery`, `signal` | Kernel ETW buffers lost. |
| `receiver.etw.session` | `buffers.written` | `buffers_written_total` | `Counter<u64>` | `{buffer}` | `signal` | Kernel ETW buffers written by the session. |
| `receiver.etw.session` | `events.lost` | `events_lost_total` | `Counter<u64>` | `{event}` | `signal` | Events lost inside kernel ETW buffers before delivery. |
| `receiver.host_metrics` | `batches.sent` | `batches_sent_total` | `Counter<u64>` | `{batch}` | none | Metrics batches sent downstream. |
| `receiver.host_metrics` | `families.scraped` | `families_scraped_total` | `Counter<u64>` | `{family}` | none | Due metric families processed. |
| `receiver.host_metrics` | `partial.errors` | `partial_errors_total` | `Counter<u64>` | `{error}` | none | Source read errors skipped because other families succeeded. |
| `receiver.host_metrics` | `scrape.duration.ns` | `scrape_duration_ns_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Wall-clock scrape duration. |
| `receiver.host_metrics` | `scrape.lag.ns` | `scrape_lag_ns_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Delay between scheduled and actual scrape start. |
| `receiver.host_metrics` | `scraper.failures` | `scraper_failures_total` | `Counter<u64>` | `{error}` | none | Scraper worker failures. |
| `receiver.host_metrics` | `scrapes.completed` | `scrapes_completed_total` | `Counter<u64>` | `{scrape}` | none | Scrape ticks that built and sent a metrics batch. |
| `receiver.host_metrics` | `scrapes.failed` | `scrapes_failed_total` | `Counter<u64>` | `{scrape}` | none | Fatal scrape failures. |
| `receiver.host_metrics` | `scrapes.overrun` | `scrapes_overrun_total` | `Counter<u64>` | `{scrape}` | none | Scrape ticks skipped or timed out before completion. |
| `receiver.host_metrics` | `scrapes.started` | `scrapes_started_total` | `Counter<u64>` | `{scrape}` | none | Scrape ticks started. |
| `receiver.host_metrics` | `send.failures` | `send_failures_total` | `Counter<u64>` | `{error}` | none | Downstream send failures. |
| `receiver.host_metrics` | `source.read.errors` | `source_read_errors_total` | `Counter<u64>` | `{error}` | none | Source read errors observed during scrapes. |
| `receiver.journald.acknowledgements` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `outcome` | Downstream acknowledgement responses. |
| `receiver.journald.acknowledgements` | `rewinds` | `rewinds_total` | `Counter<u64>` | `{rewind}` | `outcome` | Cursor rewinds triggered by NACKs. |
| `receiver.journald.checkpoints` | `commits` | `commits_total` | `Counter<u64>` | `{commit}` | `outcome` | Durable cursor commits. |
| `receiver.journald.lifecycle` | `transitions` | `transitions_total` | `Counter<u64>` | `{transition}` | `transition.type` | Journald receiver lifecycle transitions. |
| `receiver.journald.output` | `batches` | `batches_total` | `Counter<u64>` | `{batch}` | none | Log batches emitted downstream. |
| `receiver.journald.output` | `dropped.fields` | `dropped_fields_total` | `Counter<u64>` | `{field}` | none | Journald fields dropped by extraction limits. |
| `receiver.journald.output` | `records` | `records_total` | `Counter<u64>` | `{record}` | none | Log records emitted downstream. |
| `receiver.journald.source` | `events` | `events_total` | `Counter<u64>` | `{event}` | `error.type` | Journald source read failures. |
| `receiver.kafka.acknowledgements` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `signal`, `outcome` | Downstream acknowledgement responses. |
| `receiver.kafka.consumer` | `group.feedback.after_revocation` | `group_feedback_after_revocation_total` | `Counter<u64>` | `{response}` | none | Ack or Nack responses ignored due to stale partition ownership. |
| `receiver.kafka.consumer` | `group.lag` | `group_lag` | `Gauge<f64>` | `{message}` | none | Mean broker-committed consumer-group lag across owned partitions. |
| `receiver.kafka.consumer` | `group.partition.assignments` | `group_partition_assignments_total` | `Counter<u64>` | `{partition}` | none | Partitions assigned to this consumer across rebalances. |
| `receiver.kafka.consumer` | `group.partition.revocations` | `group_partition_revocations_total` | `Counter<u64>` | `{partition}` | none | Owned partitions revoked across rebalances. |
| `receiver.kafka.consumer` | `group.partitions` | `group_partitions` | `ObserveUpDownCounter<u64>` | `{partition}` | none | Partitions currently owned by this consumer. |
| `receiver.kafka.consumer` | `group.rebalance.commit_enqueue_failures` | `group_rebalance_commit_enqueue_failures_total` | `Counter<u64>` | `{error}` | none | Async commit-before-revoke calls that failed local enqueue. |
| `receiver.kafka.consumer` | `group.rebalance.resume_failures` | `group_rebalance_resume_failures_total` | `Counter<u64>` | `{error}` | none | Partition resume operations that failed after rebalance pause. |
| `receiver.kafka.consumer` | `group.rebalances` | `group_rebalances_total` | `Counter<u64>` | `{rebalance}` | none | Consumer-group assignment events. |
| `receiver.kafka.consumer` | `records.bytes` | `records_bytes_total` | `Counter<u64>` | `By` | none | Kafka record payload bytes delivered before filtering or decoding. |
| `receiver.kafka.consumer` | `records.duplicates` | `records_duplicates_total` | `Counter<u64>` | `{message}` | none | Messages skipped because offsets were already tracked in this generation. |
| `receiver.kafka.consumer` | `records.inflight` | `records_inflight` | `ObserveUpDownCounter<u64>` | `{message}` | none | Delivered records awaiting acknowledgement and offset commit. |
| `receiver.kafka.consumer` | `records.received` | `records_received_total` | `Counter<u64>` | `{message}` | none | Kafka records delivered before filtering or decoding. |
| `receiver.kafka.consumer` | `retry.feedback.stale` | `retry_feedback_stale_total` | `Counter<u64>` | `{response}` | none | Feedback ignored from an obsolete replay generation. |
| `receiver.kafka.consumer` | `retry.partitions_paused` | `retry_partitions_paused` | `ObserveUpDownCounter<u64>` | `{partition}` | none | Partitions paused during transient-NACK recovery. |
| `receiver.kafka.consumer` | `retry.pause_failures` | `retry_pause_failures_total` | `Counter<u64>` | `{error}` | none | Partition pause failures during transient-NACK recovery. |
| `receiver.kafka.consumer` | `retry.replay_attempts` | `retry_replay_attempts_total` | `Counter<u64>` | `{attempt}` | none | Replay attempts started after transient-NACK backoff. |
| `receiver.kafka.consumer` | `retry.resume_failures` | `retry_resume_failures_total` | `Counter<u64>` | `{error}` | none | Partition resume failures during transient-NACK recovery. |
| `receiver.kafka.consumer` | `retry.seek_failures` | `retry_seek_failures_total` | `Counter<u64>` | `{error}` | none | Partition seek failures during transient-NACK recovery. |
| `receiver.kafka.consumer` | `retry.transient_nacks` | `retry_transient_nacks_total` | `Counter<u64>` | `{response}` | none | Non-permanent NACK responses received from downstream. |
| `receiver.kafka.messages` | `completed` | `completed_total` | `Counter<u64>` | `{message}` | `signal` | Admitted messages whose receiver work terminated. |
| `receiver.kafka.messages` | `payload.size` | `payload_size_bytes_total` | `Counter<u64>` | `By` | `signal` | Encoded Kafka payload bytes admitted to the pipeline send path. |
| `receiver.kafka.messages` | `started` | `started_total` | `Counter<u64>` | `{message}` | `signal` | Decoded messages admitted to the pipeline send path. |
| `receiver.kafka.offset_commits` | `commits` | `commits_total` | `Counter<u64>` | `{commit}` | `outcome` | Offset commit results reported by librdkafka. |
| `receiver.kafka.rejections` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type`, `reason` | Rejected Kafka messages. |
| `receiver.kafka.transport` | `errors` | `errors_total` | `Counter<u64>` | `{error}` | `error.type` | Non-EOF errors returned by the Kafka consumer. |
| `receiver.messages` | `bytes` | `bytes_total` | `Counter<u64>` | `By` | `signal` | Encoded transport payload bytes admitted to the pipeline send path. |
| `receiver.messages` | `completed` | `completed_total` | `Counter<u64>` | `{message}` | `signal` | Admitted messages whose receiver work terminated. |
| `receiver.messages` | `started` | `started_total` | `Counter<u64>` | `{message}` | `signal` | Decoded messages admitted to the pipeline send path. |
| `receiver.microsoft_json` | `compressed.requests` | `compressed_requests_total` | `Counter<u64>` | `{request}` | none | HTTP requests received with a compressed body. |
| `receiver.microsoft_json` | `config.endpoints.failed` | `config_endpoints_failed_total` | `Counter<u64>` | `{error}` | none | Endpoint configuration files that failed startup loading. |
| `receiver.microsoft_json` | `config.endpoints.loaded` | `config_endpoints_loaded_total` | `Counter<u64>` | `{item}` | none | Endpoint configuration files loaded at startup. |
| `receiver.microsoft_json` | `cors.preflight.requests` | `cors_preflight_requests_total` | `Counter<u64>` | `{request}` | none | CORS `OPTIONS` preflight requests handled. |
| `receiver.microsoft_json` | `events.oversized` | `events_oversized_total` | `Counter<u64>` | `{item}` | none | Events rejected for exceeding the endpoint size limit. |
| `receiver.microsoft_json` | `ikey.ignored` | `ikey_ignored_total` | `Counter<u64>` | `{item}` | none | Events silently dropped by the iKey filter. |
| `receiver.microsoft_json` | `ikey.rejected` | `ikey_rejected_total` | `Counter<u64>` | `{item}` | none | Events rejected by the iKey filter. |
| `receiver.microsoft_json` | `ikey.unauthorized` | `ikey_unauthorized_total` | `Counter<u64>` | `{item}` | none | Events rejected due to disallowed origin. |
| `receiver.microsoft_json` | `received.logs.invalid` | `received_logs_invalid_total` | `Counter<u64>` | `{item}` | none | JSON events that failed parsing or endpoint validation. |
| `receiver.otap.acknowledgements` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `signal`, `outcome` | Routed or invalid OTAP acknowledgement responses. |
| `receiver.otap.batches` | `completed` | `completed_total` | `Counter<u64>` | `{batch}` | `signal` | Admitted OTAP batches whose receiver work terminated. |
| `receiver.otap.batches` | `payload.size` | `payload_size_bytes_total` | `Counter<u64>` | `By` | `signal` | Protobuf-encoded batch bytes after gRPC decompression. |
| `receiver.otap.batches` | `started` | `started_total` | `Counter<u64>` | `{batch}` | `signal` | OTAP batches admitted to the pipeline send path. |
| `receiver.otap.rejections` | `batches` | `batches_total` | `Counter<u64>` | `{batch}` | `error.type` | Rejected OTAP batches within admitted streams. |
| `receiver.otap.rejections` | `streams` | `streams_total` | `Counter<u64>` | `{stream}` | `error.type` | Rejected OTAP streaming RPCs. |
| `receiver.otlp.acknowledgements` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `signal`, `outcome` | Routed or invalid OTLP acknowledgement responses. |
| `receiver.otlp.requests` | `accepted` | `accepted_total` | `Counter<u64>` | `{request}` | `signal`, `protocol` | OTLP requests admitted to the pipeline send path. |
| `receiver.otlp.requests` | `rejected` | `rejected_total` | `Counter<u64>` | `{request}` | `protocol`, `error.type` | Rejected OTLP requests. |
| `receiver.otlp.transport` | `errors` | `errors_total` | `Counter<u64>` | `{error}` | `protocol` | Transport-level OTLP server errors. |
| `receiver.processing` | `duration` | `duration_seconds_{min,max,sum,count}` | `Mmsc` / `HistogramNormal` / `HistogramDetailed` | `s` | none | Receiver-local processing duration. |
| `receiver.pulled` | `attempts` | `attempts_total` | `Counter<u64>` | `{attempt}` | `outcome` | Accepted pull collection attempts reaching a terminal local outcome. |
| `receiver.received` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Classified external messages whose receiver-local handling terminated. |
| `receiver.received` | `payload.size` | `payload_size_bytes_total` | `Counter<u64>` | `By` | `signal`, `outcome` | Encoded application payload bytes observed before receiver decoding. |
| `receiver.syslog_cef.connections` | `active` | `active` | `UpDownCounter<u64>` | `{connection}` | none | Active syslog/CEF connections. |
| `receiver.syslog_cef.connections` | `rejected` | `rejected_total` | `Counter<u64>` | `{connection}` | none | Rejected syslog/CEF connections. |
| `receiver.syslog_cef.rejections` | `items` | `items_total` | `Counter<u64>` | `{item}` | `protocol`, `error.type`, `signal` | Rejected syslog/CEF items. |
| `receiver.syslog_cef.transport` | `errors` | `errors_total` | `Counter<u64>` | `{error}` | `protocol` | Transport-level server errors. |
| `receiver.syslog_cef.truncations` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Truncated syslog/CEF payloads. |
| `receiver.topic` | `downstream.backpressure.events` | `downstream_backpressure_events_total` | `Counter<u64>` | `{event}` | none | Downstream backpressure events blocked for at least 500ms. |
| `receiver.topic` | `lagged.messages` | `lagged_messages_total` | `Counter<u64>` | `{message}` | none | Messages missed across broadcast lag notifications. |
| `receiver.topic.bridge` | `controls` | `controls_total` | `Counter<u64>` | `{control}` | `control`, `result` | Topic bridge control operations. |
| `receiver.topic.downstream.blocked` | `duration` | `duration_seconds_{min,max,sum,count}` | `Mmsc` / `HistogramNormal` / `HistogramDetailed` | `s` | none | Time blocked forwarding a message downstream. |
| `receiver.topic.lag` | `events` | `events_total` | `Counter<u64>` | `{event}` | `event.type` | Topic lag events. |
| `receiver.traffic_generator` | `completion.drain_deadline_forced` | `completion_drain_deadline_forced_total` | `Counter<u64>` | `{drain}` | none | Drains forced to finish with unresolved batches at deadline. |
| `receiver.traffic_generator` | `completion.pending` | `completion_pending` | `Gauge<u64>` | `{batch}` | none | Subscribed batches awaiting Ack/Nack completion. |
| `receiver.traffic_generator` | `smooth.batch_interval` | `smooth_batch_interval_nanoseconds` | `Gauge<u64>` | `ns` | none | Configured smooth-mode interval between batches. |
| `receiver.traffic_generator` | `smooth.batch_tick_lateness_duration` | `smooth_batch_tick_lateness_duration_seconds_{min,max,sum,count}` | `Mmsc` | `s` | none | Lateness of smooth-mode batch ticks relative to schedule. |
| `receiver.traffic_generator` | `smooth.late_remaining_batches` | `smooth_late_remaining_batches_{min,max,sum,count}` | `Mmsc` | `{batch}` | none | Batches remaining when a smooth-mode run is late. |
| `receiver.traffic_generator` | `smooth.late_remaining_items` | `smooth_late_remaining_items_{min,max,sum,count}` | `Mmsc` | `{item}` | none | Signal items remaining when a smooth-mode run is late. |
| `receiver.traffic_generator` | `smooth.payload_generate_duration` | `smooth_payload_generate_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | none | Time spent generating or cloning one smooth-mode payload. |
| `receiver.traffic_generator` | `smooth.payload_send_duration` | `smooth_payload_send_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | none | Time spent sending one smooth-mode payload downstream. |
| `receiver.traffic_generator` | `smooth.run_batches` | `smooth_run_batches` | `Gauge<u64>` | `{batch}` | none | Configured smooth-mode batches per one-second run. |
| `receiver.traffic_generator` | `smooth.runs_started` | `smooth_runs_started_total` | `Counter<u64>` | `{run}` | none | Smooth-mode production runs started. |
| `receiver.traffic_generator` | `smooth.schedule_deadline_misses` | `smooth_schedule_deadline_misses_total` | `Counter<u64>` | `{miss}` | none | Missed smooth-mode scheduling boundaries. |
| `receiver.traffic_generator.smooth.payload.sends` | `attempts` | `attempts_total` | `Counter<u64>` | `{attempt}` | `attempt`, `outcome` | Smooth-mode payload send attempts. |
| `receiver.traffic_generator.smooth.runs` | `terminations` | `terminations_total` | `Counter<u64>` | `{run}` | `outcome` | Terminated smooth-mode production runs. |
| `receiver.user_events` | `downstream.send.blocked.ns` | `downstream_send_blocked_ns_nanoseconds_total` | `Counter<u64>` | `ns` | none | Time spent waiting for downstream channel capacity. |
| `receiver.user_events` | `flushed.batches` | `flushed_batches_total` | `Counter<u64>` | `{event}` | none | Arrow batches flushed downstream. |
| `receiver.user_events` | `forwarded.samples` | `forwarded_samples_total` | `Counter<u64>` | `{item}` | none | Perf samples forwarded downstream. |
| `receiver.user_events` | `late.registration.retries` | `late_registration_retries_total` | `Counter<u64>` | `{event}` | none | Late-registration retries while waiting for tracepoints. |
| `receiver.user_events` | `lost.perf.samples` | `lost_perf_samples_total` | `Counter<u64>` | `{item}` | none | Lost samples reported by the perf ring. |
| `receiver.user_events` | `received.samples` | `received_samples_total` | `Counter<u64>` | `{item}` | none | Perf samples read from the kernel ring. |
| `receiver.user_events` | `sessions.started` | `sessions_started_total` | `Counter<u64>` | `{event}` | none | Receiver sessions started. |
| `receiver.user_events.drops` | `samples` | `samples_total` | `Counter<u64>` | `{item}` | `reason` | Dropped user_events samples. |
| `receiver.windows_perf_counters` | `active.counters` | `active_counters` | `Gauge<u64>` | `{counter}` | none | Counters currently added to a PDH query. |
| `receiver.windows_perf_counters` | `batches.dropped` | `batches_dropped_total` | `Counter<u64>` | `{batch}` | none | Batches dropped because the downstream channel was full or closed. |
| `receiver.windows_perf_counters` | `counter.add.failures` | `counter_add_failures_total` | `Counter<u64>` | `{failure}` | none | Failed counter additions, including retries. |
| `receiver.windows_perf_counters` | `scrape.duration` | `scrape_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | none | Scrape duration from request to report. |
| `receiver.windows_perf_counters` | `scrape.failures` | `scrape_failures_total` | `Counter<u64>` | `{scrape}` | none | Scrapes or startup queries where at least one counter failed. |
| `receiver.windows_perf_counters` | `scrape.overruns` | `scrape_overruns_total` | `Counter<u64>` | `{tick}` | none | Ticks skipped because the previous scrape was still running. |
| `receiver.windows_perf_counters` | `scrapes` | `scrapes_total` | `Counter<u64>` | `{scrape}` | none | Scrapes in which at least one counter query succeeded. |

### Processors

Includes core processors, contrib processors, and the WebAssembly processor
host.

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `otap.processor.batch` | `batching.errors` | `batching_errors_total` | `Counter<u64>` | `{error}` | none | Batches that encountered batching errors. |
| `otap.processor.batch` | `dropped.conversion` | `dropped_conversion_total` | `Counter<u64>` | `{msg}` | none | Messages dropped due to conversion failures. |
| `otap.processor.batch` | `flush.age.duration` | `flush_age_duration_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Time from first pending input arrival to flush start. |
| `otap.processor.batch` | `flush.output.batches` | `flush_output_batches_{min,max,sum,count}` | `Mmsc` | `{batch}` | none | Output batches handed downstream per flush. |
| `otap.processor.batch` | `flush.output.bytes` | `flush_output_bytes_{min,max,sum,count}` | `Mmsc` | `By` | none | Output payload bytes handed downstream per flush. |
| `otap.processor.batch` | `flush.pending.bytes` | `flush_pending_bytes_{min,max,sum,count}` | `Mmsc` | `By` | none | Bytes pending at flush time. |
| `otap.processor.batch` | `flush.pending.requests` | `flush_pending_requests_{min,max,sum,count}` | `Mmsc` | `{request}` | none | Input requests pending at flush time. |
| `otap.processor.batch` | `flush.timer.lateness.duration` | `flush_timer_lateness_duration_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Delay between scheduled timer wakeup and timer flush start. |
| `otap.processor.batch` | `flushes.size` | `flushes_size_total` | `Counter<u64>` | `{flush}` | none | Flushes triggered by size threshold. |
| `otap.processor.batch` | `flushes.timer` | `flushes_timer_total` | `Counter<u64>` | `{flush}` | none | Flushes triggered by timer. |
| `otap.processor.batch` | `nacked.inbound.slots` | `nacked_inbound_slots_total` | `Counter<u64>` | `{msg}` | none | Requests NACKed due to inbound slot exhaustion. |
| `otap.processor.batch` | `nacked.outbound.slots` | `nacked_outbound_slots_total` | `Counter<u64>` | `{msg}` | none | Requests NACKed due to outbound slot exhaustion. |
| `otap.processor.batch` | `passthrough.batches` | `passthrough_batches_total` | `Counter<u64>` | `{batch}` | none | Inputs forwarded as-is because their size already fell within bounds. |
| `otap.processor.batch` | `split.budget.fallbacks` | `split_budget_fallbacks_total` | `Counter<u64>` | `{entry}` | none | Oversize resource entries emitted whole when splitting exceeded fragment budget. |
| `processor.attributes` | `transforms` | `transforms_total` | `Counter<u64>` | `{transform}` | `outcome` | Attribute transform attempts by outcome. |
| `processor.attributes.modified` | `entries` | `entries_total` | `Counter<u64>` | `{attr}` | `action`, `domain` | Attribute entries modified by action and domain. |
| `processor.content_router` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `outcome`, `reason` | Routed messages by outcome and reason. |
| `processor.debug` | `consumed.events` | `consumed_events_total` | `Counter<u64>` | `{event}` | `signal` | Log events or span events consumed. |
| `processor.debug` | `consumed.links` | `consumed_links_total` | `Counter<u64>` | `{link}` | `signal` | Span links consumed. |
| `processor.durable_buffer` | `flush.failures` | `flush_failures_total` | `Counter<u64>` | `{error}` | none | Segment finalization failures. |
| `processor.durable_buffer` | `in.flight` | `in_flight` | `Gauge<u64>` | `{bundle}` | none | Bundles currently in flight downstream. |
| `processor.durable_buffer` | `read.errors` | `read_errors_total` | `Counter<u64>` | `{error}` | none | Persistent storage read errors. |
| `processor.durable_buffer` | `retries.scheduled` | `retries_scheduled_total` | `Counter<u64>` | `{retry}` | none | Retry attempts scheduled. |
| `processor.durable_buffer` | `storage.bytes.cap` | `storage_bytes_cap_bytes` | `Gauge<u64>` | `By` | none | Configured persistent storage byte cap. |
| `processor.durable_buffer` | `storage.bytes.used` | `storage_bytes_used_bytes` | `Gauge<u64>` | `By` | none | Persistent storage bytes currently used. |
| `processor.durable_buffer` | `storage.utilization` | `storage_utilization` | `Gauge<f64>` | `{1}` | none | Persistent storage utilization ratio. |
| `processor.durable_buffer.bundles` | `resolved` | `resolved_total` | `Counter<u64>` | `{bundle}` | `outcome` | Bundles resolved by downstream outcome. |
| `processor.durable_buffer.ingest` | `failures` | `failures_total` | `Counter<u64>` | `{failure}` | `failure` | Failed durable ingest attempts. |
| `processor.durable_buffer.items` | `consumed` | `consumed_total` | `Counter<u64>` | `{item}` | `signal` | Items ingested into durable storage. |
| `processor.durable_buffer.items` | `produced` | `produced_total` | `Counter<u64>` | `{item}` | `signal` | Items sent downstream from durable storage. |
| `processor.durable_buffer.items` | `queued` | `queued` | `Gauge<u64>` | `{item}` | `signal` | Items currently queued in durable storage. |
| `processor.durable_buffer.items` | `rejected` | `rejected_total` | `Counter<u64>` | `{item}` | `signal` | Items in permanently rejected bundles. |
| `processor.durable_buffer.items` | `requeued` | `requeued_total` | `Counter<u64>` | `{item}` | `signal` | Items requeued for retry after a NACK. |
| `processor.durable_buffer.loss` | `bundles` | `bundles_total` | `Counter<u64>` | `{bundle}` | `reason` | Unresolved bundles lost to retention limits. |
| `processor.durable_buffer.loss` | `bytes` | `bytes_total` | `Counter<u64>` | `By` | `reason` | Logical payload bytes lost to retention limits. |
| `processor.durable_buffer.loss` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal`, `reason` | Items lost to retention limits. |
| `processor.durable_buffer.reclaimed` | `bytes` | `bytes_total` | `Counter<u64>` | `By` | `reason` | Persisted bytes reclaimed from removed segment files. |
| `processor.durable_buffer.reclaimed` | `segments` | `segments_total` | `Counter<u64>` | `{segment}` | `reason` | Physical segment files removed. |
| `processor.fanout` | `active` | `active` | `Gauge<u64>` | `{message}` | none | Active pipeline messages tracked by fanout. |
| `processor.fanout` | `max.inflight.config` | `max_inflight_config` | `Gauge<u64>` | `{message}` | none | Configured `max_inflight` limit, where `0` means unlimited. |
| `processor.fanout` | `throttle.episodes` | `throttle_episodes_total` | `Counter<u64>` | `{episode}` | none | Transitions from not-throttled to throttled. |
| `processor.fanout` | `throttled` | `throttled` | `Gauge<u64>` | `1` | none | `1` while fanout is refusing new PData, else `0`. |
| `processor.fanout` | `timed.out` | `timed_out_total` | `Counter<u64>` | `{message}` | `signal`, `destination` | Destination message attempts that timed out. |
| `processor.filter` | `dropped.items` | `dropped_items_total` | `Counter<u64>` | `{item}` | `signal` | Signal items dropped by filter rules. |
| `processor.log_parser` | `failures` | `failures_total` | `Counter<u64>` | `{operation}` | `error.type` | Failed local log parsing operations. |
| `processor.log_parser` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `signal`, `outcome` | Input messages processed by log parser. |
| `processor.log_parser` | `records` | `records_total` | `Counter<u64>` | `{record}` | `format`, `reason` | Preserved malformed records and observed-time fallbacks. |
| `processor.log_sampling` | `dropped.items` | `dropped_items_total` | `Counter<u64>` | `{item}` | `signal` | Log records dropped by sampling. |
| `processor.log_sampling` | `filter.buffer.reclamation.failures` | `filter_buffer_reclamation_failures_total` | `Counter<u64>` | `{error}` | `signal` | Filter buffer reclamation failures. |
| `processor.log_sampling` | `filtering.errors` | `filtering_errors_total` | `Counter<u64>` | `{error}` | `signal` | Errors encountered while filtering OTAP batches. |
| `processor.partition` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `outcome` | Partitioning operations attempted. |
| `processor.resource_validator` | `failures` | `failures_total` | `Counter<u64>` | `{batch}` | `error.type` | Batches that failed resource validation. |
| `processor.retry` | `messages.recovered` | `messages_recovered_total` | `Counter<u64>` | `{message}` | `signal` | PData messages accepted downstream after at least one retry. |
| `processor.retry` | `retries.scheduled` | `retries_scheduled_total` | `Counter<u64>` | `{retry}` | `signal` | Retries scheduled after downstream refusal. |
| `processor.retry.messages` | `terminated` | `terminated_total` | `Counter<u64>` | `{message}` | `signal`, `reason` | PData messages that exhausted retries. |
| `processor.temporal_reaggregation` | `failures` | `failures_total` | `Counter<u64>` | `{operation}` | `error.type` | Failed temporal reaggregation input operations. |
| `processor.temporal_reaggregation` | `flushes` | `flushes_total` | `Counter<u64>` | `{flush}` | `outcome`, `reason` | Non-empty reaggregation flushes by outcome and reason. |
| `processor.temporal_reaggregation` | `metrics` | `metrics_total` | `Counter<u64>` | `{metric}` | `type`, `aggregable`, `temporality` | Input metric records by type, aggregability, and temporality. |
| `processor.temporal_reaggregation` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `outcome` | Temporal reaggregation input operations by outcome. |
| `processor.transform` | `failures` | `failures_total` | `Counter<u64>` | `{operation}` | `signal`, `error.type`, `language` | Failed transform operations. |
| `processor.transform` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `signal`, `outcome`, `language` | Transform operations reaching a terminal local outcome. |
| `processor.type_router` | `signals.decision` | `signals_decision_total` | `Counter<u64>` | `{message}` | `outcome`, `reason`, `signal` | Type-routed messages by signal, outcome, and reason. |
| `processor.wasm_processor.pdata` | `guest.counter.add.calls` | `guest_counter_add_calls_total` | `Counter<u64>` | `{item}` | none | Accepted guest `counter-add` calls. |
| `processor.wasm_processor.pdata` | `guest.counter.add.rejected.cardinality` | `guest_counter_add_rejected_cardinality_total` | `Counter<u64>` | `{item}` | none | Guest `counter-add` calls rejected due to distinct-counter cardinality cap. |
| `processor.wasm_processor.pdata` | `guest.counter.add.rejected.name.len` | `guest_counter_add_rejected_name_len_total` | `Counter<u64>` | `{item}` | none | Guest `counter-add` calls rejected due to counter name length cap. |
| `processor.wasm_processor.pdata` | `guest.counter.add.value` | `guest_counter_add_value_total` | `ObserveCounter<u64>` | `{item}` | none | Cumulative sum of values accepted through guest `counter-add` calls. |
| `processor.wasm_processor.pdata` | `guest.host.service.calls.rejected` | `guest_host_service_calls_rejected_total` | `Counter<u64>` | `{item}` | none | Guest `log`, `counter-add`, or `get-config` calls rejected by rate limiter. |
| `processor.wasm_processor.pdata` | `guest.log.message.truncated` | `guest_log_message_truncated_total` | `Counter<u64>` | `{item}` | none | Guest log messages truncated to maximum retained length. |
| `processor.wasm_processor.pdata` | `guest.process.calls` | `guest_process_calls_total` | `Counter<u64>` | `{item}` | none | Guest `process` calls attempted. |
| `processor.wasm_processor.pdata` | `guest.process.errors` | `guest_process_errors_total` | `Counter<u64>` | `{item}` | none | Guest `process` calls that failed or trapped. |
| `processor.wasm_processor.pdata` | `kernel.calls` | `kernel_calls_total` | `Counter<u64>` | `{item}` | none | Host kernel invocations dispatched by the guest. |
| `processor.wasm_processor.pdata` | `pdata.dropped` | `pdata_dropped_total` | `Counter<u64>` | `{item}` | none | PData messages dropped when guest `process` returned `none`. |
| `processor.wasm_processor.pdata.records` | `records.in` | `records_in_total` | `Counter<u64>` | `{item}` | `signal` | Records entering the WASM guest. |
| `processor.wasm_processor.pdata.records` | `records.out` | `records_out_total` | `Counter<u64>` | `{item}` | `signal` | Records leaving the WASM guest after filtering. |

### Exporters and extensions

Includes shared exporter boundary instruments, core exporters, contrib
exporters, validation exporters, and pipeline extensions.

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `exporter.attempted` | `duration` | `duration_seconds_{min,max,sum,count}` | `Mmsc` / `HistogramNormal` / `HistogramDetailed` | `s` | `signal`, `outcome` | Export attempt duration including backend latency. |
| `exporter.attempted` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal`, `outcome` | Signal items handled across export attempts. |
| `exporter.attempted` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Node-local export delivery attempts, including shared retries. |
| `exporter.attempted` | `payload.size` | `payload_size_bytes_total` | `Counter<u64>` | `By` | `signal`, `outcome` | Encoded application payload bytes submitted across export attempts. |
| `exporter.azure_data_explorer` | `batches` | `batches_total` | `Counter<u64>` | `{batch}` | `signal`, `outcome` | Logical ADX batches reaching a terminal outcome. |
| `exporter.azure_data_explorer.http` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `response` | ADX HTTP export attempts by response category. |
| `exporter.azure_monitor` | `batch_size` | `batch_size_bytes_{min,max,sum,count}` | `Mmsc` | `By` | `outcome`, `signal` | Compressed batch size in bytes. |
| `exporter.azure_monitor` | `batch_uncompressed_size` | `batch_uncompressed_size_bytes_{min,max,sum,count}` | `Mmsc` | `By` | `outcome`, `signal` | Uncompressed batch size in bytes. |
| `exporter.azure_monitor` | `batches` | `batches_total` | `Counter<u64>` | `{batch}` | `outcome`, `signal` | Completed Azure Monitor batches. |
| `exporter.azure_monitor` | `in_flight_exports` | `in_flight_exports` | `Gauge<u64>` | `{export}` | none | In-flight Azure Monitor export requests. |
| `exporter.azure_monitor` | `in_flight_log_records` | `in_flight_log_records` | `Gauge<u64>` | `{log}` | none | Log records in flight or awaiting retry at the exporter. |
| `exporter.azure_monitor` | `log_entries_too_large` | `log_entries_too_large_total` | `Counter<u64>` | `{entry}` | none | Log entries rejected for exceeding the batch size limit. |
| `exporter.azure_monitor.heartbeats` | `sends` | `sends_total` | `Counter<u64>` | `{heartbeat}` | `outcome` | Completed heartbeat send attempts. |
| `exporter.azure_monitor.http` | `latency` | `latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | `response` | HTTP export attempt latency in milliseconds. |
| `exporter.azure_monitor.http` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `response` | HTTP export attempts by response category. |
| `exporter.azure_monitor.state` | `mappings` | `mappings` | `Gauge<u64>` | `{entry}` | `mapping` | Entries in the exporter state mapping. |
| `exporter.clickhouse.batches` | `transformed` | `transformed_total` | `Counter<u64>` | `{batch}` | `path` | Log batches transformed by path. |
| `exporter.clickhouse.rows` | `written` | `written_total` | `Counter<u64>` | `{row}` | `signal` | Rows written into ClickHouse. |
| `exporter.console.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type`, `format` | PData messages that failed console formatting or write. |
| `exporter.exports` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal`, `outcome` | Duration from PData dequeue to terminal export result. |
| `exporter.exports` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Messages whose export reached a terminal outcome. |
| `exporter.file` | `bytes` | `bytes_total` | `Counter<u64>` | `By` | `signal` | Frame bytes written, including delimiters. |
| `exporter.file` | `failures` | `failures_total` | `Counter<u64>` | `{failure}` | `signal`, `operation` | File open, write, sync, or rollback failures. |
| `exporter.file` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Signal items written to file frames before ACK routing. |
| `exporter.file` | `tail.recovered.bytes` | `tail_recovered_bytes_total` | `Counter<u64>` | `By` | `signal` | Bytes removed by append-tail recovery. |
| `exporter.file` | `tail.recoveries` | `tail_recoveries_total` | `Counter<u64>` | `{recovery}` | `signal` | Repaired append-mode partial tails. |
| `exporter.file.exports` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | File export messages reaching a terminal outcome. |
| `exporter.geneva.encoding` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal`, `outcome` | Time spent encoding and compressing Geneva signal data. |
| `exporter.geneva.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | Failed Geneva processing operations. |
| `exporter.geneva.skipped` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `reason` | Skipped Geneva PData messages. |
| `exporter.kafka.exports` | `bytes` | `bytes_{min,max,sum,count}` | `HistogramNormal` | `By` | `signal`, `outcome` | Encoded Kafka payload bytes reaching the producer. |
| `exporter.kafka.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | Failed Kafka export attempts. |
| `exporter.kafka.operations` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal`, `operation`, `outcome` | Time spent in the selected Kafka exporter phase. |
| `exporter.kafka.routing` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `topic.source` | Messages routed by topic source. |
| `exporter.otap.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | OTAP export messages that failed by error type. |
| `exporter.otap.streams` | `correlation.depth` | `correlation_depth_{min,max,sum,count}` | `HistogramNormal` | `{batch}` | `signal` | Response correlation queue depth before enqueueing a batch. |
| `exporter.otap.streams` | `correlation.enqueue.duration` | `correlation_enqueue_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal` | Time spent enqueueing a batch into the response correlation queue. |
| `exporter.otap.streams` | `encode.duration` | `encode_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal` | Time spent encoding an OTAP batch into outbound Arrow records. |
| `exporter.otap.streams` | `enqueue.depth` | `enqueue_depth_{min,max,sum,count}` | `HistogramNormal` | `{batch}` | `signal` | Per-signal stream task queue depth before enqueueing a batch. |
| `exporter.otap.streams` | `enqueue.duration` | `enqueue_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal` | Time spent waiting to enqueue a batch into the stream task. |
| `exporter.otap.streams` | `response.active` | `response_active_{min,max,sum,count}` | `HistogramNormal` | `{batch}` | `signal` | Yielded batches awaiting a matching server response. |
| `exporter.otap.streams` | `response.wait.duration` | `response_wait_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal` | Time spent waiting for the next server response on an OTAP stream. |
| `exporter.otlp_grpc.authentication` | `ready` | `ready` | `UpDownCounter<u64>` | `{1}` | `source` | `1` when authenticated OTLP/gRPC progress is possible, else `0`. |
| `exporter.otlp_grpc.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | OTLP/gRPC export messages that failed by error type. |
| `exporter.otlp_http.authentication` | `ready` | `ready` | `UpDownCounter<u64>` | `{1}` | `source` | `1` when authenticated OTLP/HTTP progress is possible, else `0`. |
| `exporter.otlp_http.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | OTLP/HTTP export messages that failed by error type. |
| `exporter.parquet.file` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `operation` | Parquet file lifecycle and flush operations. |
| `exporter.parquet.rows` | `written` | `written_total` | `Counter<u64>` | `{row}` | none | Rows appended to Parquet writers. |
| `exporter.topic` | `tracked.in.flight` | `tracked_in_flight` | `Gauge<u64>` | `{message}` | none | Admitted tracked topic publishes awaiting a terminal result. |
| `exporter.topic.rejections` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `reason` | Refused topic publishes by reason. |
| `exporter.topic.tracked` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `result` | Admitted tracked topic publishes by terminal result. |
| `exporter.validation` | `check.failed` | `check_failed_total` | `Counter<u64>` | `{check}` | none | Validation checks that did not match expectation. |
| `exporter.validation` | `check.passed` | `check_passed_total` | `Counter<u64>` | `{check}` | none | Validation checks that matched expectation. |
| `exporter.validation` | `finished` | `finished` | `Gauge<u64>` | `{state}` | none | `1` after idle timeout and final validation, else `0`. |
| `exporter.validation` | `valid` | `valid` | `Gauge<u64>` | `{input}` | none | Last validation comparison result: `1` for valid, `0` for invalid. |
| `extension.azure_identity_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed Azure Identity credential acquisitions. |
| `extension.azure_identity_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful credential acquisitions. |
| `extension.azure_identity_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful Azure Identity credential acquisitions. |
| `extension.azure_identity_auth` | `auth.token.publish` | `auth_token_publish_total` | `Counter<u64>` | `{token}` | none | Tokens published to consumers over the watch channel. |
| `extension.azure_key_vault_sasl_auth` | `acquisition.failures` | `acquisition_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed Key Vault SASL credential acquisitions. |
| `extension.azure_key_vault_sasl_auth` | `acquisition.success.latency` | `acquisition_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Successful Key Vault SASL acquisition duration. |
| `extension.azure_key_vault_sasl_auth` | `acquisition.successes` | `acquisition_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful Key Vault SASL credential acquisitions. |
| `extension.azure_key_vault_sasl_auth` | `credential.publishes` | `credential_publishes_total` | `Counter<u64>` | `{credential}` | none | Complete credentials published to consumers. |
| `extension.azure_key_vault_user_pass_auth` | `acquisition.failures` | `acquisition_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed Key Vault username/password acquisitions. |
| `extension.azure_key_vault_user_pass_auth` | `acquisition.success.latency` | `acquisition_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Successful Key Vault username/password acquisition duration. |
| `extension.azure_key_vault_user_pass_auth` | `acquisition.successes` | `acquisition_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful Key Vault username/password acquisitions. |
| `extension.azure_key_vault_user_pass_auth` | `credential.publishes` | `credential_publishes_total` | `Counter<u64>` | `{credential}` | none | Complete credentials published to consumers. |
| `extension.flat_file_api_key_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed flat-file API key acquisitions. |
| `extension.flat_file_api_key_auth` | `auth.publishes` | `auth_publishes_total` | `Counter<u64>` | `{credential}` | none | API keys published to consumers. |
| `extension.flat_file_api_key_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful API key acquisitions. |
| `extension.flat_file_api_key_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful flat-file API key acquisitions. |
| `extension.flat_file_sasl_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed flat-file SASL credential acquisitions. |
| `extension.flat_file_sasl_auth` | `auth.publishes` | `auth_publishes_total` | `Counter<u64>` | `{credential}` | none | SASL credentials published to consumers. |
| `extension.flat_file_sasl_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful SASL credential acquisitions. |
| `extension.flat_file_sasl_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful flat-file SASL credential acquisitions. |
| `extension.flat_file_user_pass_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed flat-file username/password acquisitions. |
| `extension.flat_file_user_pass_auth` | `auth.publishes` | `auth_publishes_total` | `Counter<u64>` | `{credential}` | none | Username/password credentials published to consumers. |
| `extension.flat_file_user_pass_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful username/password acquisitions. |
| `extension.flat_file_user_pass_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful flat-file username/password acquisitions. |
| `extension.gcp_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed GCP token acquisitions. |
| `extension.gcp_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful GCP token acquisitions. |
| `extension.gcp_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful GCP token acquisitions. |
| `extension.gcp_auth` | `auth.token.publish` | `auth_token_publish_total` | `Counter<u64>` | `{token}` | none | GCP tokens published to consumers. |
| `extension.lifecycle` | `completed.cancelled` | `completed_cancelled_total` | `Counter<u64>` | `{completion}` | none | Extension tasks terminated by cancellation. |
| `extension.lifecycle` | `completed.error` | `completed_error_total` | `Counter<u64>` | `{completion}` | none | Extension tasks terminated with an error. |
| `extension.lifecycle` | `completed.ok` | `completed_ok_total` | `Counter<u64>` | `{completion}` | none | Extension tasks that exited cleanly. |
| `extension.lifecycle` | `completed.panic` | `completed_panic_total` | `Counter<u64>` | `{completion}` | none | Extension tasks terminated by a panic. |
| `extension.lifecycle` | `shutdown.sent` | `shutdown_sent_total` | `Counter<u64>` | `{shutdown}` | none | Shutdown signals sent to extension tasks. |
| `extension.lifecycle` | `shutdown.timeout` | `shutdown_timeout_total` | `Counter<u64>` | `{timeout}` | none | Extension shutdowns that timed out before completion. |
| `extension.lifecycle` | `spawned` | `spawned_total` | `Counter<u64>` | `{spawn}` | none | Extension tasks spawned. |
| `extension.lifecycle` | `state` | `state` | `Gauge<u64>` | `{state}` | none | Current extension runtime lifecycle state code. |
| `extension.oauth2_client_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed OAuth2 token acquisitions. |
| `extension.oauth2_client_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful OAuth2 token acquisitions. |
| `extension.oauth2_client_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful OAuth2 token acquisitions. |
| `extension.oauth2_client_auth` | `auth.token.publish` | `auth_token_publish_total` | `Counter<u64>` | `{token}` | none | OAuth2 tokens published to consumers. |
