"""
report_test_summary.py

Defines a test summary reporting strategy for structured aggregation and presentation
of test execution results within a telemetry and testing framework.

This module includes:

- TestSummaryConfig: Configuration model for the test summary report strategy.

- TestSummaryReport: Report class that formats and structures test execution metadata,
  including test names, statuses, durations, and hierarchical depth for clear
  visualization.

- TestSummaryReportHook: A reporting hook that collects detailed execution data
  from test cases, steps, and hooks within a test suite context. It compiles this data
  into pandas DataFrames and produces a comprehensive summary report.

Features:
- Supports hierarchical indentation of test steps and hooks in the report output.
- Provides console colorization of test statuses for easy visual identification.
- Designed to integrate as a post-execution hook in test suites for reporting and
  diagnostics.
- Raises informative errors if used outside appropriate test element contexts.

Typical usage involves attaching the hook to test elements to generate structured
reports for CI pipelines, debugging, and analytics.

"""
from typing import ClassVar

import yaml
import pandas as pd

from .....core.context.base import BaseContext
from .....core.context.framework_element_hook_context import FrameworkElementHookContext
from .....core.context.framework_element_contexts import StepContext
from .....core.framework.report import Report
from .....runner.schema.reporting_hook_config import StandardReportingHookStrategyConfig

from .....runner.registry import hook_registry, PluginMeta

from .standard_reporting_strategy import StandardReportingStrategy

STRATEGY_NAME = "test_report"


@hook_registry.register_config(STRATEGY_NAME)
class TestSummaryConfig(StandardReportingHookStrategyConfig):
    """Configuration for test summary report"""


def color_status_console(status: str) -> str:
    color_map = {
        "error": "\033[91m",  # Red
        "skipped": "\033[90m",  # Grey
        "success": "\033[92m",  # Green
    }
    reset = "\033[0m"
    color = color_map.get(status.lower(), "\033[0m")
    return f"{color}{status}{reset}"


class TestSummaryReport(Report):

    REPORT_TYPE: ClassVar[str] = STRATEGY_NAME

    def to_template_dict(self):
        data = self.to_dict()
        data["metadata"] = yaml.dump(self.metadata, indent=2)

        for key, df in self.results.items():
            filtered_df = df[["name", "status", "duration", "depth"]].copy()

            def format_row(row):
                indent = "\t" * int(row["depth"])
                return {
                    "name": row["name"],
                    "status": color_status_console(row["status"]),
                    "duration": row["duration"],
                    "indent": indent,
                }

            records = [format_row(row) for _, row in filtered_df.iterrows()]
            data["results"][key] = records
        return data

    def display_template(self):
        return """# Test Summary Report

## Metadata:

{{ report.metadata }}
{% for key, records in report.results.items() %}
Results for: {{ key }}
{% for row in records %}
{{ row.indent }}Name: {{ row.name }}
{{ row.indent }}Status: {{ row.status }}
{{ row.indent }}Duration: {{ row.duration }}s
{{ row.indent }}--------------------------
{% endfor %}
{% endfor %}
"""


@hook_registry.register_class(STRATEGY_NAME)
class TestSummaryReportHook(StandardReportingStrategy):
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
    PLUGIN_META = PluginMeta(
        supported_contexts=[FrameworkElementHookContext.__name__],
        installs_hooks=[],
        yaml_example="""
hooks:
  post:
    - test_report:
        name: TestReport - My Cool Test
        output:
        - format:
            template: {}
          destination:
            console: {}
"""
    )
    def __init__(
        self,
        config: TestSummaryConfig,
    ):
        super().__init__(config)
        self.config = config

    def _execute(self, ctx: BaseContext) -> Report:
        if not isinstance(ctx, FrameworkElementHookContext):
            raise RuntimeError(
                f"{STRATEGY_NAME} report must be installed on standard test element (not component)"
            )
        element = ctx.get_framework_element()
        if not element:
            raise RuntimeError(
                f"{STRATEGY_NAME} error fetching test element from context"
            )
        ts = ctx.get_suite()
        results = {}
        for test_context in ts.context.child_contexts:
            if isinstance(test_context, FrameworkElementHookContext):
                continue
            test_name = test_context.name
            test_data = []
            for step_ctx in test_context.child_contexts:
                # This is either a TestStep or a TestHook
                step_data = step_ctx.to_dict()
                step_name = step_ctx.name
                step_data.pop("child_contexts", None)
                step_data["parent_context"] = test_name
                step_data["depth"] = 1
                test_data.append(step_data)
                # Only TestStep will have child hooks
                if isinstance(step_ctx, StepContext):
                    for hook_ctx in step_ctx.child_contexts:
                        hook_data = hook_ctx.to_dict()
                        hook_data.pop("child_contexts", None)
                        hook_data["parent_context"] = step_name
                        hook_data["depth"] = 2
                        test_data.append(hook_data)

            results[test_name] = pd.DataFrame(test_data)

        report = TestSummaryReport.from_context(self.config.name, ctx)
        report.set_results(results)
        return report
