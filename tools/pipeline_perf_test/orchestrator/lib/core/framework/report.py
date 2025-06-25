"""
Module: report

This module defines the `Report` class, which represents structured test or data quality reports,
including result data, metadata, and associated timestamps.

It provides utilities for:
- Serializing reports to JSON-compatible dictionaries with proper timestamp handling.
- Generating consistent, descriptive default filenames for reports.
- Aggregating multiple reports into comparison or timeseries views for analysis.

The module also defines supporting enums and types used to configure aggregation behaviors.
"""

import datetime
import json
import re
import os
from dataclasses import asdict, dataclass
from enum import Enum
from typing import Any, Dict, ClassVar, List, TYPE_CHECKING

import pandas as pd

from ..context.base import BaseContext
from ..helpers.report import slugify

if TYPE_CHECKING:
    from ..strategies.reporting_hook_strategy import ReportingHookStrategyConfig


class ReportAggregation(Enum):
    """
    Enumeration of supported test report aggregation strategies.

    This enum defines the available modes for aggregating test reports.
    These modes determine how multiple test results are combined or presented
    over time or across different configurations.

    Members:
        NONE: No aggregation is applied; reports are treated independently.
        COMPARISON: Aggregates reports for comparison across test runs or environments.
        TIMESERIES: Aggregates reports over time for trend or time-based analysis.
    """

    NONE = "none"
    COMPARISON = "comparison"
    TIMESERIES = "timeseries"


@dataclass
class Report:
    """
    A structured representation of a test report, including results, metadata, and serialization utilities.

    This class captures the output of a test execution or other checks,
    storing results as a dictionary of pandas DataFrames, along with metadata
    and a timestamp. It also supports serialization, filename generation, and
    report aggregation.

    Attributes:
        report_name (str): A human-readable name for the report.
        report_time (datetime.datetime): Timestamp indicating when the report was generated.
        metadata (Dict[str, Any]): Additional metadata describing the context or configuration of the test.
        results (Dict[str, pd.DataFrame]): A mapping from result names to pandas DataFrames containing result data.
        REPORT_TYPE (str, class variable): A string identifier for the type of report. Subclasses should override this.
    """

    report_name: str
    report_time: datetime.datetime
    metadata: Dict[str, Any]
    results: Dict[str, pd.DataFrame]

    REPORT_TYPE: ClassVar[str] = "unknown_test_type"

    @classmethod
    def aggregate(
        cls,
        reports: List["Report"],
        mode: ReportAggregation = ReportAggregation.COMPARISON,
        *,
        label_key: str = "name",
    ) -> Dict[str, pd.DataFrame]:
        """
        Aggregate multiple reports together into a comparison or timeseries report.

        Args:
            reports (List[TestReport]): List of reports to aggregate.
            mode (str): Aggregation mode - "comparison" or "timeseries".
            label_key (str): Key in metadata to use for labeling runs (e.g., name, timestamp).

        Returns:
            - If COMPARISON: A single DataFrame with side-by-side metrics.
            - If TIMESERIES: A dict mapping metric names to time series DataFrames.
        """
        raise ValueError(f"Report type does not support aggregation: {cls.REPORT_TYPE}")

    @classmethod
    def to_aggregate_template_dict(
        cls,
        results: Dict[str, pd.DataFrame],
        config: "ReportingHookStrategyConfig",
        mode: ReportAggregation = ReportAggregation.COMPARISON,
    ):
        """Abstract method for preparing a dict to pass to an output template.

        This method can be used to format aggregated report results for rendering with
        the jinja templating system. Reports will implement this method to define what
        fields are available to the template engine.

        Args:
            results: A dict mapping result dataframe names to the dataframes.
            config: The configuration for the aggregate report.
            mode: The report aggregation mode for the current set of reports.
        """

    @classmethod
    def get_template(self, mode: ReportAggregation) -> str:
        """
        Override this method in subclasses to customize default data output template.

        Args:
            mode: The report aggregation mode to fetch the template for.
        """
        templates = {
            ReportAggregation.NONE: "{{results}}",
            ReportAggregation.COMPARISON: "{{results}}",
            ReportAggregation.TIMESERIES: "{{results}}",
        }
        tmpl = templates.get(mode)
        if not tmpl:
            raise ValueError(f"Unsupported aggregation mode: {mode}")
        return tmpl

    @classmethod
    def from_json(cls, json_str: str) -> "Report":
        """
        Construct a TestReport from json data.

        Args:
            json_str: the json string containing the TestReport data.
        """
        data = json.loads(json_str)
        return cls.from_dict(data)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Report":
        """
        Construct a TestReport from a python dict.

        Args:
            data: python dictionary containing the report data.
        """
        # Convert each list-of-dicts back to a DataFrame
        results = {
            key: pd.DataFrame(records) for key, records in data["results"].items()
        }

        return cls(
            report_name=data["reprt_name"],
            report_time=data["report_time"],
            metadata=data["metadata"],
            results=results,
        )

    @classmethod
    def from_context(cls, name: str, ctx: BaseContext) -> "Report":
        """
        Create a TestReport from a test context object.

        This method builds a new named TestReport and initializes common
        metadata attributes from the provided context.

        Args:
            name: The name of the test report which can be referenced in
              aggregation reports.
            ctx: The context for test framework element in which the report
              is being generated.
        """

        def to_iso(dt):
            return dt.isoformat() if dt else None

        now = datetime.datetime.now(tz=datetime.timezone.utc)
        ctx_meta = ctx.get_metadata()
        for k, v in ctx_meta.items():
            if isinstance(v, datetime.datetime):
                ctx_meta[k] = to_iso(v)
        ts = ctx.get_suite()
        test_meta = {
            **ctx_meta,
            "test.suite.start": to_iso(ts.context.start_time),
            "test.suite": ctx_meta.get("test.suite"),
            "report.type": cls.REPORT_TYPE,
            "report.time": to_iso(now),
            "report.name": name,
        }
        test_name = ctx_meta.get("test.name")
        if test_name:
            for test_ctx in ts.context.child_contexts:
                if test_ctx.name == test_name:
                    test_meta["test.name"] = test_name
                    test_meta["test.start"] = to_iso(test_ctx.start_time)
        return cls(report_name=name, report_time=now, metadata=test_meta, results={})

    def to_dict(self) -> Dict[str, Any]:
        """
        Serialize the TestReport object to a dictionary suitable for JSON encoding.

        This method converts the TestReport instance into a standard Python dictionary,
        performing special handling for values that are not JSON-serializable by default,
        such as `pandas.Timestamp`, `datetime.datetime`, and nested metadata structures.

        Transformations:
            - Converts `report_time` to ISO 8601 string format.
            - Recursively converts timestamp-like objects within `metadata`.
            - Applies timestamp conversion to each cell in each DataFrame within `results`,
            then flattens the DataFrames into lists of dictionaries.

        Returns:
            Dict[str, Any]: A dictionary representation of the test report with all values
            made JSON-compatible.

        Example structure:
            {
                "report_time": "2025-06-16T10:15:30.123Z",
                "metadata": { ... },  # all timestamps converted
                "results": {
                    "some_check": [
                        {"col1": "value", "timestamp": "2025-06-15T08:00:00Z"},
                        ...
                    ],
                    ...
                },
                ...
            }
        """

        def convert_timestamps(val):
            if isinstance(val, (pd.Timestamp, datetime.datetime)):
                return val.isoformat()
            return val

        def convert_metadata(val):
            if isinstance(val, dict):
                return {k: convert_metadata(v) for k, v in val.items()}
            elif isinstance(val, list):
                return [convert_metadata(v) for v in val]
            return convert_timestamps(val)

        base_dict = asdict(self)
        base_dict["report_time"] = convert_timestamps(base_dict["report_time"])
        base_dict["metadata"] = convert_metadata(base_dict["metadata"])
        base_dict["results"] = {
            key: df.map(convert_timestamps).to_dict(orient="records")
            for key, df in self.results.items()
        }

        return base_dict

    def to_template_dict(self) -> Dict[str, Any]:
        """Prepare a dictionary of TestReport data to pass to a template.

        This method can be used to format report results for rendering with
        the jinja templating system. Reports will implement this method to define what
        fields are available to the template engine.
        """
        data = self.to_dict()
        return data

    def display_template(self):
        """
        Fetch the display template for the the TestReport given the current aggregation.
        """
        return self.get_template(mode=ReportAggregation.NONE)

    def set_results(self, results: Dict[str, pd.DataFrame]):
        """
        Set the TestReport results to the provided dict.

        Args:
            results: a dict of named report results.
        """
        for name, df in results.items():
            if not isinstance(df, pd.DataFrame):
                raise TypeError(f"Result '{name}' is not a DataFrame")
        self.results = results

    def default_filename(self, ext="json", directory=".") -> str:
        """
        Generate a default filename for the test report based on metadata.

        This method constructs a filename using the report's type, name, and timestamp.
        It ensures that the filename is safe for use in filesystems by slugifying the
        report type and name, and formatting the timestamp.

        Filename format:
            <report_type>-<report_name>-<timestamp>.<ext>

        Timestamp formatting:
            - If `report_time` is a `datetime`, it's formatted as YYYYMMDD_HHMMSS.
            - If it's a string, it's cleaned and stripped of delimiters.
            - If the timestamp is not recognized, "unknown_time" is used.

        Args:
            ext (str): The file extension to use (default is "json").
            directory (str): The directory to prepend to the filename (default is current directory).

        Returns:
            str: A full path string representing the generated filename.
        """
        report_type = slugify(self.REPORT_TYPE)
        report_name = slugify(self.report_name)
        start_time = self.report_time

        if isinstance(start_time, datetime.datetime):
            timestamp = start_time.strftime("%Y%m%d_%H%M%S")
        elif isinstance(start_time, str):
            timestamp = re.sub(r"[:\-T]", "_", start_time.split(".")[0])
        else:
            timestamp = "unknown_time"

        filename = f"{report_type}-{report_name}-{timestamp}.{ext}"
        return os.path.join(directory, filename)
