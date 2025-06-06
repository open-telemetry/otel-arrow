"""
Module: reporting_strategy

This module defines a set of abstract base classes that provide an interface for reporting strategies
in a load generation testbed. The goal of these strategies is to format, output, and report aggregated
monitoring and test data in various ways.

The three key components in this module are:
    - `FormatStrategy`: Defines how aggregated data should be formatted into a specific structure (e.g., JSON, CSV, etc.).
    - `DestinationStrategy`: Defines where and how the formatted data should be output (e.g., to a file, console, or external service).
    - `ReportingStrategy`: Integrates the formatting and output strategies to generate and report test data.

Classes:
    ReportingStrategyConfig(BaseModel) Base model for configuring reporting strategies.
    FormatStrategy (ABC): Interface for strategies that format aggregated test data.
    DestinationStrategy (ABC): Interface for strategies that determine where and how to output formatted data.
    ReportingStrategy (ABC): Interface for strategies that combine formatting and output to report aggregated data.
"""

from abc import ABC, abstractmethod
from typing import Any, Dict

from ..test_framework.test_data import TestData
from .base import BaseStrategyConfig


class ReportingStrategyConfig(BaseStrategyConfig):
    """Base model for Reporting Strategy config, passed to strategy init."""


class FormatStrategy(ABC):
    """
    Abstract base class for data formatting strategies.

    The `FormatStrategy` class defines how aggregated test data should be formatted into a specific
    structure (e.g., JSON, CSV). Implementations should specify the format that best suits the reporting
    needs for the testbed.

    Methods:
        format(aggregated_data): Format the aggregated test data into a desired structure.

    Args:
        aggregated_data (Dict[str, Dict[str, any]]): The data to be formatted, typically containing metrics
                                                      or test results organized by component and metric name.

    Returns:
        str: The formatted data as a string.
    """

    @abstractmethod
    def format(self, aggregated_data: Dict[str, Dict[str, Any]]) -> str:
        """
        Format the aggregated test data.

        Args:
            aggregated_data (Dict[str, Dict[str, any]]): The data to format.

        Returns:
            str: The formatted data as a string.
        """


class DestinationStrategy(ABC):
    """
    Abstract base class for data output strategies.

    The `DestinationStrategy` class defines where and how the formatted data should be sent, such as
    outputting to a file, the console, or an external service. Concrete implementations will specify
    the exact destination behavior.

    Methods:
        output(formatted_data): Output the formatted data to a specified destination.

    Args:
        formatted_data (str): The data to output, typically in the form of a string after being formatted.
    """

    @abstractmethod
    def output(self, formatted_data: str):
        """
        Output the formatted data.

        Args:
            formatted_data (str): The formatted data to be sent to the destination.
        """


class ReportingStrategy(ABC):
    """
    Abstract base class for reporting strategies.

    The `ReportingStrategy` class integrates the formatting and destination strategies to generate
    and report test data. This could involve formatting the aggregated data and sending it to a
    designated output, such as a log, file, or monitoring system.

    Methods:
        report(aggregated_data): Format and output the aggregated test data.

    Args:
        aggregated_data (Dict[str, Dict[str, any]]): The data to be reported, typically containing metrics
                                                      or test results organized by component and metric name.
    """

    @abstractmethod
    def __init__(self, config: ReportingStrategyConfig) -> None:
        """All reporting strategies must be initialized with a config object."""

    @abstractmethod
    def report(self, test_data: TestData):
        """
        Generate and report by formatting and outputting the test data.

        Args:
            test_data: The aggregated data to format and report.
        """
