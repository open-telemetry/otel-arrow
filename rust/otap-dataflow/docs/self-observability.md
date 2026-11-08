# Self-observability

This guide covers self-observability for the OTAP Dataflow Engine, including
the embedded debugging UI, administrative HTTP endpoints, internal
observability pipeline configuration, Prometheus and OTLP mapping rules, and a
complete catalog of internal metrics emitted across engine crates.

## Overview

The engine collects internal metrics and structured events in a lock-free,
NUMA-aware telemetry registry and exposes them through two paths:

- Pull and interactive inspection over the admin HTTP server configured via
  `engine.http_admin.bind_address`, which defaults to `127.0.0.1:8080`. The
  admin server hosts an embedded debugging UI, Prometheus and JSON metric
  endpoints, live semantic convention schema inspection, an optional internal
  log tap, health probes, and CPU and heap profiling routes.
- Push export through the internal observability pipeline configured under
  `engine.observability.pipeline`. The `receiver:internal_telemetry` node reads
  metrics and internal log events from the registry on a configurable interval
  and forwards them as OTAP records to any configured exporter, such as
  `exporter:otlp_grpc`, `exporter:otlp_http`, or `exporter:console`.

## Embedded debugging UI

When `engine.http_admin` is enabled, the admin HTTP server serves an embedded
browser debugging UI at `GET /` and `GET /dashboard` with static assets under
`/static/*`.

To open the UI locally while `df_engine` is running with the default admin
configuration, browse to `http://127.0.0.1:8080/`.

### Dashboard capabilities

- Live polling and playback controls poll
  `/api/v1/telemetry/metrics?format=json&reset=false` every 2 seconds without
  resetting counters. Operators can toggle `keep_all_zeroes` to include or hide
  zero-valued instruments, pause live polling to freeze the buffered time
  window, and scrub across historical samples with synchronized crosshair hover
  across charts.
- Engine summary cards read `memory.rss`, `memory.commit`, and
  `cpu.utilization` from the `engine` metric set, derive displayed uptime from
  the maximum observed `pipeline.uptime`, and count active pipeline groups,
  pipelines, and worker cores from observed metric-set attributes.
- Pipeline and runtime cards display per-core pipeline memory usage, CPU time,
  context switches, and Tokio worker thread statistics from the `pipeline` and
  `tokio.runtime` metric sets.
- Scope selectors filter the view by pipeline group ID, pipeline ID, and worker
  core ID. Selecting `ALL` aggregates metrics across all worker cores assigned
  to the selected pipeline, while selecting a single core ID isolates one
  single-threaded engine worker.
- Interactive channel DAG topology renders the live pipeline graph from
  `channel.sender` and `channel.receiver` metric sets. Edges show PData
  throughput, queue depth, capacity, and send errors. Operators can overlay
  control channels, filter out zero-activity channels, search nodes by ID or
  URN, and click any node or channel edge to inspect its attached metric sets
  and time-series charts in the side panel.
- Internal log inspection queries the in-memory log tap via
  `GET /api/v1/telemetry/logs` and streams live structured events over the
  WebSocket endpoint `GET /api/v1/telemetry/logs/stream` when
  `engine.telemetry.logs.tap.enabled` is set to `true`.

## Admin HTTP endpoints for self-observability

The admin server binds to `engine.http_admin.bind_address`. The table below
lists all endpoints related to metrics, logs, status, and profiling.

<!-- markdownlint-disable MD013 -->

| Method and path | Description |
| --- | --- |
| `GET /`, `GET /dashboard` | Embedded browser debugging dashboard. |
| `GET /api/v1/metrics` | Per-entity metrics snapshot, defaulting to Prometheus text format. |
| `GET /api/v1/telemetry/metrics` | Canonical per-entity metrics snapshot endpoint with the same handler and query parameters as `/api/v1/metrics`. |
| `GET /api/v1/telemetry/metrics/aggregate` | Cross-entity aggregated metrics grouped by `otel_scope_name` and optional `attrs`. |
| `GET /api/v1/telemetry/live-schema` | Live semantic convention registry JSON describing registered entities, metric sets, and events. |
| `GET /api/v1/telemetry/logs` | Paginated query over the in-memory internal log tap buffer. Accepts `after` and `limit`. Returns `404` when the log tap is disabled. |
| `GET /api/v1/telemetry/logs/stream` | WebSocket live stream of internal log tap records. |
| `GET /api/v1/status` | Full engine status snapshot, observed state store report, and build information. |
| `GET /api/v1/livez` | Engine-wide liveness probe across all deployed pipelines. |
| `GET /api/v1/readyz` | Engine-wide readiness probe across all deployed pipelines. |
| `GET /api/v1/groups/status` | Pipeline status grouped by pipeline group ID. |
| `GET /api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/status` | Status and condition details for a single logical pipeline. |
| `GET /api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/livez` | Per-pipeline liveness probe. |
| `GET /api/v1/groups/{pipeline_group_id}/pipelines/{pipeline_id}/readyz` | Per-pipeline readiness probe. |
| `GET /api/v1/debug/pprof/heap` | Jemalloc heap profile in pprof protobuf format (`application/x-protobuf`). Requires the `jemalloc` feature (default on Linux and macOS) and runtime activation via `MALLOC_CONF="prof:true,prof_active:true,lg_prof_sample:19"` on Linux or `_RJEM_MALLOC_CONF` on macOS (see [Profiling](../PROFILING.md#live-heap-profiling-jemalloc-pprof)). |
| `GET /api/v1/debug/pprof/profile` | Sampled CPU flamegraph or pprof protobuf profile. Accepts `seconds`, `frequency`, and `format=flamegraph` or `format=protobuf`. |

<!-- markdownlint-enable MD013 -->

### Metrics endpoint query parameters

`GET /api/v1/metrics` and `GET /api/v1/telemetry/metrics` accept the following
query parameters:

<!-- markdownlint-disable MD013 -->

| Parameter | Values | Default | Behavior |
| --- | --- | --- | --- |
| `format` | `prometheus`, `json`, `json_compact`, `line_protocol` | `prometheus` | Selects the response serialization format. |
| `reset` | `true`, `false` | `false` | Resets synchronous counters and distributions after reading when set to `true`. Leave at `false` for Prometheus scraping and dashboard polling. |
| `keep_all_zeroes` | `true`, `false` | `false` | Retains all-zero metric sets in `json` and `json_compact` responses. Ignored by `prometheus` and `line_protocol` outputs. |

<!-- markdownlint-enable MD013 -->

`GET /api/v1/telemetry/metrics/aggregate` sums counters and gauges and merges
distributions across entity instances that share the same `otel_scope_name`:

<!-- markdownlint-disable MD013 -->

| Parameter | Values | Default | Behavior |
| --- | --- | --- | --- |
| `format` | `prometheus`, `json`, `json_compact`, `line_protocol` | `prometheus` | Selects the aggregated response format. |
| `reset` | `true`, `false` | `false` | Resets metrics after reading when set to `true`. |
| `attrs` | Comma-separated attribute keys, or `*` | empty | Retains the specified entity or datapoint attributes as grouping keys. When omitted, each `otel_scope_name` collapses into a single aggregate group. |

<!-- markdownlint-enable MD013 -->

### Scraping with Prometheus

Point Prometheus at `/api/v1/metrics` or `/api/v1/telemetry/metrics` on the
configured `engine.http_admin.bind_address`:

```yaml
scrape_configs:
  - job_name: otap-dataflow-engine
    metrics_path: /api/v1/metrics
    static_configs:
      - targets: ["127.0.0.1:8080"]
```

## Internal observability pipeline and telemetry policies

### Exporting internal telemetry via the observability pipeline

To push internal metrics and logs to an OTLP backend or console sink, configure
`engine.observability.pipeline` with a `receiver:internal_telemetry` node:

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
        admin: console_async
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

When `engine.observability.pipeline` is omitted, the controller runs a default
internal observability pipeline that drains internal telemetry into
`exporter:noop` so internal ring buffers never back up.

### Controlling metric verbosity with telemetry policies

Runtime metric granularity can be tuned at the top-level `policies.telemetry`,
per pipeline group, per pipeline, or per node:

```yaml
policies:
  telemetry:
    pipeline_metrics: true
    tokio_metrics: true
    runtime_metrics: normal
```

The `runtime_metrics` level controls channel, node boundary, and latency
distribution instruments:

<!-- markdownlint-disable MD013 -->

| Level | Collected runtime instrumentation |
| --- | --- |
| `none` | Disables channel metrics, node boundary metrics, and shared receiver or exporter latency distributions. Component-specific operational counters and engine or pipeline gauges remain active. |
| `basic` | Enables channel message counters, channel queue depth and capacity gauges, node boundary message counters, and min/max/sum/count `Mmsc` duration summaries. |
| `normal` | Includes `basic` plus per-signal item counts, payload byte counters, and normal-resolution exponential histograms using `HistogramNormal` at scale 3 with 32 buckets. This is the default level. |
| `detailed` | Includes `normal` with high-resolution exponential histograms using `HistogramDetailed` at scale 5 with 160 buckets. |

<!-- markdownlint-enable MD013 -->

Individual node telemetry dimensions such as `messages`, `item_counts`, `size`,
`completion_duration`, `duration`, and `duration_distribution`, along with
processor range `flow_metrics`, can be overridden selectively. See
[Node and flow metrics](node-and-flow-metrics.md) for per-node and flow
configuration details.

## Prometheus and OTLP mapping rules

### Resource metadata and `target_info`

Attributes configured under `engine.telemetry.resource`, along with automatic
process and host resource attributes, are exposed once per Prometheus scrape as
a `target_info` gauge with value `1`:

```text
# HELP target_info Target metadata
# TYPE target_info gauge
target_info{service_name="otap-dataflow"} 1
```

### Entity scope labels

Each registered metric set is associated with an entity attribute set and a
scope name:

- The metric-set scope name is emitted on every sample as
  `otel_scope_name="<scope>"`.
- Entity attributes are prefixed with `otel_scope_` and sanitized to valid
  Prometheus label names by replacing dots and dashes with underscores. For
  example, `pipeline.id` becomes `otel_scope_pipeline_id` and `node.urn`
  becomes `otel_scope_node_urn`.

The table below summarizes the standard entity attribute sets attached to
metric sets at runtime:

<!-- markdownlint-disable MD013 -->

| Entity level | Attribute set | Entity attributes with `otel_scope_<key>` in Prometheus |
| --- | --- | --- |
| Process / engine | `engine` | none, process identity lives on `target_info` |
| Controller monitor | `controller.monitor.attrs` | `extension.id` |
| Worker core | `controller.attrs` | `core.id`, `numa.node.id` |
| Pipeline instance | `pipeline.attrs` | `pipeline.group.id`, `pipeline.id`, `deployment.generation`, `core.id`, `numa.node.id` |
| Node instance | `node.attrs` | `node.id`, `node.urn`, `node.type`, plus all `pipeline.attrs` keys, plus optional `topic`, `protocol`, or `custom` attributes |
| Node channel endpoint | `node.channel.attrs` | `channel.id`, `node.port`, `channel.kind`, `channel.mode`, `channel.type`, `channel.impl`, plus all `node.attrs` keys |
| Extension instance | `extension.attrs` | `extension.id`, `extension.variant`, `scope.kind`, plus all `pipeline.attrs` keys |
| Extension channel endpoint | `extension.channel.attrs` | `channel.id`, `channel.mode`, `channel.impl`, plus all `extension.attrs` keys |
| Processor flow range | `flow.attrs` | `flow.id`, `flow.node.start`, `flow.node.end`, `flow.purpose`, `flow.node.decision`, plus all `pipeline.attrs` keys |

<!-- markdownlint-enable MD013 -->

### Datapoint attributes

Measurement and registration attributes declared on a `#[metric_set]`, such as
`signal`, `outcome`, `error.type`, or `protocol`, are emitted directly as
Prometheus sample labels without the `otel_scope_` prefix. Dot characters in
attribute keys are converted to underscores, so `error.type` renders as
`error_type`.

### Metric naming, units, and distributions

Prometheus metric names are derived from the instrument field name, UCUM unit,
and instrument type, not prefixed by `otel_scope_name`. Use the
`otel_scope_name` label in PromQL selectors when two scopes define the same
instrument name, such as `messages_total{otel_scope_name="node.input"}`.

- Dots and dashes in the instrument name are replaced with underscores.
- Standard UCUM units are expanded to Prometheus unit words and appended to the
  metric name when not already present, and a `# UNIT <metric> <unit>` line is
  emitted. In particular, `By` maps to `bytes`, `s` maps to `seconds`, `ms`
  maps to `milliseconds`, `ns` maps to `nanoseconds`, and `%` maps to
  `percent`. Bracketed count annotations such as `{item}`, `{message}`, or
  `{batch}` are treated as dimensionless count units and do not add a unit
  word suffix.
- `Counter` and `ObserveCounter` instruments append `_total` and emit
  `# TYPE <metric> counter`.
- `UpDownCounter`, `ObserveUpDownCounter`, and `Gauge` instruments emit
  `# TYPE <metric> gauge` without a `_total` suffix.
- `Mmsc`, `HistogramNormal`, and `HistogramDetailed` distribution instruments
  expand into four Prometheus series when their observation count is non-zero,
  producing `<name>_min` as `gauge`, `<name>_max` as `gauge`, `<name>_sum` as
  `counter`, and `<name>_count` as `counter`. When queried with `?format=json`,
  exponential histograms also include populated bucket counts and estimated
  `p50`, `p90`, and `p99` quantiles.

## Self-observability metrics reference

The tables below list every production metric set and instrument defined across
the OTAP Dataflow Engine crates.

### Controller and engine metrics

Process-wide resource metrics, console writer metrics, rate admission metrics,
and controller runtime supervision metrics.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `admission.rate_limiter` | `refusals` | `refusals_total` | `Counter<u64>` | `{decision}` | `dimension`, `refusal` | Admission attempts declined, or that would have been declined in observe-only mode. |
| `controller.monitor` | `observed_pipelines.live` | `observed_pipelines_live` | `Gauge<u64>` | `{pipeline}` | none | Observed logical pipelines currently considered live. |
| `controller.monitor` | `observed_pipelines.ready` | `observed_pipelines_ready` | `Gauge<u64>` | `{pipeline}` | none | Observed logical pipelines currently considered ready. |
| `controller.monitor` | `observed_pipelines.total` | `observed_pipelines_total` | `Gauge<u64>` | `{pipeline}` | none | Observed logical pipelines currently tracked by the controller. |
| `controller.monitor` | `telemetry.entities` | `telemetry_entities` | `Gauge<u64>` | `{entity}` | none | Registered telemetry entities visible to the controller monitor. |
| `controller.monitor` | `telemetry.metric_sets` | `telemetry_metric_sets` | `Gauge<u64>` | `{metric_set}` | none | Registered telemetry metric sets visible to the controller monitor. |
| `engine` | `cpu.utilization` | `cpu_utilization` | `Gauge<f64>` | `{1}` | none | Process-wide CPU utilization as a ratio in [0, 1], normalized across all logical CPU cores on the system, not just engine-assigned cores. Aligned with the OTel semantic convention `process.cpu.utilization`. The `cpu.mode` attribute is not set; this reports combined user + system time. |
| `engine` | `memory.commit` | `memory_commit_bytes` | `ObserveUpDownCounter<u64>` | `By` | none | Process-wide private committed memory, including paged-out private memory. |
| `engine` | `memory.pressure.state` | `memory_pressure_state` | `Gauge<u64>` | `{state}` | none | Process-wide memory limiter state encoded as `0=normal`, `1=soft`, `2=hard`. |
| `engine` | `memory.rss` | `memory_rss` | `ObserveUpDownCounter<u64>` | `{By}` | none | Process-wide Resident Set Size, physical RAM currently used by the process. Matches what external tools report, e.g. `kubectl top pod`, `htop`, `ps rss`. |
| `engine` | `process.memory.hard.limit.bytes` | `process_memory_hard_limit_bytes` | `Gauge<u64>` | `{By}` | none | Effective process-wide memory limiter hard limit, in bytes. |
| `engine` | `process.memory.soft.limit.bytes` | `process_memory_soft_limit_bytes` | `Gauge<u64>` | `{By}` | none | Effective process-wide memory limiter soft limit, in bytes. |
| `engine` | `process.memory.usage.bytes` | `process_memory_usage_bytes` | `Gauge<u64>` | `{By}` | none | Most recent process-wide memory limiter sample, in bytes. |
| `engine.console_output` | `bytes.written` | `bytes_written_bytes_total` | `ObserveCounter<u64>` | `By` | `console.stream` | Bytes written to the stream. |
| `engine.console_output` | `diagnostics.dropped` | `diagnostics_dropped_total` | `ObserveCounter<u64>` | `{frame}` | `console.stream` | Best-effort diagnostics dropped because the queue was full or the frame exceeded the stream's byte budget. |
| `engine.console_output` | `frames.enqueue.failed` | `frames_enqueue_failed_total` | `ObserveCounter<u64>` | `{frame}` | `console.stream` | Frames rejected because the queue was closed or full, the frame was too large, or its writer had failed. |
| `engine.console_output` | `frames.submitted` | `frames_submitted_total` | `ObserveCounter<u64>` | `{frame}` | `console.stream` | Frames accepted into the stream's queue. |
| `engine.console_output` | `frames.written` | `frames_written_total` | `ObserveCounter<u64>` | `{frame}` | `console.stream` | Frames written to the stream. |
| `engine.console_output` | `write.errors` | `write_errors_total` | `ObserveCounter<u64>` | `{error}` | `console.stream` | Failed writes or flushes. Each one stops the stream's writer. |

<!-- markdownlint-enable MD013 -->

### Pipeline, memory limiter, and Tokio runtime metrics

Per-core pipeline CPU and memory metrics, Tokio async runtime metrics, pipeline
control-plane metrics, and pipeline memory-limiter metrics.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `pipeline` | `context.switches.involuntary` | `context_switches_involuntary_total` | `ObserveCounter<u64>` | `{1}` | none | Number of times the pipeline thread was preempted by the scheduler since start. A rising rate often indicates CPU contention or time-slice expirations. When paired with `cpu_utilization`, it helps diagnose CPU time lost to preemption. |
| `pipeline` | `context.switches.voluntary` | `context_switches_voluntary_total` | `ObserveCounter<u64>` | `{1}` | none | Number of times the pipeline thread voluntarily yielded the CPU since start. A rising rate can indicate scheduling pressure due to blocking work, I/O, locks, backpressure, and helps distinguish "waiting" from "CPU-bound". |
| `pipeline` | `cpu.time` | `cpu_time_total` | `Counter<f64>` | `{s}` | none | Total CPU seconds used by the pipeline since start. |
| `pipeline` | `cpu.utilization` | `cpu_utilization` | `Gauge<f64>` | `{1}` | none | Difference in pipeline cpu time since the last measurement, divided by the elapsed time. Reported as a ratio in the range [0, 1]. |
| `pipeline` | `memory.allocated` | `memory_allocated_total` | `ObserveCounter<u64>` | `{By}` | none | Memory allocated to the heap by the pipeline instance since start. |
| `pipeline` | `memory.allocated.delta` | `memory_allocated_delta_total` | `Counter<u64>` | `{By}` | none | Bytes allocated to the heap by this pipeline during the last measurement window. |
| `pipeline` | `memory.freed` | `memory_freed_total` | `ObserveCounter<u64>` | `{By}` | none | Memory freed to the heap by the pipeline instance since start. |
| `pipeline` | `memory.freed.delta` | `memory_freed_delta_total` | `Counter<u64>` | `{By}` | none | Bytes returned to the heap by this pipeline during the last measurement window. |
| `pipeline` | `memory.usage` | `memory_usage` | `ObserveUpDownCounter<u64>` | `{By}` | none | The amount of heap memory in use by the pipeline instance running on a specific core. |
| `pipeline` | `page.faults.major` | `page_faults_major_total` | `ObserveCounter<u64>` | `{1}` | none | Number of major page faults, requiring disk I/O, since start. Any non-trivial rate typically indicates severe memory pressure or swapping, and correlates strongly with pipeline latency spikes. |
| `pipeline` | `page.faults.minor` | `page_faults_minor_total` | `ObserveCounter<u64>` | `{1}` | none | Number of minor page faults, served without disk I/O, since start. A rising rate indicates memory access pain, poor locality, working-set churn, even if heap bytes are stable. |
| `pipeline` | `uptime` | `uptime` | `Gauge<f64>` | `{s}` | none | The time the pipeline instance has been running. |
| `pipeline.completion` | `ack.attempted` | `ack_attempted_total` | `Counter<u64>` | `{message}` | none | Count of Ack completions routed toward an interested upstream node. This increments once the dispatcher finds the closest interested frame and attempts delivery, even if the node-control channel is temporarily full and the send must be retried later. Level: `normal`. |
| `pipeline.completion` | `ack.delivered` | `ack_delivered_total` | `Counter<u64>` | `{message}` | none | Count of Ack control messages actually delivered to upstream nodes. This increments only when the node-control send succeeds, either immediately or later via the completion dispatcher's pending-send retry loop. Level: `normal`. |
| `pipeline.completion` | `ack.dropped_no_interest` | `ack_dropped_no_interest_total` | `Counter<u64>` | `{message}` | none | Count of Ack completions dropped because no upstream frame was interested. This distinguishes "successful unwind with no listener" from send failure or dispatcher backlog, which is useful when validating routing expectations and subscription placement. Level: `normal`. |
| `pipeline.completion` | `deliver_ack.received` | `deliver_ack_received_total` | `Counter<u64>` | `{message}` | none | Count of `DeliverAck` messages received by the completion dispatcher. This shows inbound successful completion pressure on the shared completion path and helps explain downstream-to-upstream unwind load. Level: `normal`. |
| `pipeline.completion` | `deliver_nack.received` | `deliver_nack_received_total` | `Counter<u64>` | `{message}` | none | Count of `DeliverNack` messages received by the completion dispatcher. This is the failure-side counterpart to received acks and is especially useful when investigating retry storms or elevated downstream refusal rates. Level: `normal`. |
| `pipeline.completion` | `nack.attempted` | `nack_attempted_total` | `Counter<u64>` | `{message}` | none | Count of Nack completions routed toward an interested upstream node. This is the failure-side counterpart to attempted Acks and captures how many downstream refusals found an upstream subscriber and entered the delivery path. Level: `normal`. |
| `pipeline.completion` | `nack.delivered` | `nack_delivered_total` | `Counter<u64>` | `{message}` | none | Count of Nack control messages actually delivered to upstream nodes. This is the delivery-side counterpart to attempted Nacks and helps distinguish routing intent from successful node-control delivery under backpressure. Level: `normal`. |
| `pipeline.completion` | `nack.dropped_no_interest` | `nack_dropped_no_interest_total` | `Counter<u64>` | `{message}` | none | Count of Nack completions dropped because no upstream frame was interested. This is particularly important for liveness and correctness analysis: it means failure information reached the dispatcher but no upstream frame subscribed to receive it. Level: `normal`. |
| `pipeline.completion` | `pending_sends.buffered` | `pending_sends_buffered` | `Gauge<u64>` | `{message}` | none | Number of completion deliveries currently buffered for later retry. A sustained backlog here means the completion dispatcher is making progress by deferring sends to full node-control inboxes instead of blocking, but it also indicates unwind traffic is under pressure. Level: `basic`. |
| `pipeline.completion` | `unwind.depth` | `unwind_depth_{min,max,sum,count}` | `Mmsc` | `{frame}` | none | Distribution of unwind stack depth consumed per completion outcome. Larger values indicate completions are traversing more frames before finding an interested upstream node or concluding that none exists, which helps explain completion-path cost and routing behavior. Level: `detailed`. |
| `pipeline.runtime_control` | `cancel_telemetry_timer.received` | `cancel_telemetry_timer_received_total` | `Counter<u64>` | `{message}` | none | Count of telemetry timer cancel requests received by the runtime manager. This is mainly useful for checking that telemetry scheduling remains bounded and does not accumulate stale per-node timers. Level: `normal`. |
| `pipeline.runtime_control` | `cancel_timer.received` | `cancel_timer_received_total` | `Counter<u64>` | `{message}` | none | Count of `CancelTimer` control messages received by the runtime manager. Comparing this with timer starts helps diagnose timer churn, leaks, or mismatches between node scheduling and node cleanup behavior. Level: `normal`. |
| `pipeline.runtime_control` | `collect_telemetry.sent` | `collect_telemetry_sent_total` | `Counter<u64>` | `{message}` | none | Count of `CollectTelemetry` control messages sent to nodes. This helps confirm telemetry timers are progressing under load and that observability work is not being starved by other runtime-control traffic. Level: `normal`. |
| `pipeline.runtime_control` | `downstream_shutdown.sent` | `downstream_shutdown_sent_total` | `Counter<u64>` | `{message}` | none | Count of downstream `Shutdown` fan-out operations to non-receivers. This marks the handoff from receiver draining to downstream convergence, which is useful when debugging where shutdown latency is spent. Level: `normal`. |
| `pipeline.runtime_control` | `drain.active` | `drain_active` | `Gauge<u64>` | `{1}` | none | Whether the runtime is currently in graceful drain mode. This is the top-level liveness phase signal for shutdown analysis: `1` means ingress has been stopped and the runtime is trying to converge to quiescence, `0` means normal operation. Level: `basic`. |
| `pipeline.runtime_control` | `drain.pending_receivers` | `drain_pending_receivers` | `Gauge<u64>` | `{node}` | none | Number of receivers that still need to report `ReceiverDrained`. This shows whether shutdown is still blocked in the receiver-first drain phase and helps identify a stuck or slow receiver delaying downstream shutdown. Level: `basic`. |
| `pipeline.runtime_control` | `drain.receiver_phase_duration_ns` | `drain_receiver_phase_duration_ns_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Distribution of time spent waiting for receivers to finish draining. This isolates the receiver-gated phase of shutdown and is useful for determining whether slow graceful shutdown is ingress-bound or further downstream in the pipeline. Level: `detailed`. |
| `pipeline.runtime_control` | `drain.total_duration_ns` | `drain_total_duration_ns_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Distribution of total graceful shutdown duration for the runtime. This is the top-level latency metric for shutdown liveness and shows how long the runtime took to move from shutdown acceptance to completion or forced deadline exit. Level: `detailed`. |
| `pipeline.runtime_control` | `drain_ingress.sent` | `drain_ingress_sent_total` | `Counter<u64>` | `{message}` | none | Count of `DrainIngress` fan-out operations sent to receivers. This confirms that shutdown moved from "requested" to "stop admitting new ingress", which is the first active liveness step of graceful drain. Level: `normal`. |
| `pipeline.runtime_control` | `pending_sends.buffered` | `pending_sends_buffered` | `Gauge<u64>` | `{message}` | none | Number of node-control deliveries currently buffered for later retry. A sustained non-zero value indicates the runtime manager is no longer delivering control work immediately and is absorbing backpressure from full node-control inboxes instead of blocking the runtime. Level: `basic`. |
| `pipeline.runtime_control` | `receiver_drained.received` | `receiver_drained_received_total` | `Counter<u64>` | `{message}` | none | Count of `ReceiverDrained` notifications received from receivers. This shows actual receiver-side drain progress and helps distinguish "no work admitted anymore" from "receivers have really finished draining". Level: `normal`. |
| `pipeline.runtime_control` | `shutdown.deadline_forced` | `shutdown_deadline_forced_total` | `Counter<u64>` | `{message}` | none | Count of shutdowns that reached the deadline before natural completion. Any non-zero value here is a direct signal that graceful drain did not complete in time and that liveness relied on the shutdown deadline. Level: `normal`. |
| `pipeline.runtime_control` | `shutdown.received` | `shutdown_received_total` | `Counter<u64>` | `{message}` | none | Count of `RuntimeControlMsg::Shutdown` requests accepted by this runtime. This marks the start of the runtime-managed drain sequence and is the main counter used to correlate later drain-phase transitions. Level: `normal`. |
| `pipeline.runtime_control` | `start_telemetry_timer.received` | `start_telemetry_timer_received_total` | `Counter<u64>` | `{message}` | none | Count of telemetry timer start requests received by the runtime manager. This shows how much of the runtime-control stream is driven by metrics collection scheduling rather than ordinary data-plane coordination. Level: `normal`. |
| `pipeline.runtime_control` | `start_timer.received` | `start_timer_received_total` | `Counter<u64>` | `{message}` | none | Count of `StartTimer` control messages received by the runtime manager. This helps quantify timer creation pressure and can explain growth in `timers.active` or bursts of later `timer_tick.sent` activity. Level: `normal`. |
| `pipeline.runtime_control` | `telemetry_timers.active` | `telemetry_timers_active` | `Gauge<u64>` | `{timer}` | none | Number of telemetry collection timers currently registered. This distinguishes telemetry scheduling pressure from normal timer traffic and helps explain unexpected `CollectTelemetry` activity. Level: `basic`. |
| `pipeline.runtime_control` | `timer_tick.sent` | `timer_tick_sent_total` | `Counter<u64>` | `{message}` | none | Count of `TimerTick` control messages sent to nodes. This confirms that due timers are actually being delivered and helps detect starvation where timers accumulate but ticks do not make progress. Level: `normal`. |
| `pipeline.runtime_control` | `timers.active` | `timers_active` | `Gauge<u64>` | `{timer}` | none | Number of ordinary per-node timers currently registered. This is useful for understanding timer pressure and for spotting leaks where timers are started but not canceled or consumed as expected. Level: `basic`. |
| `tokio.runtime` | `blocking.task.queue.size` | `blocking_task_queue_size` | `ObserveUpDownCounter<u64>` | `{task}` | none | Current number of tasks pending in the blocking thread pool. This counts tasks spawned using `spawn_blocking` that have not yet started. |
| `tokio.runtime` | `blocking.thread.count` | `blocking_thread_count` | `ObserveUpDownCounter<u64>` | `{thread}` | none | Current number of threads created for the blocking thread pool. |
| `tokio.runtime` | `blocking.thread.idle.count` | `blocking_thread_idle_count` | `ObserveUpDownCounter<u64>` | `{thread}` | none | Current number of idle threads in the blocking thread pool. |
| `tokio.runtime` | `budget.forced.yield.count` | `budget_forced_yield_count_total` | `ObserveCounter<u64>` | `{yield}` | none | Total number of times tasks were forced to yield after exhausting their cooperative budget. This metric requires `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime` | `global.task.queue.size` | `global_task_queue_size` | `ObserveUpDownCounter<u64>` | `{task}` | none | Current number of tasks pending in the runtime's global/injection queue. This queue is used when tasks are spawned or woken from outside the runtime. |
| `tokio.runtime` | `io.driver.fd.deregistered.count` | `io_driver_fd_deregistered_count_total` | `ObserveCounter<u64>` | `{fd}` | none | Total number of file descriptors deregistered from the runtime's I/O driver. This metric requires `tokio_unstable` and 64-bit atomics, and Tokio's `net` feature. |
| `tokio.runtime` | `io.driver.fd.registered.count` | `io_driver_fd_registered_count_total` | `ObserveCounter<u64>` | `{fd}` | none | Total number of file descriptors registered with the runtime's I/O driver. This metric requires `tokio_unstable` and 64-bit atomics, and Tokio's `net` feature. |
| `tokio.runtime` | `io.driver.ready.count` | `io_driver_ready_count_total` | `ObserveCounter<u64>` | `{ready}` | none | Total number of ready events processed by the runtime's I/O driver. This metric requires `tokio_unstable` and 64-bit atomics, and Tokio's `net` feature. |
| `tokio.runtime` | `remote.schedule.count` | `remote_schedule_count_total` | `ObserveCounter<u64>` | `{remote}` | none | Total number of tasks scheduled from outside the runtime since it was created. These schedules go through the global injection queue and can be slower than worker-local scheduling. This metric requires `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime` | `spawned.tasks.count` | `spawned_tasks_count_total` | `ObserveCounter<u64>` | `{spawn}` | none | Total number of tasks spawned in the runtime since it was created. This counter is monotonically increasing. This metric requires `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime` | `task.active.count` | `task_active_count` | `ObserveUpDownCounter<u64>` | `{task}` | none | Current number of alive tasks in the runtime. This value increases when tasks are spawned and decreases when tasks complete. |
| `tokio.runtime` | `worker.busy.time` | `worker_busy_time_total` | `ObserveCounter<f64>` | `{s}` | none | Total worker busy time, summed across all runtime workers, since runtime creation. With multiple workers, this sum can exceed wall-clock time because workers can be busy concurrently. This metric is only available on targets with 64-bit atomics. |
| `tokio.runtime` | `worker.count` | `worker_count` | `ObserveUpDownCounter<u64>` | `{thread}` | none | Number of worker threads used by the Tokio runtime. For Tokio's `current_thread` runtime, this is always `1`. |
| `tokio.runtime` | `worker.local.queue.size` | `worker_local_queue_size` | `ObserveUpDownCounter<u64>` | `{task}` | none | Current number of tasks pending in worker-local queues, summed across all workers. |
| `tokio.runtime` | `worker.local.schedule.count` | `worker_local_schedule_count_total` | `ObserveCounter<u64>` | `{schedule}` | none | Total number of tasks scheduled onto worker-local queues, summed across workers, since runtime creation. This metric requires `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime` | `worker.noop.count` | `worker_noop_count_total` | `ObserveCounter<u64>` | `{noop}` | none | Total number of "noop" unpark events, summed across workers, since runtime creation. A noop unpark is when a worker is unparked but finds no work before parking again. This metric requires `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime` | `worker.overflow.count` | `worker_overflow_count_total` | `ObserveCounter<u64>` | `{overflow}` | none | Total number of worker local-queue overflows, summed across workers, since runtime creation. This metric only applies to Tokio's multi-threaded scheduler. This metric requires `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime` | `worker.park.count` | `worker_park_count_total` | `ObserveCounter<u64>` | `{park}` | none | Total number of times runtime workers parked, summed across all workers, since runtime creation. A worker "parks" when it becomes idle and waits for new work. This metric is only available on targets with 64-bit atomics. |
| `tokio.runtime` | `worker.park.unpark.count` | `worker_park_unpark_count_total` | `ObserveCounter<u64>` | `{unpark}` | none | Total number of park/unpark transitions, summed across all workers, since runtime creation. An odd value means at least one worker is currently parked; an even value means all workers are currently active. This metric is only available on targets with 64-bit atomics. |
| `tokio.runtime` | `worker.poll.count` | `worker_poll_count_total` | `ObserveCounter<u64>` | `{poll}` | none | Total number of task polls, summed across workers, since runtime creation. This metric requires `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime` | `worker.steal.attempt.count` | `worker_steal_attempt_count_total` | `ObserveCounter<u64>` | `{steal}` | none | Total number of steal attempts, summed across workers, since runtime creation. This metric requires `tokio_unstable` and 64-bit atomics. |
| `tokio.runtime` | `worker.steal.success.count` | `worker_steal_success_count_total` | `ObserveCounter<u64>` | `{steal}` | none | Total number of successful steal operations, summed across workers, since runtime creation. This metric requires `tokio_unstable` and 64-bit atomics. |

<!-- markdownlint-enable MD013 -->

### Channel and local scheduler metrics

Data-plane and control-plane channel endpoint metrics under `channel.sender` and
`channel.receiver`, plus node-local wakeup scheduler metrics. On
`channel.sender` and `channel.receiver`, the `channel.kind` entity attribute
distinguishes `pdata` channels, which carry the `signal` datapoint attribute,
from `control` channels.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `channel.receiver` | `capacity` | `capacity` | `Gauge<u64>` | `{message}` | none | Configured maximum number of buffered messages. |
| `channel.receiver` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | none | Control messages successfully dequeued from the channel. |
| `channel.receiver` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal` | Messages successfully dequeued from the channel. |
| `channel.receiver` | `queue.depth` | `queue_depth` | `Gauge<u64>` | `{message}` | none | Current number of messages buffered in the channel. |
| `channel.sender` | `failures` | `failures_total` | `Counter<u64>` | `{message}` | `error.type` | Failed or refused immediate control-message send attempts. |
| `channel.sender` | `failures` | `failures_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | Failed or refused immediate send attempts. |
| `channel.sender` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `outcome` | Control messages whose immediate channel send attempt terminated. |
| `channel.sender` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Messages whose immediate channel send attempt terminated. |
| `node.local_wakeup.scheduler` | `cancel.ignored_shutdown` | `cancel_ignored_shutdown_total` | `Counter<u64>` | `{attempt}` | none | Count of cancel attempts ignored after shutdown was latched. |
| `node.local_wakeup.scheduler` | `cancel.missed` | `cancel_missed_total` | `Counter<u64>` | `{attempt}` | none | Count of cancel attempts that found no live wakeup in the slot. |
| `node.local_wakeup.scheduler` | `cancel.removed` | `cancel_removed_total` | `Counter<u64>` | `{wakeup}` | none | Count of cancel attempts that removed a live wakeup. |
| `node.local_wakeup.scheduler` | `capacity` | `capacity` | `Gauge<u64>` | `{wakeup}` | none | Configured live wakeup slot capacity. |
| `node.local_wakeup.scheduler` | `live` | `live` | `Gauge<u64>` | `{wakeup}` | none | Current number of live wakeups. |
| `node.local_wakeup.scheduler` | `pop.due` | `pop_due_total` | `Counter<u64>` | `{wakeup}` | none | Count of wakeups popped for delivery to the processor. |
| `node.local_wakeup.scheduler` | `set.error_capacity` | `set_error_capacity_total` | `Counter<u64>` | `{attempt}` | none | Count of set attempts rejected because the live-slot capacity was full. |
| `node.local_wakeup.scheduler` | `set.error_shutdown` | `set_error_shutdown_total` | `Counter<u64>` | `{attempt}` | none | Count of set attempts rejected after shutdown was latched. |
| `node.local_wakeup.scheduler` | `set.inserted` | `set_inserted_total` | `Counter<u64>` | `{wakeup}` | none | Count of newly inserted wakeups. |
| `node.local_wakeup.scheduler` | `set.replaced` | `set_replaced_total` | `Counter<u64>` | `{wakeup}` | none | Count of wakeups that replaced an existing live slot. |
| `node.local_wakeup.scheduler` | `shutdown.cleared` | `shutdown_cleared_total` | `Counter<u64>` | `{wakeup}` | none | Count of live wakeups dropped when shutdown was latched. |

<!-- markdownlint-enable MD013 -->

### Node boundary and processor flow metrics

Engine-managed node input, node output, node completion, processor compute
duration, and multi-node processor flow range metrics. See
[Node and flow metrics](node-and-flow-metrics.md) for detailed semantics.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `flow.compute` | `duration` | `duration_seconds_{min,max,sum,count}` | `Mmsc` / `HistogramNormal` / `HistogramDetailed` | `s` | `signal` | Sum of per-node compute durations, in seconds, for messages traversing the flow range. |
| `flow.dropped` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Number of signal items, log records, spans, or metric data points, a decision node chose to drop. |
| `flow.input` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Number of signal items, log records, spans, or metric data points, entering the flow range. |
| `flow.input` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal` | Number of PData messages entering the flow range. |
| `flow.input` | `size` | `size_bytes_total` | `Counter<u64>` | `By` | `signal` | Logical payload bytes entering the flow range. |
| `flow.output` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Number of signal items, log records, spans, or metric data points, leaving the flow range. |
| `flow.output` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal` | Number of PData messages leaving the flow range. |
| `flow.output` | `size` | `size_bytes_total` | `Counter<u64>` | `By` | `signal` | Logical payload bytes leaving the flow range. |
| `node.completion` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal`, `outcome` | Duration from the tracked node boundary until the corresponding terminal ack or nack is routed, in seconds. |
| `node.completion_emission` | `notify_ack.routed` | `notify_ack_routed_total` | `Counter<u64>` | `{message}` | none | Count of successful `notify_ack` routings into the shared completion lane. |
| `node.completion_emission` | `notify_nack.routed` | `notify_nack_routed_total` | `Counter<u64>` | `{message}` | none | Count of successful `notify_nack` routings into the shared completion lane. |
| `node.input` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal`, `outcome` | Input signal items, grouped by the `signal` datapoint attribute. |
| `node.input` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Input messages, grouped by `signal` and `outcome` datapoint attributes. |
| `node.input` | `size` | `size_bytes_total` | `Counter<u64>` | `By` | `signal`, `outcome` | Input logical payload size, grouped by `signal` and `outcome`. |
| `node.output` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal`, `outcome` | Output signal items, grouped by the `signal` datapoint attribute. |
| `node.output` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Output messages, grouped by `signal` and `outcome` datapoint attributes. |
| `node.output` | `size` | `size_bytes_total` | `Counter<u64>` | `By` | `signal`, `outcome` | Output logical payload size, grouped by `signal` and `outcome`. |
| `processor.compute` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `outcome` | Wall-clock duration of processor-local synchronous compute. |

<!-- markdownlint-enable MD013 -->

### Shared receiver and exporter metrics

Transport-independent receiver, OTLP request, and exporter outcome metrics
shared across built-in OTAP and OTLP components in `otel-arrow-dfe-otap`.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `exporter.attempted` | `duration` | `duration_seconds_{min,max,sum,count}` | `Mmsc` / `HistogramNormal` / `HistogramDetailed` | `s` | `signal`, `outcome` | Time spent performing export attempts, including backend latency. |
| `exporter.attempted` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal`, `outcome` | Number of signal items handled across export attempts. |
| `exporter.attempted` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Number of node-local delivery attempts. Retries represented as shared attempts count again. This differs from `node.input.messages`, which counts PData messages entering the exporter. |
| `exporter.attempted` | `payload.size` | `payload_size_bytes_total` | `Counter<u64>` | `By` | `signal`, `outcome` | Encoded application payload size produced or submitted across export attempts. |
| `exporter.exports` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal`, `outcome` | Time from dequeuing PData through its terminal local or backend export result. Ack/Nack notification time is excluded. |
| `exporter.exports` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Number of messages whose export reached a terminal outcome. |
| `receiver.messages` | `bytes` | `bytes_total` | `Counter<u64>` | `By` | `signal` | Encoded transport payload bytes admitted to the pipeline send path. |
| `receiver.messages` | `completed` | `completed_total` | `Counter<u64>` | `{message}` | `signal` | Number of admitted messages whose receiver work terminated. |
| `receiver.messages` | `started` | `started_total` | `Counter<u64>` | `{message}` | `signal` | Number of decoded messages admitted to the pipeline send path. |
| `receiver.otlp.acknowledgements` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `signal`, `outcome` | Number of routed or invalid acknowledgement responses. |
| `receiver.otlp.requests` | `accepted` | `accepted_total` | `Counter<u64>` | `{request}` | `signal`, `protocol` | Number of OTLP requests admitted to the pipeline send path. |
| `receiver.otlp.requests` | `rejected` | `rejected_total` | `Counter<u64>` | `{request}` | `protocol`, `error.type` | Number of rejected requests. |
| `receiver.otlp.transport` | `errors` | `errors_total` | `Counter<u64>` | `{error}` | `protocol` | Number of transport-level server errors. |
| `receiver.processing` | `duration` | `duration_seconds_{min,max,sum,count}` | `Mmsc` / `HistogramNormal` / `HistogramDetailed` | `s` | none | Node-defined receiver-local processing time. |
| `receiver.pulled` | `attempts` | `attempts_total` | `Counter<u64>` | `{attempt}` | `outcome` | Number of accepted collection attempts reaching a terminal local outcome. |
| `receiver.received` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Number of classified external messages whose receiver-local handling terminated. |
| `receiver.received` | `payload.size` | `payload_size_bytes_total` | `Counter<u64>` | `By` | `signal`, `outcome` | Encoded application payload size observed before receiver decoding. |

<!-- markdownlint-enable MD013 -->

### Core receiver metrics

Metrics emitted by receivers in `otel-arrow-dfe-core-nodes`.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `receiver.host_metrics` | `batches.sent` | `batches_sent_total` | `Counter<u64>` | `{batch}` | none | Number of batches sent downstream. |
| `receiver.host_metrics` | `families.scraped` | `families_scraped_total` | `Counter<u64>` | `{family}` | none | Number of due metric families processed. |
| `receiver.host_metrics` | `partial.errors` | `partial_errors_total` | `Counter<u64>` | `{error}` | none | Number of source read errors skipped because other families succeeded. |
| `receiver.host_metrics` | `scrape.duration.ns` | `scrape_duration_ns_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Wall-clock scrape duration. |
| `receiver.host_metrics` | `scrape.lag.ns` | `scrape_lag_ns_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Delay between scheduled and actual scrape start. |
| `receiver.host_metrics` | `scraper.failures` | `scraper_failures_total` | `Counter<u64>` | `{error}` | none | Number of scraper worker failures. |
| `receiver.host_metrics` | `scrapes.completed` | `scrapes_completed_total` | `Counter<u64>` | `{scrape}` | none | Number of scrape ticks that built and sent a metrics batch. |
| `receiver.host_metrics` | `scrapes.failed` | `scrapes_failed_total` | `Counter<u64>` | `{scrape}` | none | Number of fatal scrape failures. |
| `receiver.host_metrics` | `scrapes.overrun` | `scrapes_overrun_total` | `Counter<u64>` | `{scrape}` | none | Number of scrape ticks skipped or timed out before completion. |
| `receiver.host_metrics` | `scrapes.started` | `scrapes_started_total` | `Counter<u64>` | `{scrape}` | none | Number of scrape ticks started. |
| `receiver.host_metrics` | `send.failures` | `send_failures_total` | `Counter<u64>` | `{error}` | none | Number of downstream send failures. |
| `receiver.host_metrics` | `source.read.errors` | `source_read_errors_total` | `Counter<u64>` | `{error}` | none | Number of source read errors seen during scrapes. |
| `receiver.journald.acknowledgements` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `outcome` | Number of downstream responses. |
| `receiver.journald.acknowledgements` | `rewinds` | `rewinds_total` | `Counter<u64>` | `{rewind}` | `outcome` | Number of rewinds triggered by NACKs. |
| `receiver.journald.checkpoints` | `commits` | `commits_total` | `Counter<u64>` | `{commit}` | `outcome` | Number of durable cursor commits. |
| `receiver.journald.lifecycle` | `transitions` | `transitions_total` | `Counter<u64>` | `{transition}` | `transition.type` | Number of lifecycle transitions. |
| `receiver.journald.output` | `batches` | `batches_total` | `Counter<u64>` | `{batch}` | none | Number of log batches emitted downstream. |
| `receiver.journald.output` | `dropped.fields` | `dropped_fields_total` | `Counter<u64>` | `{field}` | none | Number of journald fields dropped by extraction limits. |
| `receiver.journald.output` | `records` | `records_total` | `Counter<u64>` | `{record}` | none | Number of log records emitted downstream. |
| `receiver.journald.source` | `events` | `events_total` | `Counter<u64>` | `{event}` | `error.type` | Number of source read failures. |
| `receiver.otap.acknowledgements` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `signal`, `outcome` | Number of routed or invalid acknowledgement responses. |
| `receiver.otap.batches` | `completed` | `completed_total` | `Counter<u64>` | `{batch}` | `signal` | Number of admitted batches whose receiver work terminated. |
| `receiver.otap.batches` | `payload.size` | `payload_size_bytes_total` | `Counter<u64>` | `By` | `signal` | Protobuf-encoded batch bytes after gRPC transport decompression. |
| `receiver.otap.batches` | `started` | `started_total` | `Counter<u64>` | `{batch}` | `signal` | Number of batches admitted to the pipeline send path. |
| `receiver.otap.rejections` | `batches` | `batches_total` | `Counter<u64>` | `{batch}` | `error.type` | Number of rejected OTAP batches within admitted streams. |
| `receiver.otap.rejections` | `streams` | `streams_total` | `Counter<u64>` | `{stream}` | `error.type` | Number of rejected OTAP streaming RPCs. |
| `receiver.syslog_cef.connections` | `active` | `active` | `UpDownCounter<u64>` | `{connection}` | none | Active connections. |
| `receiver.syslog_cef.connections` | `rejected` | `rejected_total` | `Counter<u64>` | `{connection}` | none | Rejected connections. |
| `receiver.syslog_cef.rejections` | `items` | `items_total` | `Counter<u64>` | `{item}` | `protocol`, `error.type`, `signal` | Number of items rejected. |
| `receiver.syslog_cef.transport` | `errors` | `errors_total` | `Counter<u64>` | `{error}` | `protocol` | Number of transport-level server errors. |
| `receiver.syslog_cef.truncations` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Truncated payloads. |
| `receiver.topic` | `downstream.backpressure.events` | `downstream_backpressure_events_total` | `Counter<u64>` | `{event}` | none | Number of downstream backpressure events, >= 500ms blocked. |
| `receiver.topic` | `lagged.messages` | `lagged_messages_total` | `Counter<u64>` | `{message}` | none | Total messages missed across lag notifications. |
| `receiver.topic.bridge` | `controls` | `controls_total` | `Counter<u64>` | `{control}` | `control`, `result` | Number of bridge controls. |
| `receiver.topic.downstream.blocked` | `duration` | `duration_seconds_{min,max,sum,count}` | `Mmsc` / `HistogramNormal` / `HistogramDetailed` | `s` | none | Time blocked while forwarding one message to downstream. |
| `receiver.topic.lag` | `events` | `events_total` | `Counter<u64>` | `{event}` | `event.type` | Number of lag events. |

<!-- markdownlint-enable MD013 -->

### Core processor metrics

Metrics emitted by processors in `otel-arrow-dfe-core-nodes`.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `otap.processor.batch` | `batching.errors` | `batching_errors_total` | `Counter<u64>` | `{error}` | none | Number of batches for which errors encountered. |
| `otap.processor.batch` | `dropped.conversion` | `dropped_conversion_total` | `Counter<u64>` | `{msg}` | none | Number of messages dropped due to conversion failures. |
| `otap.processor.batch` | `flush.age.duration` | `flush_age_duration_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Time from first pending input arrival to actual flush start. |
| `otap.processor.batch` | `flush.output.batches` | `flush_output_batches_{min,max,sum,count}` | `Mmsc` | `{batch}` | none | Number of output batches successfully handed downstream by each flush. |
| `otap.processor.batch` | `flush.output.bytes` | `flush_output_bytes_{min,max,sum,count}` | `Mmsc` | `By` | none | Final-representation bytes successfully handed downstream by each flush, when all output sizes are known; excludes transport compression and framing. |
| `otap.processor.batch` | `flush.pending.bytes` | `flush_pending_bytes_{min,max,sum,count}` | `Mmsc` | `By` | none | Number of bytes pending at flush time when byte size is known. |
| `otap.processor.batch` | `flush.pending.requests` | `flush_pending_requests_{min,max,sum,count}` | `Mmsc` | `{request}` | none | Number of input requests pending at flush time. |
| `otap.processor.batch` | `flush.timer.lateness.duration` | `flush_timer_lateness_duration_nanoseconds_{min,max,sum,count}` | `Mmsc` | `ns` | none | Delay between scheduled timer wakeup and actual timer flush start. |
| `otap.processor.batch` | `flushes.size` | `flushes_size_total` | `Counter<u64>` | `{flush}` | none | Number of flushes triggered by size threshold, all signals. |
| `otap.processor.batch` | `flushes.timer` | `flushes_timer_total` | `Counter<u64>` | `{flush}` | none | Number of flushes triggered by timer, all signals. |
| `otap.processor.batch` | `nacked.inbound.slots` | `nacked_inbound_slots_total` | `Counter<u64>` | `{msg}` | none | Number of requests nacked due to inbound slot exhaustion. |
| `otap.processor.batch` | `nacked.outbound.slots` | `nacked_outbound_slots_total` | `Counter<u64>` | `{msg}` | none | Number of requests nacked due to inbound slot exhaustion. |
| `otap.processor.batch` | `passthrough.batches` | `passthrough_batches_total` | `Counter<u64>` | `{batch}` | none | Number of inputs forwarded as-is, without re-batching or completion tracking, because their size was already within the configured [min_size, max_size] range on arrival. |
| `otap.processor.batch` | `split.budget.fallbacks` | `split_budget_fallbacks_total` | `Counter<u64>` | `{entry}` | none | Number of oversize resource entries emitted whole, best-effort, because splitting them would have exceeded the configured fragment budget. |
| `processor.attributes` | `transforms` | `transforms_total` | `Counter<u64>` | `{transform}` | `outcome` | No description. |
| `processor.attributes.modified` | `entries` | `entries_total` | `Counter<u64>` | `{attr}` | `action`, `domain` | No description. |
| `processor.content_router` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `outcome`, `reason` | Number of messages with this outcome. |
| `processor.debug` | `consumed.events` | `consumed_events_total` | `Counter<u64>` | `{event}` | `signal` | Named log events for logs, or span events for traces. |
| `processor.debug` | `consumed.links` | `consumed_links_total` | `Counter<u64>` | `{link}` | `signal` | Span links. |
| `processor.durable_buffer` | `flush.failures` | `flush_failures_total` | `Counter<u64>` | `{error}` | none | Number of segment finalization failures. |
| `processor.durable_buffer` | `in.flight` | `in_flight` | `Gauge<u64>` | `{bundle}` | none | Current number of bundles in flight to downstream. |
| `processor.durable_buffer` | `read.errors` | `read_errors_total` | `Counter<u64>` | `{error}` | none | Number of read errors. |
| `processor.durable_buffer` | `retries.scheduled` | `retries_scheduled_total` | `Counter<u64>` | `{retry}` | none | Number of retry attempts scheduled. |
| `processor.durable_buffer` | `storage.bytes.cap` | `storage_bytes_cap_bytes` | `Gauge<u64>` | `By` | none | Configured storage capacity cap. |
| `processor.durable_buffer` | `storage.bytes.used` | `storage_bytes_used_bytes` | `Gauge<u64>` | `By` | none | Current bytes used by persistent storage. |
| `processor.durable_buffer` | `storage.utilization` | `storage_utilization` | `Gauge<f64>` | `{1}` | none | Current storage utilization ratio. |
| `processor.durable_buffer.bundles` | `resolved` | `resolved_total` | `Counter<u64>` | `{bundle}` | `outcome` | Number of bundles resolved by downstream outcome. |
| `processor.durable_buffer.ingest` | `failures` | `failures_total` | `Counter<u64>` | `{failure}` | `failure` | Number of failed ingest attempts. |
| `processor.durable_buffer.items` | `consumed` | `consumed_total` | `Counter<u64>` | `{item}` | `signal` | Number of items ingested to durable storage. |
| `processor.durable_buffer.items` | `produced` | `produced_total` | `Counter<u64>` | `{item}` | `signal` | Number of items sent downstream. |
| `processor.durable_buffer.items` | `queued` | `queued` | `Gauge<u64>` | `{item}` | `signal` | Current number of items queued in durable storage. |
| `processor.durable_buffer.items` | `rejected` | `rejected_total` | `Counter<u64>` | `{item}` | `signal` | Number of items in permanently rejected bundles. |
| `processor.durable_buffer.items` | `requeued` | `requeued_total` | `Counter<u64>` | `{item}` | `signal` | Number of items requeued for retry after a NACK. |
| `processor.durable_buffer.loss` | `bundles` | `bundles_total` | `Counter<u64>` | `{bundle}` | `reason` | Number of bundles attributed to retention loss. This may be only the unresolved subset of a removed segment. |
| `processor.durable_buffer.loss` | `bytes` | `bytes_total` | `Counter<u64>` | `By` | `reason` | Logical payload bytes attributed to retention loss. |
| `processor.durable_buffer.loss` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal`, `reason` | Number of items in bundles attributed to retention loss. |
| `processor.durable_buffer.reclaimed` | `bytes` | `bytes_total` | `Counter<u64>` | `By` | `reason` | Full persisted bytes reclaimed from removed segment files. |
| `processor.durable_buffer.reclaimed` | `segments` | `segments_total` | `Counter<u64>` | `{segment}` | `reason` | Number of physical segment files removed. |
| `processor.fanout` | `active` | `active` | `Gauge<u64>` | `{message}` | none | Current number of active pipeline messages tracked by the processor. |
| `processor.fanout` | `max.inflight.config` | `max_inflight_config` | `Gauge<u64>` | `{message}` | none | Configured max_inflight value, 0 means unlimited. |
| `processor.fanout` | `throttle.episodes` | `throttle_episodes_total` | `Counter<u64>` | `{episode}` | none | Increments on transition from not-throttled to throttled. |
| `processor.fanout` | `throttled` | `throttled` | `Gauge<u64>` | `1` | none | 1 when fanout is currently refusing new pdata via accept_pdata,, else 0. |
| `processor.fanout` | `timed.out` | `timed_out_total` | `Counter<u64>` | `{message}` | `signal`, `destination` | Destination message attempts that timed out. |
| `processor.filter` | `dropped.items` | `dropped_items_total` | `Counter<u64>` | `{item}` | `signal` | Number of signal items, log records, spans, or metric data points, a decision node chose to drop. |
| `processor.log_parser` | `failures` | `failures_total` | `Counter<u64>` | `{operation}` | `error.type` | Failed local parsing operations. |
| `processor.log_parser` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `signal`, `outcome` | Matching input messages whose local parsing operation terminated. |
| `processor.log_parser` | `records` | `records_total` | `Counter<u64>` | `{record}` | `format`, `reason` | Preserved malformed records and committed observed-time fallbacks. |
| `processor.log_sampling` | `dropped.items` | `dropped_items_total` | `Counter<u64>` | `{item}` | `signal` | Log records dropped by sampling. |
| `processor.log_sampling` | `filter.buffer.reclamation.failures` | `filter_buffer_reclamation_failures_total` | `Counter<u64>` | `{error}` | `signal` | How many times we fail to reclaim the underlying filter buffer. |
| `processor.log_sampling` | `filtering.errors` | `filtering_errors_total` | `Counter<u64>` | `{error}` | `signal` | Errors encountered while filtering OTAP batches. |
| `processor.partition` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `outcome` | Number of partitioning operations attempted by this processor. |
| `processor.retry` | `messages.recovered` | `messages_recovered_total` | `Counter<u64>` | `{message}` | `signal` | Number of PData messages accepted downstream after at least one retry. |
| `processor.retry` | `retries.scheduled` | `retries_scheduled_total` | `Counter<u64>` | `{retry}` | `signal` | Number of retries successfully scheduled after a downstream refusal. |
| `processor.retry.messages` | `terminated` | `terminated_total` | `Counter<u64>` | `{message}` | `signal`, `reason` | Number of PData messages the retry processor stopped retrying. |
| `processor.temporal_reaggregation` | `failures` | `failures_total` | `Counter<u64>` | `{operation}` | `error.type` | No description. |
| `processor.temporal_reaggregation` | `flushes` | `flushes_total` | `Counter<u64>` | `{flush}` | `outcome`, `reason` | No description. |
| `processor.temporal_reaggregation` | `metrics` | `metrics_total` | `Counter<u64>` | `{metric}` | `type`, `aggregable`, `temporality` | Number of input metric records containing metric data. |
| `processor.temporal_reaggregation` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `outcome` | No description. |
| `processor.transform` | `failures` | `failures_total` | `Counter<u64>` | `{operation}` | `signal`, `error.type`, `language` | Number of failed transform operations. |
| `processor.transform` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `signal`, `outcome`, `language` | Number of matching input messages whose local transform operation terminated. |
| `processor.type_router` | `signals.decision` | `signals_decision_total` | `Counter<u64>` | `{message}` | `outcome`, `reason`, `signal` | Number of messages with this outcome. |

<!-- markdownlint-enable MD013 -->

### Core exporter metrics

Metrics emitted by exporters in `otel-arrow-dfe-core-nodes`.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `exporter.console.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type`, `format` | Number of PData messages that failed for the classified reason. |
| `exporter.file` | `bytes` | `bytes_total` | `Counter<u64>` | `By` | `signal` | Successfully written frame bytes, including newline delimiters. |
| `exporter.file` | `failures` | `failures_total` | `Counter<u64>` | `{failure}` | `signal`, `operation` | Open, write, sync, or rollback failures. |
| `exporter.file` | `items` | `items_total` | `Counter<u64>` | `{item}` | `signal` | Signal items in frames successfully written before ACK routing. |
| `exporter.file` | `tail.recovered.bytes` | `tail_recovered_bytes_total` | `Counter<u64>` | `By` | `signal` | Bytes removed by successful append-tail recovery. |
| `exporter.file` | `tail.recoveries` | `tail_recoveries_total` | `Counter<u64>` | `{recovery}` | `signal` | Append-mode partial tails successfully repaired. |
| `exporter.file.exports` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `outcome` | Telemetry messages whose export reached a terminal outcome. |
| `exporter.otap.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | Number of PData messages that failed for the classified error type. |
| `exporter.otap.streams` | `correlation.depth` | `correlation_depth_{min,max,sum,count}` | `HistogramNormal` | `{batch}` | `signal` | Occupancy of the response correlation queue before enqueueing a yielded batch. |
| `exporter.otap.streams` | `correlation.enqueue.duration` | `correlation_enqueue_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal` | Time spent enqueueing a yielded batch into the response correlation queue. |
| `exporter.otap.streams` | `encode.duration` | `encode_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal` | Time spent encoding an OTAP batch into outbound Arrow batch records. |
| `exporter.otap.streams` | `enqueue.depth` | `enqueue_depth_{min,max,sum,count}` | `HistogramNormal` | `{batch}` | `signal` | Occupancy of the per-signal stream task queue before enqueueing a batch. |
| `exporter.otap.streams` | `enqueue.duration` | `enqueue_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal` | Time spent waiting to enqueue a batch into the per-signal stream task. |
| `exporter.otap.streams` | `response.active` | `response_active_{min,max,sum,count}` | `HistogramNormal` | `{batch}` | `signal` | Number of yielded batches actively awaiting a matching server response. |
| `exporter.otap.streams` | `response.wait.duration` | `response_wait_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal` | Time spent waiting for the next server response on an OTAP stream. |
| `exporter.otlp_grpc.authentication` | `ready` | `ready` | `UpDownCounter<u64>` | `{1}` | `source` | Whether authenticated progress is currently possible, 0=false, 1=true. |
| `exporter.otlp_grpc.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | Number of PData messages that failed for the classified error type. |
| `exporter.otlp_http.authentication` | `ready` | `ready` | `UpDownCounter<u64>` | `{1}` | `source` | Whether authenticated progress is currently possible, 0=false, 1=true. |
| `exporter.otlp_http.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | Number of PData messages that failed for the classified error type. |
| `exporter.parquet.file` | `operations` | `operations_total` | `Counter<u64>` | `{operation}` | `operation` | Number of Parquet file lifecycle and flush operations. |
| `exporter.parquet.rows` | `written` | `written_total` | `Counter<u64>` | `{row}` | none | Total number of rows written into Parquet writers, appended, not necessarily flushed yet. |
| `exporter.topic` | `tracked.in.flight` | `tracked_in_flight` | `Gauge<u64>` | `{message}` | none | Number of admitted tracked publishes waiting for a terminal result. |
| `exporter.topic.rejections` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `reason` | Number of topic publishes refused for the classified reason. |
| `exporter.topic.tracked` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `result` | Number of admitted tracked publishes reaching the classified result. |

<!-- markdownlint-enable MD013 -->

### Contrib receiver metrics

Metrics emitted by optional feature-gated receivers in
`otel-arrow-dfe-contrib-nodes`.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `receiver.etw.decode` | `errors` | `errors_total` | `Counter<u64>` | `{event}` | `signal` | Number of ETW events that failed TDH decode/parsing. Note that these events remain eligible for forwarding downstream with empty decoded fields. |
| `receiver.etw.drops` | `events` | `events_total` | `Counter<u64>` | `{event}` | `reason`, `signal` | Number of dropped ETW events. |
| `receiver.etw.events` | `forwarded` | `forwarded_total` | `Counter<u64>` | `{event}` | `signal` | Events successfully forwarded downstream in batches. |
| `receiver.etw.events` | `received` | `received_total` | `Counter<u64>` | `{event}` | `signal` | Total events produced/received by the session before channel enqueue. This is the ingress denominator and authoritative total. |
| `receiver.etw.session` | `buffers.lost` | `buffers_lost_total` | `Counter<u64>` | `{buffer}` | `delivery`, `signal` | Number of kernel ETW buffers lost. |
| `receiver.etw.session` | `buffers.written` | `buffers_written_total` | `Counter<u64>` | `{buffer}` | `signal` | Number of kernel ETW buffers written by the session. |
| `receiver.etw.session` | `events.lost` | `events_lost_total` | `Counter<u64>` | `{event}` | `signal` | Number of events lost inside kernel ETW buffers before delivery, buffer overrun. Distinct from receiver-level drops and not included in received total. |
| `receiver.kafka.acknowledgements` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `signal`, `outcome` | Number of downstream acknowledgement responses. |
| `receiver.kafka.consumer` | `group.feedback.after_revocation` | `group_feedback_after_revocation_total` | `Counter<u64>` | `{response}` | none | Ack or nack responses ignored because their partition ownership was stale. |
| `receiver.kafka.consumer` | `group.lag` | `group_lag` | `Gauge<f64>` | `{message}` | none | Mean broker-committed consumer-group lag across every owned partition. |
| `receiver.kafka.consumer` | `group.partition.assignments` | `group_partition_assignments_total` | `Counter<u64>` | `{partition}` | none | Partitions newly acquired by this consumer across rebalances. |
| `receiver.kafka.consumer` | `group.partition.revocations` | `group_partition_revocations_total` | `Counter<u64>` | `{partition}` | none | Owned partitions revoked from this consumer across rebalances. |
| `receiver.kafka.consumer` | `group.partitions` | `group_partitions` | `ObserveUpDownCounter<u64>` | `{partition}` | none | Current number of partitions owned by this consumer. |
| `receiver.kafka.consumer` | `group.rebalance.commit_enqueue_failures` | `group_rebalance_commit_enqueue_failures_total` | `Counter<u64>` | `{error}` | none | Async commit-before-revoke calls that failed to enqueue locally during partition revocation. A broker rejection of the commit is not counted here; it is reported by `offset_commits` with `outcome="failure"`. |
| `receiver.kafka.consumer` | `group.rebalance.resume_failures` | `group_rebalance_resume_failures_total` | `Counter<u64>` | `{error}` | none | Partition resume operations that failed while clearing rebalance pause state. |
| `receiver.kafka.consumer` | `group.rebalances` | `group_rebalances_total` | `Counter<u64>` | `{rebalance}` | none | Consumer-group assignment events observed by this consumer. |
| `receiver.kafka.consumer` | `records.bytes` | `records_bytes_total` | `Counter<u64>` | `By` | none | Kafka record payload bytes delivered before filtering or decoding. |
| `receiver.kafka.consumer` | `records.duplicates` | `records_duplicates_total` | `Counter<u64>` | `{message}` | none | Messages skipped because their offsets were already tracked in this ownership generation. |
| `receiver.kafka.consumer` | `records.inflight` | `records_inflight` | `ObserveUpDownCounter<u64>` | `{message}` | none | Current number of delivered records awaiting acknowledgement and commit progress. |
| `receiver.kafka.consumer` | `records.received` | `records_received_total` | `Counter<u64>` | `{message}` | none | Kafka records delivered by the consumer before filtering or decoding. |
| `receiver.kafka.consumer` | `retry.feedback.stale` | `retry_feedback_stale_total` | `Counter<u64>` | `{response}` | none | Feedback ignored because it belongs to an obsolete replay generation. |
| `receiver.kafka.consumer` | `retry.partitions_paused` | `retry_partitions_paused` | `ObserveUpDownCounter<u64>` | `{partition}` | none | Current number of partitions paused during transient-NACK recovery. |
| `receiver.kafka.consumer` | `retry.pause_failures` | `retry_pause_failures_total` | `Counter<u64>` | `{error}` | none | Partition pause operations that failed during transient-NACK recovery. |
| `receiver.kafka.consumer` | `retry.replay_attempts` | `retry_replay_attempts_total` | `Counter<u64>` | `{attempt}` | none | Kafka replay attempts started after transient NACK backoff. |
| `receiver.kafka.consumer` | `retry.resume_failures` | `retry_resume_failures_total` | `Counter<u64>` | `{error}` | none | Partition resume operations that failed during transient-NACK recovery. |
| `receiver.kafka.consumer` | `retry.seek_failures` | `retry_seek_failures_total` | `Counter<u64>` | `{error}` | none | Partition seek operations that failed during transient-NACK recovery. |
| `receiver.kafka.consumer` | `retry.transient_nacks` | `retry_transient_nacks_total` | `Counter<u64>` | `{response}` | none | Non-permanent NACK responses received from downstream. |
| `receiver.kafka.messages` | `completed` | `completed_total` | `Counter<u64>` | `{message}` | `signal` | Number of admitted messages whose receiver work terminated. |
| `receiver.kafka.messages` | `payload.size` | `payload_size_bytes_total` | `Counter<u64>` | `By` | `signal` | Encoded Kafka payload bytes admitted to the pipeline send path. |
| `receiver.kafka.messages` | `started` | `started_total` | `Counter<u64>` | `{message}` | `signal` | Number of decoded messages admitted to the pipeline send path. |
| `receiver.kafka.offset_commits` | `commits` | `commits_total` | `Counter<u64>` | `{commit}` | `outcome` | Number of offset commit results reported by librdkafka. |
| `receiver.kafka.rejections` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type`, `reason` | Number of rejected Kafka messages. |
| `receiver.kafka.transport` | `errors` | `errors_total` | `Counter<u64>` | `{error}` | `error.type` | Number of non-EOF errors returned by the Kafka consumer. |
| `receiver.microsoft_json` | `compressed.requests` | `compressed_requests_total` | `Counter<u64>` | `{request}` | none | Number of HTTP requests received with an encoded body. |
| `receiver.microsoft_json` | `config.endpoints.failed` | `config_endpoints_failed_total` | `Counter<u64>` | `{error}` | none | Number of endpoint configuration files that failed startup loading. |
| `receiver.microsoft_json` | `config.endpoints.loaded` | `config_endpoints_loaded_total` | `Counter<u64>` | `{item}` | none | Number of endpoint configuration files loaded at startup. |
| `receiver.microsoft_json` | `cors.preflight.requests` | `cors_preflight_requests_total` | `Counter<u64>` | `{request}` | none | Number of CORS preflight, `OPTIONS`, requests handled. |
| `receiver.microsoft_json` | `events.oversized` | `events_oversized_total` | `Counter<u64>` | `{item}` | none | Number of events rejected for exceeding the endpoint event-size limit. |
| `receiver.microsoft_json` | `ikey.ignored` | `ikey_ignored_total` | `Counter<u64>` | `{item}` | none | Number of events silently dropped by the iKey filter. |
| `receiver.microsoft_json` | `ikey.rejected` | `ikey_rejected_total` | `Counter<u64>` | `{item}` | none | Number of events rejected by the iKey filter. |
| `receiver.microsoft_json` | `ikey.unauthorized` | `ikey_unauthorized_total` | `Counter<u64>` | `{item}` | none | Number of events rejected because their origin is not allowed. |
| `receiver.microsoft_json` | `received.logs.invalid` | `received_logs_invalid_total` | `Counter<u64>` | `{item}` | none | Number of JSON events that failed parsing or endpoint validation. |
| `receiver.user_events` | `downstream.send.blocked.ns` | `downstream_send_blocked_ns_nanoseconds_total` | `Counter<u64>` | `ns` | none | Total time spent waiting for downstream channel capacity. |
| `receiver.user_events` | `flushed.batches` | `flushed_batches_total` | `Counter<u64>` | `{event}` | none | Number of Arrow batches flushed downstream. |
| `receiver.user_events` | `forwarded.samples` | `forwarded_samples_total` | `Counter<u64>` | `{item}` | none | Number of perf samples forwarded downstream. |
| `receiver.user_events` | `late.registration.retries` | `late_registration_retries_total` | `Counter<u64>` | `{event}` | none | Number of late-registration retries attempted while waiting for tracepoints. |
| `receiver.user_events` | `lost.perf.samples` | `lost_perf_samples_total` | `Counter<u64>` | `{item}` | none | Number of lost samples reported by the perf ring. |
| `receiver.user_events` | `received.samples` | `received_samples_total` | `Counter<u64>` | `{item}` | none | Number of perf samples received from the kernel ring. |
| `receiver.user_events` | `sessions.started` | `sessions_started_total` | `Counter<u64>` | `{event}` | none | Number of receiver sessions successfully started. |
| `receiver.user_events.drops` | `samples` | `samples_total` | `Counter<u64>` | `{item}` | `reason` | Number of dropped samples. |
| `receiver.windows_perf_counters` | `active.counters` | `active_counters` | `Gauge<u64>` | `{counter}` | none | Counters currently added to a query. |
| `receiver.windows_perf_counters` | `batches.dropped` | `batches_dropped_total` | `Counter<u64>` | `{batch}` | none | Batches dropped because downstream was full or closed. |
| `receiver.windows_perf_counters` | `counter.add.failures` | `counter_add_failures_total` | `Counter<u64>` | `{failure}` | none | Failed counter adds, including retries. |
| `receiver.windows_perf_counters` | `scrape.duration` | `scrape_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | none | Time from scrape request to report. |
| `receiver.windows_perf_counters` | `scrape.failures` | `scrape_failures_total` | `Counter<u64>` | `{scrape}` | none | Scrapes in which at least one counter's query failed to open or collect, plus startup if a query failed to open or prime. |
| `receiver.windows_perf_counters` | `scrape.overruns` | `scrape_overruns_total` | `Counter<u64>` | `{tick}` | none | Ticks skipped because the previous scrape was still running. |
| `receiver.windows_perf_counters` | `scrapes` | `scrapes_total` | `Counter<u64>` | `{scrape}` | none | Scrapes in which at least one counter's query collected. |

<!-- markdownlint-enable MD013 -->

### Contrib processor metrics

Metrics emitted by optional feature-gated processors in
`otel-arrow-dfe-contrib-nodes`.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `processor.resource_validator` | `failures` | `failures_total` | `Counter<u64>` | `{batch}` | `error.type` | Number of batches that failed resource validation. |

<!-- markdownlint-enable MD013 -->

### Contrib exporter metrics

Metrics emitted by optional feature-gated exporters in
`otel-arrow-dfe-contrib-nodes`.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `exporter.azure_data_explorer` | `batches` | `batches_total` | `Counter<u64>` | `{batch}` | `signal`, `outcome` | Number of logical ADX batches reaching a terminal outcome. |
| `exporter.azure_data_explorer.http` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `response` | Number of HTTP attempts by response category. |
| `exporter.azure_monitor` | `batch_size` | `batch_size_bytes_{min,max,sum,count}` | `Mmsc` | `By` | `outcome`, `signal` | Compressed batch size in bytes. |
| `exporter.azure_monitor` | `batch_uncompressed_size` | `batch_uncompressed_size_bytes_{min,max,sum,count}` | `Mmsc` | `By` | `outcome`, `signal` | Uncompressed batch size in bytes. |
| `exporter.azure_monitor` | `batches` | `batches_total` | `Counter<u64>` | `{batch}` | `outcome`, `signal` | Number of completed batches. |
| `exporter.azure_monitor` | `in_flight_exports` | `in_flight_exports` | `Gauge<u64>` | `{export}` | none | Current number of in-flight export requests. |
| `exporter.azure_monitor` | `in_flight_log_records` | `in_flight_log_records` | `Gauge<u64>` | `{log}` | none | Current number of log records in-flight at the exporter, enqueued export requests awaiting completion, including records being retried. |
| `exporter.azure_monitor` | `log_entries_too_large` | `log_entries_too_large_total` | `Counter<u64>` | `{entry}` | none | Number of log entries rejected for exceeding the batch size limit. |
| `exporter.azure_monitor.heartbeats` | `sends` | `sends_total` | `Counter<u64>` | `{heartbeat}` | `outcome` | Number of completed heartbeat send attempts. |
| `exporter.azure_monitor.http` | `latency` | `latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | `response` | HTTP export attempt latency in milliseconds, min/max/sum/count. |
| `exporter.azure_monitor.http` | `responses` | `responses_total` | `Counter<u64>` | `{response}` | `response` | Number of HTTP export attempts by response category. |
| `exporter.azure_monitor.state` | `mappings` | `mappings` | `Gauge<u64>` | `{entry}` | `mapping` | Current number of entries in the exporter state mapping. |
| `exporter.clickhouse.batches` | `transformed` | `transformed_total` | `Counter<u64>` | `{batch}` | `path` | Total number of log batches transformed by the given path. |
| `exporter.clickhouse.rows` | `written` | `written_total` | `Counter<u64>` | `{row}` | `signal` | Total number of rows written successfully into clickhouse. |
| `exporter.geneva.encoding` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal`, `outcome` | Time spent encoding and compressing signal data. |
| `exporter.geneva.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | Number of failed Geneva processing operations. |
| `exporter.geneva.skipped` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `reason` | Number of skipped PData messages. |
| `exporter.kafka.exports` | `bytes` | `bytes_{min,max,sum,count}` | `HistogramNormal` | `By` | `signal`, `outcome` | Encoded Kafka payload bytes for attempts that reached the producer. |
| `exporter.kafka.failures` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `error.type` | Number of failed export attempts. |
| `exporter.kafka.operations` | `duration` | `duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | `signal`, `operation`, `outcome` | Time spent in the selected exporter phase. |
| `exporter.kafka.routing` | `messages` | `messages_total` | `Counter<u64>` | `{message}` | `signal`, `topic.source` | Number of messages routed using the selected bounded source. |

<!-- markdownlint-enable MD013 -->

### Extension metrics

Extension lifecycle metrics and authentication extension token acquisition and
refresh metrics in `otel-arrow-dfe-engine` and
`otel-arrow-dfe-contrib-extensions`.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `extension.azure_identity_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Number of failed credential acquisitions. |
| `extension.azure_identity_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful acquisitions in milliseconds, min/max/sum/count. |
| `extension.azure_identity_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Number of successful credential acquisitions. |
| `extension.azure_identity_auth` | `auth.token.publish` | `auth_token_publish_total` | `Counter<u64>` | `{token}` | none | Number of tokens published to consumers via the watch channel. |
| `extension.azure_key_vault_sasl_auth` | `acquisition.failures` | `acquisition_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed complete acquisitions, with no partial credential publication. |
| `extension.azure_key_vault_sasl_auth` | `acquisition.success.latency` | `acquisition_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Successful complete acquisition duration. |
| `extension.azure_key_vault_sasl_auth` | `acquisition.successes` | `acquisition_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful complete username/password acquisitions. |
| `extension.azure_key_vault_sasl_auth` | `credential.publishes` | `credential_publishes_total` | `Counter<u64>` | `{credential}` | none | Complete credentials published to consumers. |
| `extension.azure_key_vault_user_pass_auth` | `acquisition.failures` | `acquisition_failures_total` | `Counter<u64>` | `{acquisition}` | none | Failed acquisitions, without partial credential publication. |
| `extension.azure_key_vault_user_pass_auth` | `acquisition.success.latency` | `acquisition_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Successful complete acquisition duration. |
| `extension.azure_key_vault_user_pass_auth` | `acquisition.successes` | `acquisition_successes_total` | `Counter<u64>` | `{acquisition}` | none | Successful complete username/password acquisitions. |
| `extension.azure_key_vault_user_pass_auth` | `credential.publishes` | `credential_publishes_total` | `Counter<u64>` | `{credential}` | none | Complete credentials published to consumers. |
| `extension.flat_file_api_key_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Number of failed API key acquisitions. |
| `extension.flat_file_api_key_auth` | `auth.publishes` | `auth_publishes_total` | `Counter<u64>` | `{credential}` | none | Number of API keys published to consumers. |
| `extension.flat_file_api_key_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful acquisitions in milliseconds. |
| `extension.flat_file_api_key_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Number of successful API key acquisitions. |
| `extension.flat_file_sasl_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Number of failed credential acquisitions. |
| `extension.flat_file_sasl_auth` | `auth.publishes` | `auth_publishes_total` | `Counter<u64>` | `{credential}` | none | Number of credentials published to consumers via the watch channel. |
| `extension.flat_file_sasl_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful acquisitions in milliseconds, min/max/sum/count. |
| `extension.flat_file_sasl_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Number of successful credential acquisitions. |
| `extension.flat_file_user_pass_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Number of failed credential acquisitions. |
| `extension.flat_file_user_pass_auth` | `auth.publishes` | `auth_publishes_total` | `Counter<u64>` | `{credential}` | none | Number of credentials published to consumers via the watch channel. |
| `extension.flat_file_user_pass_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful acquisitions in milliseconds, min/max/sum/count. |
| `extension.flat_file_user_pass_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Number of successful credential acquisitions. |
| `extension.gcp_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Number of failed token acquisitions. |
| `extension.gcp_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful acquisitions in milliseconds, min/max/sum/count. |
| `extension.gcp_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Number of successful token acquisitions. |
| `extension.gcp_auth` | `auth.token.publish` | `auth_token_publish_total` | `Counter<u64>` | `{token}` | none | Number of tokens published to consumers via the watch channel. |
| `extension.lifecycle` | `completed.cancelled` | `completed_cancelled_total` | `Counter<u64>` | `{completion}` | none | No description. |
| `extension.lifecycle` | `completed.error` | `completed_error_total` | `Counter<u64>` | `{completion}` | none | No description. |
| `extension.lifecycle` | `completed.ok` | `completed_ok_total` | `Counter<u64>` | `{completion}` | none | No description. |
| `extension.lifecycle` | `completed.panic` | `completed_panic_total` | `Counter<u64>` | `{completion}` | none | No description. |
| `extension.lifecycle` | `shutdown.sent` | `shutdown_sent_total` | `Counter<u64>` | `{shutdown}` | none | No description. |
| `extension.lifecycle` | `shutdown.timeout` | `shutdown_timeout_total` | `Counter<u64>` | `{timeout}` | none | No description. |
| `extension.lifecycle` | `spawned` | `spawned_total` | `Counter<u64>` | `{spawn}` | none | No description. |
| `extension.lifecycle` | `state` | `state` | `Gauge<u64>` | `{state}` | none | No description. |
| `extension.oauth2_client_auth` | `auth.failures` | `auth_failures_total` | `Counter<u64>` | `{acquisition}` | none | Number of failed token acquisitions. |
| `extension.oauth2_client_auth` | `auth.success.latency` | `auth_success_latency_milliseconds_{min,max,sum,count}` | `Mmsc` | `ms` | none | Latency of successful acquisitions in milliseconds, min/max/sum/count. |
| `extension.oauth2_client_auth` | `auth.successes` | `auth_successes_total` | `Counter<u64>` | `{acquisition}` | none | Number of successful token acquisitions. |
| `extension.oauth2_client_auth` | `auth.token.publish` | `auth_token_publish_total` | `Counter<u64>` | `{token}` | none | Number of tokens published to consumers via the watch channel. |

<!-- markdownlint-enable MD013 -->

### Development, scraper, and WASM host metrics

Metrics emitted by development and benchmarking nodes in
`otel-arrow-dfe-dev-nodes`, database scraper components in
`otel-arrow-dfe-scraper`, validation helpers, and the WebAssembly processor
host in `otel-arrow-dfe-wasm-host`.

<!-- markdownlint-disable MD013 -->

| Scope `otel_scope_name` | Instrument name | Prometheus name | Type | Unit | Datapoint attributes | Description |
| --- | --- | --- | --- | --- | --- | --- |
| `exporter.validation` | `check.failed` | `check_failed_total` | `otel_arrow_dfe_telemetry::instrument::Counter<u64>` | `{check}` | none | Number of validation checks that did not match expectation. |
| `exporter.validation` | `check.passed` | `check_passed_total` | `otel_arrow_dfe_telemetry::instrument::Counter<u64>` | `{check}` | none | Number of validation checks that did match expectation. |
| `exporter.validation` | `finished` | `finished` | `otel_arrow_dfe_telemetry::instrument::Gauge<u64>` | `{state}` | none | Whether the exporter has finished processing 0 -> still receiving / processing 1 -> idle timeout reached, final validation performed. |
| `exporter.validation` | `valid` | `valid` | `otel_arrow_dfe_telemetry::instrument::Gauge<u64>` | `{input}` | none | The value of the last comparison result 0 -> not valid 1 -> valid. |
| `processor.wasm_processor.pdata` | `guest.counter.add.calls` | `guest_counter_add_calls_total` | `Counter<u64>` | `{item}` | none | Number of guest `counter-add` calls accepted, name within both the length and cardinality bounds. |
| `processor.wasm_processor.pdata` | `guest.counter.add.rejected.cardinality` | `guest_counter_add_rejected_cardinality_total` | `Counter<u64>` | `{item}` | none | Number of guest `counter-add` calls rejected, silently no-op'd, because accepting the name would exceed the host's distinct-counter cardinality bound. Means the plugin is tracking more counters than the host allows, and some of its counts are being lost. |
| `processor.wasm_processor.pdata` | `guest.counter.add.rejected.name.len` | `guest_counter_add_rejected_name_len_total` | `Counter<u64>` | `{item}` | none | Number of guest `counter-add` calls rejected, silently no-op'd, because the counter name exceeded the host's maximum name length. Distinct from the cardinality rejection below: this one means the plugin is emitting a malformed name. |
| `processor.wasm_processor.pdata` | `guest.counter.add.value` | `guest_counter_add_value_total` | `ObserveCounter<u64>` | `{item}` | none | Cumulative sum of values accepted through guest `counter-add` calls. Saturates at `u64::MAX` for the lifetime of the plugin instance. |
| `processor.wasm_processor.pdata` | `guest.host.service.calls.rejected` | `guest_host_service_calls_rejected_total` | `Counter<u64>` | `{item}` | none | Number of guest `log`/`counter-add`/`get-config` calls rejected, silently no-op'd, because the shared token-bucket rate limiter was empty. The limiter is scoped to the plugin instance's whole lifetime, not reset per `initialize`/`process` call. A non-zero value is how an operator tells a throttled plugin apart from a quiet one. |
| `processor.wasm_processor.pdata` | `guest.log.message.truncated` | `guest_log_message_truncated_total` | `Counter<u64>` | `{item}` | none | Number of guest `log` messages truncated to the host's maximum retained length before being forwarded to telemetry. |
| `processor.wasm_processor.pdata` | `guest.process.calls` | `guest_process_calls_total` | `Counter<u64>` | `{item}` | none | Number of guest `process` calls attempted. |
| `processor.wasm_processor.pdata` | `guest.process.errors` | `guest_process_errors_total` | `Counter<u64>` | `{item}` | none | Number of guest `process` calls that failed or trapped. |
| `processor.wasm_processor.pdata` | `kernel.calls` | `kernel_calls_total` | `Counter<u64>` | `{item}` | none | Total host kernel invocations dispatched by the guest. |
| `processor.wasm_processor.pdata` | `pdata.dropped` | `pdata_dropped_total` | `Counter<u64>` | `{item}` | none | Number of pdata messages intentionally dropped by guest `process` returning `none`. |
| `processor.wasm_processor.pdata.records` | `records.in` | `records_in_total` | `Counter<u64>` | `{item}` | `signal` | Records entering the guest, root batch row count. |
| `processor.wasm_processor.pdata.records` | `records.out` | `records_out_total` | `Counter<u64>` | `{item}` | `signal` | Records leaving the guest after filtering. |
| `receiver.database` | `acks` | `acks_total` | `Counter<u64>` | `{ack}` | none | Downstream acknowledgements matching an in-flight page. |
| `receiver.database` | `batches.sent` | `batches_sent_total` | `Counter<u64>` | `{batch}` | none | Pages sent downstream. |
| `receiver.database` | `cancellations` | `cancellations_total` | `Counter<u64>` | `{cancellation}` | none | Active database operations cancelled by a control message. |
| `receiver.database` | `checkpoint.cleanup.failures` | `checkpoint_cleanup_failures_total` | `Counter<u64>` | `{failure}` | none | Stale checkpoint revisions that could not be removed. |
| `receiver.database` | `checkpoint.commits` | `checkpoint_commits_total` | `Counter<u64>` | `{commit}` | none | Durable checkpoint commits. |
| `receiver.database` | `checkpoint.failures` | `checkpoint_failures_total` | `Counter<u64>` | `{failure}` | none | Durable checkpoint write failures. |
| `receiver.database` | `drains` | `drains_total` | `Counter<u64>` | `{drain}` | none | Drain requests observed, including during checkpoint writes and retries. |
| `receiver.database` | `encoded.bytes.sent` | `encoded_bytes_sent_bytes_total` | `Counter<u64>` | `By` | none | Encoded OTLP bytes sent downstream. |
| `receiver.database` | `event.time.fallbacks` | `event_time_fallbacks_total` | `Counter<u64>` | `{record}` | none | Records whose source event time cannot fit OTLP's timestamp range. |
| `receiver.database` | `nacks` | `nacks_total` | `Counter<u64>` | `{nack}` | none | Downstream negative acknowledgements matching an in-flight page. |
| `receiver.database` | `polls` | `polls_total` | `Counter<u64>` | `{poll}` | none | Bounded page polls attempted against the database. |
| `receiver.database` | `query.failures` | `query_failures_total` | `Counter<u64>` | `{failure}` | none | Query-page attempts that failed, including operation setup. |
| `receiver.database` | `reconnects` | `reconnects_total` | `Counter<u64>` | `{attempt}` | none | Reconnect operations started after transient database failures. |
| `receiver.database` | `rejection.paused` | `rejection_paused` | `Gauge<u64>` | `1` | none | One while polling is paused by permanent rejection, otherwise zero. |
| `receiver.database` | `replays` | `replays_total` | `Counter<u64>` | `{replay}` | none | Replay schedules created after a NACK, not completed query attempts. |
| `receiver.database` | `rows.sent` | `rows_sent_total` | `Counter<u64>` | `{row}` | none | Database rows sent downstream. |
| `receiver.database` | `shutdowns` | `shutdowns_total` | `Counter<u64>` | `{shutdown}` | none | Shutdown requests observed, including during checkpoint writes and retries. |
| `receiver.database` | `stale.feedback` | `stale_feedback_total` | `Counter<u64>` | `{feedback}` | none | ACK/NACK feedback discarded as malformed, wrong-generation, stale, or duplicate. |
| `receiver.database` | `starts` | `starts_total` | `Counter<u64>` | `{start}` | none | Receiver starts. |
| `receiver.traffic_generator` | `completion.drain_deadline_forced` | `completion_drain_deadline_forced_total` | `Counter<u64>` | `{drain}` | none | Number of drains forced to finish with unresolved batches at the deadline. |
| `receiver.traffic_generator` | `completion.pending` | `completion_pending` | `Gauge<u64>` | `{batch}` | none | Number of subscribed batches waiting for Ack/Nack completion. |
| `receiver.traffic_generator` | `smooth.batch_interval` | `smooth_batch_interval_nanoseconds` | `Gauge<u64>` | `ns` | none | Smooth-mode configured interval between batches. |
| `receiver.traffic_generator` | `smooth.batch_tick_lateness_duration` | `smooth_batch_tick_lateness_duration_seconds_{min,max,sum,count}` | `Mmsc` | `s` | none | Lateness of smooth-mode batch ticks relative to their scheduled instant. |
| `receiver.traffic_generator` | `smooth.late_remaining_batches` | `smooth_late_remaining_batches_{min,max,sum,count}` | `Mmsc` | `{batch}` | none | Number of batches remaining when smooth mode detects that a run is late. |
| `receiver.traffic_generator` | `smooth.late_remaining_items` | `smooth_late_remaining_items_{min,max,sum,count}` | `Mmsc` | `{item}` | none | Number of signal items remaining when smooth mode detects that a run is late. |
| `receiver.traffic_generator` | `smooth.payload_generate_duration` | `smooth_payload_generate_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | none | Wall-clock time spent generating or cloning one smooth-mode payload. |
| `receiver.traffic_generator` | `smooth.payload_send_duration` | `smooth_payload_send_duration_seconds_{min,max,sum,count}` | `HistogramNormal` | `s` | none | Wall-clock time spent sending one smooth-mode payload into the downstream channel. |
| `receiver.traffic_generator` | `smooth.run_batches` | `smooth_run_batches` | `Gauge<u64>` | `{batch}` | none | Smooth-mode configured batches per one-second run. |
| `receiver.traffic_generator` | `smooth.runs_started` | `smooth_runs_started_total` | `Counter<u64>` | `{run}` | none | Number of smooth-mode production runs started. |
| `receiver.traffic_generator` | `smooth.schedule_deadline_misses` | `smooth_schedule_deadline_misses_total` | `Counter<u64>` | `{miss}` | none | Number of times smooth mode detects a missed scheduling boundary. |
| `receiver.traffic_generator.smooth.payload.sends` | `attempts` | `attempts_total` | `Counter<u64>` | `{attempt}` | `attempt`, `outcome` | Number of smooth-mode payload send attempts. |
| `receiver.traffic_generator.smooth.runs` | `terminations` | `terminations_total` | `Counter<u64>` | `{run}` | `outcome` | Number of smooth-mode production runs terminated. |

<!-- markdownlint-enable MD013 -->
