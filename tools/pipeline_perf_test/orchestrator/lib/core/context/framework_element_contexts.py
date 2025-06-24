"""
Module: framework_element_contexts

This module defines the core context classes used for managing state and shared data throughout
the lifecycle of a test suite execution. These context classes enable structured access to components
and metadata at different granularities: suite-wide, per-test, and per-step.

Classes:
    SuiteContext:
        Holds global state for a test suite run, including shared components and metadata.
        Provides methods to register and retrieve components by name.

    ScenarioContext:
        Represents the execution context for a single Scenario.
        Tracks execution metadata (timing, status, errors) and maintains access to the suite-level context.
        Allows per-test interaction with shared components.

    StepContext:
        Represents the execution context for an individual test step within a test.
        Provides access to the parent test and suite contexts, as well as step-level status and metadata.

Each context layer provides helper methods to retrieve components by name, ensuring consistent
access across the execution lifecycle.
"""

from __future__ import annotations
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Dict, List, Optional, TYPE_CHECKING

from .base import BaseContext
from ..telemetry.framework_event import FrameworkEvent


if TYPE_CHECKING:
    from ..framework.scenario import Scenario
    from ..framework.step import Step
    from ..framework.suite import Suite
    from ..framework.element import FrameworkElement
    from ..component.component import Component, ComponentHookContext


class FrameworkElementContext(BaseContext, ABC):
    """
    Abstract base class for all test framework element contexts.
    Used to represent the polymorphic parent/child relationship between
    SuiteContext, ScenarioContext, and StepContext.
    """

    @abstractmethod
    def get_framework_element(self) -> Optional["FrameworkElement"]:
        """Get the FrameworkElement associated with the context if any."""

    def get_suite(self) -> Optional["Suite"]:
        """Get the root test suite object from a given context."""
        if self.parent_ctx:
            return self.parent_ctx.get_suite()
        raise RuntimeError(
            "SuiteContext.suite must be set to access the root test suite."
        )

    def get_component_by_name(self, name: str) -> Optional["Component"]:
        """Get a component instance from the test suite context by the name of the component.

        Args:
            name: The name of the component to return

        Returns: The named component or none if not found.
        """
        if self.parent_ctx:
            return self.parent_ctx.get_component_by_name(name)
        raise RuntimeError(
            f"Can't get parent context fetching component by name: {name}"
        )


@dataclass
class SuiteContext(FrameworkElementContext):
    """
    Holds global state for a test suite run, including all shared components.
    """

    components: Dict[str, Component] = field(default_factory=dict)
    child_contexts: List["ScenarioContext"] = field(default_factory=list)
    suite: Optional["Suite"] = None

    def __post_init__(self):
        """
        Performs additional initialization after object creation.
        """
        self.start_event_type = FrameworkEvent.SUITE_START
        self.end_event_type = FrameworkEvent.SUITE_END
        self.metadata["test.suite"] = self.name
        self.span_name = f"Run Test Suite: {self.name}"

    def get_framework_element(self) -> Optional["FrameworkElement"]:
        """Test Suite has no direct FrameworkElement, return None."""
        return self.suite

    def add_component(self, name: str, component: Component):
        "Add a component to the test suite context by name"
        self.components[name] = component

    def get_components(self) -> Dict[str, "Component"]:
        """Get all components on the context indexed by name.

        Returns: a dict of component names to component instances.
        """
        return self.components

    def get_component_by_name(self, name: str) -> Optional["Component"]:
        """Get a component instance by the name of the component.

        Args:
            name: The name of the component to return

        Returns: The named component or none if not found.
        """
        return self.components.get(name)

    def get_suite(self) -> Optional["Suite"]:
        """Get the root test suite object from a given context."""
        if self.suite:
            return self.suite
        raise RuntimeError(
            "SuiteContext.suite must be set to access the root test suite."
        )


@dataclass
class ScenarioContext(FrameworkElementContext):
    """
    Context for executing a single Scenario, scoped per test.
    """

    parent_ctx: Optional["SuiteContext"] = None
    child_contexts: List["StepContext"] = field(default_factory=list)
    scenario_definition: Optional["Scenario"] = None

    def __post_init__(self):
        """
        Performs additional initialization after object creation.
        """
        super().__post_init__()
        self.start_event_type = FrameworkEvent.TEST_START
        self.end_event_type = FrameworkEvent.TEST_END
        if self.parent_ctx:
            merged_metadata = {**self.parent_ctx.metadata, **self.metadata}
            self.metadata = merged_metadata
        self.metadata["test.name"] = self.scenario_definition.name
        self.span_name = f"Run Test: {self.scenario_definition.name}"

    def get_framework_element(self) -> Optional["FrameworkElement"]:
        """Get the Scenario associated with this execution context."""
        return self.scenario_definition


@dataclass
class StepContext(FrameworkElementContext):
    """
    Context for an individual test step execution.
    """

    parent_ctx: Optional["ScenarioContext"] = None
    child_contexts: List["ComponentHookContext"] = field(default_factory=list)
    step: "Step" = None

    def __post_init__(self):
        """
        Performs additional initialization after object creation.
        """
        super().__post_init__()
        self.start_event_type = FrameworkEvent.STEP_START
        self.end_event_type = FrameworkEvent.STEP_END
        if self.parent_ctx:
            merged_metadata = {**self.parent_ctx.metadata, **self.metadata}
            self.metadata = merged_metadata
        self.metadata["test.step"] = self.step.name
        self.span_name = f"Run Test Step: {self.step.name}"

    def get_step_component(self) -> Optional["Component"]:
        "Get the component targeted for this step (if applicable)."
        return self.step.component

    def set_step_component(self, component: "Component"):
        """
        Set the component targeted for this step.

        Args:
            component: The Component associated with the step.
        """
        self.step.set_component(component)

    def get_framework_element(self) -> Optional["FrameworkElement"]:
        """Get the Step associated with this context."""
        return self.step
