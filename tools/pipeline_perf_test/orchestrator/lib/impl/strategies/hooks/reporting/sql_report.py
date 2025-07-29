
from typing import ClassVar, Optional, List, Dict, Any

import yaml
import pandas as pd
from pydantic import BaseModel, Field

from .....core.context.base import BaseContext
from .....core.context import FrameworkElementHookContext
from .....core.telemetry.metric import MetricDataFrame
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


STRATEGY_NAME = "sql_report"


@hook_registry.register_config(STRATEGY_NAME)
class SQLReportConfig(StandardReportingHookStrategyConfig):
    load_generator: str = "load-generator"
    system_under_test: str = "otel-collector"
    backend: str = "backend-service"




@hook_registry.register_class(STRATEGY_NAME)
class SQLReportHook(StandardReportingStrategy):

    PLUGIN_META = PluginMeta(
        supported_contexts=[FrameworkElementHookContext.__name__],
        installs_hooks=[],
        yaml_example="""
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
            sql:
                - select * from wahtever
            save_tables:
                foo_table:
                    path:
                    format:
          # OR report_config_file: ./report_configs/whatever.yaml
          output:
            - format:
                template: {}
              destination:
                console: {}
""",
        report_meta=ReportMeta(
            supported_aggregations=[
                ReportAggregation.COMPARISON.value,
                ReportAggregation.NONE.value,
            ],
            sample_output={},
        ),
    )

    def __init__(self, config: SQLReportConfig):
        super().__init__(config)
        self.config = config


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

        # Remove leading/trailing rows where value == 0
        non_zero = counter_rates["value"].ne(0) & counter_rates["value"].notna()
        if non_zero.any():
            first_idx = non_zero.idxmax()
            last_idx = non_zero[::-1].idxmax()
            counter_rates = counter_rates.loc[first_idx:last_idx]

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
