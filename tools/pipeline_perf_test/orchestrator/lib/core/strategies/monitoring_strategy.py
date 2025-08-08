"""
Module: monitoring_strategy

This module defines the `MonitoringStrategy` abstract base class, which provides
a common interface for monitoring components in a load generation testbed.

Monitoring strategies are responsible for gathering, starting, and stopping monitoring
for components during tests. This can involve collecting performance metrics, logging,
or gathering any relevant statistics or insights from the component.

Classes:
    MonitoringStrategyConfig(BaseModel): Base class for configuring monitoring strategies.
    MonitoringStrategy (ABC): Abstract base class for all monitoring strategies.
"""

from abc import abstractmethod
from typing import TYPE_CHECKING

from ..context.framework_element_contexts import StepContext, ScenarioContext
from .base import BaseStrategyConfig, BaseStrategy

if TYPE_CHECKING:
    from ..component.component import Component


class MonitoringStrategyConfig(BaseStrategyConfig):
    """Base model for Monitoring Strategy config, passed to strategy init."""


class MonitoringStrategy(BaseStrategy):
    """
    Abstract base class for monitoring strategies.

    Monitoring strategies define how to start, stop, and collect data from a component's monitoring
    system. Concrete implementations should specify how to track, log, and aggregate monitoring
    data for a given component.

    Methods:
        start(component, ctx): Begin the monitoring process for the component.
        stop(component, ctx): Stop the monitoring process.
        collect(component, ctx): Collect and return monitoring data as a dictionary.
    """

    @abstractmethod
    def __init__(self, config: MonitoringStrategyConfig) -> None:
        """All monitoring strategies must be initialized with a config object."""

    @abstractmethod
    def start(self, component: "Component", ctx: StepContext):
        """
        Start the monitoring process.

        This method initializes and starts the collection of monitoring data for the component.
        Args:
            component: The component instance to stop.
            ctx: The current execution context for the containing test step.
        """

    @abstractmethod
    def stop(self, component: "Component", ctx: StepContext):
        """
        Stop the monitoring process.

        This method shuts down any active monitoring and ensures data collection is concluded.
        Args:
            component: The component instance to stop.
            ctx: The current execution context for the containing test step.
        """

    @abstractmethod
    def collect(self, component: "Component", ctx: ScenarioContext) -> dict:
        """
        Collect and return monitoring data.

        This method aggregates and returns the collected monitoring data as a dictionary.

        Args:
            component: The component instance to stop.
            ctx: The current execution context for the containing test step.

        Returns:
            dict: A dictionary of collected monitoring data.
        """
