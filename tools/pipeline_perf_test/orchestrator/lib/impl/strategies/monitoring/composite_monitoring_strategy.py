"""
This module defines the CompositeMonitoringStrategy class, which allows combining
multiple monitoring strategies into a unified interface.

The CompositeMonitoringStrategy class enables flexible and modular monitoring by
delegating lifecycle methods (`start`, `stop`, and `collect`) to a collection of
individual monitoring strategies. This pattern is especially useful when a component
needs to be monitored through multiple data sources (e.g., process-level stats,
Docker metrics, custom probes) simultaneously.

Usage:
    - Instantiate with a list of concrete MonitoringStrategy implementations.
    - Call `start`, `stop`, and `collect` as you would with a single strategy.
"""
from typing import List

from ....core.component.component import Component
from ....core.strategies.monitoring_strategy import MonitoringStrategy
from ....core.context.framework_element_contexts import StepContext, ScenarioContext


class CompositeMonitoringStrategy(MonitoringStrategy):
    """
    Composite Monitoring Strategy that aggregates multiple individual monitoring strategies.

    This class combines a list of monitoring strategies and delegates the start, stop, and collect
    operations to all the included strategies. It effectively consolidates results from multiple sources
    into a single set of monitoring data.

    Methods:
        start(): Starts all monitoring strategies.
        stop(): Stops all monitoring strategies.
        collect(): Collects and returns combined monitoring data from all strategies.
    """

    def __init__(self, strategies: List[MonitoringStrategy]):
        """
        Initializes the CompositeMonitoringStrategy with a list of individual monitoring strategies.

        Args:
            strategies (List[MonitoringStrategy]): A list of monitoring strategy instances to be combined.
        """
        self.strategies = strategies

    def start(self, component: Component, ctx: StepContext):
        """
        Start all monitoring strategies.

        This method iterates over each individual strategy and calls its `start()` method.

        Args:
            component: The component which is running this strategy.
            ctx: The test execution context for the step where this is invoked.
        """
        for strategy in self.strategies:
            strategy.start(component, ctx)

    def stop(self, component: Component, ctx: StepContext):
        """
        Stop all monitoring strategies.

        This method iterates over each individual strategy and calls its `stop()` method.

        Args:
            component: The component which is running this strategy.
            ctx: The test execution context for the step where this is invoked.
        """
        for strategy in self.strategies:
            strategy.stop(component, ctx)

    def collect(self, component: Component, ctx: ScenarioContext) -> dict:
        """
        Collect combined monitoring data from all strategies.

        This method aggregates the data collected from each individual monitoring strategy and
        returns a dictionary containing the combined results.

        Args:
            component: The component which is running this strategy.
            ctx: The test execution context for the step where this is invoked.

        Returns:
            dict: The combined monitoring data from all strategies.
        """
        results = {}
        for strategy in self.strategies:
            res = strategy.collect(component, ctx)
            if res:
                results.update(res)  # Combine results from all strategies
        return results
