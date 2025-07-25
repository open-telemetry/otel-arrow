import pytest
from io import BytesIO
from pathlib import Path
from lib.impl.strategies.common.report import (
    FileDestination,
    FileDestinationConfig,
    JsonFormatterConfig,
    JsonFormatter,
)


class DummyContext:
    def get_logger(self, name):
        import logging

        return logging.getLogger(name)


class DummyReport:
    def default_filename(self, ext="json", directory=".") -> str:
        return str(Path(directory) / f"default_report.{ext}")


def test_write_text_report(tmp_path):
    config = FileDestinationConfig(
        directory=str(tmp_path), name="text_report", extension="txt"
    )
    dest = FileDestination(config)
    ctx = DummyContext()
    report = DummyReport()

    dest.write("hello world", ctx, report)

    output_file = tmp_path / "text_report.txt"
    assert output_file.exists()
    assert output_file.read_text() == "hello world"


def test_write_binary_report(tmp_path):
    binary_data = BytesIO(b"\x50\x41\x52\x31")  # mock parquet magic bytes

    config = FileDestinationConfig(
        directory=str(tmp_path), name="bin_report", extension="parquet"
    )
    dest = FileDestination(config)
    ctx = DummyContext()
    report = DummyReport()

    dest.write(binary_data, ctx, report)

    output_file = tmp_path / "bin_report.parquet"
    assert output_file.exists()
    assert output_file.read_bytes() == b"\x50\x41\x52\x31"


def test_path_override(tmp_path):
    file_path = tmp_path / "override.json"
    config = FileDestinationConfig(path=str(file_path))
    dest = FileDestination(config)
    ctx = DummyContext()
    report = DummyReport()

    dest.write("data via path", ctx, report)

    assert file_path.exists()
    assert file_path.read_text() == "data via path"


def test_default_filename_used(tmp_path, monkeypatch):
    config = FileDestinationConfig(directory=str(tmp_path))
    dest = FileDestination(config)
    ctx = DummyContext()

    class ReportWithDefault:
        def default_filename(self, ext="json", directory="."):
            return str(Path(directory) / f"autonamed.{ext}")

    dest.write("defaulted content", ctx, ReportWithDefault())

    output_file = tmp_path / "autonamed.json"
    assert output_file.exists()
    assert output_file.read_text() == "defaulted content"


def test_json_formatter_replaces_nan():

    class DummyReport:
        def to_dict(self):
            return {
                "value": float("nan"),
                "items": [1, float("inf"), "test"],
                "nested": {"x": -float("inf")},
            }

    config = JsonFormatterConfig(indent=2)
    formatter = JsonFormatter(config)
    result = formatter.format(DummyReport(), _ctx=None)

    assert '"value": null' in result
    assert '"x": null' in result
    assert '"items": [\n    1,\n    null,\n    "test"\n  ]' in result


def test_json_formatter_basic_formatting():
    class DummyReport:
        def to_dict(self):
            return {"a": 1, "b": "text", "c": True}

    config = JsonFormatterConfig(indent=4)
    formatter = JsonFormatter(config)
    output = formatter.format(DummyReport(), _ctx=None)

    assert '"a": 1' in output
    assert '"b": "text"' in output
    assert output.count("    ") > 0
