"""
test_config.py

Configuration Models for Test Suite, Test Definitions, and Test Steps

This module contains configuration models used to define and manage the structure
of a test suite, its individual tests, and the steps involved in each test. These
models are designed using Pydantic's `BaseModel` to provide validation and parsing
of test configurations.

The following models are included:

1. `TestStepConfig`: Represents the configuration for an individual test step.
   - Defines the name, action, and hooks associated with each test step.
   - Allows validation and parsing of the action section, ensuring correct action type and configuration.

2. `TestDefinitionConfig`: Represents the configuration for a full test within a suite.
   - Defines the test name, the list of steps involved, the reporting strategies, and lifecycle hooks.
   - Parses and validates reporting strategies, ensuring each one is correctly configured.

3. `TestSuiteConfig`: Represents the overall configuration for a test suite.
   - Defines components, tests, and hooks for the entire test suite.
   - Allows grouping of reusable components and sets up lifecycle hooks for the suite.

"""

from typing import Dict, List, Optional, Any
from pydantic import BaseModel, model_validator, Field

from ...impl.component.managed_component import (
    ManagedComponentConfiguration,
)
from ..registry import reporting_registry, test_step_action_registry
from ..wrappers import ReportingWrapper, TestStepActionWrapper
from ...core.test_framework.test_element import TestLifecyclePhase
from .hook_config import HooksConfig


class TestStepConfig(BaseModel):
    """
    Represents the configuration for a single test step in a test framework.

    Attributes:
        name (str): The name of the test step.
        action (TestStepActionWrapper): The action config to be performed during this test step, wrapped via TestStepActionWrapper.
        hooks (Dict[TestLifecyclePhase, HooksConfig]): A dictionary of hooks that are applied at different points in the test lifecycle. The keys are lifecycle phases, and the values are the corresponding hook configurations.

    Methods:
        parse_action_section(cls, data: Any) -> Any:
            A class method that validates and processes the 'action' section of the test step configuration. If the 'action' is provided as a dictionary, it wraps the configuration into the appropriate action class, ensuring the action type is valid.

    Example:
        test_step_config = TestStepConfig(
            name="Step 1",
            action=TestStepActionWrapper(element_type="action_type", config=config_data),
            hooks={TestLifecyclePhase.START: start_hook_config}
        )
    """

    name: str
    action: TestStepActionWrapper
    hooks: Dict[TestLifecyclePhase, HooksConfig] = Field(default_factory=dict)

    @model_validator(mode="before")
    @classmethod
    def parse_action_section(cls, data: Any) -> Any:
        """
        A class method that validates and processes the 'action' section in the test step configuration.

        This method checks if the 'action' field exists in the input `data` and whether it is provided as a dictionary.
        If it is, the method iterates over the items in the dictionary, validates the action type,
        creates the appropriate configuration object, and wraps it into a `TestStepActionWrapper`.

        The wrapped action is then assigned back to the 'action' key in the `data` dictionary.

        Args:
            data (Any): The input data for the test step configuration, typically a dictionary.

        Returns:
            Any: The processed data, with the 'action' field updated to contain the wrapped action configuration.

        Raises:
            ValueError: If an unknown action type is encountered during processing.
        """
        action = data.get("action")

        # If 'action' exists and is a dictionary, process it.
        if action and isinstance(action, dict):
            for action_type, config_data in action.items():
                # Look up the corresponding configuration class for the action type.
                config_cls = test_step_action_registry.config.get(action_type)

                # If no configuration class is registered, raise an error.
                if not config_cls:
                    raise ValueError(f"Unknown test step action type: '{action_type}'")

                # Instantiate the configuration class with the provided data.
                action_config = config_cls(**config_data)

                # Wrap the action configuration into a TestStepActionWrapper.
                wrapped = TestStepActionWrapper(
                    element_type=action_type, config=action_config
                )

            # Update the 'action' field in the input data with the wrapped configuration.
            data["action"] = wrapped
        return data


class TestDefinitionConfig(BaseModel):
    """
    Base configuration model for defining a test, including its steps, reporting, and hooks.

    Attributes:
        name (str): The name of the test definition.
        steps (Optional[List[TestStepConfig]]): A list of steps that define the sequence of actions for this test. Each step is configured via a `TestStepConfig`.
        reporting (Optional[Dict[str, ReportingWrapper]]): A dictionary of reporting strategies to be applied during the test. The keys are reporting strategy types, and the values are the corresponding `ReportingWrapper` configurations.
        hooks (Dict[TestLifecyclePhase, HooksConfig]): A dictionary of hooks applied at different phases of the test lifecycle. The keys represent lifecycle phases, and the values are the corresponding `HooksConfig`.

    Methods:
        parse_reporting_section(cls, data: Any) -> Any:
            A class method that validates and processes the 'reporting' section of the test definition configuration. If the 'reporting' field is provided as a dictionary, it wraps each reporting strategy into the appropriate wrapper class, ensuring the strategy type is valid.

    Example:
        test_def_config = TestDefinitionConfig(
            name="Test 1",
            steps=[step_1_config, step_2_config],
            reporting={"strategy_1": strategy_1_config, "strategy_2": strategy_2_config},
            hooks={TestLifecyclePhase.START: start_hook_config, TestLifecyclePhase.END: end_hook_config}
        )
    """

    name: str
    steps: Optional[List[TestStepConfig]]
    reporting: Optional[Dict[str, ReportingWrapper]]
    hooks: Dict[TestLifecyclePhase, HooksConfig] = Field(default_factory=dict)

    @model_validator(mode="before")
    @classmethod
    def parse_reporting_section(cls, data: Any) -> Any:
        """
        A class method that validates and processes the 'reporting' section in the test definition configuration.

        This method checks if the 'reporting' field exists in the input `data` and whether it is provided as a dictionary.
        If it is, the method iterates over the items in the dictionary, validates the reporting strategy type,
        creates the corresponding configuration object, and wraps it into a `ReportingWrapper`.

        The wrapped reporting strategies are then assigned back to the 'reporting' key in the `data` dictionary.

        Args:
            data (Any): The input data for the test definition configuration, typically a dictionary.

        Returns:
            Any: The processed data, with the 'reporting' field updated to contain the wrapped reporting strategies.

        Raises:
            ValueError: If an unknown reporting strategy type is encountered during processing.
        """
        reports = data.get("reporting")
        if reports and isinstance(reports, dict):
            parsed = {}
            for strategy_type, config_data in reports.items():
                # Look up the corresponding configuration class for the strategy type.
                config_cls = reporting_registry.config.get(strategy_type)

                # If no configuration class is registered for the strategy type, raise an error.
                if not config_cls:
                    raise ValueError(f"Unknown reporting strategy: '{strategy_type}'")

                # Instantiate the configuration class with the provided data for this reporting strategy.
                strategy_config = config_cls(**config_data)

                # Wrap the strategy configuration into a ReportingWrapper and store it in the parsed dictionary.
                parsed[strategy_type] = ReportingWrapper(
                    element_type=strategy_type, config=strategy_config
                )

            # Update the 'reporting' field in the input data with the parsed and wrapped strategies.
            data["reporting"] = parsed
        return data


class TestSuiteConfig(BaseModel):
    """
    Base configuration model for defining a test suite, including its components, tests, and lifecycle hooks.

    Attributes:
        components (Optional[Dict[str, ManagedComponentConfiguration]]):
            A dictionary of reusable components used throughout the test suite.
            The keys represent component names, and the values are their corresponding configurations.

        tests (Optional[List[TestDefinitionConfig]]):
            A list of test definitions that make up the test suite. Each test is defined using a `TestDefinitionConfig`.

        hooks (Dict[TestLifecyclePhase, HooksConfig]):
            A dictionary of lifecycle hooks to be executed at various phases of the test suite execution.
            The keys are lifecycle phases (e.g., setup, teardown), and the values are `HooksConfig` objects.

    Example:
        suite_config = TestSuiteConfig(
            components={"db": db_component_config, "api": api_component_config},
            tests=[test_1_config, test_2_config],
            hooks={TestLifecyclePhase.BEFORE_ALL: before_all_hook}
        )
    """

    components: Optional[Dict[str, ManagedComponentConfiguration]]
    tests: Optional[List[TestDefinitionConfig]]
    hooks: Dict[TestLifecyclePhase, HooksConfig] = Field(default_factory=dict)
