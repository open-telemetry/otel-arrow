from unittest.mock import MagicMock

import pandas as pd
import pytest

from lib.impl import actions, strategies  # noqa: F401
from lib.core.telemetry.metric import MetricDataFrame
from lib.impl.strategies.hooks.reporting.pipeline_perf_report import (
    PipelinePerfReportConfig,
    PipelinePerfReportHook,
)

CPU_COUNTER = "otelcol_process_cpu_seconds_total"
SENT_COUNTER = "otelcol_exporter_sent_log_records_total"
START = pd.Timestamp("2026-01-01T00:00:00Z")


def metric_rows(
    component,
    name,
    values,
    metric_attributes=None,
    resource_attributes=None,
    scope_attributes=None,
):
    rows = []
    for index, value in enumerate(values):
        attributes = {"component_name": component, **(metric_attributes or {})}
        # Dictionary insertion order is not part of a series' identity.
        if index % 2:
            attributes = dict(reversed(list(attributes.items())))
        rows.append(
            {
                "timestamp": START + pd.Timedelta(seconds=10 * index),
                "metric_name": name,
                "metric_type": "Gauge",
                "value": value,
                "metric_attributes": attributes,
                "resource_attributes": resource_attributes or {},
                "scope_attributes": scope_attributes or {},
            }
        )
    return rows


@pytest.fixture
def run_report():
    def run(rows, reverse=False):
        rows = [
            *metric_rows("load-generator", "sent", [1000, 1040, 1070]),
            *metric_rows("load-generator", "failed", [17, 17, 17]),
            *metric_rows("backend-service", "received_logs", [900, 940, 970]),
            *rows,
        ]
        metrics = MetricDataFrame(rows[::-1] if reverse else rows)
        ctx = MagicMock()
        ctx.get_metadata.return_value = {"test.suite": "counter-series"}
        ctx.get_suite().context.start_time = START
        ctx.get_telemetry_client.return_value.metrics.query_metrics.side_effect = (
            metrics.query_metrics
        )
        hook = PipelinePerfReportHook(PipelinePerfReportConfig(name="counter-report"))
        return hook._execute(ctx)

    return run


def component_values(report, metric_name):
    return (
        report.results["component_summary"]
        .query_metrics(
            metric_name=metric_name, metric_attrs={"component_name": "otel-collector"}
        )["value"]
        .tolist()
    )


# Scenario: One component/metric has independent target, pod, label, resource or scope series.
# Guarantees: The legacy report directs callers to SQL instead of mixing counter series.
@pytest.mark.parametrize("identity", ["target", "pod", "label", "resource", "scope"])
def test_report_rejects_multiple_counter_series(run_report, identity):
    rows = []
    for index, values in enumerate(([100, 110, 130], [900, 930, 940])):
        rows.extend(
            metric_rows(
                "otel-collector",
                CPU_COUNTER,
                values,
                metric_attributes={
                    "component.target.name": (
                        str(index) if identity == "target" else "one"
                    ),
                    "k8s.pod.uid": str(index) if identity == "pod" else "same-pod",
                    "receiver": str(index) if identity == "label" else "otlp",
                },
                resource_attributes={
                    "service.instance.id": (
                        str(index) if identity == "resource" else "one"
                    )
                },
                scope_attributes={
                    "scope_name": str(index) if identity == "scope" else "prometheus"
                },
            )
        )
    with pytest.raises(ValueError, match="one counter series.*use sql_report"):
        run_report(rows, reverse=True)


# Scenario: Each component/metric has a single independent counter series.
# Guarantees: Existing names, deltas, per-interval rates and summary values remain intact.
def test_report_preserves_single_instance_counters(run_report):
    rows = [
        *metric_rows("otel-collector", CPU_COUNTER, [100, 140, 170]),
        *metric_rows("otel-collector", SENT_COUNTER, [100, 140, 170]),
    ]
    report = run_report(rows, reverse=True)
    for name in (CPU_COUNTER, SENT_COUNTER):
        assert component_values(report, f"delta({name})") == [70]
        assert component_values(report, f"mean(rate({name}))") == [3.5]
        rates = report.results["component_detail"].query_metrics(
            metric_name=f"rate({name})"
        )
        assert rates["value"].tolist() == [4, 3]
    summary = report.results["summary"].set_index("metric_name")["value"]
    assert summary["Logs successfully sent by loadgen"] == 70
    assert summary["Logs received by backend"] == 70
    assert summary["Logs receive rate (avg)"] == 3.5
    assert summary["Total logs lost"] == 0


# Scenario: A counter has an invalid middle observation.
# Guarantees: Rates and deltas use numeric observations with valid timestamps.
@pytest.mark.parametrize("invalid", [float("nan"), None, "invalid"])
def test_report_skips_invalid_counter_values(run_report, invalid):
    report = run_report(
        metric_rows("otel-collector", CPU_COUNTER, [0, invalid, 20, 40])
    )
    assert component_values(report, f"delta({CPU_COUNTER})") == [40]
    assert component_values(report, f"mean(rate({CPU_COUNTER}))") == [1.5]
    rates = report.results["component_detail"].query_metrics(
        metric_name=f"rate({CPU_COUNTER})"
    )
    assert rates["value"].tolist() == [1, 2]


# Scenario: A counter has fewer than two valid observations.
# Guarantees: Missing or invalid baselines fail explicitly rather than fabricate rates.
@pytest.mark.parametrize("values", [[900], [0, None, "invalid"], [None, float("nan")]])
def test_report_rejects_counter_series_without_valid_baselines(run_report, values):
    with pytest.raises(ValueError, match="two distinct.*timestamps"):
        run_report(metric_rows("otel-collector", CPU_COUNTER, values))


# Scenario: A legacy single series has uneven observation spacing.
# Guarantees: Its existing arithmetic mean remains unchanged.
def test_report_preserves_single_series_mean_with_uneven_intervals(run_report):
    rows = metric_rows("otel-collector", CPU_COUNTER, [0, 10, 50])
    rows[-1]["timestamp"] = START + pd.Timedelta(seconds=30)
    report = run_report(rows)
    assert component_values(report, f"mean(rate({CPU_COUNTER}))") == [1.5]
