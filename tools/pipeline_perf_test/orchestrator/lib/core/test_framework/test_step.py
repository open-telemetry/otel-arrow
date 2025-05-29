"""
Module: test_step

This module defines the `TestStep` class, which represents a single step in a test execution sequence.
Each test step has a name and an associated action (a callable function) that is executed when the step is run.

The `TestStep` class is designed to be used within the broader context of a test definition, where multiple
test steps are executed in sequence to complete a full test.

Classes:
    TestStep: A class representing a single step in a test, which includes a name and an action to execute.
"""

from typing import Any, Callable, Optional, TYPE_CHECKING

from ..context.test_contexts import TestStepContext


if TYPE_CHECKING:
    from ..component.lifecycle_component import LifecycleComponent


class TestStep:
    """
    Represents a single step in a test execution sequence.

    A test step consists of a name and an associated action, which is a callable function that is executed
    when the step is run. Test steps are typically used in a sequence to build up the logic for a complete test.

    Attributes:
        name (str): The name of the test step.
        action (Callable): The action (a callable function) to be executed for this test step.

    Methods:
        run(context): Executes the action associated with the test step, providing the context to the action.
    """

    def __init__(
        self,
        name: str,
        action: Callable[[TestStepContext], Any],
        component: Optional["LifecycleComponent"] = None,
    ):
        """
        Initializes a test step with a name and an associated action.

        Args:
            name (str): The name of the test step.
            action (Callable[[TestStepContext], any]): A callable function that defines the action to execute when the test step is run.
        """
        self.name = name
        self.action = action
        self.component = component

    def run(self, context: TestStepContext):
        """
        Executes the action associated with the test step.

        This method prints the name of the test step and then runs the action, passing the provided context
        to the action.

        Args:
            context (Context): The context containing data and components to be used during the step execution.

        Returns:
            The result of the action execution, which is whatever is returned by the action callable.
        """

        return self.action(context)
