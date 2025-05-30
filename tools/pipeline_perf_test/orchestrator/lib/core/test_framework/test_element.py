"""
Module: test_element

This module defines the `TestFrameworkElement` abstract base class and supporting
constructs that represent units of test execution within a test orchestration system.

The test framework element is the foundational abstraction for anything that can be
executed in a test such as a test case, test suite, or a test step. It supports
execution hooks that can be attached to various phases of the test lifecycle,
allowing users to inject behavior before or after execution (e.g., setup, validation,
cleanup).

The module also defines enums for lifecycle phases and mechanisms to run registered hooks.

Key Concepts:
    - Lifecycle Phases: Defined via `TestLifecyclePhase`, represent execution stages.
    - Hook Integration: Hooks conforming to `HookStrategy` can be attached and run dynamically.

Typical usage:
    - Subclass `TestFrameworkElement` and implement the `run` method.
    - Attach hooks using `add_hook` to customize behavior for different lifecycle phases.

Enums:
    TestLifecyclePhase: Represents core phases in a test element's execution.

Classes:
    TestFrameworkElement (ABC): Base class for all testable elements in the framework,
                                with support for lifecycle hook execution.
"""

from abc import ABC, abstractmethod
from enum import Enum
from collections import defaultdict
from typing import Optional, List, Dict, Callable, Any

from ..context.test_element_hook_context import (
    HookableTestPhase,
    TestElementHookContext,
)
from ..strategies.hook_strategy import HookStrategy
from ..context.base import BaseContext
from ..context.test_contexts import TestFrameworkElementContext
from ..context.base import ExecutionStatus
from ..runtime import Runtime


class TestLifecyclePhase(Enum):
    """
    Enum representing the various primary phases in the lifecycle of a TestFrameworkElement.

    These phases help manage the orchestration of components during test execution.

    Phases include:
        - RUN        (Run the test suite/definition/step)
    """

    RUN = "run"


class TestFrameworkElement(ABC):
    """
    Abstract base class for test elements within the orchestrator.
    """

    def __init__(self) -> None:
        self._hooks: Dict[HookableTestPhase, List[HookStrategy]] = defaultdict(list)
        self.runtime: Runtime = Runtime()

    def add_hook(self, phase: HookableTestPhase, hook: HookStrategy) -> None:
        """Register hooks to trigger at the specified phase of the element.

        Args:
            phase: The hookable phase of the test element (e.g. pre_run, post_run)
            hook: The hook strategy to execute.
        """
        self._hooks[phase].append(hook)

    def _run_hooks(
        self, phase: HookableTestPhase, ctx: "TestFrameworkElementContext"
    ) -> None:
        """Run hooks for the specified phase with provided context.

        Args:
            phase: The hookable phase of the test element (e.g. pre_run, post_run)
            ctx: The context for the current test element.
        """
        for hook in self._hooks.get(phase, []):
            hook_context = TestElementHookContext(
                phase=phase, name=f"{hook.__class__.__name__} ({phase.value})"
            )
            ctx.add_child_ctx(hook_context)
            try:
                hook_context.start()
                hook.execute(hook_context)
                if hook_context.status == ExecutionStatus.RUNNING:
                    hook_context.status = ExecutionStatus.SUCCESS
            except Exception as e:  # pylint: disable=broad-except
                hook_context.status = ExecutionStatus.ERROR
                hook_context.error = e
                hook_context.log(f"Hook failed: {e}")
                break
            finally:
                hook_context.end()

    def get_or_create_runtime(self, namespace: str, factory: Callable[[], Any]) -> Any:
        """Get an existing runtime data structure or initialize a new one.

        Args:
            namespace: The namespace to get/create data for.
            factory: The initialization method if no namespace data exists.
        """
        return self.runtime.get_or_create(namespace, factory)

    def set_runtime_data(self, namespace: str, data: Any):
        """Set the data value on the component's runtime with the specified namespace.

        Args:
            namespace: The namespace to set the data value on.
            data: The data to set.
        """
        self.runtime.set(namespace, data)

    @abstractmethod
    def run(self, ctx: Optional[BaseContext] = None):
        """Run the test element logic."""
