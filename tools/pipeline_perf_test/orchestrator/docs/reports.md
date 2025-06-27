# Reports Guide

## Overview

In this framework, reports are a flexible and extensible mechanism for
summarizing test execution data. They are most commonly used to extract
insights such as:

- System resource usage (CPU, memory, network)
- Load test behavior
- Error rates or pass/fail ratios
- Custom metrics tied to application-specific behavior

Reports are implemented as a type of Strategy Hook and can be attached to any
test framework element (e.g., Suite, Scenario, Step). Most commonly, they are
run as post-hooks at the Suite level.

## How Reports Work

### Report Lifecycle

Reports are generated after execution, typically via a post-hook.

During execution:

- Metrics, logs, and trace data are collected via OpenTelemetry and stored in
    pandas DataFrames.
- Events mark important phases (observation_start, observation_stop, etc.).
- Reports access the context, filter relevant data, and generate structured outputs.

Each report consists of:

- A configuration model to define behavior
- A subclass of the Report dataclass (with metadata and result data)
- A hook strategy that creates the report from runtime data

### Anatomy of a Report

```python
@dataclass
class Report:
    report_name: str
    report_time: datetime
    metadata: Dict[str, Any]
    results: Dict[str, pd.DataFrame]
    REPORT_TYPE: ClassVar[str] = "unknown_test_type"
```

Reports are dataframe-first objects. This makes them both easy to process (e.g.,
with pandas or numpy) and flexible to export (e.g., Markdown, JSON, Parquet).

### Configuration and Hook Strategy

Each report type is paired with a config model and a hook strategy.
The strategy class (usually a subclass of StandardReportingStrategy) is
responsible for:

- Interpreting the configuration
- Accessing telemetry/context data
- Filtering and aggregating metrics
- Constructing and returning a Report object

Example YAML configuration:

```yaml
hooks:
  run:
    post:
      - process_report:
          name: resource_report
          between_events: ["steady_start", "steady_stop"]
```

### Using Events to Define Time Windows

Many reports support time window filtering. This is accomplished via named
events, which are emitted during execution.

```yaml
between_events: ["observation_start", "observation_stop"]
```

The hook strategy will retrieve the timestamps for these events and slice
telemetry data accordingly. This enables highly focused analysis, such as
“CPU usage during steady state” or “error rate during ramp-up.”

### Output and Aggregation

Reports can be serialized in multiple formats via "FormattingStrategies":

- Markdown (via templates)
- JSON
- Parquet (planned)

And output to multiple destinations via "DestinationStrategies":

- Console or file output
- S3 upload (planned)

### Aggregation Modes

|Mode       |  Description                                                 |
|-----------|--------------------------------------------------------------|
|NONE       | Single-run report                                            |
|COMPARISON | Combines multiple reports to compare test runs               |
|TIMESERIES | Tracks values across time or test runs (optional/extendable) |

Each report subclass can define its own aggregate logic to merge and compare results.

## Developing a Custom Report

To create a new report:

### 1. Define a Config Class

```python
class MyReportConfig(BaseModel):
    my_option: bool = True
```

### 2. Create a Hook Strategy

```python
class MyReportHook(StandardReportingStrategy):
    def _execute(self, ctx: BaseContext) -> Report:
        # collect data, slice by event, compute metrics...
        report = MyReport(...)
        report.set_results({...})
        return report
```

### 3. Subclass the Report

```python
class MyReport(Report):
    REPORT_TYPE = "my_report_type"

    def to_template_dict(self):
        # transform DataFrames into markdown/JSON-safe dicts
```

### 4. Register with the Hook Registry

```python
@hook_registry.register_config("my_report")
class MyReportConfig(BaseModel): ...

@hook_registry.register_class("my_report")
class MyReportHook(StandardReportingStrategy): ...
```

### Best Practices

- Use event-based time slicing to isolate key test phases.
- Leverage telemetry and context objects — don't reinvent the wheel.
- Name your reports clearly for traceability and reproducibility.
- Include metadata such as component versions, timestamps, and configuration.
- Format outputs using consistent unit conversions and naming (bytes, cores, etc.).
- Start simple, and build incrementally (e.g., just a summary first, then details).
