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

from ..component.lifecycle_component import LifecycleComponent
from ..context.base import BaseContext

if TYPE_CHECKING:
    from .test_definition import TestDefinition
    from .test_step import TestStep


@dataclass
class TestSuiteContext(BaseContext):
    """
    Holds global state for a test suite run, including all shared components.
    """
    components: Dict[str, LifecycleComponent] = field(default_factory=dict)
    metadata: dict = field(default_factory=dict)

    def add_component(self, name: str, component: LifecycleComponent):
        "Add a component to the test suite context by name"
        self.components[name] = component

    def get_component(self, name: str) -> Optional[LifecycleComponent]:
        "Cat a component from the test suite context by name"
        return self.components.get(name)


@dataclass
class TestExecutionContext(BaseContext):
    """
    Context for executing a single TestDefinition, scoped per test.
    """
    test_definition: "TestDefinition"
    suite_context: "TestSuiteContext"
    step_contexts: List["TestStepContext"] = field(default_factory=list)

    def get_component(self, name: str) -> Optional[LifecycleComponent]:
        "Cat a component from the test suite context by name"
        return self.suite_context.get_component(name)


@dataclass
class TestStepContext(BaseContext):
    """
    Context for an individual test step execution.
    """
    step: "TestStep"
    test_definition: "TestDefinition"
    test_context: "TestExecutionContext"

    def get_component(self, name: str) -> Optional[LifecycleComponent]:
        "Cat a component from the test suite context by name"
        return self.test_context.suite_context.get_component(name)
