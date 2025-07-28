"""
signal_retriever.py

Defines the abstract base class SignalRetriever, which provides an interface
for retrieving telemetry data. This is intended to be
subclassed by concrete implementations for specific telemetry signal types,
such as metrics or traces.
"""

from abc import ABC, abstractmethod
from typing import Dict, Any


class SignalRetriever(ABC):
    """
    Abstract interface for querying telemetry signal data.

    Implementations of this interface must define how to retrieve
    the schema that describes the structure of the signal data.
    """

    @abstractmethod
    def get_schema(self) -> Dict[str, Any]:
        """
        Return the schema of the telemetry signal.

        The schema describes the columns and their associated data types.

        Returns:
            Dict[str, Any]: A dictionary where keys are column names and
            values represent the expected data types or metadata for each column.
        """
