"""
telemetry_client.py

Defines the TelemetryClient class, a lightweight container that aggregates
telemetry data sources such as metrics and spans. This class allows easy
access to both types of telemetry from a single interface.
"""

from dataclasses import dataclass

from .metric import MetricsRetriever
from .span import SpanRetriever


@dataclass
class TelemetryClient:
    """
    A data class that provides access to telemetry data sources.

    Attributes:
        metrics (MetricsRetriever): An object responsible for retrieving
            metric data such as counters, gauges, and histograms.
        spans (SpanRetriever): An object responsible for retrieving
            tracing spans, useful for distributed tracing and performance
            monitoring.
    """

    metrics: MetricsRetriever
    spans: SpanRetriever
