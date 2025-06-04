"""
Module: test_definition

This module defines the `TestDefinition` class, which encapsulates a test scenario in a
testbed. The `TestDefinition` class allows for defining a sequence of test steps, executing them, and
reporting the results through specified reporting strategies.

The class provides the functionality to aggregate monitoring data during the test execution and apply
different reporting strategies to the results.

Classes:
    TestDefinition: A class that defines a test, including the steps to run and the reporting strategies to use.
"""

from typing import List, Optional, TYPE_CHECKING

from ..component.component_data import ComponentData
from ..context.base import ExecutionStatus, BaseContext
from ..context.test_contexts import TestExecutionContext, TestStepContext
from .test_data import TestData
from .test_step import TestStep
from .test_element import TestFrameworkElement
from ..context.test_element_hook_context import HookableTestPhase

if TYPE_CHECKING:
    from ..strategies.reporting_strategy import ReportingStrategy


class TestDefinition(TestFrameworkElement):
    """
    Defines a test scenario composed of sequential steps and reporting strategies.

    Attributes:
        name (str): The test's name.
        steps (List[TestStep]): Ordered list of test steps.
        reporting_strategies (List[ReportingStrategy]): Strategies for reporting test results.

    Methods:
        run(ctx): Executes all steps and applies reporting strategies.
        get_test_data(ctx) Compile and return a TestData object from components and steps.
    """

    def __init__(
        self,
        name: str,
        steps: List[TestStep],
        reporting_strategies: Optional[List["ReportingStrategy"]] = None,
    ):
        """
        Initializes the test definition with the given name, steps, and reporting strategies.

        Args:
            name (str): The name of the test.
            steps (List[TestStep]): A list of test steps to be executed in sequence.
            reporting_strategies (List[ReportingStrategy], optional): A list of reporting strategies to apply
                                                                     after the test. Defaults to an empty list.
        """
        super().__init__()
        self.name = name
        self.steps = steps
        self.reporting_strategies = reporting_strategies or []

    def run(self, ctx: Optional[BaseContext] = None) -> None:
        """
        Runs the test by executing steps and any pre/post run hooks.

        Args:
            ctx (TestExecutionContext): The test execution context.
        """

        assert isinstance(ctx, TestExecutionContext), "Expected TestExecutionContext"
        self._run_hooks(HookableTestPhase.PRE_RUN, ctx)

        for step in self.steps:
            step_ctx = TestStepContext(name=step.name, step=step)
            ctx.add_child_ctx(step_ctx)

            try:
                step_ctx.start()
                step.run(step_ctx)
                if step_ctx.status == ExecutionStatus.RUNNING:
                    step_ctx.status = ExecutionStatus.SUCCESS
            except Exception as e:
                step_ctx.status = ExecutionStatus.ERROR
                step_ctx.error = e
                step_ctx.log(f"Step '{step.name}' failed: {e}")
                raise  # or continue, based on policy
            finally:
                step_ctx.end()

        self._run_hooks(HookableTestPhase.POST_RUN, ctx)

    def get_test_data(self, ctx: TestExecutionContext) -> TestData:
        """
        Aggregates test data, including the current test context and monitoring data from all components.

        Returns:
            TestData: A dictionary of aggregated monitoring data, indexed by component name along with the context
        """

        component_data = {}
        for component_name, component in ctx.get_components().items():
            component_data[component_name] = ComponentData.from_component(
                component, ctx
            )

        return TestData(context=ctx, component_data=component_data)
