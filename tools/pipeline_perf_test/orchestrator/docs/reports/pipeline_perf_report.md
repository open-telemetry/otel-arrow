# Report Plugin: pipeline_perf_report

**Class**: `lib.impl.strategies.hooks.reporting.pipeline_perf_report.PipelinePerfReportHook`

**Config Class**: `lib.impl.strategies.hooks.reporting.pipeline_perf_report.PipelinePerfReportConfig`

**Supported Contexts:**

- FrameworkElementHookContext

**Description:**

```python
"""
Reporting strategy hook for generating a pipeline performance report.

This hook collects telemetry metrics from the load generator, system under test,
and backend components within a specified time window, computes aggregate metrics,
and compiles a summary and detailed report of the pipeline's performance.

Attributes:
    config (PipelinePerfReportConfig): Configuration instance defining components
        involved and report sections to include.
    report_start (Optional[pd.Timestamp]): Timestamp marking the start of the report period.
    report_end (Optional[pd.Timestamp]): Timestamp marking the end of the report period.
    duration (Optional[float]): Duration of the report period in seconds.
"""
```

**Example YAML:**

```yaml
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - OTLP
            - format:
                template: {}
              destination:
                console: {}
          between_events:
            start:
              name: test_framework.test_start
              attributes:
                test.name: Test OTLP - Max Logs / Sec
            end:
              name: test_framework.test_end
              attributes:
                test.name: Test OTLP - Max Logs / Sec
```

## Supported Aggregations

- comparison

## Sample Outputs

### Comparison Aggregation

```markdown
# Pipeline Perf Comparison Report

## Metadata:
...

## Summary:

| metric_name                       |   PerfReprort - OTLP |   PerfReprort - OTAP |
|:----------------------------------|---------------------:|---------------------:|
| Duration                          |          45.8919     |          55.851      |
| Logs receive rate (avg)           |      855638          |      943012          |
| Logs failed at loadgen            |           0          |           0          |
| Logs lost in transit              |           0          |           0          |
| Logs received by backend          |           2.2475e+07 |           2.5535e+07 |
| Logs successfully sent by loadgen |           2.2475e+07 |           2.5535e+07 |
| Percentage of logs lost           |           0          |           0          |
| Total logs attempted              |           2.2475e+07 |           2.5535e+07 |
| Total logs lost                   |           0          |           0          |
```
