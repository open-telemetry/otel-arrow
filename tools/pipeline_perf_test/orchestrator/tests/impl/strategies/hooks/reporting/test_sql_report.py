import pytest
import pandas as pd
from pathlib import Path
from pydantic import ValidationError
import tempfile
import yaml
import duckdb
from unittest.mock import MagicMock, patch

from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader

from lib.impl import actions, strategies  # noqa: F401
from lib.core.framework.element import FrameworkLifecyclePhase
from lib.core.strategies.monitoring_strategy import MonitoringStrategy
from lib.core.telemetry.metric import FrameworkMetricBackend, FrameworkMetricExporter
from lib.impl.component.managed_component import ManagedComponentConfiguration
from lib.runner.factory import build_managed_component
from lib.runner.schema.framework_element_config import ScenarioConfig
from lib.impl.strategies.hooks.reporting.sql_report import (
    SQLReportConfig,
    SQLReportDetails,
    QueryConfig,
    ResultTable,
    LoadTableConfig,
    SQLReportHook,
    WriteTableConfig,
    TableIOConfig,
)


def test_valid_config_with_inline_report_config():
    config = SQLReportConfig(
        name="inline_test",
        report_config=SQLReportDetails(
            queries=[QueryConfig(name="q1", sql="SELECT 1")],
            result_tables=[ResultTable(name="result")],
        ),
    )
    assert config.report_config is not None
    assert config.report_config_file is None


# Scenario: Inline or file-based reports inherit parameterized SQL and override results.
# Guarantees: Both entry points consume the same template and preserve caller overrides.
@pytest.mark.parametrize("file_based", [False, True])
def test_report_templates_expand_before_validation_and_keep_overrides(
    tmp_path, file_based
):
    template = tmp_path / "report-template.yaml"
    template.write_text(
        "queries:\n"
        "  - name: Parameterized result\n"
        "    sql: CREATE TABLE answer AS SELECT {{ value }} AS value\n"
        "result_tables:\n"
        "  - name: replaced_by_caller\n"
    )
    details = {
        "from_template": {"path": str(template), "variables": {"value": 42}},
        "result_tables": [{"name": "answer", "finite_columns": ["value"]}],
    }
    if file_based:
        path = tmp_path / "report.yaml"
        path.write_text(yaml.safe_dump(details))
        config = SQLReportConfig(name="templated", report_config_file=path)
    else:
        config = SQLReportConfig(name="templated", report_config=details)
    hook = SQLReportHook(config)
    with duckdb.connect() as connection:
        hook.conn = connection
        for query in hook.config.report_config.queries:
            connection.execute(query.sql)
        assert hook._build_result_dataframes()["answer"]["value"].tolist() == [42]


# Scenario: A templated report includes an unsupported outer configuration field.
# Guarantees: Template expansion does not weaken the existing strict report schema.
def test_report_template_rejects_unknown_overrides(tmp_path):
    template = tmp_path / "report.yaml"
    template.write_text("queries: []\n")
    with pytest.raises(ValidationError, match="extra_forbidden"):
        SQLReportDetails.model_validate(
            {"from_template": {"path": str(template)}, "unsupported": True}
        )


# Scenario: The shared observation flow reports queue depth, not CPU or memory.
# Guarantees: Callers choose arbitrary metrics and SQL without resource assumptions.
@pytest.mark.parametrize("adopt", [False, True])
def test_observation_template_accepts_arbitrary_metric_report(adopt):
    template = Path(__file__).resolve().parents[5] / "templates/observe-existing.yaml"
    scenario = ScenarioConfig.model_validate(
        {
            "from_template": {
                "path": str(template),
                "variables": {
                    "adopt": adopt,
                    "report_config": {
                        "queries": [
                            {
                                "name": "Queue depth",
                                "sql": (
                                    "CREATE TABLE queue_summary AS SELECT "
                                    "AVG(value) AS mean_depth FROM metrics "
                                    "WHERE metric_name = 'queue_depth'"
                                ),
                            }
                        ],
                        "result_tables": [
                            {
                                "name": "queue_summary",
                                "finite_columns": ["mean_depth"],
                            }
                        ],
                    },
                },
            }
        }
    )
    phases = [
        step.action.config.phase.value
        for step in scenario.steps
        if step.action.element_type == "multi_component_action"
    ]
    assert ("adopt" in phases) == adopt
    hook = scenario.steps[-1].hooks[FrameworkLifecyclePhase.RUN].post[0].build_element()
    with duckdb.connect() as connection:
        hook.conn = connection
        connection.register(
            "metrics",
            pd.DataFrame(
                {
                    "metric_name": ["queue_depth", "queue_depth", "other"],
                    "value": [7, 13, 99],
                }
            ),
        )
        for query in hook.config.report_config.queries:
            connection.execute(query.sql)
        assert hook._build_result_dataframes()["queue_summary"][
            "mean_depth"
        ].tolist() == [10]


# Scenario: A monitor records its final gauge value while periodic export is pending.
# Guarantees: Shutdown exports that value; SQL reporting reads it without flushing.
def test_monitoring_shutdown_exports_final_metrics_before_reporting():
    backend = FrameworkMetricBackend()
    provider = MeterProvider(
        metric_readers=[
            PeriodicExportingMetricReader(
                FrameworkMetricExporter(backend), export_interval_millis=600000
            )
        ]
    )
    hook = SQLReportHook(
        SQLReportConfig(
            name="pending metrics",
            report_config={
                "queries": [
                    {
                        "name": "Latest metric",
                        "sql": (
                            "CREATE TABLE snapshot AS SELECT SUM(value) AS total "
                            "FROM metrics WHERE metric_name = 'queue_depth'"
                        ),
                    }
                ],
                "result_tables": [{"name": "snapshot", "finite_columns": ["total"]}],
            },
        )
    )
    ctx = MagicMock()
    ctx.get_metadata.return_value = {}
    ctx.get_suite().context.start_time = None
    ctx.get_suite().get_runtime.return_value.meter_provider = provider
    tc = ctx.get_telemetry_client.return_value
    tc.metrics.query_metrics.side_effect = backend.get_metrics_df
    tc.spans.query_spans.return_value = pd.DataFrame({"name": []})
    tc.spans.query_span_events.return_value = pd.DataFrame({"name": []})
    component = build_managed_component("service", ManagedComponentConfiguration())
    monitor = MagicMock(spec=MonitoringStrategy)
    component.monitoring = monitor
    gauge = provider.get_meter("pending").create_gauge("queue_depth")
    monitor.stop.side_effect = lambda *_args: gauge.set(13)
    try:
        with (
            patch.object(backend, "add", wraps=backend.add) as export,
            patch.object(provider, "force_flush", wraps=provider.force_flush) as flush,
        ):
            gauge.set(7)
            export.assert_not_called()
            component.stop_monitoring(ctx)
            export.assert_called_once()
            flush.assert_called_once_with()
            report = hook._execute(ctx)
            assert report.results["snapshot"]["total"].tolist() == [13]
            flush.assert_called_once_with()
    finally:
        if hook.conn is not None:
            hook.conn.close()
        provider.shutdown()


def test_valid_config_with_report_config_file(tmp_path):
    config_data = {
        "queries": [{"name": "q1", "sql": "SELECT 1"}],
        "result_tables": [{"name": "result"}],
    }
    config_path = tmp_path / "report.yaml"
    config_path.write_text(yaml.dump(config_data))

    config = SQLReportConfig(
        name="file_test",
        report_config_file=config_path,
    )

    # Simulate internal method that loads file contents
    config._load_report_config_from_file = lambda: setattr(
        config, "report_config", SQLReportDetails(**config_data)
    )
    config._load_report_config_from_file()

    assert config.report_config_file == config_path
    assert config.report_config is not None
    assert config.report_config.queries[0].name == "q1"


def test_invalid_config_raises_error_when_missing_both():
    with pytest.raises(ValidationError) as exc_info:
        SQLReportConfig(name="invalid_test")

    assert "Either 'report_config' or 'report_config_file'" in str(exc_info.value)


def test_table_io_config_requires_path_or_template():
    with pytest.raises(ValidationError) as exc_info:
        LoadTableConfig(format="csv")

    assert "Either 'path' or 'path_template' must be specified" in str(exc_info.value)


def test_both_inline_and_file_report_config(tmp_path):
    config_data = {
        "queries": [{"name": "q1", "sql": "SELECT 1"}],
        "result_tables": [{"name": "result"}],
    }
    config_path = tmp_path / "config.yaml"
    config_path.write_text(yaml.dump(config_data))

    inline_config = SQLReportDetails(
        queries=[QueryConfig(name="inline", sql="SELECT 42")],
        result_tables=[ResultTable(name="inline_result")],
    )

    config = SQLReportConfig(
        name="dual_config", report_config=inline_config, report_config_file=config_path
    )

    hook = SQLReportHook(config)
    # File-based config should override inline
    assert config.report_config.queries[0].name == "q1"


def test_report_config_file_not_found():
    config = SQLReportConfig(
        name="bad_path", report_config_file=Path("non_existent.yaml")
    )
    with pytest.raises(FileNotFoundError):
        hook = SQLReportHook(config)


def test_report_config_file_structurally_invalid(tmp_path):
    bad_yaml_path = tmp_path / "bad.yaml"
    bad_yaml_path.write_text(":::: not valid yaml")

    config = SQLReportConfig(name="bad_yaml", report_config_file=bad_yaml_path)

    with pytest.raises(ValidationError):
        SQLReportHook(config)._load_report_config_from_file()


def test_missing_result_tables_should_error():
    config = SQLReportConfig(
        name="missing_result_tables",
        report_config=SQLReportDetails(
            queries=[QueryConfig(name="q1", sql="SELECT 1")],
            result_tables=[ResultTable(name="doesnt_exist")],
        ),
    )
    hook = SQLReportHook(config)
    with pytest.raises(
        AttributeError
    ):  # because .result_tables is None and expected as list
        hook._build_result_dataframes()


def test_invalid_path_template_renders_to_empty_string():
    with pytest.raises(ValueError) as exc_info:
        TableIOConfig(path_template="", format="csv")
    assert "Either 'path' or 'path_template'" in str(exc_info.value)


def test_unsupported_table_format_raises():
    with pytest.raises(ValueError) as exc_info:
        TableIOConfig(path="some_path.csv", format="unknown")
    assert "Input should be 'parquet', 'json' or 'csv'" in str(exc_info.value)


def test_load_table_with_default_ddl_on_no_match(monkeypatch):
    dummy_config = LoadTableConfig(
        path_template="no/match/*.csv",
        format="csv",
        default_ddl="CREATE TABLE dummy (id INTEGER);",
    )
    hook = SQLReportHook(
        SQLReportConfig(
            name="ddl_fallback",
            report_config=SQLReportDetails(load_tables={"dummy": dummy_config}),
        )
    )
    hook.conn = hook.conn or __import__("duckdb").connect()

    class DummyLogger:
        def debug(self, *args, **kwargs):
            pass

    report = type("FakeReport", (), {"metadata": {}})
    hook._load_external_tables(DummyLogger(), report)  # Should not raise


def test_load_table_missing_and_no_ddl_raises(tmp_path):
    dummy_config = LoadTableConfig(
        path_template=str(tmp_path / "nothing.csv"), format="csv"
    )
    config = SQLReportConfig(
        name="missing_file",
        report_config=SQLReportDetails(load_tables={"missing": dummy_config}),
    )
    hook = SQLReportHook(config)
    hook.conn = __import__("duckdb").connect()

    class DummyLogger:
        def debug(self, *args, **kwargs):
            pass

    report = type("FakeReport", (), {"metadata": {}})
    with pytest.raises(FileNotFoundError):
        hook._load_external_tables(DummyLogger(), report)


def test_query_fails_due_to_invalid_sql():
    config = SQLReportConfig(
        name="bad_sql",
        report_config=SQLReportDetails(
            queries=[QueryConfig(name="fail", sql="SELECT * FROM non_existent")],
            result_tables=[ResultTable(name="non_existent")],
        ),
    )
    hook = SQLReportHook(config)
    hook.conn = __import__("duckdb").connect()
    with pytest.raises(Exception):
        hook._run_sql_queries(lambda *_: None)


def test_write_table_config_rejects_invalid_format():
    with pytest.raises(ValidationError) as exc_info:
        WriteTableConfig(path="foo.out", format="invalid")

    assert "Input should be 'parquet', 'json' or 'csv'" in str(exc_info.value)


# Scenario: A required benchmark value is NaN.
# Guarantees: SQL reporting fails instead of publishing an invalid benchmark.
def test_result_table_rejects_non_finite_values():
    table = ResultTable(name="gh_actions_benchmark", finite_columns=["value"])

    with pytest.raises(ValueError, match="contains non-finite values"):
        SQLReportHook._assert_finite_columns(
            table, pd.DataFrame({"name": ["loss"], "value": [float("nan")]})
        )


# Scenario: Every required benchmark value is finite.
# Guarantees: SQL reporting accepts valid benchmark output without value thresholds.
def test_result_table_accepts_finite_values():
    table = ResultTable(name="gh_actions_benchmark", finite_columns=["value"])

    SQLReportHook._assert_finite_columns(
        table, pd.DataFrame({"name": ["loss", "throughput"], "value": [0.0, 1.0]})
    )


# Scenario: A benchmark table omits a configured KPI while all emitted values are finite.
# Guarantees: SQL reporting fails rather than publishing an incomplete benchmark set.
def test_result_table_rejects_missing_required_values():
    table = ResultTable(
        name="gh_actions_benchmark",
        required_values={"name": ["loss", "produced_rate"]},
    )

    with pytest.raises(ValueError, match=r"required values \['produced_rate'\]"):
        SQLReportHook._assert_required_values(
            table, pd.DataFrame({"name": ["loss"], "value": [0.0]})
        )


# Scenario: A benchmark table contains every configured KPI.
# Guarantees: Required-value validation accepts complete benchmark output.
def test_result_table_accepts_required_values():
    table = ResultTable(
        name="gh_actions_benchmark",
        required_values={"name": ["loss", "produced_rate"]},
    )

    SQLReportHook._assert_required_values(
        table,
        pd.DataFrame({"name": ["loss", "produced_rate"], "value": [0.0, 1.0]}),
    )


# Scenario: A configured SQL result table contains an invalid required KPI
# value.
# Guarantees: Building report results invokes configured validity checks.
def test_build_result_dataframes_validates_configured_table():
    table = ResultTable(
        name="gh_actions_benchmark",
        finite_columns=["value"],
        required_values={"name": ["loss", "produced_rate"]},
    )
    hook = SQLReportHook(
        SQLReportConfig(
            name="validated_report",
            report_config=SQLReportDetails(result_tables=[table]),
        )
    )
    hook.conn = __import__("duckdb").connect()
    hook.conn.execute("""
        CREATE TABLE gh_actions_benchmark AS
        SELECT 'loss' AS name, CAST('NaN' AS DOUBLE) AS value
        """)

    with pytest.raises(ValueError, match=r"required values \['produced_rate'\]"):
        hook._build_result_dataframes()
