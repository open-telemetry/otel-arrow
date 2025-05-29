"""
Module: test_definition

This module defines the `TestDefinition` class, which encapsulates a test scenario in a load generation
testbed. The `TestDefinition` class allows for defining a sequence of test steps, executing them, and
reporting the results through specified reporting strategies.

The class provides the functionality to aggregate monitoring data during the test execution and apply
different reporting strategies to the results.

Classes:
    TestDefinition: A class that defines a test, including the steps to run and the reporting strategies to use.
"""

from typing import List, Optional, TYPE_CHECKING

from ..component.component_data import ComponentData
from ..context.base import ExecutionStatus
from ..context.test_contexts import TestExecutionContext, TestStepContext
from .test_data import TestData
from .test_step import TestStep

if TYPE_CHECKING:
    from ..strategies.reporting_strategy import ReportingStrategy


class TestDefinition:
    """
    A class that defines a test scenario, including its steps and reporting strategies.

    The `TestDefinition` class encapsulates the details of a test, including the steps to execute and
    the reporting strategies to use after the test has run. It provides the functionality to run the test,
    aggregate monitoring data from components, and report the results using different strategies.

    Attributes:
        name (str): The name of the test.
        steps (List[TestStep]): A list of test steps to be executed in sequence.
        reporting_strategies (List[ReportingStrategy]): A list of reporting strategies used to report results.

    Methods:
        run(context): Executes the test, runs the test steps, and reports results using the specified strategies.
        aggregate_monitoring_data(context): Collects and aggregates monitoring data from all components.
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
        self.name = name
        self.steps = steps
        self.reporting_strategies = reporting_strategies or []
        self.context = None

    def run(self, context: TestExecutionContext):
        """
        Executes the test by running all its steps and then reporting the results.

        This method prints the test name, executes each test step in sequence, and collects aggregated monitoring
        data from all components. Afterward, it reports the aggregated results using the provided reporting strategies.

        Args:
            context (TestExecutionContext): The context that contains all the components and relevant data for the test.
        """
        print(f"Running test: {self.name}")
        for step in self.steps:
            step_ctx = TestStepContext(name=step.name, step=step)
            context.add_child_ctx(step_ctx)
            try:
                step_ctx.start()
                step.run(step_ctx)
                if step_ctx.status == ExecutionStatus.RUNNING:
                    step_ctx.status = ExecutionStatus.SUCCESS
            except Exception as e:
                step_ctx.status = ExecutionStatus.ERROR
                step_ctx.error = e
                step_ctx.log(f"Step '{step.name}' failed: {e}")
                # TODO: Depending on policy: raise or continue
                raise
            finally:
                step_ctx.end()

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
