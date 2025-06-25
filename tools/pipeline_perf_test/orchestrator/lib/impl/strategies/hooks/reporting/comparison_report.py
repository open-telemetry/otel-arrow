"""
Module: comparison_report.py

Provides classes to aggregate, compare, and render reports from multiple test runs or processes.

Classes:
    ComparisonReportConfig:
        Configuration model for the comparison reporting strategy.
        Defines which reports to aggregate, label keys, and additional report-specific configuration.

    ComparisonReportHook:
        Reporting hook that collects multiple prior reports from the test suite context,
        verifies their compatibility, and aggregates their results into a unified
        ComparisonReport using the specified configuration.

    ComparisonReport:
        Report class that acts as a wrapper to aggregate and format multiple individual reports
        into a comparative summary.
        Delegates templating and rendering to the underlying report type.

This module supports flexible report comparison scenarios by consolidating multiple reports,
validating consistent report types, and generating side-by-side comparative outputs
for improved visibility and analysis.
"""
from typing import ClassVar, Optional, List, Dict, Any

from pydantic import Field

from .....core.context.base import BaseContext
from .....core.context import FrameworkElementHookContext
from .....core.framework.report import Report, ReportAggregation
from .....runner.registry import hook_registry, PluginMeta
from .....runner.schema.reporting_hook_config import StandardReportingHookStrategyConfig
from .standard_reporting_strategy import StandardReportingStrategy, ReportRuntime


STRATEGY_NAME = "comparison_report"


@hook_registry.register_config(STRATEGY_NAME)
class ComparisonReportConfig(StandardReportingHookStrategyConfig):
    """
    Configuration model for the ComparisonReportHook strategy.

    Attributes:
        reports (List[str]): List of report names to be aggregated and compared.
        label_key (Optional[str]): Key used to label columns in the aggregated report.
            Defaults to "report.name".
        report_config (Optional[Dict[str, Any]]): Additional optional configuration
            parameters for report generation, stored as a dictionary.

    This config class extends the standard reporting hook configuration to specify
    parameters needed for comparing multiple reports within a test or performance
    suite context.
    """
    reports: List[str]
    label_key: Optional[str] = "report.name"
    report_config: Optional[Dict[str, Any]] = Field(default_factory=dict)


class ComparisonReport(Report):
    """
    Report class that aggregates and formats comparison data from multiple individual reports.

    Attributes:
        report_class (Optional[type[Report]]): The type of the underlying report being aggregated.
        config (Optional[ComparisonReportConfig]): Configuration object containing parameters
            for report aggregation and rendering.

    Methods:
        to_template_dict():
            Converts aggregated report results into a dictionary formatted for templating,
            delegating to the underlying report class's aggregation template method.

        display_template():
            Returns the template string for rendering the aggregated report. Raises an error
            if the underlying report class is not set.

    This class acts as a wrapper around other report classes to combine their outputs
    into a comparative view, supporting flexible templating and configuration.
    """
    report_class: Optional[type[Report]] = None
    config: Optional[ComparisonReportConfig] = None
    REPORT_TYPE: ClassVar[str] = STRATEGY_NAME

    def to_template_dict(self):
        config_cls = hook_registry.config.get(self.report_class.REPORT_TYPE)
        config_obj = config_cls(name=self.config.name, **self.config.report_config)
        return self.report_class.to_aggregate_template_dict(
            self.results, config_obj, mode=ReportAggregation.COMPARISON
        )

    def display_template(self):
        if not self.report_class:
            raise ValueError(
                f"Unable to render report to template: {STRATEGY_NAME}, missing report class."
            )
        return self.report_class.get_template(ReportAggregation.COMPARISON)


@hook_registry.register_class(STRATEGY_NAME)
class ComparisonReportHook(StandardReportingStrategy):
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
    PLUGIN_META = PluginMeta(
        supported_contexts=[FrameworkElementHookContext.__name__],
        installs_hooks=[],
        yaml_example="""
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
"""
    )

    def __init__(self, config: ComparisonReportConfig):
        super().__init__(config)
        self.config = config

    def _execute(self, ctx: BaseContext) -> Report:
        this_report: ComparisonReport = ComparisonReport.from_context(
            self.config.name, ctx
        )
        this_report.config = self.config

        ts = ctx.get_suite()
        runtime: ReportRuntime = ts.get_runtime("reports")

        if not runtime:
            raise ValueError(
                f"Unable to find reports in the current context. They must run before {STRATEGY_NAME}."
            )

        reports = []
        report_type = None
        for report_name in self.config.reports:
            agg_report = runtime.reports.get(report_name)
            if not agg_report:
                raise ValueError(
                    f"Unable to find report name '{report_name}' in the current context. It must run before {STRATEGY_NAME}."
                )
            if not report_type:
                report_type = agg_report.REPORT_TYPE
                this_report.report_class = agg_report.__class__
            else:
                if report_type != agg_report.REPORT_TYPE:
                    raise ValueError(
                        f"All report types must match in {STRATEGY_NAME}. Found {agg_report.REPORT_TYPE}, expected {report_type}"
                    )
            reports.append(agg_report)

        if not this_report.report_class:
            raise ValueError(
                f"At least one report type must be provided to {STRATEGY_NAME}. Unable to determine report class."
            )
        aggregated = this_report.report_class.aggregate(
            reports,
            ReportAggregation.COMPARISON,
            label_key=self.config.label_key or "report.name",
        )
        this_report.results = aggregated

        return this_report
