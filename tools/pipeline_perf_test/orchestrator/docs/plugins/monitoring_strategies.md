# `monitoring_strategies`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
|-----------|--------|-------|--------------|----------------------|
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
