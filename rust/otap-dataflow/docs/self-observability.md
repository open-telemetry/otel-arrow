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
- In Prometheus, metric names derive from the instrument field name, UCUM unit,
  and type without an `otel_scope_name` prefix. Filter by `otel_scope_name` in
  PromQL when scopes share an instrument name, such as
  `messages_total{otel_scope_name="node.input"}`. In OTLP, default `Metric.name`
  values are qualified as `<metric_set>.<field>` (such as `node.input.messages`)
  unless overridden by a metric view.
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

### Core engine, pipeline, runtime, channel, node, flow, and boundary metrics

The authoritative catalog of core metrics, scopes (`otel_scope_name`),
instrument types, units, datapoint attributes, and descriptions is generated
directly from the Weaver registry (`semconv/groups/metric.engine.yaml`) in the
[Core metrics reference](telemetry/core-metrics-reference.md).

Within `tokio.runtime`, `worker.count`, `task.active.count`,
`global.task.queue.size`, `worker.busy.time`, `worker.park.count`, and
`worker.park.unpark.count` are available without `tokio_unstable`. All other
`tokio.runtime` instruments listed in the reference require a build with
`RUSTFLAGS="--cfg tokio_unstable" cargo build --release`, run from
`rust/otap-dataflow`. The cumulative Tokio runtime counters also require
64-bit atomics. Enabling `tokio_metrics` or increasing `runtime_metrics`
does not enable instruments omitted at compile time.

On builds with `tokio_unstable` and 64-bit atomics,
`poll.time.histogram.enabled` indicates whether task poll timing is active.
With `engine.runtime.tokio.task_poll_time_histogram: true` and
`policies.telemetry.tokio_metrics` enabled, `poll.time.histogram.bucket.count`
reports a cumulative poll
count for each disjoint `bucket.range.us` datapoint attribute. Ranges are in
microseconds: `[0,100)`, `[100,200)`, and so on through `[800,900)`; the final
`[900,+inf)` includes **all** polls lasting at least 900 microseconds.
These are not cumulative upper-bound buckets: each poll belongs to one range.

When upgrading from an engine that used `LocalSet` to one that uses
`LocalRuntime`, expect a change in what Tokio counts: pipeline tasks that
`LocalSet` previously managed are now counted directly by the runtime.
Both `task.active.count` (tasks currently alive) and `spawned.tasks.count`
(tasks spawned since runtime startup) can be higher under the same workload.
Use a post-upgrade baseline when comparing task counts or tuning alerts.

On `channel.sender` and `channel.receiver`, the `channel.kind` entity attribute
distinguishes `pdata` channels, which carry the `signal` datapoint attribute,
from `control` channels. See [Node and flow metrics](node-and-flow-metrics.md)
for node boundary and flow semantics.

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
