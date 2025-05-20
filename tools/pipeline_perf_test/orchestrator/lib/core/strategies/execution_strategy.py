"""
Module: execution_strategy

This module defines the `ExecutionStrategy` abstract base class, which provides a unified
interface for controlling the execution behavior of testbed components in a load generation
environment.

Execution strategies encapsulate how a component runs or behaves at runtime. This separation
of concerns allows different execution modes to be applied to the same deployment and configuration
mechanisms.

Typical implementations of this interface include:
    - GenerateLoad: Starts a component that actively generates load (e.g., otlp, arrow, etc).
    - ReceiveLoad: Starts a component that passively receives load (e.g., validating otlp receiver).

Classes:
    ExecutionStrategy (ABC): Abstract interface for starting and stopping a component’s workload execution.
"""

from abc import ABC, abstractmethod


class ExecutionStrategy(ABC):
    """
    Abstract base class for execution strategies.

    Execution strategies define how a component behaves when it is running.
    This interface is responsible for triggering and controlling runtime execution
    of a component’s main workload or behavior.

    Methods:
        start(component): Begin execution of the component’s workload.
        stop(component): Stop execution of the component’s workload.
    """

    @abstractmethod
    def start(self, component):
        """
        Start executing the component's workload.

        Args:
            component: The component instance to execute.
        """

    @abstractmethod
    def stop(self, component):
        """
        Stop executing the component's workload.

        Args:
            component: The component instance to stop.
        """
