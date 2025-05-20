"""
Module: monitoring_strategy

This module defines the `MonitoringStrategy` abstract base class, which provides
a common interface for monitoring components in a load generation testbed.

Monitoring strategies are responsible for gathering, starting, and stopping monitoring
for components during tests. This can involve collecting performance metrics, logging,
or gathering any relevant statistics or insights from the component.

Typical implementations of this interface include:
    - ResourceMonitoring: Monitors system resources such as CPU, memory, and network usage.
    - HealthMonitoring: Monitors the health and availability of the component (e.g., via heartbeats).
    - ResourceLogging: Monitors the application’s logs from file, docker, k8s.

Classes:
    MonitoringStrategy (ABC): Abstract base class for all monitoring strategies.
"""

from abc import ABC, abstractmethod
from typing import List


class MonitoringStrategy(ABC):
    """
    Abstract base class for monitoring strategies.

    Monitoring strategies define how to start, stop, and collect data from a component’s monitoring
    system. Concrete implementations should specify how to track, log, and aggregate monitoring
    data for a given component.

    Methods:
        start(): Begin the monitoring process for the component.
        stop(): Stop the monitoring process.
        collect(): Collect and return monitoring data as a dictionary.
    """

    @abstractmethod
    def start(self):
        """
        Start the monitoring process.

        This method initializes and starts the collection of monitoring data for the component.
        """

    @abstractmethod
    def stop(self):
        """
        Stop the monitoring process.

        This method shuts down any active monitoring and ensures data collection is concluded.
        """

    @abstractmethod
    def collect(self) -> dict:
        """
        Collect and return monitoring data.

        This method aggregates and returns the collected monitoring data as a dictionary.

        Returns:
            dict: A dictionary of collected monitoring data.
        """
