"""
pipeline_perf_report.py

This module defines classes and logic for generating, aggregating, and formatting
performance reports related to pipeline processing. It integrates with a telemetry
framework to collect, analyze, and present metrics from various pipeline components
such as load generators, backends, and systems under test.

Key components:

- PipelinePerfReportIncludesConfig: Configuration flags to specify which report
  sections to include (summary, component summaries, detailed metrics).

- PipelinePerfReportConfig: Main configuration class for the pipeline performance
  reporting strategy. Includes component identifiers and section inclusion options.

- PipelinePerfReportHook: Implements the reporting strategy hook that orchestrates
  metric collection, aggregation, and report generation within a telemetry context.
  Supports calculation of key performance metrics such as logs sent, failed, received,
  lost in transit, and throughput rates.

- PipelinePerfReport: Report class that provides aggregation of multiple pipeline
  performance reports, formatting of results into markdown tables, and template
  generation for rendering in different aggregation modes (comparison, timeseries).

The module supports flexible aggregation and visualization of pipeline performance
data, enabling detailed insight into component-level and overall pipeline behavior
during testing or production monitoring.

Dependencies include pandas for data manipulation, pydantic for configuration
modeling, and integration with telemetry clients and metric querying utilities.
"""

from typing import ClassVar, Optional, List, Dict, Any

import yaml
import pandas as pd
from pydantic import BaseModel, Field

from .....core.context.base import BaseContext
from .....core.context import FrameworkElementHookContext
from .....core.helpers.report import group_by_populated_columns
from .....core.helpers.metrics import (
    compute_rate_over_time,
    pivot_aggregated_metrics,
    delta,
    concat_metrics_df,
    split_raw_metrics_by_group,
    format_metrics_by_ordered_rules,
    format_bytes,
    append_string,
)
from .....core.framework.report import Report, ReportAggregation
from .....core.telemetry.telemetry_client import TelemetryClient
from .....runner.registry import hook_registry, PluginMeta, ReportMeta
from .....runner.schema.reporting_hook_config import StandardReportingHookStrategyConfig
from ...common.events import get_start_end_event
from .standard_reporting_strategy import StandardReportingStrategy


STRATEGY_NAME = "pipeline_perf_report"


class PipelinePerfReportIncludesConfig(BaseModel):
    """
    Configuration for which sections to include in the pipeline performance report.

    Attributes:
        summary (Optional[bool]): Whether to include the overall summary section in the report.
            Defaults to True.
        component_summary (Optional[bool]): Whether to include per-component summary information.
            Defaults to True.
        component_detail (Optional[bool]): Whether to include detailed information for each component.
            Defaults to False.
    """

    summary: Optional[bool] = True
    component_summary: Optional[bool] = True
    component_detail: Optional[bool] = False


@hook_registry.register_config(STRATEGY_NAME)
class PipelinePerfReportConfig(StandardReportingHookStrategyConfig):
    """
    Configuration model for the pipeline performance reporting strategy.

    Defines the primary systems involved in the test as well as which sections
    to include in the generated report.

    Attributes:
        load_generator (str): Identifier for the load generator component. Defaults to "load-generator".
        system_under_test (str): Identifier for the system being evaluated. Defaults to "otel-collector".
        backend (str): Identifier for the backend service receiving data from the system under test.
            Defaults to "backend-service".
        include_sections (Optional[PipelinePerfReportIncludesConfig]): Sectional inclusion configuration
            for the report output. Defaults to including summary and component summary, excluding detail.
    """

    load_generator: str = "load-generator"
    system_under_test: str = "otel-collector"
    backend: str = "backend-service"
    include_sections: Optional[PipelinePerfReportIncludesConfig] = Field(
        default_factory=PipelinePerfReportIncludesConfig
    )


class PipelinePerfReport(Report):
    """
    Report class for aggregating and presenting pipeline performance test results.

    This class extends the base Report class to provide functionality for
    aggregating multiple pipeline performance reports, formatting results for
    templating, and producing different views depending on aggregation mode.

    Attributes:
        config (Optional[PipelinePerfReportConfig]): Configuration instance
            that controls report generation and section inclusion.
        REPORT_TYPE (ClassVar[str]): Identifier for this report type, linked to
            the associated reporting strategy.

    Methods:
        aggregate(reports, mode=ReportAggregation.COMPARISON, label_key="name"):
            Aggregate multiple PipelinePerfReport instances into combined
            dataframes. Supports comparison (wide) and timeseries (long) modes.

        to_aggregate_template_dict(results, config, mode=ReportAggregation.COMPARISON):
            Format aggregated dataframes into markdown tables suitable for
            templating and presentation, respecting the config's section flags.

        to_template_dict():
            Converts the report results and metadata into a dictionary with
            markdown-formatted strings for each included report section.

        get_template(mode):
            Returns a Jinja2 template string for rendering reports in the
            specified aggregation mode.
    """

    config: Optional[PipelinePerfReportConfig] = None
    REPORT_TYPE: ClassVar[str] = STRATEGY_NAME

    @classmethod
    def aggregate(
        cls,
        reports: List["Report"],
        mode: ReportAggregation = ReportAggregation.COMPARISON,
        *,
        label_key: str = "name",
    ) -> Dict[str, pd.DataFrame]:
        """
        Aggregate multiple pipeline performance reports into combined dataframes.

        Supports two aggregation modes:
            - COMPARISON: Wide-format combining metrics from multiple reports
              with columns representing each run.
            - TIMESERIES: Long-format suitable for timeseries visualization, splitting
              metrics by name with associated labels.

        Args:
            reports (List[Report]): List of pipeline performance reports to aggregate.
            mode (ReportAggregation, optional): Aggregation mode, either COMPARISON
                (default) or TIMESERIES.
            label_key (str, optional): Metadata key to use for labeling reports in output.

        Returns:
            Dict[str, pd.DataFrame]: Dictionary containing aggregated dataframes keyed by
                'summary', 'component_summary', and 'component_detail' for COMPARISON mode,
                or by metric name for TIMESERIES mode.
        """

        def get_perf_test_component(
            component_name: str, config: PipelinePerfReportConfig
        ) -> str:
            if config.load_generator == component_name:
                return "load_generator"
            if config.backend == component_name:
                return "backend"
            if config.system_under_test == component_name:
                return "system_under_test"

        summary_tables = [r.results["summary"] for r in reports]
        component_summary_tables = [r.results["component_summary"] for r in reports]
        component_detail_tables = [r.results["component_detail"] for r in reports]
        labels = [
            r.metadata.get(label_key, f"Run {i+1}") for i, r in enumerate(reports)
        ]
        configs = [r.config for r in reports]

        if mode == ReportAggregation.COMPARISON:
            # Wide-format: rows = metrics, columns = runs
            merged_summary = summary_tables[0].rename(columns={"value": labels[0]})
            for i, df in enumerate(summary_tables[1:], 1):
                df_renamed = df.rename(columns={"value": labels[i]})
                merged_summary = pd.merge(
                    merged_summary, df_renamed, on="metric_name", how="outer"
                )

            component_summary_merged = {}
            for i, df in enumerate(component_summary_tables):
                tables = split_raw_metrics_by_group(
                    df, "metric_attributes.component_name"
                )
                for component, component_df in tables.items():
                    perf_test_component = get_perf_test_component(component, configs[i])
                    df_clean = component_df.drop(columns=["timestamp"]).copy()
                    df_clean = (
                        df_clean.set_index("metric_name")
                        .rename(columns={"value": labels[i]})
                        .reset_index()
                    )
                    if component_summary_merged.get(perf_test_component) is None:
                        component_summary_merged[perf_test_component] = df_clean
                    else:
                        component_summary_merged[perf_test_component] = pd.merge(
                            component_summary_merged[perf_test_component],
                            df_clean,
                            on="metric_name",
                            how="outer",
                        )

            component_detail_merged = {}
            for i, df in enumerate(component_detail_tables):
                tables = split_raw_metrics_by_group(
                    df, "metric_attributes.component_name"
                )
                for component, component_df in tables.items():
                    perf_test_component = get_perf_test_component(component, configs[i])
                    df_clean = component_df.drop(columns=["timestamp"]).copy()
                    df_clean = df_clean.rename(columns={"value": labels[i]})
                    df_clean["sample_number"] = df_clean.groupby(
                        "metric_name"
                    ).cumcount()
                    if component_detail_merged.get(perf_test_component) is None:
                        component_detail_merged[perf_test_component] = df_clean
                    else:
                        component_detail_merged[perf_test_component] = pd.merge(
                            component_detail_merged[perf_test_component],
                            df_clean,
                            on=["metric_name", "sample_number"],
                            how="outer",
                        )
                        component_detail_merged[perf_test_component] = (
                            component_detail_merged[perf_test_component].reset_index(
                                drop=True
                            )
                        )
                        component_detail_merged[perf_test_component].index.name = (
                            "index"
                        )
                        component_detail_merged[perf_test_component] = (
                            component_detail_merged[perf_test_component].reset_index()
                        )
            for component, df_merged in component_detail_merged.items():
                df_merged["Time Offset"] = df_merged.groupby("metric_name").cumcount()
                cols = df_merged.columns.tolist()
                metric_idx = cols.index("metric_name")
                # Move 'Time Offset' to the correct place
                cols.insert(metric_idx + 1, cols.pop(cols.index("Time Offset")))
                df_merged = df_merged[cols]
                df_merged = df_merged.drop(
                    columns=["index", "sample_number"], errors="ignore"
                )
                component_detail_merged[component] = df_merged

            return {
                "summary": merged_summary,
                "component_summary": component_summary_merged,
                "component_detail": component_detail_merged,
            }

        elif mode == ReportAggregation.TIMESERIES:
            # Long-format, split by metric
            all_rows = []
            for df, label in zip(summary_tables, labels):
                temp = df.copy()
                temp["Label"] = label
                all_rows.append(temp)

            combined = pd.concat(all_rows, ignore_index=True)

            # Split by metric
            return {
                metric: group[["Label", "value"]].reset_index(drop=True)
                for metric, group in combined.groupby("metric_name")
            }

        else:
            raise ValueError(f"Unsupported mode: {mode}")

    @classmethod
    def to_aggregate_template_dict(
        cls,
        results: Dict[str, pd.DataFrame],
        config: PipelinePerfReportConfig,
        mode: ReportAggregation = ReportAggregation.COMPARISON,
    ) -> Dict[str, Any]:
        """
        Format aggregated report dataframes into markdown tables for templating.

        Applies formatting rules to metric columns and selectively includes report
        sections based on the provided config.

        Args:
            results (Dict[str, pd.DataFrame]): Aggregated report dataframes.
            config (PipelinePerfReportConfig): Configuration controlling which sections
                to include in the output.
            mode (ReportAggregation, optional): Aggregation mode, defaults to COMPARISON.

        Returns:
            Dict[str, Any]: Dictionary mapping section names to markdown-formatted tables
                or nested dictionaries of tables for component sections.
        """
        data = {}
        if mode == ReportAggregation.COMPARISON:
            format_rules = [
                (r".*bytes.*", format_bytes),
                (r"rate\(", append_string("/s")),
            ]
            if config.include_sections.summary:
                df = results.get("summary")
                df = df.copy()
                df = format_metrics_by_ordered_rules(
                    df, metric_col="metric_name", format_rules=format_rules
                )
                data["summary"] = df.to_markdown(index=False)

            if config.include_sections.component_summary:
                cs = results.get("component_summary")
                data["component_summary"] = {}
                for component, df in cs.items():
                    df = df.copy()
                    df = format_metrics_by_ordered_rules(
                        df, metric_col="metric_name", format_rules=format_rules
                    )
                    data["component_summary"][component] = df.to_markdown(index=False)

            if config.include_sections.component_detail:
                cd = results.get("component_detail")
                data["component_detail"] = {}
                for component, df in cd.items():
                    df = df.copy()
                    df = format_metrics_by_ordered_rules(
                        df,
                        metric_col="metric_name",
                        format_rules=format_rules,
                        exclude_columns=["Time Offset"],
                    )
                    data["component_detail"][component] = df.to_markdown(index=False)
        return data

    def to_template_dict(self) -> Dict[str, Any]:
        """
        Convert the report's results and metadata into a dictionary suitable for templating.

        Includes markdown-rendered tables for each configured section and serializes metadata
        to YAML format.

        Returns:
            Dict[str, Any]: Dictionary containing report metadata and markdown-formatted sections.
        """
        data = self.to_dict()
        # ordered_meta = reorder_metadata(self.metadata, self.PRIORITY_KEYS)
        data["metadata"] = yaml.dump(self.metadata, indent=2)

        if self.config.include_sections.summary:
            data["summary"] = self.results.get("summary").to_markdown(index=False)

        format_rules = [(r".*bytes.*", format_bytes), (r"rate\(", append_string("/s"))]

        if self.config.include_sections.component_summary:
            data["component_summary"] = {}
            component_summary = self.results.get("component_summary")
            component_summary_pivoted = pivot_aggregated_metrics(
                component_summary, group_key="metric_attributes.component_name"
            )
            for key in [
                self.config.backend,
                self.config.load_generator,
                self.config.system_under_test,
            ]:
                df = component_summary_pivoted.get(key)
                df = group_by_populated_columns(df, ["delta", "min", "max", "mean"])
                df = format_metrics_by_ordered_rules(
                    df, metric_col="metric_name", format_rules=format_rules
                )
                df = df.fillna("")
                table_md = df.to_markdown(index=False)
                data["component_summary"][key] = table_md

        if self.config.include_sections.component_detail:
            data["component_detail"] = {}
            component_detail = self.results.get("component_detail")
            component_detail_pivoted = split_raw_metrics_by_group(
                component_detail, group_key="metric_attributes.component_name"
            )
            for key in [
                self.config.backend,
                self.config.load_generator,
                self.config.system_under_test,
            ]:
                df = component_detail_pivoted.get(key)
                table_md = df.to_markdown(index=False)
                data["component_detail"][key] = table_md

        return data

    @classmethod
    def get_template(cls, mode: ReportAggregation) -> str:
        default = """# Pipeline Perf Report
{% if report.metadata %}
## Metadata:

{{ report.metadata }}
{% endif %}

{% if report.summary %}
## Summary:

{{report.summary}}
{% endif %}
{% if report.component_summary %}
{% for process, table in report.component_summary.items() %}
## Element: {{ process }}

{{table}}
{% endfor %}
{% endif %}
{% if report.component_detail %}
{% for process, table in report.component_detail.items() %}
## Element Detail: {{ process }}

{{table}}
{% endfor %}
{% endif %}
"""
        templates = {
            ReportAggregation.NONE: default,
            ReportAggregation.COMPARISON: default.replace(
                "Pipeline Perf Report", "Pipeline Perf Comparison Report"
            ),
            ReportAggregation.TIMESERIES: """# Time Series Report

## Metadata:

{{ report.metadata }}

## Timeseries by Metric:

{% for metric, df in report.timeseries_by_metric.items() %}
### Metric: {{ metric }}

{{ df }}
{% endfor %}
""",
        }
        return templates.get(mode, "Unsupported aggregation mode")


@hook_registry.register_class(STRATEGY_NAME)
class PipelinePerfReportHook(StandardReportingStrategy):
    """
    Reporting strategy hook for generating a pipeline performance report.

    This hook collects telemetry metrics from the load generator, system under test,
    and backend components within a specified time window, computes aggregate metrics,
    and compiles a summary and detailed report of the pipeline's performance.

    Attributes:
        config (PipelinePerfReportConfig): Configuration instance defining components
            involved and report sections to include.
        report_start (Optional[pd.Timestamp]): Timestamp marking the start of the report period.
        report_end (Optional[pd.Timestamp]): Timestamp marking the end of the report period.
        duration (Optional[float]): Duration of the report period in seconds.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[FrameworkElementHookContext.__name__],
        installs_hooks=[],
        yaml_example="""
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - OTLP
            - format:
                template: {}
              destination:
                console: {}
          between_events:
            start:
              name: test_framework.test_start
              attributes:
                test.name: Test OTLP - Max Logs / Sec
            end:
              name: test_framework.test_end
              attributes:
                test.name: Test OTLP - Max Logs / Sec
""",
        report_meta=ReportMeta(
            supported_aggregations=[
                ReportAggregation.COMPARISON.value,
                ReportAggregation.NONE.value,
            ],
            sample_output={
                "Without Aggregation": """
# Pipeline Perf Report

## Metadata:
...

## Summary:

| metric_name                       |           value |
|:----------------------------------|----------------:|
| Total logs attempted              |      2.2475e+07 |
| Logs successfully sent by loadgen |      2.2475e+07 |
| Logs failed at loadgen            |      0          |
| Logs received by backend          |      2.2475e+07 |
| Logs lost in transit              |      0          |
| Duration                          |     45.8919     |
| Logs receive rate (avg)           | 855638          |
| Total logs lost                   |      0          |
| Percentage of logs lost           |      0          |
""",
                "Comparison Aggregation": """
# Pipeline Perf Comparison Report

## Metadata:
...

## Summary:

| metric_name                       |   PerfReprort - OTLP |   PerfReprort - OTAP |
|:----------------------------------|---------------------:|---------------------:|
| Duration                          |          45.8919     |          55.851      |
| Logs receive rate (avg)           |      855638          |      943012          |
| Logs failed at loadgen            |           0          |           0          |
| Logs lost in transit              |           0          |           0          |
| Logs received by backend          |           2.2475e+07 |           2.5535e+07 |
| Logs successfully sent by loadgen |           2.2475e+07 |           2.5535e+07 |
| Percentage of logs lost           |           0          |           0          |
| Total logs attempted              |           2.2475e+07 |           2.5535e+07 |
| Total logs lost                   |           0          |           0          |
""",
            },
        ),
    )

    def __init__(self, config: PipelinePerfReportConfig):
        super().__init__(config)
        self.config = config
        self.report_start = None
        self.report_end = None
        self.duration = None

    def _get_summary_table(self, aggregated_metrics):
        def safe_lookup(df, metric_name):
            result = df.loc[df["metric_name"] == metric_name, "value"]
            return result.iloc[0] if not result.empty else float("nan")

        metric_map = {
            "Total logs attempted": safe_lookup(aggregated_metrics, "sum(total_sent)"),
            "Logs successfully sent by loadgen": safe_lookup(
                aggregated_metrics, "delta(sent)"
            ),
            "Logs failed at loadgen": safe_lookup(aggregated_metrics, "delta(failed)"),
            "Logs received by backend": safe_lookup(
                aggregated_metrics, "delta(received_logs)"
            ),
            "Logs lost in transit": aggregated_metrics.loc[
                aggregated_metrics["metric_name"] == "delta(sent)", "value"
            ].iloc[0]
            - aggregated_metrics.loc[
                aggregated_metrics["metric_name"] == "delta(received_logs)", "value"
            ].iloc[0],
            "Duration": self.duration,
            "Logs receive rate (avg)": safe_lookup(
                aggregated_metrics, "mean(rate(received_logs))"
            ),
            "Total logs lost": aggregated_metrics.loc[
                aggregated_metrics["metric_name"] == "sum(total_sent)", "value"
            ].iloc[0]
            - aggregated_metrics.loc[
                aggregated_metrics["metric_name"] == "delta(received_logs)", "value"
            ].iloc[0],
            "Percentage of logs lost": (
                aggregated_metrics.loc[
                    aggregated_metrics["metric_name"] == "sum(total_sent)", "value"
                ].iloc[0]
                - aggregated_metrics.loc[
                    aggregated_metrics["metric_name"] == "delta(received_logs)", "value"
                ].iloc[0]
            )
            / aggregated_metrics.loc[
                aggregated_metrics["metric_name"] == "sum(total_sent)", "value"
            ].iloc[0],
        }
        summary = [
            {"metric_name": label, "value": value}
            for label, value in metric_map.items()
        ]
        summary = pd.DataFrame(summary)

        return summary

    def _execute(self, ctx: BaseContext) -> Report:
        tc: TelemetryClient = ctx.get_telemetry_client()
        report: PipelinePerfReport = PipelinePerfReport.from_context(
            self.config.name, ctx
        )
        report.config = self.config

        if self.config.between_events:
            (report_start_event, report_end_event) = get_start_end_event(
                self.config.between_events, tc
            )
            self.report_start = report_start_event.iloc[0]["timestamp"]
            report.metadata["report.observation.start"] = (
                pd.to_datetime(self.report_start).isoformat()
                if self.report_start
                else None
            )
            self.report_end = report_end_event.iloc[0]["timestamp"]
            report.metadata["report.observation.end"] = (
                pd.to_datetime(self.report_end).isoformat() if self.report_end else None
            )

            if self.report_start and self.report_end:
                self.duration = (
                    pd.to_datetime(self.report_end) - pd.to_datetime(self.report_start)
                ).total_seconds()
                report.metadata["report.observation.duration_seconds"] = self.duration

        results = {}
        otel_collector_metrics_type = {
            "counter": [
                "otelcol_exporter_send_failed_log_records_total",
                "otelcol_exporter_sent_log_records_total",
                "otelcol_receiver_accepted_log_records_total",
                "otelcol_process_cpu_seconds_total",
                "otelcol_exporter_send_failed_log_records",
                "otelcol_exporter_sent_log_records",
                "otelcol_receiver_accepted_log_records",
                "otelcol_process_cpu_seconds",
            ],
            "gauge": ["otelcol_process_memory_rss_bytes"],
        }
        loadgen_metrics_type = {"counter": ["sent", "failed", "bytes_sent"]}
        backend_metrics_type = {"counter": ["received_logs"]}

        otel_counter_metrics = tc.metrics.query_metrics(
            metric_name=otel_collector_metrics_type.get("counter"),
            metric_attrs={"component_name": self.config.system_under_test},
            time_range=(self.report_start, self.report_end),
        )
        backend_counter_metrics = tc.metrics.query_metrics(
            metric_name=backend_metrics_type.get("counter"),
            metric_attrs={"component_name": self.config.backend},
            time_range=(self.report_start, self.report_end),
        )
        loadgen_counter_metrics = tc.metrics.query_metrics(
            metric_name=loadgen_metrics_type.get("counter"),
            metric_attrs={"component_name": self.config.load_generator},
            time_range=(self.report_start, self.report_end),
        )
        otel_gauge_metrics = tc.metrics.query_metrics(
            metric_name=otel_collector_metrics_type.get("gauge"),
            metric_attrs={"component_name": self.config.system_under_test},
            time_range=(self.report_start, self.report_end),
        )

        counter_metrics = concat_metrics_df(
            [otel_counter_metrics, backend_counter_metrics, loadgen_counter_metrics],
            ignore_index=True,
        )
        counter_rates = compute_rate_over_time(
            counter_metrics, by=["metric_attributes.component_name", "metric_name"]
        )

        gauge_metrics = concat_metrics_df(
            [counter_rates, otel_gauge_metrics], ignore_index=True
        )

        gauge_aggregates = gauge_metrics.with_aggregation(
            by=["metric_attributes.component_name", "metric_name"],
            agg_func=["min", "mean", "max"],
        )
        counter_deltas = counter_metrics.with_aggregation(
            by=["metric_attributes.component_name", "metric_name"], agg_func=delta
        )

        all_aggregates = concat_metrics_df(
            [gauge_aggregates, counter_deltas], ignore_index=True
        )
        component_aggregates = concat_metrics_df(
            [gauge_aggregates, counter_deltas], ignore_index=True
        )
        component_details = gauge_metrics

        sent_failed = (
            all_aggregates.query_metrics(
                metric_name=["delta(sent)", "delta(failed)"],
                metric_attrs={"component_name": self.config.load_generator},
            )
            .with_aggregation(
                by=["metric_attributes.component_name"],
                agg_func=["sum"],
                collapsed_metric_name="total_sent",
            )
            .with_attributes(
                attributes={"metric_attributes": {"component_name": STRATEGY_NAME}}
            )
        )
        all_aggregates = concat_metrics_df(
            [sent_failed, all_aggregates], ignore_index=True
        )

        transit_lost = (
            all_aggregates.query_metrics(
                metric_name=["sum(total_sent)", "delta(received_logs)"],
            )
            .with_aggregation(
                agg_func=delta,
                collapsed_metric_name="transit_lost",
            )
            .with_attributes(
                attributes={"metric_attributes": {"component_name": STRATEGY_NAME}}
            )
        )

        all_aggregates = concat_metrics_df(
            [transit_lost, all_aggregates], ignore_index=True
        )

        results["summary"] = self._get_summary_table(aggregated_metrics=all_aggregates)
        results["component_summary"] = component_aggregates
        results["component_detail"] = component_details

        report.set_results(results)
        return report
