import pytest
import pandas as pd
from pathlib import Path
from pydantic import ValidationError
import tempfile
import yaml

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
    hook.conn.execute(
        """
        CREATE TABLE gh_actions_benchmark AS
        SELECT 'loss' AS name, CAST('NaN' AS DOUBLE) AS value
        """
    )

    with pytest.raises(ValueError, match=r"required values \['produced_rate'\]"):
        hook._build_result_dataframes()
