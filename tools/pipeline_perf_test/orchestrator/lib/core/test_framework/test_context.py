"""
Module: test_context

This module defines the core context classes used for managing state and shared data throughout
the lifecycle of a test suite execution. These context classes enable structured access to components
and metadata at different granularities: suite-wide, per-test, and per-step.

Classes:
    TestSuiteContext:
        Holds global state for a test suite run, including shared components and metadata.
        Provides methods to register and retrieve components by name.

    TestExecutionContext:
        Represents the execution context for a single TestDefinition.
        Tracks execution metadata (timing, status, errors) and maintains access to the suite-level context.
        Allows per-test interaction with shared components.

    TestStepContext:
        Represents the execution context for an individual test step within a test.
        Provides access to the parent test and suite contexts, as well as step-level status and metadata.

Each context layer provides helper methods to retrieve components by name, ensuring consistent
access across the execution lifecycle.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Dict, List, Optional, TYPE_CHECKING

from ..context.base import BaseContext

if TYPE_CHECKING:
    from .test_definition import TestDefinition
    from .test_step import TestStep
    from ..component.lifecycle_component import LifecycleComponent, LifecycleHookContext


@dataclass
class TestSuiteContext(BaseContext):
    """
    Holds global state for a test suite run, including all shared components.
    """

    components: Dict[str, LifecycleComponent] = field(default_factory=dict)
    clients: Dict[str, object] = field(default_factory=dict)
    child_contexts: List["TestExecutionContext"] = field(default_factory=list)

    def add_component(self, name: str, component: LifecycleComponent):
        "Add a component to the test suite context by name"
        self.components[name] = component

    def get_components(self) -> Dict[str, "LifecycleComponent"]:
        """Get all components on the context indexed by name.

        Returns: a dict of component names to component instances.
        """
        return self.components

    def get_component_by_name(self, name: str) -> Optional["LifecycleComponent"]:
        """Get a component instance by the name of the component.

        Args:
            name: The name of the component to return

        Returns: The named component or none if not found.
        """
        return self.components.get(name)

    def get_client(
        self, name: str, client_factory: Optional[callable] = None
    ) -> object:
        """
        Retrieve a client from the context, creating it if it does not exist.

        name: The name of the client.
        client_factory: A factory function to create the client if it does not exist.

        return: The client object.
        """
        client = self.clients.get(name)
        if not client:
            return client_factory()
        return client


@dataclass
class TestExecutionContext(BaseContext):
    """
    Context for executing a single TestDefinition, scoped per test.
    """

    parent_ctx: Optional["TestSuiteContext"] = None
    child_contexts: List["TestStepContext"] = field(default_factory=list)
    test_definition: Optional["TestDefinition"] = None

    def get_test_definition(self) -> Optional["TestDefinition"]:
        """Get the Test Definition on which this context is operating."""
        return self.test_definition


@dataclass
class TestStepContext(BaseContext):
    """
    Context for an individual test step execution.
    """

    parent_ctx: Optional["TestExecutionContext"] = None
    child_contexts: List["LifecycleHookContext"] = field(default_factory=list)
    step: Optional["TestStep"] = None

    def get_step_component(self) -> Optional["LifecycleComponent"]:
        "Get the component targeted for this step (if applicable)."
        return self.step.component

    def get_step(self) -> Optional["TestStep"]:
        """Get the Test Step on which this context is operating."""
        return self.step
