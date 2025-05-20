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
import time
from typing import List, Dict

from .test_context import TestExecutionContext, TestStepContext
from .test_step import TestStep
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
    def __init__(self, name: str, steps: List[TestStep], reporting_strategies: List[ReportingStrategy] = None):
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
        context.start_time = time.time()
        for step in self.steps:
            step_ctx = TestStepContext(
                step=step,
                test_definition=self,
                test_context=context,
            )
            try:
                step_ctx.start_time = time.time()
                step.run(step_ctx)
                step_ctx.status = "success"
            except Exception as e:
                step_ctx.status = "error"
                step_ctx.error = e
                print(f"Step '{step.name}' failed: {e}")
                #TODO: Depending on policy: break or continue
                break
            finally:
                step_ctx.end_time = time.time()

        context.step_contexts.append(step_ctx)
        context.end_time = time.time()

        # Report using all reporting strategies
        aggregated_data = self.aggregate_monitoring_data(context)
        for strategy in self.reporting_strategies:
            strategy.report(aggregated_data)

    def aggregate_monitoring_data(self, context) -> Dict[str, Dict[str, any]]:
        """
        Aggregates monitoring data from all components in the context.

        This method collects monitoring data from each component in the provided context and combines it
        into a single dictionary, where the keys are component names and the values are the respective
        monitoring data.

        Args:
            context (TestExecutionContext): The context containing all components

        Returns:
            Dict[str, Dict[str, any]]: A dictionary of aggregated monitoring data, indexed by component name.
        """
        aggregated_data = {}

        # Collect data from each component
        for component_name, component in context.suite_context.components.items():
            # Collect the monitoring data from the component
            collected_data = component.collect_monitoring_data()

            # Aggregate it by component name
            aggregated_data[component_name] = collected_data

        return aggregated_data
