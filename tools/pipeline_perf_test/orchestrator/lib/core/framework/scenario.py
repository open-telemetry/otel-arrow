"""
Module: scenario

This module defines the `Scenario` class, which encapsulates a test scenario in a
testbed. The `Scenario` class allows for defining a sequence of test steps, with optional
hooks for modifying the behavior or reporting results.


Classes:
    Scenario: A class that defines a test, including the steps to run.
"""

from typing import List, Optional, TYPE_CHECKING

from ..context.base import ExecutionStatus
from ..context.framework_element_contexts import ScenarioContext, StepContext
from ..context.framework_element_hook_context import HookableTestPhase
from .element import FrameworkElement

if TYPE_CHECKING:
    from .step import Step
    from ..context.base import BaseContext


class Scenario(FrameworkElement):
    """
    Defines a test scenario composed of sequential steps.

    Attributes:
        name (str): The test's name.
        steps (List[Step]): Ordered list of test steps.

    Methods:
        run(ctx): Executes all steps.
    """

    def __init__(
        self,
        name: str,
        steps: List["Step"],
    ):
        """
        Initializes the test definition with the given name and steps.

        Args:
            name (str): The name of the test.
            steps (List[Step]): A list of test steps to be executed in sequence.
        """
        super().__init__()
        self.name = name
        self.steps = steps

    def run(self, ctx: Optional["BaseContext"] = None) -> None:
        """
        Runs the test by executing steps and any pre/post run hooks.

        Args:
            ctx (ScenarioContext): The test execution context.
        """

        logger = ctx.get_logger(__name__)
        assert isinstance(ctx, ScenarioContext), "Expected ScenarioContext"
        self._run_hooks(HookableTestPhase.PRE_RUN, ctx)

        logger.info("Running %d test steps...", len(self.steps))
        for step in self.steps:
            step_ctx = StepContext(name=step.name, step=step, parent_ctx=ctx)
            ctx.add_child_ctx(step_ctx)
            with step_ctx:
                try:
                    step.run(step_ctx)
                    if step_ctx.status == ExecutionStatus.RUNNING:
                        step_ctx.status = ExecutionStatus.SUCCESS
                except Exception as e:
                    step_ctx.status = ExecutionStatus.ERROR
                    step_ctx.error = e
                    logger.error("Error: Test Step: %s failed: %s", step.name, e)
                    raise  # or continue, based on policy

        self._run_hooks(HookableTestPhase.POST_RUN, ctx)
