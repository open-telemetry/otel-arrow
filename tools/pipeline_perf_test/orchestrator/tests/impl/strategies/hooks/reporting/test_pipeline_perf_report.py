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
        hook = PipelinePerfReportHook(PipelinePerfReportConfig(name="replica-counters"))
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


# Scenario: Two increasing counters differ by pod, label, resource, or scope identity.
# Guarantees: Reports sum independent deltas and aligned rates, never cross-subtract.
@pytest.mark.parametrize("identity", ["pod", "label", "resource", "scope"])
def test_report_aggregates_independent_counter_series(run_report, identity):
    rows = []
    for index, values in enumerate(([100, 110, 130], [900, 930, 940])):
        attributes = {
            "metric_attributes": {
                "k8s.pod.uid": f"uid-{index}" if identity == "pod" else "same-pod",
                "receiver": f"receiver-{index}" if identity == "label" else "otlp",
            },
            "resource_attributes": {
                "service.instance.id": (
                    str(index) if identity == "resource" else "runner"
                ),
                "tags": ["performance", "collector"],
            },
            "scope_attributes": {
                "scope_name": f"scope-{index}" if identity == "scope" else "prometheus"
            },
        }
        for name in (CPU_COUNTER, SENT_COUNTER):
            rows.extend(metric_rows("otel-collector", name, values, **attributes))
    report = run_report(rows, reverse=True)

    for name in (CPU_COUNTER, SENT_COUNTER):
        assert component_values(report, f"delta({name})") == [70]
        for aggregation, expected in (("min", 3), ("mean", 3.5), ("max", 4)):
            assert component_values(report, f"{aggregation}(rate({name}))") == [
                expected
            ]
        rates = report.results["component_detail"].query_metrics(
            metric_name=f"rate({name})",
            metric_attrs={"component_name": "otel-collector"},
        )
        assert rates["timestamp"].tolist() == [
            START + pd.Timedelta(seconds=10),
            START + pd.Timedelta(seconds=20),
        ]
        assert rates["value"].tolist() == [4, 3]
    summary = report.results["summary"].set_index("metric_name")["value"]
    assert summary["Logs successfully sent by loadgen"] == 70
    assert summary["Logs received by backend"] == 70
    assert summary["Logs receive rate (avg)"] == 3.5
    assert summary["Total logs lost"] == 0


# Scenario: A legacy single-instance component has no pod or other series labels.
# Guarantees: Existing counter names, deltas, and per-interval rates stay unchanged.
def test_report_preserves_single_instance_counters(run_report):
    report = run_report(metric_rows("otel-collector", CPU_COUNTER, [100, 140, 170]))
    assert component_values(report, f"delta({CPU_COUNTER})") == [70]
    assert component_values(report, f"mean(rate({CPU_COUNTER}))") == [3.5]
    rates = report.results["component_detail"].query_metrics(
        metric_name=f"rate({CPU_COUNTER})",
        metric_attrs={"component_name": "otel-collector"},
    )
    assert rates["value"].tolist() == [4, 3]


# Scenario: One replica has only a single counter observation beside a healthy peer.
# Guarantees: Insufficient data fails explicitly instead of inventing a replica delta.
def test_report_rejects_counter_series_without_a_baseline(run_report):
    rows = [
        *metric_rows(
            "otel-collector",
            CPU_COUNTER,
            [100, 110, 130],
            metric_attributes={"k8s.pod.uid": "healthy"},
        ),
        *metric_rows(
            "otel-collector",
            CPU_COUNTER,
            [900],
            metric_attributes={"k8s.pod.uid": "missing-baseline"},
        ),
    ]
    with pytest.raises(ValueError, match="two distinct.*timestamps"):
        run_report(rows)


# Scenario: Replica intervals initially differ, then share a complete interval.
# Guarantees: Only complete aligned totals contribute to rates; no replica is omitted.
def test_report_uses_complete_aligned_replica_intervals(run_report):
    rows = metric_rows(
        "otel-collector",
        CPU_COUNTER,
        [100, 110, 130],
        metric_attributes={"component.target.name": "first"},
    )
    other = metric_rows(
        "otel-collector",
        CPU_COUNTER,
        [900, 940, 950],
        metric_attributes={"component.target.name": "second"},
    )
    for row, seconds in zip(other, [1, 10, 20]):
        row["timestamp"] = START + pd.Timedelta(seconds=seconds)
    report = run_report([*rows, *other])
    assert component_values(report, f"mean(rate({CPU_COUNTER}))") == [3]
    assert component_values(report, f"delta({CPU_COUNTER})") == [80]
    rates = report.results["component_detail"].query_metrics(
        metric_name=f"rate({CPU_COUNTER})",
        metric_attrs={"component_name": "otel-collector"},
    )
    assert rates["timestamp"].tolist() == [START + pd.Timedelta(seconds=20)]
    assert rates["value"].tolist() == [3]


# Scenario: Replicas share end timestamps but have no matching rate intervals.
# Guarantees: The report rejects fabricated totals instead of dropping a replica.
def test_report_rejects_replica_rates_without_any_complete_interval(run_report):
    rows = metric_rows(
        "otel-collector",
        CPU_COUNTER,
        [100, 110, 130],
        metric_attributes={"component.target.name": "first"},
    )
    other = metric_rows(
        "otel-collector",
        CPU_COUNTER,
        [900, 950],
        metric_attributes={"component.target.name": "second"},
    )
    other[-1]["timestamp"] = START + pd.Timedelta(seconds=20)
    with pytest.raises(ValueError, match="No complete aligned"):
        run_report([*rows, *other])
