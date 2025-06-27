# Report Plugin: test_report

**Class**: `lib.impl.strategies.hooks.reporting.report_test_summary.TestSummaryReportHook`

**Config Class**: `lib.impl.strategies.hooks.reporting.report_test_summary.TestSummaryConfig`

**Supported Contexts:**

- FrameworkElementHookContext

**Description:**

```python
"""
Reporting hook that generates a structured summary report from a test suite execution.

This hook traverses the context hierarchy of a test suite to collect detailed execution
metadata from individual test cases, steps, and hooks. The result is compiled into a
pandas DataFrame for each test case and included in a `TestSummaryReport`.

Key behavior:
    - Operates only when attached to test elements (not components).
    - Iterates through test case contexts in the suite.
    - Captures metadata from each test step and any associated hooks.
    - Produces per-test-case DataFrames, each containing both steps and hooks.
    - Metadata includes context name, execution status, duration, depth, and parent linkage.

Raises:
    RuntimeError: If the hook is mistakenly used outside of a `FrameworkElementHookContext`.
    RuntimeError: If the framework element cannot be resolved from the context.

Returns:
    TestSummaryReport: An enriched report object containing execution summaries
    for each test in the suite.

Typical usage:
    This hook is useful for:
    - Generating structured test execution summaries.
    - Producing post-run analytics or artifacts for CI pipelines.
    - Debugging and traceability in complex test environments.
"""
```

**Example YAML:**

```yaml
hooks:
  post:
    - test_report:
        name: TestReport - My Cool Test
        output:
        - format:
            template: {}
          destination:
            console: {}
```

## Supported Aggregations

*None.*

## Sample Outputs
