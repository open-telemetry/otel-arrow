"""
Module for SQL-based Telemetry Reporting using DuckDB.

This module defines a reporting strategy that leverages SQL queries to generate
reports from telemetry data, such as metrics, spans, and events. It uses DuckDB
as the execution engine and supports loading data from external files, in-memory
dataframes, and configurable YAML-based report definitions.

Core Components:
----------------
- 'SQLReportConfig': Configuration schema for specifying report details either
  directly or via an external YAML file.
- 'SQLReportDetails': Structured schema defining external tables to load, SQL
  queries to run, and output tables to produce.
- 'SQLReport': An implementation of the abstract 'Report' class that renders
  query results and metadata into markdown-formatted output.
- 'SQLReportHook': A registered hook class that executes the full report
  workflow:
    - Loads metadata and telemetry
    - Registers in-memory DuckDB tables
    - Executes SQL queries from configuration
    - Outputs result tables in CSV/JSON/Parquet formats
    - Returns a rendered report object

Features:
---------
- Supports flexible templated file paths for loading/writing tables.
- Handles missing data gracefully with default DDL options.
- Integrates with a telemetry context to query live telemetry.
- Renders results into markdown for templated output generation.
- Allows both inline and file-based configuration for report definition.

Intended Usage:
---------------
This module is intended to be registered as part of a plugin system,
executed post-pipeline to generate telemetry insights based on user-defined
SQL queries and table configurations.

Example YAML Configuration:
---------------------------
hooks:
  run:
    post:
      - sql_report:
          name: "Performance Report"
          report_config_file: "./report_configs/perf.yaml"
          output:
            - format:
                template: {}
              destination:
                console: {}
"""

import os
from typing import ClassVar, Optional, List, Dict, Any, Literal
from pathlib import Path

import duckdb
import pandas as pd
import yaml
from jinja2 import Template
from pydantic import BaseModel, Field, model_validator, ConfigDict

from .....core.context.base import BaseContext
from .....core.context import FrameworkElementHookContext
from .....core.framework.report import Report, ReportAggregation
from .....core.telemetry.telemetry_client import TelemetryClient
from .....runner.registry import hook_registry, PluginMeta, ReportMeta
from .....runner.schema.reporting_hook_config import StandardReportingHookStrategyConfig
from .standard_reporting_strategy import StandardReportingStrategy


STRATEGY_NAME = "sql_report"


@hook_registry.register_config(STRATEGY_NAME)
class SQLReportConfig(StandardReportingHookStrategyConfig):
    report_config: Optional["SQLReportDetails"] = None
    report_config_file: Optional[Path] = None

    @model_validator(mode="before")
    def validate_source(cls, values):
        if not values.get("report_config") and not values.get("report_config_file"):
            raise ValueError(
                "Either 'report_config' or 'report_config_file' must be specified in SQLReportConfig"
            )
        return values


class TableIOConfig(BaseModel):
    path: Optional[str] = None
    path_template: Optional[str] = None
    format: Literal["parquet", "json", "csv"] = "parquet"

    @model_validator(mode="before")
    def validatepath(cls, values):
        if not values.get("path_template") and not values.get("path"):
            raise ValueError(
                "Either 'path' or 'path_template' must be specified in SqlRSQLReportConfig.WriteTableConfig."
            )
        return values


class LoadTableConfig(TableIOConfig):
    default_ddl: Optional[str] = None


class WriteTableConfig(TableIOConfig):
    pass


class QueryConfig(BaseModel):
    name: str
    sql: str


class ResultTable(BaseModel):
    name: str
    description: Optional[str] = None
    display: bool = True


class SQLReportDetails(BaseModel):
    load_tables: Optional[Dict[str, LoadTableConfig]] = Field(default_factory=dict)
    queries: List[QueryConfig] = Field(default_factory=list)
    result_tables: List[ResultTable] = Field(default_factory=list)
    write_tables: Optional[Dict[str, WriteTableConfig]] = Field(default_factory=dict)

    model_config = ConfigDict(extra="forbid")


class SQLReport(Report):
    """
    Report class for interacting with the metrics / logs / traces tables via SQL.

    This class extends the base Report class to provide flexible SQL driven reporting
    from a yaml config.

    Attributes:
        config (Optional[PipelinePerfReportConfig]): Configuration instance
            that controls report generation and table inclusion.
        REPORT_TYPE (ClassVar[str]): Identifier for this report type, linked to
            the associated reporting strategy.

    Methods:
        to_template_dict():
            Converts the report results and metadata into a dictionary with
            markdown-formatted strings for each included report section.

        get_template(mode):
            Returns a Jinja2 template string for rendering reports in the
            specified aggregation mode.
    """

    config: Optional[SQLReportConfig] = None
    REPORT_TYPE: ClassVar[str] = STRATEGY_NAME

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
        data["markdown_results"] = {}
        data["table_descriptions"] = {
            conf.name: conf.description
            for conf in self.config.report_config.result_tables
        }
        data["table_display"] = {
            conf.name: conf.display for conf in self.config.report_config.result_tables
        }
        data["result_tables"] = [
            conf.name for conf in self.config.report_config.result_tables
        ]
        for table_name, df in self.results.items():
            data["markdown_results"][table_name] = df.to_markdown(index=False)

        return data

    @classmethod
    def get_template(cls, mode: ReportAggregation) -> str:
        default = """# {{ report.metadata["report.name"] }}
{%- if report.markdown_results %}
{% for table_name in report.result_tables if report.table_display[table_name] %}
## Table: {{ table_name }}
  {% if report.table_descriptions and table_name in report.table_descriptions and report.table_descriptions[table_name] %}
**Description**: {{ report.table_descriptions[table_name] }}
  {% endif %}
{{ report.markdown_results[table_name] }}

{% endfor %}
{% endif %}
"""
        templates = {
            ReportAggregation.NONE: default,
            ReportAggregation.COMPARISON: default,
            ReportAggregation.TIMESERIES: default,
        }
        return templates.get(mode, "Unsupported aggregation mode")


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
            queries:
            - name: "aggregate_metrics"
                sql: |
            result_tables:
                - foo
                - bar
            write_tables:
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
                ReportAggregation.NONE.value,
            ],
            sample_output={},
        ),
    )

    def __init__(self, config: SQLReportConfig):
        """Initialize the report with a config and optionally load from file."""
        super().__init__(config)
        self.config = config
        self.conn = None
        if self.config.report_config_file:
            self._load_report_config_from_file()

    def _load_report_config_from_file(self):
        """Load and validate an external report config file."""
        path: Path = self.config.report_config_file
        parsed = yaml.safe_load(path.read_text())
        self.config.report_config = SQLReportDetails(**parsed)

    def _load_external_tables(self, logger, report):
        """Load external tables from csv, json, parquet files.

        Args:
            logger: instance for logging information about execution status
            report: instance of a report class used to expand templates

        Raises:
            ValueError on unknown table format
            FileNotFoundError on missing source table and no default create statement
        """
        if not self.config.report_config or not self.config.report_config.load_tables:
            return

        for table_name, cfg in self.config.report_config.load_tables.items():
            logger.debug("Loading external table %s", table_name)
            path = cfg.path
            if cfg.path_template:
                path = Template(cfg.path_template).render(
                    metadata=report.metadata, table_name=table_name
                )
            matched_files = list(Path(path).parent.glob(Path(path).name))

            if not matched_files:
                if cfg.default_ddl:
                    self.conn.execute(cfg.default_ddl)
                    continue
                raise FileNotFoundError(
                    f"No files matching pattern '{path}' for table '{table_name}' found."
                )

            read_funcs = {
                "parquet": "read_parquet",
                "json": "read_json_auto",
                "csv": "read_csv_auto",
            }
            if cfg.format in read_funcs:
                reader_func = read_funcs[cfg.format]
                self.conn.execute(
                    f"CREATE OR REPLACE TABLE {table_name} AS SELECT * FROM {reader_func}('{path}')"
                )
            else:
                raise ValueError(
                    f"Unsupported format '{cfg.format}' for table '{table_name}'"
                )

    def _run_sql_queries(self, logger):
        """Run the sql queries defined in the report config"""
        for query in self.config.report_config.queries:
            logger.debug("Running sql query %s", query.name)
            self.conn.execute(query.sql)

    def _register_in_memory_tables(self, metrics, spans, events):
        """Flatten and register in-memory telemetry tables and regaister in duckdb

        This method extracts the various attribute dicts into their own columns for easier querying.
        """
        # Flatten and register metrics
        metrics = flatten_columns(
            metrics, ["metric_attributes", "resource_attributes", "scope_attributes"]
        )
        self.conn.register("metrics", metrics)

        # Flatten and register spans
        spans = flatten_columns(spans, ["resource", "attributes"])
        self.conn.register("spans", spans)

        # Flatten and register events
        events = flatten_columns(events, ["attributes"])
        self.conn.register("events", events)

    def _build_result_dataframes(self):
        """Loop through the result_tables config and convert them to result dataframes"""
        results = {}
        for table in self.config.report_config.result_tables:
            results[table.name] = self.conn.execute(
                f"SELECT * FROM {table.name}"
            ).fetchdf()
        return results

    def _build_metadata_table(self, metadata):
        """Register duckdb tables containing context metadata."""
        meta_df = pd.DataFrame([metadata])
        meta_kv_df = meta_df.transpose().reset_index()
        meta_kv_df.columns = ["Attribute", "Value"]
        meta_kv_df = meta_kv_df.sort_values(by="Attribute", ascending=False)
        self.conn.register("metadata", meta_kv_df)
        self.conn.register("metadata_row", meta_df)

    def _write_table_to_parquet(self, table_name, path):
        """Output the given table name to the specified path in parquet format"""
        self.conn.execute(
            f"COPY {table_name} TO '{path}' (FORMAT PARQUET, OVERWRITE TRUE);"
        )

    def _write_table_to_json(self, table_name, path):
        """Output the given table name to the specified path in json format"""
        # OVERWRITE TRUE is not compatible with json
        if os.path.exists(path):
            os.remove(path)
        self.conn.execute(f"COPY {table_name} TO '{path}' (FORMAT JSON, ARRAY);")

    def _write_table_to_csv(self, table_name, path):
        """Output the given table name to the specified path in csv format"""
        self.conn.execute(
            f"COPY {table_name} TO '{path}' (FORMAT CSV, HEADER, DELIMITER ',', OVERWRITE TRUE);"
        )

    def _write_tables(self, logger, report):
        """Write any configured tables to disk in the specified format"""
        for table_name, write_conf in self.config.report_config.write_tables.items():
            logger.debug("Writing SQL table %s to disk", table_name)
            path = write_conf.path
            if write_conf.path_template:
                path = Template(write_conf.path_template).render(
                    metadata=report.metadata, table_name=table_name
                )
            # Ensure the output directory exists
            os.makedirs(os.path.dirname(path), exist_ok=True)
            if write_conf.format == "parquet":
                self._write_table_to_parquet(table_name, path)
            elif write_conf.format == "json":
                self._write_table_to_json(table_name, path)
            elif write_conf.format == "csv":
                self._write_table_to_csv(table_name, path)
            else:
                raise RuntimeError(f"Unknown table output format: {write_conf.format}")

    def _execute(self, ctx: BaseContext) -> Report:
        """
        Executes the SQL report workflow using the provided context.

        This method performs the full report generation process, including:
        - Initializing the report object and telemetry client
        - Connecting to an in-memory DuckDB instance
        - Building metadata and optionally loading external tables
        - Querying telemetry data (metrics, spans, events)
        - Registering in-memory tables for SQL querying
        - Running configured SQL queries
        - Optionally writing result tables to disk
        - Collecting and returning query results in a Report object

        Args:
            ctx (BaseContext): The execution context containing telemetry, logger, and configuration data.

        Returns:
            Report: A populated SQLReport object containing results and metadata.

        Raises:
            Exception: If writing result tables to disk fails.
        """
        logger = ctx.get_logger(__name__)
        report: SQLReport = SQLReport.from_context(self.config.name, ctx)
        report.config = self.config
        tc: TelemetryClient = ctx.get_telemetry_client()
        self.conn = duckdb.connect()

        self._build_metadata_table(report.metadata)

        if self.config.report_config.load_tables:
            self._load_external_tables(logger, report)

        # Fetch the metrics, spans, events dataframes
        metrics = tc.metrics.query_metrics()
        spans = tc.spans.query_spans()
        events = tc.spans.query_span_events(
            where=lambda df: df[df["name"] != "log"].reset_index(drop=True)
        )
        self._register_in_memory_tables(metrics, spans, events)

        self._run_sql_queries(logger)

        if self.config.report_config.write_tables:
            try:
                self._write_tables(logger, report)
            except Exception as e:
                logger.error("SQL Report failed to write tables %s", e)
                raise

        results = self._build_result_dataframes()
        report.set_results(results)
        return report


def flatten_columns(df: pd.DataFrame, columns_to_flatten: list[str]) -> pd.DataFrame:
    """Flattens specified JSON-like columns in a DataFrame."""
    for col in columns_to_flatten:
        if col in df.columns:
            flattened = pd.json_normalize(df[col])
            flattened.columns = [f"{col}.{subcol}" for subcol in flattened.columns]
            df = pd.concat([df.drop(columns=[col]), flattened], axis=1)
    return df
