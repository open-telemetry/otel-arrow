"""
Module: suite

This module defines the `Suite` class, which is responsible for managing and running a series of tests
on a set of components. It allows the execution of multiple tests in a sequence, providing the necessary
context and components to each test.

The `Suite` class organizes tests and components, managing their execution in a structured manner.
Each test is provided with a context that includes all the components needed for the test to run.

Classes:
    Suite: A class that manages a collection of tests and components, runs the tests, and provides
    context to each test.
"""

from typing import List, Dict, Optional, TYPE_CHECKING

from ..context.base import ExecutionStatus, BaseContext
from .scenario import Scenario
from ..context.framework_element_contexts import SuiteContext, ScenarioContext
from .element import FrameworkElement
from ..context.framework_element_hook_context import HookableTestPhase
from ..telemetry.telemetry_runtime import TelemetryRuntime

if TYPE_CHECKING:
    from ..component.component import Component


class Suite(FrameworkElement):
    """
    A test suite class for managing and executing a series of tests on a set of components.

    The `Suite` class is designed to execute a list of tests on a set of components. Each test is provided
    with a `TestExecutionContext` that contains all components, enabling the test to interact with and
    manipulate the components during execution.

    Attributes:
        tests (List[Scenario]): A list of test definitions that define the tests to run.
        components (Dict[str, Component]): A dictionary of components, indexed by their names.
        name str: The name for the test suite.
        telemetry_runtime Optional[TelemetryRuntime]: A runtime object that holds Opentelemetry trace/meter providers.

    Methods:
        run(): Executes all the tests in the test suite, providing each test with the necessary context.
    """

    def __init__(
        self,
        tests: List[Scenario],
        components: Dict[str, "Component"],
        name: Optional[str] = "Suite",
        telemetry_runtime: Optional[TelemetryRuntime] = None,
    ):
        """
        Initializes the test suite with a list of tests and a dictionary of components.

        This constructor sets up the test suite by storing the tests and components, and initializing
        a `Context` to manage the components during test execution.

        Args:
            tests (List[Scenario]): The list of test definitions to be executed in the test suite.
            components (Dict[str, Component]): A dictionary of components to be used in the tests.
        """
        super().__init__()
        self.name = name
        self.tests = tests
        self.components = components
        self.context = SuiteContext(name=self.name)
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
        self.context.suite = self
        logger = self.context.get_logger(__name__)
        with self.context:
            # PRE_RUN runs outside the try/finally below: if a pre hook (e.g.
            # starting a receiver) fails, there is nothing for POST_RUN to tear
            # down, and the post hooks themselves assume their paired pre hook
            # ran successfully.
            self._run_hooks(HookableTestPhase.PRE_RUN, self.context)
            # Once the pre hooks have run, POST_RUN must always run so suite
            # teardown (e.g. stopping a receiver started in PRE_RUN) is not
            # skipped when a test raises. The try/finally guarantees this; the
            # finally re-raises any test error in preference to a cleanup error.
            test_error: Optional[BaseException] = None
            try:
                for test_definition in self.tests:
                    test_execution_context = ScenarioContext(
                        name=test_definition.name,
                        scenario_definition=test_definition,
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
                self.context.status = ExecutionStatus.SUCCESS
            except BaseException as e:  # noqa: BLE001 (re-raised below)
                test_error = e
                logger.error("test_error: %s", test_error)
                raise
            finally:
                try:
                    self._run_hooks(HookableTestPhase.POST_RUN, self.context)
                except Exception as post_error:
                        logger.error("POST_RUN hooks failed: post: %s", test_error)
                        raise
