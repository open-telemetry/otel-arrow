"""
Standard Reporting strategy module for structured test output and report persistence.

This module provides the foundational classes for building and executing reporting
hooks in a test framework. It supports generating structured reports from test suite
execution contexts and running them through configurable post-processing pipelines.

Classes:
    - ReportRuntime: A runtime data structure used to store reports generated during
      the execution of a test suite. It maps report names to `Report` objects and can
      be attached to a suite's runtime for later access.

    - StandardReportingStrategy: An abstract base class for hook strategies that generate
      reports. It defines the interface for implementing report generation via `_execute`,
      and handles common concerns such as executing output pipelines and saving reports
      to the suite's runtime.

Typical usage:
    A reporting hook subclass implements `_execute()` to extract data from the test
    context and produce a `Report` object. The `StandardReportingStrategy` base class
    then handles pipeline execution (e.g., saving to file, uploading to dashboards)
    and ensures the report is stored and accessible in the suite context.

Example:
    class MyCustomReportHook(StandardReportingStrategy):
        def _execute(self, ctx: BaseContext) -> Report:
            # logic to create a Report from test data
            return MyReport(...)

        # Then `execute()` will automatically:
        # - run pipelines
        # - store report in suite's ReportRuntime

This module is intended to be extended by plugin authors and test engineers who need to
generate and persist structured data from test execution for analytics, traceability,
and debugging.
"""

from abc import abstractmethod
from dataclasses import dataclass, field
from typing import Optional, Dict

from .....core.context.base import BaseContext
from .....core.framework.report import Report
from .....core.strategies.reporting_hook_strategy import ReportingHookStrategy
from .....runner.schema.reporting_hook_config import StandardReportingHookStrategyConfig
from ...common.report import get_report_pipelines


@dataclass
class ReportRuntime:
    """
    Runtime container for storing named reports generated during test execution.

    This class is used to persist one or more reports associated with a test suite,
    making them available for downstream consumption (e.g. display, export, assertions).

    Attributes:
        reports (Dict[str, Report]): A dictionary mapping report names to Report objects.
            Initialized as an empty dictionary by default.
    """

    reports: Optional[Dict[str, Report]] = field(default_factory=dict)


class StandardReportingStrategy(ReportingHookStrategy):
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

    def __init__(self, config: StandardReportingHookStrategyConfig):
        self.config = config
        self.pipelines = get_report_pipelines(self.config)

    @abstractmethod
    def _execute(self, ctx: BaseContext) -> Report:
        """Implemented by strategy hooks to turn contexts into reports."""

    def execute(self, ctx: BaseContext):
        """Generates a report, passes it through pipelines, and stores result in the report runtime."""
        report = self._execute(ctx)

        # Run the report through any configured output pipelines
        for pipeline in self.pipelines:
            pipeline.execute(report=report, ctx=ctx)

        # Save reports to the runtime
        ts = ctx.get_suite()
        runtime: ReportRuntime = ts.get_or_create_runtime("reports", ReportRuntime)
        runtime.reports[report.report_name] = report
        ts.set_runtime_data("reports", runtime)
