# `monitoring_strategies`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
| --------- | ------ | ----- | ------------ | -------------------- |
| `docker_component` | `lib.impl.strategies.monitoring.docker_component` | `DockerComponentMonitoringStrategy` | `DockerComponentMonitoringConfig` | Strategy for monitoring a docker container resource mentrics via the python docker API client |
| `process_component` | `lib.impl.strategies.monitoring.process_component` | `ProcessComponentMonitoringStrategy` | `ProcessComponentMonitoringConfig` | Strategy for monitoring a Process via the python psutil library |
| `prometheus` | `lib.impl.strategies.monitoring.prometheus` | `PrometheusMonitoringStrategy` | `PrometheusMonitoringConfig` | Strategy for monitoring a prometheus endpoint for a component |

---

## `docker_component`

**Class**: `lib.impl.strategies.monitoring.docker_component.DockerComponentMonitoringStrategy`

**Config Class**: `lib.impl.strategies.monitoring.docker_component.DockerComponentMonitoringConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Strategy for monitoring a docker container resource mentrics via the python docker API client.

Monitoring strategies define how to start, stop, and collect data from a component's monitoring
system. Concrete implementations should specify how to track, log, and aggregate monitoring
data for a given component.

Methods:
    start(component, ctx): Begin the monitoring process for the component.
    stop(component, ctx): Stop the monitoring process.
    collect(component, ctx): Collect and return monitoring data as a dictionary.
"""
```

**Example YAML:**

```yaml
components:
  backend-service:
    deployment:
      docker: ...   # Component should be deployed via docker strategy.
    monitoring:
      docker_component:
        interval: 1
```

## `process_component`

**Class**: `lib.impl.strategies.monitoring.process_component.ProcessComponentMonitoringStrategy`

**Config Class**: `lib.impl.strategies.monitoring.process_component.ProcessComponentMonitoringConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Strategy for monitoring a Process via the python psutil library.

Monitoring strategies define how to start, stop, and collect data from a component's monitoring
system. Concrete implementations should specify how to track, log, and aggregate monitoring
data for a given component.

Methods:
    start(component, ctx): Begin the monitoring process for the component.
    stop(component, ctx): Stop the monitoring process.
    collect(component, ctx): Collect and return monitoring data as a dictionary.
"""
```

**Example YAML:**

```yaml
components:
  backend-service:
    deployment:
      process: ...   # Component should be deployed via process strategy.
    monitoring:
      process_component:
        interval: 1
```

## `prometheus`

**Class**: `lib.impl.strategies.monitoring.prometheus.PrometheusMonitoringStrategy`

**Config Class**: `lib.impl.strategies.monitoring.prometheus.PrometheusMonitoringConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Strategy for monitoring a prometheus endpoint for a component.

Monitoring strategies define how to start, stop, and collect data from a component's monitoring
system. Concrete implementations should specify how to track, log, and aggregate monitoring
data for a given component.

Methods:
    start(component, ctx): Begin the monitoring process for the component.
    stop(component, ctx): Stop the monitoring process.
    collect(component, ctx): Collect and return monitoring data as a dictionary.
"""
```

**Example YAML:**

```yaml
components:
  backend-service:
    monitoring:
      prometheus:
        endpoint: http://localhost:8888/metrics
        include:
          - otelcol_exporter_send_failed_log_records_total
          - otelcol_exporter_sent_log_records_total
          - otelcol_process_cpu_seconds_total
```

---

## Memory Metrics: What We Measure and How

### `docker_component` — `container.memory.usage`

- **Source**: Docker Python SDK — `container.stats(stream=False)["memory_stats"]["usage"]`
- **What it is**: The total memory charged to the container's **Linux cgroup**.
  This includes the process's own allocations **plus** kernel page cache, buffers,
  and other memory the kernel charged to the cgroup.
- **Unit**: bytes
- **OTel instrument**: `Gauge("container.memory.usage")`
- **Polling**: Background thread at a configurable interval (default 1 s).

### `process_component` — `process.memory.usage`

- **Source**: `psutil.Process(pid).memory_info().rss` (summed over the process and
  all its children, recursively).
- **What it is**: The **Resident Set Size (RSS)** — physical RAM pages currently
  mapped into the process's address space. Does *not* include swap or
  file-backed pages that have been evicted.
- **Unit**: bytes
- **OTel instrument**: `Gauge("process.memory.usage")`
- **Polling**: Background thread at a configurable interval (default 1 s).

### How These Compare to Production Observability Tools

| Tool / Metric | Underlying value | Relation to our metrics |
|---|---|---|
| `kubectl top pods` | `container_memory_working_set_bytes` = cgroup usage − inactive file cache | **Lower** than `container.memory.usage`, **close to but ≥** RSS |
| `docker stats` (MEM USAGE column) | cgroup `usage` (same as ours) | **Equal** to `container.memory.usage` |
| `htop` / `ps rss` | RSS | **Equal** to `process.memory.usage` |
| Kubernetes OOM killer | triggers on cgroup `usage` hitting the limit | **Equal** to `container.memory.usage` threshold |
| Prometheus `container_memory_rss` (cAdvisor) | cgroup-level RSS (sum of all processes in cgroup) | Close to `process.memory.usage` but may differ slightly |

### TODO / Open Questions

- **Emit working-set alongside cgroup usage for Docker components.**
  The Docker stats response contains `memory_stats.stats.inactive_file`, so we
  could compute `working_set = usage - inactive_file` to mirror what
  `kubectl top pods` reports. This would make perf-test numbers directly
  comparable to what users see in Kubernetes dashboards.

- **Emit RSS for Docker components too.**
  `memory_stats.stats.rss` is available in the Docker stats payload and would
  give a number comparable to `process.memory.usage`, enabling apples-to-apples
  comparison between Docker-deployed and process-deployed components.

- **Clarify which metric matters most for different audiences.**
  Platform teams care about cgroup usage (it determines OOM kills and resource
  quota). Developers optimizing code care about RSS (isolates their code's
  footprint from kernel caching). Should we surface both in reports, or pick a
  primary and make the other opt-in?

- **Peak vs. average.**
  We currently report instantaneous samples at the polling interval. Reports
  aggregate min/mean/max over observation windows. Consider whether we need a
  high-water-mark (peak RSS or peak cgroup usage) captured at a finer
  granularity than the polling interval.
