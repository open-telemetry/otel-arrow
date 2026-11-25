# `monitoring_strategies`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
| --------- | ------ | ----- | ------------ | -------------------- |
| `docker_component` | `lib.impl.strategies.monitoring.docker_component` | `DockerComponentMonitoringStrategy` | `DockerComponentMonitoringConfig` | Strategy for monitoring a docker container resource mentrics via the python docker API client |
| `kubernetes_component` | `lib.impl.strategies.monitoring.kubernetes_component` | `KubernetesComponentMonitoringStrategy` | `KubernetesComponentMonitoringConfig` | Monitor adopted containers using Kubernetes metrics-server without a proxy |
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

## `kubernetes_component`

**Class**: `lib.impl.strategies.monitoring.kubernetes_component.KubernetesComponentMonitoringStrategy`

**Config Class**: `lib.impl.strategies.monitoring.kubernetes_component.KubernetesComponentMonitoringConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Monitor adopted containers using Kubernetes metrics-server without a proxy.
"""
```

**Notes:**

Requires existing.kubernetes, successful adoption, and metrics-server. Uses the
adoption kubeconfig; no engine metrics endpoint is required.

**Example YAML:**

```yaml
components:
  existing-service:
    existing:
      kubernetes:
        namespace: telemetry
        label_selector: app=my-service
    monitoring:
      kubernetes_component:
        interval: 5
        max_age_seconds: 120
```

**Collection contract:**

Uses the adoption kubeconfig, context, namespace, and selector without persisting
credential changes. Any cluster exposing `metrics.k8s.io` is supported; credentials
need `list` access to core `pods` and `metrics.k8s.io` `pods` in the selected namespace.
It does not install metrics-server or require application telemetry or a proxy.
Defaults are `interval: 5`, `max_age_seconds: 120`, and `request_timeout: 10`
(seconds per API read).

Every regular container and native sidecar is included:

| Instrument | Raw unit | Meaning |
| ---------- | -------- | ------- |
| `container.cpu.usage` | CPU cores | All container threads/cores, averaged over the source window; not normalized to its CPU limit |
| `container.memory.working_set` | Bytes | Container cgroup working set, not complete process RSS |

Attributes preserve namespace, pod name/UID, node, container name/ID,
`sample.timestamp`, `sample.window_seconds`, and `sample.observed_at`.
Only fresh source timestamps produce measurements; faster polling does not create
independent metrics-server observations. Duplicate or older windows are skipped
per pod without lowering its last recorded timestamp. Older windows log a warning;
newer windows from other pods are still recorded. All batches must pass the age,
completeness, and identity checks, even when their timestamps repeat or regress.
Framework row timestamps are collection times. Use the source timestamps/windows
for measurement boundaries, including windows that overlap warmup or begin before
monitoring starts.

Missing, stale, malformed, or incomplete measurements and changed pod/container
identities fail monitoring. Startup validates an initial batch synchronously.
Background errors are logged and propagated at monitoring shutdown or collection.

The [example](../../examples/existing-kubernetes.yaml) demonstrates source-window
filtering with the existing SQL report hook. Report metric selection, conversions,
aggregation, and presentation remain suite-owned.

The [shared observation template](../../templates/README.md) supplies the lifecycle
without prescribing metrics or report queries. The Kubernetes example's CPU/memory
report is an example choice, not a built-in resource-reporting policy.

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

### Named component endpoints

Select a named endpoint on every published component target:

```yaml
monitoring:
  prometheus:
    target_endpoint: metrics
    interval: 1
    request_timeout: 10
```

Set exactly one of `endpoint` and `target_endpoint`. Missing targets/endpoints fail
startup. The provider owns discovery, authentication, validation and cleanup;
Prometheus consumes URLs or `fetch(timeout) -> str` endpoints. See the
[component model](../../lib/core/component/README.md#components-targets-and-endpoints).

The [Kubernetes provider](existing_strategies.md#kubernetes) publishes per-pod
endpoints reached through adoption-owned authenticated port-forward tunnels.

Each sample carries `component_name` and `component.target.name` to identify its
workload and instance. Provider identity takes precedence over scraped labels.
Named targets log scrape errors and retry on the next pass, matching direct-URL
behavior. Healthy targets continue sampling. Finite `count` bounds attempted
passes. Target discovery and identity failures stop monitoring.

SQL reports expose `monitoring.<component>.status` (`complete`, `gaps`, or
`incomplete`) and failed-scrape counts. Reports validate required measurements and
coverage. Raw `metrics`, `spans`, and `events` exports precede SQL/validation to
retain diagnostics.

`pipeline_perf_report` supports one counter series per component and metric.
It rejects independent series that would require rate alignment; use
`sql_report` for multi-target reporting.
