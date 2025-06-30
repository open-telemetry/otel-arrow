# `report_writers`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
|-----------|--------|-------|--------------|----------------------|
| `noop` | `lib.impl.strategies.common.report` | `NoopDestination` | `NoopDestinationConfig` | A destination writer that performs no action |
| `console` | `lib.impl.strategies.common.report` | `ConsoleDestination` | `ConsoleDestinationConfig` | Writes report data to the console (stdout) or via a logger |

---

## `noop`

**Class**: `lib.impl.strategies.common.report.NoopDestination`

**Config Class**: `lib.impl.strategies.common.report.NoopDestinationConfig`

**Description:**

```python
"""
A destination writer that performs no action.

This writer can be used as a placeholder or stub where a destination
is required but no actual writing or output should be performed.

Args:
    config (NoopDestinationConfig): Configuration for the writer (not used).

Methods:
    write(formatted_data, ctx):
        Intentionally does nothing with the provided data.
"""
```

**Example YAML:**

```yaml
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - Max Rate
          output:
            - format:
                template: {}
              destination:
                noop: {}
```

## `console`

**Class**: `lib.impl.strategies.common.report.ConsoleDestination`

**Config Class**: `lib.impl.strategies.common.report.ConsoleDestinationConfig`

**Description:**

```python
"""
Writes report data to the console (stdout) or via a logger.

Depending on configuration, this destination can either print the formatted
report directly to stdout or send it to a configured logger at a specified
log level.

Args:
    config (ConsoleDestinationConfig): Configuration for this destination writer.

Methods:
    write(formatted_data, ctx):
        Outputs the formatted data to the console or logger.
"""
```

**Example YAML:**

```yaml
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - Max Rate
          output:
            - format:
                template: {}
              destination:
                console: {}
```
