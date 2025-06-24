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
    ExecutionStrategyConfig(BaseModel):  Base model for Execution Strategy config.
    ExecutionStrategy (ABC): Abstract interface for starting and stopping a component's workload execution.
"""

from abc import abstractmethod
from typing import TYPE_CHECKING

from ..context.framework_element_contexts import StepContext
from .base import BaseStrategyConfig, BaseStrategy


if TYPE_CHECKING:
    from ..component.component import Component


class ExecutionStrategyConfig(BaseStrategyConfig):
    """Base model for Execution Strategy config, passed to strategy init."""


class ExecutionStrategy(BaseStrategy):
    """
    Abstract base class for execution strategies.

    Execution strategies define how a component behaves when it is running.
    This interface is responsible for triggering and controlling runtime execution
    of a component's main workload or behavior.

    Methods:
        start(component, ctx): Begin execution of the component's workload.
        stop(component, ctx): Stop execution of the component's workload.
    """

    @abstractmethod
    def __init__(self, config: ExecutionStrategyConfig) -> None:
        """All execution strategies must be initialized with a config object."""

    @abstractmethod
    def start(self, component: "Component", ctx: StepContext):
        """
        Start executing the component's workload.

        Args:
            component: The component instance to execute.
            ctx: The current execution context for the containing test step.
        """

    @abstractmethod
    def stop(self, component: "Component", ctx: StepContext):
        """
        Stop executing the component's workload.

        Args:
            component: The component instance to stop.
            ctx: The current execution context for the containing test step.
        """
