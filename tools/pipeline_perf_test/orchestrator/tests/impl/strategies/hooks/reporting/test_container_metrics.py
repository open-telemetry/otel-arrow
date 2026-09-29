"""Regression coverage for retained Docker gauges in scenario SQL reports."""

import datetime
from pathlib import Path
from unittest.mock import MagicMock, patch

import duckdb
import pandas as pd
import pytest
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import InMemoryMetricReader

from lib.core.telemetry.metric import FrameworkMetricBackend
from lib.impl.strategies.common.docker import (
    CONTAINER_MONITOR_START_EVENT,
    ComponentDockerRuntime,
)
from lib.impl.strategies.hooks.reporting.sql_report import (
    QueryConfig,
    ResultTable,
    SQLReport,
    SQLReportConfig,
    SQLReportDetails,
    SQLReportHook,
)
from lib.impl.strategies.monitoring.docker_component import (
    DockerComponentMonitoringConfig,
    DockerComponentMonitoringStrategy,
)

SUITES = Path(__file__).resolve().parents[6] / "test_suites"
REPORTS = sorted((SUITES / "integration/configs").glob("*report*.yaml")) + sorted(
    (SUITES / "comparison_dashboard/reports").glob("report_*.yaml")
)
RESOURCE_QUERIES = {
    "Calculate observation window for this test",
    "Calculate observation window",
    "Container Network Rates",
    "Container Network Per-Timestamp Rates",
    "Calculate component core counts",
    "Component Resource Per-Timestamp Values",
    "Create component metric aggregates",
    "Create component resource metrics",
    "Calculate component resources",
}


def timestamp(seconds):
    return pd.Timestamp("2026-09-29T00:00:00Z") + pd.Timedelta(seconds=seconds)


def metadata():
    return {
        "test.name": "current",
        "test.start": timestamp(15).isoformat(),
        "test.suite": "regression",
        "report.time": timestamp(30).isoformat(),
    }


def event(name, seconds, **attributes):
    return {
        "name": name,
        "timestamp": timestamp(seconds).value,
        "attributes": {"test.name": "current", **attributes},
    }


@pytest.fixture
def retained_metrics():
    """Collect real SDK gauges, then place collections on a deterministic timeline."""
    reader = InMemoryMetricReader()
    provider = MeterProvider(metric_readers=[reader])
    meter = provider.get_meter("regression")
    gauges = {
        name: meter.create_gauge(name)
        for name in (
            "container.cpu.usage",
            "container.memory.usage",
            "container.cpu.allocated",
            "container.network.rx",
            "container.network.tx",
        )
    }
    previous = {"component_name": "df-engine", "container_id": "old"}
    current = {"component_name": "df-engine", "container_id": "current"}
    frames = []

    def collect(seconds):
        backend = FrameworkMetricBackend()
        backend.add(reader.get_metrics_data())
        frame = backend.get_metrics_df()
        frame["timestamp"] = timestamp(seconds)
        frames.append(frame)

    try:
        for name, value in zip(gauges, (0.65, 128 * 1024**2, 8, 10**6, 2 * 10**6)):
            gauges[name].set(value, previous)
        gauges["container.cpu.usage"].set(
            0.9, {"component_name": "go-collector", "container_id": "retired"}
        )
        collect(5)

        # Process and Prometheus measurements do not carry Docker identities.
        meter.create_gauge("process.cpu.usage").set(0.25, {"component_name": "process"})
        meter.create_gauge("logs_produced").set(
            10000, {"component_name": "load-generator"}
        )
        for i, seconds in enumerate((19, 20, 21), start=1):
            for name, value in zip(gauges, (0.38, 32 * 1024**2, 1, i * 100, i * 200)):
                gauges[name].set(value, current)
            # Two container streams in one scenario must have separate LAGs.
            for name in ("container.network.rx", "container.network.tx"):
                gauges[name].set(
                    10**8 + i * (100 if name.endswith("rx") else 200),
                    {"component_name": "df-engine", "container_id": "second"},
                )
            collect(seconds)
        yield pd.concat(frames, ignore_index=True)
    finally:
        provider.shutdown()


@pytest.fixture
def scenario_events():
    return pd.DataFrame(
        [
            # Same test name from an earlier execution must not match.
            event(
                CONTAINER_MONITOR_START_EVENT,
                1,
                component_name="df-engine",
                container_id="old",
            ),
            # Same time but a different scenario must not match.
            event(
                CONTAINER_MONITOR_START_EVENT,
                16,
                component_name="go-collector",
                container_id="retired",
                **{"test.name": "other"},
            ),
            event(
                CONTAINER_MONITOR_START_EVENT,
                16,
                component_name="df-engine",
                container_id="current",
            ),
            event(
                CONTAINER_MONITOR_START_EVENT,
                16,
                component_name="df-engine",
                container_id="second",
            ),
            event("observation_start", 20),
            event("observation_stop", 21),
        ]
    )


# Scenario: SDK 1.45 reexports old containers during the next scenario's window.
# Guarantees: All shipped SQL reports isolate CPU, RAM, core counts, and network rates.
@pytest.mark.parametrize(
    "report_path", REPORTS, ids=lambda p: str(p.relative_to(SUITES))
)
def test_reports_exclude_retained_containers(
    report_path, retained_metrics, scenario_events
):
    hook = SQLReportHook(
        SQLReportConfig(name="regression", report_config_file=report_path)
    )
    assert hook.config.report_config.scope_container_metrics_to_scenario
    # Ensure this fixture really contains the SDK behavior that broke the dashboard.
    stale = retained_metrics[
        retained_metrics["metric_attributes"].map(
            lambda a: a.get("container_id") == "old"
        )
        & (retained_metrics["timestamp"] == timestamp(20))
    ]
    assert not stale.empty

    scoped = hook._scope_container_metrics(
        retained_metrics, scenario_events, metadata()
    )
    assert {a.get("container_id") for a in scoped["metric_attributes"]} == {
        "current",
        "second",
        None,
    }
    assert "logs_produced" in set(scoped["metric_name"])
    with duckdb.connect() as conn:
        hook.conn = conn
        hook._build_metadata_table(metadata())
        hook._register_in_memory_tables(
            scoped, pd.DataFrame({"name": []}), scenario_events
        )
        for query in hook.config.report_config.queries:
            if query.name in RESOURCE_QUERIES:
                conn.execute(query.sql)

        table = (
            "component_resources"
            if "clickhouse" in report_path.name
            else "component_resource_metrics"
        )
        values = dict(
            conn.execute(
                f"SELECT metric_name, avg_value FROM {table} WHERE component_name = 'df-engine'"
            ).fetchall()
        )
        assert values["container.cpu.usage"] == pytest.approx(0.38)
        assert values["container.memory.usage"] == 32 * 1024**2
        assert (
            conn.execute(
                f"SELECT COUNT(*) FROM {table} WHERE component_name = 'go-collector'"
            ).fetchone()[0]
            == 0
        )

        query_names = {q.name for q in hook.config.report_config.queries}
        if "Calculate component core counts" in query_names:
            assert (
                conn.execute(
                    "SELECT core_count FROM component_core_counts WHERE component_name = 'df-engine'"
                ).fetchone()[0]
                == 1
            )
        if "Container Network Rates" in query_names:
            rates = dict(
                conn.execute(
                    "SELECT metric_name, average_rate FROM container_network_rates"
                    " WHERE component_name = 'df-engine'"
                ).fetchall()
            )
            assert rates == {
                "container.network.rx": pytest.approx(100),
                "container.network.tx": pytest.approx(200),
            }


# Scenario: A Docker monitor starts before its container is later destroyed.
# Guarantees: The scenario records the same shortened ID used by its gauge labels.
def test_monitor_records_scenario_container_id(dummy_component):
    dummy_component.name = "df-engine"
    dummy_component.set_runtime_data(
        ComponentDockerRuntime.type,
        ComponentDockerRuntime(container_id="0123456789abcdef"),
    )
    ctx = MagicMock()
    strategy = DockerComponentMonitoringStrategy(DockerComponentMonitoringConfig())
    with (
        patch("lib.impl.strategies.monitoring.docker_component.threading.Thread"),
        patch(
            "lib.impl.strategies.monitoring.docker_component.get_or_create_docker_client"
        ),
    ):
        strategy.start(dummy_component, ctx)
    ctx.record_event.assert_called_once_with(
        CONTAINER_MONITOR_START_EVENT,
        component_name="df-engine",
        container_id="0123456789ab",
    )


# Scenario: A report runs through the public hook with scoping enabled or disabled.
# Guarantees: Opted-in reports filter before SQL; existing unscoped reports retain all rows.
@pytest.mark.parametrize("scoped", [True, False])
def test_execute_applies_container_scope(scoped, retained_metrics, scenario_events):
    hook = SQLReportHook(
        SQLReportConfig(
            name="regression",
            report_config=SQLReportDetails(
                scope_container_metrics_to_scenario=scoped,
                queries=[
                    QueryConfig(
                        name="cpu",
                        sql="""
                CREATE TABLE cpu AS SELECT AVG(value) AS value
                FROM metrics WHERE metric_name = 'container.cpu.usage'
                  AND "metric_attributes.component_name" = 'df-engine'
            """,
                    )
                ],
                result_tables=[ResultTable(name="cpu")],
            ),
        )
    )
    ctx = MagicMock()
    client = ctx.get_telemetry_client.return_value
    client.metrics.query_metrics.return_value = retained_metrics
    client.spans.query_spans.return_value = pd.DataFrame({"name": []})
    client.spans.query_span_events.return_value = scenario_events
    report = SQLReport(
        "regression", datetime.datetime.now(datetime.timezone.utc), metadata(), {}
    )
    try:
        with patch.object(SQLReport, "from_context", return_value=report):
            result = hook._execute(ctx)
        cpu = result.results["cpu"]["value"].iloc[0]
        if scoped:
            assert cpu == pytest.approx(0.38)
        else:
            assert cpu > 0.5
    finally:
        hook.conn.close()


# Scenario: A process-only scenario follows Docker scenarios in the same suite.
# Guarantees: No previous Docker samples survive; process data and the SQL schema remain valid.
def test_process_only_scenario_preserves_non_container_metrics(retained_metrics):
    hook = SQLReportHook(
        SQLReportConfig(name="process", report_config=SQLReportDetails())
    )
    events = pd.DataFrame(
        [event("observation_start", 20), event("observation_stop", 21)]
    )
    scoped = hook._scope_container_metrics(retained_metrics, events, metadata())
    assert set(scoped["metric_name"]) == {"process.cpu.usage", "logs_produced"}
    with duckdb.connect() as conn:
        hook.conn = conn
        hook._register_in_memory_tables(scoped, pd.DataFrame({"name": []}), events)
        rows = conn.execute(
            'SELECT "metric_attributes.component_name", value FROM metrics'
            " WHERE metric_name = 'process.cpu.usage'"
        ).fetchall()
        assert rows == [("process", 0.25)] * 3
        assert (
            conn.execute(
                'SELECT COUNT("metric_attributes.container_id") FROM metrics'
            ).fetchone()[0]
            == 0
        )


# Scenario: Scoping is requested without a scenario start time.
# Guarantees: Reporting rejects missing identity metadata instead of publishing mixed results.
def test_scoping_requires_scenario_metadata(retained_metrics, scenario_events):
    hook = SQLReportHook(
        SQLReportConfig(name="invalid", report_config=SQLReportDetails())
    )
    with pytest.raises(ValueError, match="test.name and test.start"):
        hook._scope_container_metrics(retained_metrics, scenario_events, {})
