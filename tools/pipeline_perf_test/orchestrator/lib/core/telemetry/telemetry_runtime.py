"""
telemetry_runtime.py

Defines the TelemetryRuntime class, which encapsulates runtime components
used for telemetry collection and querying. This includes OpenTelemetry's
TracerProvider and MeterProvider for generating traces and metrics, as well
as a TelemetryClient for accessing structured telemetry data via the framework's
in-memory storage or via a remote backend.

This class serves as a central point for accessing both OpenTelemetry
instrumentation and high-level telemetry retrieval utilities.
"""

from dataclasses import dataclass
from typing import ClassVar, Literal

from opentelemetry.sdk.metrics import Meter, MeterProvider
from opentelemetry.sdk.trace import Tracer, TracerProvider

from .telemetry_client import TelemetryClient


@dataclass
class TelemetryRuntime:
    """Base telemetry runtime."""

    type: ClassVar[Literal["telemetry"]] = "telemetry"

    tracer_provider: TracerProvider
    meter_provider: MeterProvider

    telemetry_client: TelemetryClient

    def get_tracer(self, name="default") -> Tracer:
        """
        Get a tracer from the TracerProvider

        Args:
            name: The name of the tracer or "default"
        Returns:
            An opentelmetry Tracer initialized with the tracer_provider.
        """
        return self.tracer_provider.get_tracer(name)

    def get_meter(self, name="default") -> Meter:
        """
        Get a meter from the MeterProvider

        Args:
            name: The name of the meter or "default"

        Returns:
            An opentelmetry Meter initialized with the meter_provider.
        """
        return self.meter_provider.get_meter(name)

    def get_client(self) -> TelemetryClient:
        """
        Get the runtime telemetry client.

        Returns:
            TelemetryClient with access to span and metrics retrievers.
        """
        return self.telemetry_client
