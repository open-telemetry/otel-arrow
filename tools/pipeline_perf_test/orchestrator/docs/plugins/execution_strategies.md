# `execution_strategies`

## Plugin Summary

| Type Name | Module | Class | Config Class | Description Summary |
|-----------|--------|-------|--------------|----------------------|
| `pipeline_perf_loadgen` | `lib.impl.strategies.execution.pipeline_perf_loadgen` | `PipelinePerfLoadgenExecution` | `PipelinePerfLoadgenConfig` | Execution strategy implementation for controlling the pipeline performance load generator |

---

## `pipeline_perf_loadgen`

**Class**: `lib.impl.strategies.execution.pipeline_perf_loadgen.PipelinePerfLoadgenExecution`

**Config Class**: `lib.impl.strategies.execution.pipeline_perf_loadgen.PipelinePerfLoadgenConfig`

**Supported Contexts:**

- StepContext

**Description:**

```python
"""
Execution strategy implementation for controlling the pipeline performance load generator.

This strategy starts and stops the load generator by issuing HTTP POST requests
to designated 'start' and 'stop' endpoints.

Attributes:
    type (ClassVar[Literal["pipeline_perf_loadgen"]]): Identifier for this strategy.
    config (PipelinePerfLoadgenConfig): Configuration instance with load parameters.
    default_hooks (dict): Placeholder for lifecycle hooks (empty by default).
    start_endpoint (str): Fully qualified URL for starting the load generator.
    stop_endpoint (str): Fully qualified URL for stopping the load generator.

Methods:
    start(component, ctx): Sends a POST request to start load generation with configured parameters.
    stop(component, ctx): Sends a POST request to stop load generation.
"""
```

**Example YAML:**

```yaml
components:
  load-generator:
    execution:
      pipeline_perf_loadgen:
        endpoint:  "http://localhost:5001/"
        threads: 1
        body_size: 25
        num_attributes: 2
        attribute_value_size: 15
        batch_size: 10000
```
