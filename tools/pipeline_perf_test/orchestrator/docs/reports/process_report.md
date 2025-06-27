# Report Plugin: process_report

**Class**: `lib.impl.strategies.hooks.reporting.process_report.ProcessReportHook`

**Config Class**: `lib.impl.strategies.hooks.reporting.process_report.ProcessReportHookConfig`

**Supported Contexts:**

- FrameworkElementHookContext

**Description:**

```python
"""
Reporting strategy that generates a process-level metrics report across components.

This hook collects telemetry metrics (CPU, memory, network) for all or a subset
of components in the test suite and builds a structured report summarizing
system resource usage during execution.

The report includes:
    - Component-level summaries (min/mean/max for gauges; delta for counters)
    - Optional per-component detailed time series data
    - Optional observation window start/end times and total duration (if configured)

Attributes:
    config (ProcessReportHookConfig): Configuration specifying what to include,
        how to filter components, and whether to restrict the report to a
        specific time window using start/end events.
    report_start (datetime or None): Timestamp for the beginning of the observation
        window, inferred from telemetry events.
    report_end (datetime or None): Timestamp for the end of the observation window.
    duration (float or None): Duration of the observation window in seconds.

Methods:
    _execute(ctx: BaseContext) -> Report:
        Internal method to collect telemetry data, transform it, and populate
        a structured `ProcessReport` instance with both raw and aggregated metrics.

Raises:
    RuntimeError: If `ctx` does not conform to expected context structure or
        if telemetry data cannot be resolved as expected.
"""
```

**Example YAML:**

```yaml
hooks:
  run:
    post:
    - process_report:
        name: resource_observation
        between_events: ["observation_start", "observation_stop"]
        include_sections:
          component_summary: true
          component_detail: false
```

## Supported Aggregations

- comparison

## Sample Outputs

### Comparison Aggregation

```markdown
# Process Comparison Report

## Metadata:

test.suite: Test OTLP Vs OTAP
...

## Process: otel-collector

| metric_name                      | Process OTLP   | Process OTAP   |
|:---------------------------------|:---------------|:---------------|
| delta(container.network.rx)      | 434.61 MB      | 115.77 MB      |
| delta(container.network.tx)      | 434.11 MB      | 473.57 MB      |
| max(container.cpu.usage)         | 4.05 cores     | 3.43 cores     |
| max(container.memory.usage)      | 128.87 MB      | 222.38 MB      |
| max(rate(container.network.rx))  | 59.04 MB/s     | 14.77 MB/s     |
| max(rate(container.network.tx))  | 58.78 MB/s     | 60.57 MB/s     |
| mean(container.cpu.usage)        | 3.82 cores     | 3.35 cores     |
| mean(container.memory.usage)     | 117.46 MB      | 213.00 MB      |
| mean(rate(container.network.rx)) | 53.88 MB/s     | 14.54 MB/s     |
| mean(rate(container.network.tx)) | 53.81 MB/s     | 59.47 MB/s     |
| min(container.cpu.usage)         | 3.25 cores     | 3.29 cores     |
| min(container.memory.usage)      | 111.25 MB      | 200.57 MB      |
| min(rate(container.network.rx))  | 48.50 MB/s     | 14.08 MB/s     |
| min(rate(container.network.tx))  | 48.25 MB/s     | 57.77 MB/s     |
```
