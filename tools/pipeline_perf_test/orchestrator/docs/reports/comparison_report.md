# Report Plugin: comparison_report

**Class**: `lib.impl.strategies.hooks.reporting.comparison_report.ComparisonReportHook`

**Config Class**: `lib.impl.strategies.hooks.reporting.comparison_report.ComparisonReportConfig`

**Supported Contexts:**

- FrameworkElementHookContext

**Description:**

```python
"""
A reporting hook that aggregates multiple reports of the same type for comparison.

This hook retrieves previously executed reports specified in the configuration,
validates that all reports share the same report type, and then aggregates their
results side-by-side using a comparison mode. The aggregated data is stored in a
new `ComparisonReport` instance.

Key behaviors:

- Ensures prerequisite reports have run and are available in the runtime context.
- Validates consistency of report types among the reports to aggregate.
- Uses the report type's aggregation logic to produce a combined comparison report.
- Supports custom labeling of columns via configuration.

Raises:

- ValueError if prerequisite reports are missing or have mismatched types.
- ValueError if no valid report type can be determined.

Returns:

- A `ComparisonReport` containing aggregated results ready for further
reporting or rendering.

Typical usage:

- Attach as a post-processing hook to combine and compare metrics across multiple
report executions, e.g., for comparing performance runs or test results.
"""
```

**Example YAML:**

```yaml
hooks:
  run:
    post:
    - comparison_report:
          name: Compare PipelinePerf
          reports:
            - PerfReprort - OTLP
            - PerfReprort - OTAP
          report_config:
            include_sections:
              component_detail: true
          output:
            - format:
                template: {}
              destination:
                console: {}
```

## Supported Aggregations

*None.*

## Sample Outputs

### Example Aggregation

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
