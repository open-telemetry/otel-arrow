"""
Module: reporting_hook_strategy

This module defines the core abstractions and orchestration logic for a flexible
and extensible test reporting pipeline. It provides interfaces for formatting,
writing, and integrating report data into broader systems via hook strategies.

Classes:
    ReportFormatter (abstract): Interface for formatting raw test reports into a desired output format.
    DestinationWriter (abstract): Interface for writing formatted report data to a specific destination.
    ReportOutputPipeline: Orchestrates the formatting and writing of test reports using pluggable components.
    ReportingHookStrategy (abstract): Interface for injecting reporting logic into application or testing lifecycles.

These components are designed to be modular and easily extensible, enabling customized reporting workflows
that can adapt to different output formats (e.g., JSON, HTML, XML) and destinations (e.g., file system, APIs, databases).
"""

from abc import abstractmethod
from typing import Any

from pydantic import BaseModel

from ..context.base import BaseContext
from ..test_framework.test_report import TestReport
from .hook_strategy import HookStrategyConfig, HookStrategy


class ReportFormatterConfig(BaseModel):
    """Base model for formatting report output."""


class DestinationWriterConfig(BaseModel):
    """Base model for writing reports to destinations."""


class OutputPipelineConfig(BaseModel):
    """Model for mapping format and destination configs together."""


class ReportingHookStrategyConfig(HookStrategyConfig):
    """Base model for Reporting Strategy config, passed to strategy init."""

    name: str


class ReportFormatter:
    """
    Abstract base class for formatting test reports.

    This class provides a template for creating different report formatters
    by defining a consistent interface. Subclasses must implement the `format`
    method to handle the transformation of a `TestReport` object into the
    desired format, using a given execution context.

    Attributes:
        config (ReportFormatterConfig): Configuration object containing formatter-specific settings.
    """

    @abstractmethod
    def __init__(self, config: ReportFormatterConfig) -> None:
        """
        Initialize the report formatter with a configuration.

        Args:
            config (ReportFormatterConfig): A configuration object used to customize the formatter's behavior.
        """

    def format(self, report: TestReport, ctx: BaseContext) -> Any:
        """
        Format a test report using the provided context.

        This method must be implemented by subclasses.

        Args:
            report (TestReport): The test report object to be formatted.
            ctx (BaseContext): The context in which the formatting is occurring,
                               which may provide environmental or runtime data.

        Returns:
            Any: The formatted output, the exact type of which depends on the formatter implementation.

        Raises:
            NotImplementedError: If the method is not implemented by a subclass.
        """
        raise NotImplementedError


class DestinationWriter:
    """
    Abstract base class for writing formatted report data to a destination.

    This class defines a consistent interface for all destination writers,
    which are responsible for delivering or storing formatted report output
    (e.g., to files, databases, network endpoints, etc.).

    Attributes:
        config (DestinationWriterConfig): Configuration object containing writer-specific settings.
    """

    @abstractmethod
    def __init__(self, config: DestinationWriterConfig) -> None:
        """Initialize with a config object."""

    def write(self, formatted_data: Any, ctx: BaseContext):
        """
        Write the formatted report data to a target destination.

        This method must be implemented by subclasses.

        Args:
            formatted_data (Any): The data to be written, typically the output of a ReportFormatter.
            ctx (BaseContext): The context in which the writing is occurring,
                               which may provide environmental or runtime data.

        Raises:
            NotImplementedError: If the method is not implemented by a subclass.
        """
        raise NotImplementedError


class ReportOutputPipeline:
    """
    Coordinates the process of formatting and writing a test report.

    This class ties together a `ReportFormatter` and a `DestinationWriter`
    to create a complete output pipeline. It first formats a report using
    the provided formatter and then writes the formatted result using
    the writer, if the formatter returns a non-None value.

    Attributes:
        formatter (ReportFormatter): The formatter responsible for transforming the raw test report.
        writer (DestinationWriter): The writer responsible for delivering the formatted report to its destination.
    """

    def __init__(self, formatter: ReportFormatter, writer: DestinationWriter):
        """
        Initialize the output pipeline with a formatter and writer.

        Args:
            formatter (ReportFormatter): An instance of a report formatter.
            writer (DestinationWriter): An instance of a destination writer.
        """
        self.formatter = formatter
        self.writer = writer

    def execute(self, report: TestReport, ctx: BaseContext):
        """
        Execute the report output pipeline.

        Formats the test report and, if formatting succeeds, writes the
        result to the configured destination.

        Args:
            report (TestReport): The test report to process.
            ctx (BaseContext): Context information to use during formatting and writing.
        """
        formatted_data = self.formatter.format(report, ctx)
        if formatted_data is not None:
            self.writer.write(formatted_data, ctx)


class ReportingHookStrategy(HookStrategy):
    """
    Abstract base class for reporting hook strategies.

    This class defines the interface for strategies that manage how and when
    reporting logic is hooked into a larger process or lifecycle. All concrete
    implementations must be initialized with a configuration object that defines
    their behavior.

    Attributes:
        config (ReportingHookStrategyConfig): Configuration object containing strategy-specific settings.
    """

    @abstractmethod
    def __init__(self, config: ReportingHookStrategyConfig) -> None:
        """
        Initialize the reporting hook strategy with a configuration.

        Args:
            config (ReportingHookStrategyConfig): A configuration object used to customize the strategy's behavior.
        """
