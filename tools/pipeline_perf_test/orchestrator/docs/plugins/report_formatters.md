# `report_formatters`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
|-----------|--------|-------|--------------|----------------------|
| `noop` | `lib.impl.strategies.common.report` | `NoopFormatter` | `NoopFormatterConfig` | A report formatter that performs no formatting and returns an empty string |
| `json` | `lib.impl.strategies.common.report` | `JsonFormatter` | `JsonFormatterConfig` | Formats a report as a JSON string using the specified configuration |
| `template` | `lib.impl.strategies.common.report` | `TemplateFormatter` | `TemplateFormatterConfig` | Formats a report using a Jinja2 template specified either by file path or inline string |

---

## `noop`

**Class**: `lib.impl.strategies.common.report.NoopFormatter`

**Config Class**: `lib.impl.strategies.common.report.NoopFormatterConfig`

**Description:**

```python
"""
A report formatter that performs no formatting and returns an empty string.

Useful as a placeholder or default when no output formatting is desired.
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
                noop: {}
              destination:
                console: {}
```

## `json`

**Class**: `lib.impl.strategies.common.report.JsonFormatter`

**Config Class**: `lib.impl.strategies.common.report.JsonFormatterConfig`

**Description:**

```python
"""
Formats a report as a JSON string using the specified configuration.

Args:
    config (JsonFormatterConfig): Configuration specifying JSON formatting options such as indentation.

Methods:
    format(report, _ctx) -> str:
        Converts the given report into a JSON-formatted string with the configured indentation.
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
                json: {}
              destination:
                console: {}
```

## `template`

**Class**: `lib.impl.strategies.common.report.TemplateFormatter`

**Config Class**: `lib.impl.strategies.common.report.TemplateFormatterConfig`

**Description:**

```python
"""
Formats a report using a Jinja2 template specified either by file path or inline string.

Args:
    config (TemplateFormatterConfig): Configuration specifying the template source.

Methods:
    format(report, _ctx) -> str:
        Renders the report using the template and returns the resulting string.

Raises:
    FileNotFoundError: If a template file path is provided but the file does not exist.
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
