"""
factory.py

This module provides factory functions for constructing core objects in the test
framework from their configuration representations. These include managed components,
test steps, test definitions, and entire test suites.

Each factory function is responsible for:
- Instantiating the appropriate element from its config model.
- Building and injecting optional strategies (e.g., deployment, monitoring, execution).
- Registering lifecycle hooks defined in the configuration.
- Ensuring consistency and encapsulating construction logic.

Functions:
- build_managed_component: Creates a ManagedComponent from its configuration.
- build_test_step: Creates a TestStep with action and hooks from its config.
- build_test_definition: Builds a TestDefinition including steps and reporting strategies.
- build_test_suite: Constructs a complete TestSuite with components, tests, and hooks.
- register_hooks_from_config: Utility to attach pre/post hooks based on config and phase enums.
- get_hookable_phase: Resolves enum members for lifecycle hook registration.

This module centralizes framework object construction to promote modularity, reuse,
and consistency across the system.
"""

from enum import Enum
from typing import Type
from ..core.context.component_hook_context import HookableComponentPhase
from ..core.test_framework import (
    TestSuite,
    TestDefinition,
    TestStep,
    TestFrameworkElement,
)
from ..core.context.test_element_hook_context import HookableTestPhase
from ..impl.component.managed_component import (
    ManagedComponent,
    ManagedComponentConfiguration,
)
from ..impl.strategies.monitoring.composite_monitoring_strategy import (
    CompositeMonitoringStrategy,
)

from .schema.test_config import (
    TestSuiteConfig,
    TestDefinitionConfig,
    TestStepConfig,
)

# Trigger all the default strategy / action registrations
# pylint: disable=unused-import
from ..impl import strategies  # Do not remove
from ..impl import actions  # Do not remove


def get_hookable_phase(enum_cls: Type[Enum], base_phase: str, timing: str) -> Enum:
    """
    Retrieve a specific enum member representing a hookable phase based on timing and phase name.

    Args:
        enum_cls (Type[Enum]): The enumeration class containing hookable phase members.
        base_phase (str): The base name of the phase (e.g., "run").
        timing (str): The timing of the hook relative to the phase (e.g., "pre", "post").

    Returns:
        Enum: The corresponding enum member (e.g., enum_cls["PRE_START"]).

    Raises:
        KeyError: If the constructed name does not match any member of the enum.
    """
    name = f"{timing.upper()}_{base_phase.upper()}"
    return enum_cls[name]


def register_hooks_from_config(
    target: TestFrameworkElement | ManagedComponent,
    config: (
        ManagedComponentConfiguration
        | TestSuiteConfig
        | TestDefinitionConfig
        | TestStepConfig
    ),
    enum_cls: Type[Enum],
):
    """
    Registers lifecycle hooks from a configuration object onto a target element.

    This function iterates over the hook phases defined in the configuration object,
    builds the corresponding hook elements (both pre and post), and attaches them
    to the appropriate lifecycle phase of the target element using the provided enum class.

    The hookable phase is determined dynamically by combining the phase name (e.g., "start")
    with a timing prefix ("pre" or "post"), and matching it against the given enum class
    (e.g., enum_cls["PRE_START"]).

    Args:
        target (TestFrameworkElement | ManagedComponent): The object that will have hooks attached.
        config (ManagedComponentConfiguration | TestSuiteConfig | TestDefinitionConfig | TestStepConfig):
            The configuration containing hook definitions for different lifecycle phases.
        enum_cls (Type[Enum]): Enum class representing hookable lifecycle phases.

    Raises:
        KeyError: If a generated hook phase name (e.g., "PRE_START") is not found in the enum.
    """
    for phase, hooks_config in config.hooks.items():
        base_phase = phase.value  # e.g., "start", "deploy"

        for hook_wrapper in hooks_config.pre:
            hook = hook_wrapper.build_element()
            lifecycle_phase = get_hookable_phase(enum_cls, base_phase, "pre")
            target.add_hook(lifecycle_phase, hook)

        for hook_wrapper in hooks_config.post:
            hook = hook_wrapper.build_element()
            lifecycle_phase = get_hookable_phase(enum_cls, base_phase, "post")
            target.add_hook(lifecycle_phase, hook)


def build_managed_component(
    name: str, config: ManagedComponentConfiguration
) -> ManagedComponent:
    """
    Constructs a ManagedComponent instance from its configuration.

    This function builds and assembles various optional strategies defined in the
    `ManagedComponentConfiguration`, including deployment, monitoring, configuration,
    and execution strategies. It also registers any lifecycle hooks specified in
    the configuration.

    Args:
        name (str): The name to assign to the managed component.
        config (ManagedComponentConfiguration): The configuration object that defines
            the component's behavior, strategies, and hooks.

    Returns:
        ManagedComponent: A fully constructed and hook-registered managed component.

    Notes:
        - If a strategy (e.g., deployment, configuration) is not specified in the config,
          it will be set to None.
        - The monitoring strategy supports multiple sub-strategies combined into a
          CompositeMonitoringStrategy.
        - Lifecycle hooks are registered using the `HookableComponentPhase` enum.
    """
    deployment = config.deployment.build_element() if config.deployment else None
    monitoring = CompositeMonitoringStrategy(
        strategies=[
            strat.build_element() for _, strat in (config.monitoring or {}).items()
        ]
    )
    configuration = (
        config.configuration.build_element() if config.configuration else None
    )
    execution = config.execution.build_element() if config.execution else None

    component = ManagedComponent(
        name=name,
        config=config,
        deployment_strategy=deployment,
        monitoring_strategy=monitoring,
        configuration_strategy=configuration,
        execution_strategy=execution,
    )
    register_hooks_from_config(component, config, HookableComponentPhase)
    return component


def build_test_step(config: TestStepConfig) -> TestStep:
    """
    Constructs a TestStep instance from its configuration.

    This function initializes a TestStep using the provided configuration by:
    - Building the action element defined in the config.
    - Registering any pre- and post-execution hooks defined in the config using the
      `HookableTestPhase` enum.

    Args:
        config (TestStepConfig): The configuration object defining the test step's
            name, action, and associated hooks.

    Returns:
        TestStep: A fully initialized and hook-registered test step instance.
    """
    ts = TestStep(name=config.name, action=config.action.build_element())
    register_hooks_from_config(ts, config, HookableTestPhase)
    return ts


def build_test_definition(config: TestDefinitionConfig) -> TestDefinition:
    """
    Constructs a TestDefinition instance from its configuration.

    This function builds the individual test steps and reporting strategies defined
    in the configuration, and registers any lifecycle hooks associated with the test
    definition using the `HookableTestPhase` enum.

    Steps:
    - Each configured test step is built using `build_test_step`.
    - Reporting strategies are built and aggregated.
    - A TestDefinition object is instantiated with the name, steps, and strategies.
    - Pre and post-execution hooks are registered.

    Args:
        config (TestDefinitionConfig): Configuration defining the test name, steps,
            reporting strategies, and hooks.

    Returns:
        TestDefinition: A fully constructed and hook-registered test definition.
    """
    steps = [build_test_step(s) for s in config.steps or []]
    reporting_strategies = []
    for _, strat in config.reporting.items():
        reporting_strategies.append(strat.build_element())

    td = TestDefinition(
        name=config.name, steps=steps, reporting_strategies=reporting_strategies
    )
    register_hooks_from_config(td, config, HookableTestPhase)
    return td


def build_test_suite(config: TestSuiteConfig) -> TestSuite:
    """
    Constructs a TestSuite instance from its configuration.

    This function builds all managed components and test definitions defined
    in the test suite configuration. It also registers any lifecycle hooks
    using the `HookableTestPhase` enum.

    Steps:
    - For each component in the configuration, build a ManagedComponent using
      `build_managed_component`.
    - For each test definition, build a TestDefinition using `build_test_definition`.
    - Create a TestSuite with the assembled components and tests.
    - Register pre- and post-phase hooks defined in the suite config.

    Args:
        config (TestSuiteConfig): Configuration object defining the suite's
            components, tests, and associated hooks.

    Returns:
        TestSuite: A fully constructed and hook-registered test suite instance.
    """
    components = {}
    for component_name, component_def in config.components.items():
        component = build_managed_component(component_name, component_def)
        components[component_name] = component

    tests = []
    for test_def in config.tests:
        tests.append(build_test_definition(test_def))

    ts = TestSuite(components=components, tests=tests)
    register_hooks_from_config(ts, config, HookableTestPhase)
    return ts
