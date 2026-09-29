# Report Plugin: sql_report

**Class**: `lib.impl.strategies.hooks.reporting.sql_report.SQLReportHook`

**Config Class**: `lib.impl.strategies.hooks.reporting.sql_report.SQLReportConfig`

**Supported Contexts:**

- FrameworkElementHookContext

**Description:**

```python
"""
Base class for reporting hooks that generate and persist structured reports.

This abstract strategy defines a consistent workflow for generating reports from
execution context data and sending them through any configured report pipelines.

Subclasses must implement `_execute(ctx) -> Report`, which defines the logic for
generating the report based on the context.

Lifecycle:
    1. `_execute` is called to produce a `Report` object.
    2. All configured pipelines from the hook config are executed on the report.
    3. The report is saved to the suite's `ReportRuntime` under its name.

Attributes:
    config (StandardReportingHookStrategyConfig): Configuration object containing metadata
        and any registered pipelines.
    pipelines (List[ReportingPipeline]): Pipelines that process or export the report after creation.

Methods:
    _execute(ctx: BaseContext) -> Report:
        Abstract method that must be implemented by subclasses to define report generation logic.

    execute(ctx: BaseContext):
        Executes the full report generation lifecycle, including pipeline execution and persistence.

Raises:
    NotImplementedError: If `_execute` is not implemented in a subclass.
"""
```

**Example YAML:**

```yaml
hooks:
  run:
    post:
      - sql_report:
          name: PerfReprort - OTLP
          report_config:
            load_tables:
                foo_table:
                    path:
                    format:
            queries:
            - name: "aggregate_metrics"
                sql: |
            result_tables:
                - foo
                - bar
            write_tables:
                foo_table:
                    path:
                    format:
          # OR report_config_file: ./report_configs/whatever.yaml
          output:
            - format:
                template: {}
              destination:
                console: {}
```

## Scenario container metrics

Set `scope_container_metrics_to_scenario: true` in the report definition
(alongside `queries`, including in an external report YAML file) to exclude
Docker measurements from previous scenarios. The benchmark reports enable this
option for CPU, memory, allocated-core counts, and network calculations.

The Docker monitor records a `container_monitor_start` event with its component
name and shortened container ID. The report uses events from the current test
execution to filter the `metrics` table before running SQL. This works after
containers have been removed and with OpenTelemetry SDK versions that continue
exporting the last gauge value with a fresh collection timestamp.

Measurements without a container ID, such as process and Prometheus metrics,
are preserved. Network window functions must also partition by
`"metric_attributes.container_id"` to avoid subtracting counters from different
containers. That nullable column is available even for process-only scenarios.

The option defaults to false for existing custom and suite-wide reports.
Enabling it requires `test.name` and `test.start` metadata and Docker monitor
events from the same run; historical data without these events cannot identify
the current scenario's containers.

## Supported Aggregations

*None.*

## Sample Outputs
