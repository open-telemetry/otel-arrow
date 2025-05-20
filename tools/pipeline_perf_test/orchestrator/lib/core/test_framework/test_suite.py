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

from typing import List, Dict

from .test_definition import TestDefinition
from ..component.lifecycle_component import LifecycleComponent
from .test_context import TestSuiteContext, TestExecutionContext


class TestSuite:
    """
    A test suite class for managing and executing a series of tests on a set of components.

    The `TestSuite` class is designed to execute a list of tests on a set of components. Each test is provided
    with a `TestExecutionContext` that contains all components, enabling the test to interact with and
    manipulate the components during execution.

    Attributes:
        tests (List[TestDefinition]): A list of test definitions that define the tests to run.
        components (Dict[str, Component]): A dictionary of components, indexed by their names.

    Methods:
        run(): Executes all the tests in the test suite, providing each test with the necessary context.
    """

    def __init__(
        self, tests: List[TestDefinition], components: Dict[str, LifecycleComponent]
    ):
        """
        Initializes the test suite with a list of tests and a dictionary of components.

        This constructor sets up the test suite by storing the tests and components, and initializing
        a `Context` to manage the components during test execution.

        Args:
            tests (List[TestDefinition]): The list of test definitions to be executed in the test suite.
            components (Dict[str, Component]): A dictionary of components to be used in the tests.
        """
        self.tests = tests
        self.components = components
        self.context = TestSuiteContext()
        for k, v in components.items():
            self.context.add_component(k, v)

    def run(self):
        """
        Run all tests in the test suite.

        This method iterates through the list of tests and runs each one, passing the context object
        to each test. The context provides access to the components, allowing the test to interact
        with them as needed.
        """
        for test_definition in self.tests:
            test_execution_context = TestExecutionContext(
                test_definition=test_definition, suite_context=self.context
            )
            test_definition.run(test_execution_context)
