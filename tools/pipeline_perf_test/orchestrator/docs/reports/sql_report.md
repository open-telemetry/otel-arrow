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

## Supported Aggregations

*None.*

## Sample Outputs
