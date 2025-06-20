"""
Module: test_step

This module defines the `TestStep` class, which represents a single step in a test execution sequence.
Each test step has a name and an associated action (a callable function) that is executed when the step is run.

The `TestStep` class is designed to be used within the broader context of a test definition, where multiple
test steps are executed in sequence to complete a full test.

Classes:
    TestStep: A class representing a single step in a test, which includes a name and an action to execute.
"""

from abc import abstractmethod, ABC
from typing import Optional, TYPE_CHECKING

from pydantic import BaseModel

from ..context.base import BaseContext
from ..context.test_contexts import TestStepContext
from ..context.test_element_hook_context import HookableTestPhase
from ..telemetry.test_event import TestEvent
from .test_element import TestFrameworkElement

if TYPE_CHECKING:
    from ..component.component import Component


class TestStepActionConfig(BaseModel):
    """Base model for Test Step Action configs, passed to TestStepAction init."""


class TestStepAction(ABC):
    """
    Abstract base class representing a test step action.

    This class defines the interface for all test step actions that can be
    performed in a testing framework. Each action must be initialized with
    a configuration object and must implement an execution method that takes
    the current test step context.

    Subclasses are required to implement the __init__ and execute methods.
    """

    @abstractmethod
    def __init__(self, config: TestStepActionConfig) -> None:
        """All test step actions must be initialized with a config object."""

    @abstractmethod
    def execute(self, ctx: TestStepContext) -> None:
        """Execute the step action and pass it the current context."""


class TestStep(TestFrameworkElement):
    """
    Represents a single step in a test execution sequence.

    A test step consists of a name and an associated action, which is a callable function that is executed
    when the step is run. Test steps are typically used in a sequence to build up the logic for a complete test.

    Attributes:
        name (str): The name of the test step.
        action (TestStepAction): The action to be executed for this test step.

    Methods:
        run(context): Executes the action associated with the test step, providing the context to the action.
    """

    def __init__(
        self,
        name: str,
        action: TestStepAction,
        component: Optional["Component"] = None,
    ):
        """
        Initializes a test step with a name and an associated action.

        Args:
            name (str): The name of the test step.
            action (Callable[[TestStepContext], any]): A callable function that defines the action to execute when the test step is run.
            component: Optional component associated with the test step.
        """
        super().__init__()
        self.name = name
        self.action = action
        self.component = component

    def set_component(self, component):
        """
        Allows test actions to set the step target component at runtime.

        Args:
            component: The Component that is the target of a given step action.
        """
        self.component = component

    def run(self, ctx: Optional[BaseContext] = None) -> None:
        """
        Executes the action associated with the test step.

        This method prints the name of the test step and then runs the action, passing the provided context
        to the action.

        Args:
            context (Context): The context containing data and components to be used during the step execution.

        Returns:
            The result of the action execution, which is whatever is returned by the action callable.
        """
        assert isinstance(ctx, TestStepContext), "Expected TestExecutionContext"

        logger = ctx.get_logger(__name__)
        self._run_hooks(HookableTestPhase.PRE_RUN, ctx)
        ctx.record_event(TestEvent.STEP_EXECUTE_START.namespaced())
        logger.info("Executing main step action...")
        self.action.execute(ctx)
        logger.debug("Main step action complete...")
        ctx.record_event(TestEvent.STEP_EXECUTE_END.namespaced())
        self._run_hooks(HookableTestPhase.POST_RUN, ctx)
