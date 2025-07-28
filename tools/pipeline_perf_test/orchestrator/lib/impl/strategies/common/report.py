import json
import math
import os

from io import BytesIO
from typing import Literal, Optional, Any, List

from jinja2 import Template, Environment, FileSystemLoader

from ....core.context.base import BaseContext
from ....core.strategies.reporting_hook_strategy import (
    DestinationWriter,
    ReportFormatter,
    Report,
    ReportFormatterConfig,
    DestinationWriterConfig,
    ReportOutputPipeline,
)

from ....runner.schema.reporting_hook_config import StandardReportingHookStrategyConfig

from ....runner.registry import (
    report_writer_registry,
    report_formatter_registry,
    PluginMeta,
)


@report_formatter_registry.register_config("noop")
class NoopFormatterConfig(ReportFormatterConfig):
    """
    Configuration for the NoopFormatter.

    This formatter does not require any configuration options.
    """


@report_formatter_registry.register_class("noop")
class NoopFormatter(ReportFormatter):
    """
    A report formatter that performs no formatting and returns an empty string.

    Useful as a placeholder or default when no output formatting is desired.
    """

    PLUGIN_META = PluginMeta(
        installs_hooks=[],
        supported_contexts=[],
        yaml_example="""
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - Max Rate
          output:
            - format:
                noop: {}
              destination:
                console: {}
""",
    )

    def __init__(self, config: NoopFormatterConfig):
        self.config = config

    def format(self, report: Report, _ctx: BaseContext) -> Report:
        return None


@report_formatter_registry.register_config("json")
class JsonFormatterConfig(ReportFormatterConfig):
    """
    Configuration for JSON report formatting.

    Attributes:
        indent (Optional[int]): Number of spaces for JSON indentation. Defaults to 2.
    """

    indent: Optional[int] = 2


@report_formatter_registry.register_class("json")
class JsonFormatter(ReportFormatter):
    """
    Formats a report as a JSON string using the specified configuration.

    Args:
        config (JsonFormatterConfig): Configuration specifying JSON formatting options such as indentation.

    Methods:
        format(report, _ctx) -> str:
            Converts the given report into a JSON-formatted string with the configured indentation.
    """

    PLUGIN_META = PluginMeta(
        installs_hooks=[],
        supported_contexts=[],
        yaml_example="""
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - Max Rate
          output:
            - format:
                json: {}
              destination:
                console: {}
""",
    )

    def __init__(self, config: JsonFormatterConfig):
        self.config = config

    def format(self, report: Report, _ctx: BaseContext) -> str:
        """Convert the report to a standards-compliant JSON string with indentation."""
        data = report.to_dict()
        clean_data = self._replace_nans(data)

        return json.dumps(clean_data, indent=self.config.indent, allow_nan=False)

    def _replace_nans(self, obj):
        """Recursively replace NaN, Infinity, -Infinity with None."""
        if isinstance(obj, float):
            if math.isnan(obj) or math.isinf(obj):
                return None
            return obj
        elif isinstance(obj, dict):
            return {k: self._replace_nans(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [self._replace_nans(v) for v in obj]
        else:
            return obj


@report_formatter_registry.register_config("template")
class TemplateFormatterConfig(ReportFormatterConfig):
    """
    Configuration for template-based report formatting.

    Attributes:
        path (Optional[str]): Optional filesystem path to a template file. If provided,
            the template will be loaded from this file.
        string (Optional[str]): Optional inline template string. Used if `path` is not provided.
    """

    path: Optional[str] = None
    string: Optional[str] = None


@report_formatter_registry.register_class("template")
class TemplateFormatter(ReportFormatter):
    """
    Formats a report using a Jinja2 template specified either by file path or inline string.

    Args:
        config (TemplateFormatterConfig): Configuration specifying the template source.

    Methods:
        format(report, _ctx) -> str:
            Renders the report using the template and returns the resulting string.

    Raises:
        FileNotFoundError: If a template file path is provided but the file does not exist.
    """

    PLUGIN_META = PluginMeta(
        installs_hooks=[],
        supported_contexts=[],
        yaml_example="""
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - Max Rate
          output:
            - format:
                template: {}
              destination:
                console: {}
""",
    )

    def __init__(self, config: TemplateFormatterConfig):
        self.config = config

    def format(self, report: Report, _ctx: BaseContext) -> str:
        """
        Render the report using the configured template.

        - If `path` is set, loads and renders the template from the specified file.
        - If `string` is set (and `path` is not), uses the inline template string.
        - Otherwise, uses the report's default display template.

        Args:
            report (Report): The report to format.
            _ctx (BaseContext): The context (not used here).

        Returns:
            str: The formatted report as a string.

        Raises:
            FileNotFoundError: If the template file path does not exist.
        """
        report_dict = report.to_template_dict()

        if self.config.path:
            # Ensure file exists
            if not os.path.isfile(self.config.path):
                raise FileNotFoundError(f"Template file not found: {self.config.path}")
            env = Environment(
                loader=FileSystemLoader(os.path.dirname(self.config.path))
            )
            template = env.get_template(os.path.basename(self.config.path))
        elif self.config.string:
            # Render using the inline template string
            template = Template(self.config.string)
        else:
            template = Template(report.display_template())

        # Render the template with the report data
        return template.render(report=report_dict)


LogLevel = Literal["debug", "info", "warning", "error", "critical"]


@report_writer_registry.register_config("noop")
class NoopDestinationConfig(DestinationWriterConfig):
    """
    Configuration for the NoopDestination writer.

    This writer does not require any configuration parameters.
    """


@report_writer_registry.register_class("noop")
class NoopDestination(DestinationWriter):
    """
    A destination writer that performs no action.

    This writer can be used as a placeholder or stub where a destination
    is required but no actual writing or output should be performed.

    Args:
        config (NoopDestinationConfig): Configuration for the writer (not used).

    Methods:
        write(formatted_data, ctx):
            Intentionally does nothing with the provided data.
    """

    PLUGIN_META = PluginMeta(
        installs_hooks=[],
        supported_contexts=[],
        yaml_example="""
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - Max Rate
          output:
            - format:
                template: {}
              destination:
                noop: {}
""",
    )

    def __init__(self, config: NoopDestinationConfig):
        self.config = config

    def write(self, formatted_data: Any, ctx: BaseContext, report: Report):
        pass  # Intentionally do nothing


@report_writer_registry.register_config("file")
class FileDestinationConfig(DestinationWriterConfig):
    """
    Configuration for the FileDestination writer.

    Attributes:
        path (Optional): Full path to the report output destination.
            overrides directory / name / extension if provided.
        directory (Optional): Output directory to write to.
        name (Optional): Base name of the output file (without extension)
        extension (Optional): Suffix to append to the filename.
    """

    path: Optional[str] = None
    directory: Optional[str] = "./results"
    name: Optional[str] = None
    extension: Optional[str] = None


@report_writer_registry.register_class("file")
class FileDestination(DestinationWriter):
    """
    A destination writer that outputs to a local file.

    This writer will attempt to write the formatted report to a file locally.

    Args:
        config (FileDestinationConfig): Configuration for the writer.

    Methods:
        write(formatted_data, ctx):
            Write the provided data to the file specified in the config.
    """

    PLUGIN_META = PluginMeta(
        installs_hooks=[],
        supported_contexts=[],
        yaml_example="""
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - Max Rate
          output:
            - format:
                template: {}
              destination:
                file:
                    directory: ./reports
                    name: my_report
                    extension: .json
""",
    )

    def __init__(self, config: FileDestinationConfig):
        self.config = config

    def _resolve_path(self, report: Report) -> str:
        """
        Resolve the full file path from the configuration.
        """
        if self.config.path:
            return self.config.path

        # Extension handling
        ext = self.config.extension
        if ext:
            ext = ext.lstrip(".")  # Remove leading dot if present
        else:
            ext = "json"  # Default fallback

        # Directory handling
        directory = self.config.directory or "."
        os.makedirs(directory, exist_ok=True)

        # Filename handling
        if self.config.name:
            filename = f"{self.config.name}.{ext}"
            return os.path.join(directory, filename)
        else:
            return report.default_filename(ext=ext, directory=directory)

    def write(self, formatted_data: Any, ctx: BaseContext, report: Report):
        """
        Write the formatted report data to a local file.
        """
        path = self._resolve_path(report)
        logger = ctx.get_logger(__name__)
        if isinstance(formatted_data, (bytes, BytesIO)):
            mode = "wb"
            with open(path, mode) as f:
                if isinstance(formatted_data, BytesIO):
                    f.write(formatted_data.getvalue())
                else:
                    f.write(formatted_data)
            return
        try:
            with open(path, "w", encoding="utf-8") as f:
                f.write(formatted_data)
        except Exception as e:
            logger.error(f"Failed to write report to file: {e}")
            raise


@report_writer_registry.register_config("console")
class ConsoleDestinationConfig(DestinationWriterConfig):
    """
    Configuration for ConsoleDestination writer.

    Attributes:
        use_logger (Optional[bool]): Whether to use a logger to output the report.
            If False, outputs directly to stdout using print(). Defaults to False.
        log_level (Optional[LogLevel]): The log level to use if use_logger is True.
            Defaults to "warning".
        logger_name (Optional[str]): The name of the logger to use when use_logger is True.
            Defaults to "report".
    """

    use_logger: Optional[bool] = False
    log_level: Optional[LogLevel] = "warning"
    logger_name: Optional[str] = "report"


@report_writer_registry.register_class("console")
class ConsoleDestination(DestinationWriter):
    """
    Writes report data to the console (stdout) or via a logger.

    Depending on configuration, this destination can either print the formatted
    report directly to stdout or send it to a configured logger at a specified
    log level.

    Args:
        config (ConsoleDestinationConfig): Configuration for this destination writer.

    Methods:
        write(formatted_data, ctx):
            Outputs the formatted data to the console or logger.
    """

    PLUGIN_META = PluginMeta(
        installs_hooks=[],
        supported_contexts=[],
        yaml_example="""
hooks:
  run:
    post:
      - pipeline_perf_report:
          name: PerfReprort - Max Rate
          output:
            - format:
                template: {}
              destination:
                console: {}
""",
    )

    def __init__(self, config: ConsoleDestinationConfig):
        self.config = config

    def write(self, formatted_data: Any, ctx: BaseContext, report: Report):
        """
        Write the formatted report data.

        If `use_logger` is True in the configuration, logs the data at the configured
        log level using the specified logger. Otherwise, prints the data to stdout.

        Args:
            formatted_data (Any): The formatted report data to output.
            ctx (BaseContext): The current execution context, used for logger retrieval.
        """
        if self.config.use_logger:
            logger = ctx.get_logger(self.config.logger_name)
            log_fn = getattr(logger, self.config.log_level, logger.info)
            log_fn(formatted_data)
        else:
            print(formatted_data)


def get_default_writer() -> DestinationWriter:
    """
    Create and return the default report destination writer.

    Returns:
        DestinationWriter: A NoopDestination instance which performs no output.
    """
    return NoopDestination(NoopDestinationConfig())


def get_default_formatter() -> ReportFormatter:
    """
    Create and return the default report formatter.

    Returns:
        ReportFormatter: A JsonFormatter instance configured with 2-space indentation.
    """
    return JsonFormatter(JsonFormatterConfig(indent=2))


def get_report_pipelines(
    config: StandardReportingHookStrategyConfig, default_template: Optional[str] = None
) -> List[ReportOutputPipeline]:
    """
    Construct and return a list of report output pipelines based on the given configuration.

    Each pipeline consists of a formatter and a writer, determined by the specified
    format and destination strategies in the configuration. If no pipelines are
    configured, a default pipeline using the JSON formatter and Noop writer is returned.

    Args:
        config (StandardReportingHookStrategyConfig): Configuration defining the output pipelines,
            including formatters and destinations.
        default_template (Optional[str]): Optional default template string to use when
            a 'template' formatter strategy is specified without an inline template.

    Raises:
        ValueError: If a formatter or writer strategy or class is unknown/missing.

    Returns:
        List[ReportOutputPipeline]: A list of fully constructed report output pipelines.
    """
    pipelines: List[ReportOutputPipeline] = []
    for pipeline in config.output:
        rept_fmt = get_default_formatter()
        writer = get_default_writer()
        if pipeline.format:
            for strategy_type, config_data in pipeline.format.items():
                config_cls = report_formatter_registry.config.get(strategy_type)
                if not config_cls:
                    raise ValueError(
                        f"Unknown report format strategy: '{strategy_type}'"
                    )
                fmt_class = report_formatter_registry.element.get(strategy_type)
                if not fmt_class:
                    raise ValueError(f"Unknown report format class: '{strategy_type}'")
                if strategy_type == "template":
                    config_data.string = default_template
                rept_fmt = fmt_class(config_data)
        if pipeline.destination:
            for strategy_type, config_data in pipeline.destination.items():
                config_cls = report_writer_registry.config.get(strategy_type)
                if not config_cls:
                    raise ValueError(
                        f"Unknown report destination strategy: '{strategy_type}'"
                    )
                # fmt_config = config_cls(**config_data)
                writer_class = report_writer_registry.element.get(strategy_type)
                if not writer_class:
                    raise ValueError(
                        f"Unknown report destination class: '{strategy_type}'"
                    )
                writer = writer_class(config_data)

        pipelines.append(ReportOutputPipeline(formatter=rept_fmt, writer=writer))
    if not pipelines:
        return [
            ReportOutputPipeline(
                formatter=get_default_formatter(), writer=get_default_writer()
            )
        ]
    return pipelines
