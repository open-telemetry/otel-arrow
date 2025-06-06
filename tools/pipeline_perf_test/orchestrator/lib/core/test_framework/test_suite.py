"""
Module: test_suite

This module defines the `TestSuite` class, which is responsible for managing and running a series of tests
on a set of components. It allows the execution of multiple tests in a sequence, providing the necessary
context and components to each test.

The `TestSuite` class organizes tests and components, managing their execution in a structured manner.
Each test is provided with a context that includes all the components needed for the test to run.

Classes:
    TestSuite: A class that manages a collection of tests and components, runs the tests, and provides
    context to each test.
"""

from typing import List, Dict, Optional, TYPE_CHECKING

from ...core.context.base import ExecutionStatus, BaseContext
from .test_definition import TestDefinition
from ..context.test_contexts import TestSuiteContext, TestExecutionContext
from .test_element import TestFrameworkElement
from ..context.test_element_hook_context import HookableTestPhase
from ..telemetry.telemetry_runtime import TelemetryRuntime

if TYPE_CHECKING:
    from ..component.component import Component


class TestSuite(TestFrameworkElement):
    """
    A test suite class for managing and executing a series of tests on a set of components.

    The `TestSuite` class is designed to execute a list of tests on a set of components. Each test is provided
    with a `TestExecutionContext` that contains all components, enabling the test to interact with and
    manipulate the components during execution.

    Attributes:
        tests (List[TestDefinition]): A list of test definitions that define the tests to run.
        components (Dict[str, Component]): A dictionary of components, indexed by their names.
        name str: The name for the test suite.
        telemetry_runtime Optional[TelemetryRuntime]: A runtime object that holds Opentelemetry trace/meter providers.

    Methods:
        run(): Executes all the tests in the test suite, providing each test with the necessary context.
    """

    def __init__(
        self,
        tests: List[TestDefinition],
        components: Dict[str, "Component"],
        name: Optional[str] = "TestSuite",
        telemetry_runtime: Optional[TelemetryRuntime] = None,
    ):
        """
        Initializes the test suite with a list of tests and a dictionary of components.

        This constructor sets up the test suite by storing the tests and components, and initializing
        a `Context` to manage the components during test execution.

        Args:
            tests (List[TestDefinition]): The list of test definitions to be executed in the test suite.
            components (Dict[str, Component]): A dictionary of components to be used in the tests.
        """
        super().__init__()
        self.name = name
        self.tests = tests
        self.components = components
        self.context = TestSuiteContext(name=self.name)
        self.set_runtime_data(TelemetryRuntime.type, telemetry_runtime)
        for k, v in components.items():
            self.context.add_component(k, v)

    def run(self, _ctx: Optional[BaseContext] = None) -> None:
        """
        Run all tests in the test suite.

        This method iterates through the list of tests and runs each one, passing the context object
        to each test. The context provides access to the components, allowing the test to interact
        with them as needed.

        Args:
            _ctx: unused context object, defaults to None.
        """
        self.context.test_suite = self
        logger = self.context.get_logger(__name__)
        with self.context:
            self._run_hooks(HookableTestPhase.PRE_RUN, self.context)
            for test_definition in self.tests:
                test_execution_context = TestExecutionContext(
                    name=test_definition.name,
                    test_definition=test_definition,
                    parent_ctx=self.context,
                )
                self.context.add_child_ctx(test_execution_context)
                with test_execution_context:
                    logger.info("Starting Test: %s", test_definition.name)
                    try:
                        test_definition.run(test_execution_context)
                        if test_execution_context.status == ExecutionStatus.RUNNING:
                            test_execution_context.status = ExecutionStatus.SUCCESS
                    except Exception as e:
                        test_execution_context.status = ExecutionStatus.ERROR
                        test_execution_context.error = e
                        logger.error("Test %s failed %s", test_definition.name, e)
                        raise

                logger.debug(
                    "Test %s finished. Running reports...", test_definition.name
                )
                # Report using all defined reporting strategies
                test_data = test_definition.get_test_data(test_execution_context)
                for strategy in test_definition.reporting_strategies:
                    strategy.report(test_data)
            self._run_hooks(HookableTestPhase.POST_RUN, self.context)
            self.context.status = ExecutionStatus.SUCCESS
