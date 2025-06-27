# Plugin Development Guide

This guide explains how to develop, register, and integrate custom plugins for
the Test Orchestration Framework. Plugins enable developers to extend the
framework with custom behaviors such as deployment logic, execution workflows,
telemetry monitoring, and reporting.

Plugins are modular, type-safe, and discoverable via YAML configuration. Each
plugin adheres to a specific interface and is dynamically loaded at runtime
based on its registered type.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Plugin Types and Responsibilities](#2-plugin-types-and-responsibilities)
3. [Plugin Anatomy](#3-plugin-anatomy)
4. [Registration and Discovery](#4-registration-and-discovery)
5. [Contexts and Hook Points](#5-contexts-and-hook-points)
6. [Configuration Schema Definition](#6-configuration-schema-definition)
7. [Plugin Class Implementation](#7-plugin-class-implementation)
8. [Registering Plugins with the Framework Registry](#8-registering-plugins-with-the-framework-registry)
9. [Logging and Error Handling](#9-logging-and-error-handling)
10. [Testing and Validation](#10-testing-and-validation)
11. [Extending the Framework](#11-extending-the-framework)

---

## 1. Overview

The Test Orchestration Framework is designed to support modular, pluggable
strategies for each phase of the test lifecycle. Plugins are the building
blocks of this extensibility. They allow you to customize:

- How components are **configured** (e.g., load from external source, template
    rendering, etc)
- How components are **deployed** (e.g., via Docker, processes, Kubernetes)
- How components are **executed** (e.g., send load, simulate telemetry)
- How components are **monitored** (e.g., via Prometheus or logs)
- What **hooks** run before or after test phases (e.g., run shell commands,
    collect data)
- How **reports** are generated at the end of a test suite

Each plugin implements a well-defined interface and is registered under a
unique type name. These type names are used in the YAML configuration to
associate behaviors with components or test phases.

This guide walks you through the structure, expectations, and lifecycle of
plugin development in the framework. Whether you're creating a new load
generator, integrating a monitoring system, or automating test analysis,
plugins are the mechanism to do it cleanly and consistently.

---

## 2. Plugin Types and Responsibilities

Plugins are grouped by their role in the test lifecycle. Each plugin type
corresponds to a specific phase or aspect of orchestrated test execution. This
section provides an overview of each type and what it is responsible for.

| Plugin Type                | Base Class                    | Purpose                                                                 |
|----------------------------|-------------------------------|-------------------------------------------------------------------------|
| **Configuration Strategy** | `ConfigurationStrategy`       | Prepares config files or settings before deployment                     |
| **Deployment Strategy**    | `DeploymentStrategy`          | Launches components into the test environment (e.g., Docker, processes) |
| **Execution Strategy**     | `ExecutionStrategy`           | Drives component behavior during the test (send traffic, run scripts)   |
| **Monitoring Strategy**    | `MonitoringStrategy`          | Collects telemetry and health data (e.g., metrics, logs, status checks) |
| **Hook Strategy**          | `HookStrategy`                | Executes logic before or after lifecycle events (e.g., setup, teardown) |
| **Reporting Hook**         | `ReportingHookStrategy`       | Generates and outputs reports after test execution                      |
| **Test Step Action**       | `TestStepAction`              | Defines reusable actions used within test step blocks                   |

Each plugin must:

- Inherit from the appropriate base class (from `lib/core/strategies/`)
- Define a configuration schema class inheriting from `*StrategyConfig`
- Be registered using decorators:
  `@<registry>.register_config("type_name")` and
  `@<registry>.register_class("type_name")`
- Define metadata via `PLUGIN_META`, including supported contexts and
example YAML usage

Plugin instances are created and invoked automatically during test execution
based on the test suite's YAML configuration.

The following sections walk through the anatomy of a plugin and how to implement
each type.

Reporting Strategies have additional considerations, and are covered seperately
in the [Reports Guide](./reports.md).

---

## 3. Plugin Anatomy

Each plugin follows a consistent structure to ensure it can be discovered,
validated, and executed correctly by the orchestrator. This section breaks down
the key components of a well-formed plugin.

### Core Components

A plugin typically consists of two main classes:

1. **Configuration Schema**
   - Inherits from `*StrategyConfig`
   - Defines all configurable parameters (with types, defaults, and docstrings)
   - Registered with `@<registry>.register_config("type_name")`

2. **Implementation Class**
   - Inherits from the appropriate `*Strategy` base class
   - Implements required methods (`start`, `stop`, `execute`, etc.)
   - Registered with `@<registry>.register_class("type_name")`
   - Defines a `PLUGIN_META` block with cli flags, context, and YAML metadata

---

### Example: Hook Plugin

```python
from ....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from ....runner.registry import hook_registry, PluginMeta

@hook_registry.register_config("run_command")
class RunCommandConfig(HookStrategyConfig):
    command: str  # Shell command to execute

@hook_registry.register_class("run_command")
class RunCommandHook(HookStrategy):
    PLUGIN_META = PluginMeta(
        supported_contexts=["FrameworkElementHookContext", "ComponentHookContext"],
        installs_hooks=[],
        yaml_example="""
hooks:
  run:
    pre:
      - run_command:
          command: echo 'Hello'
"""
    )

    def __init__(self, config: RunCommandConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        import subprocess
        logger = ctx.get_logger(__name__)
        logger.debug(f"Executing: {self.config.command}")
        subprocess.run([self.config.command], shell=True, check=True)
```

---

### File and Directory Structure

Plugins live under:

```shell
lib/impl/strategies/<plugin_type>/
```

Examples:

- `lib/impl/strategies/execution/my_custom_loadgen.py`
- `lib/impl/strategies/hooks/record_custom_metric.py`

Each plugin file typically contains both the `*Config` class and the strategy class.

---

### PluginMeta

Every plugin class can define a `PLUGIN_META` attribute with the following fields:

| Field               | Purpose                                              |
|---------------------|------------------------------------------------------|
| `supported_contexts` | List of context types the plugin supports                      |
| `installs_hooks`     | Hook types the plugin automatically installs (if any)          |
| `yaml_example`       | Example YAML block showing how to use the plugin               |
| `cli_flags`          | List of additional cli flags to inject into the argpase parser |

This metadata is used by documentation generators, validation tools, and helps
developers quickly understand how to use the plugin.

---

## 4. Registration and Discovery

All plugins must be registered with the framework to be discoverable and
instantiable based on their YAML type. The framework uses a centralized
registry system for each plugin type, found in `lib/runner/registry.py`.

Each registry maps a string-based `type` identifier (from the YAML config) to:

- A **configuration class** that validates and parses user-defined settings
- An **implementation class** that executes the plugin behavior

---

### How Registration Works

The registration process uses decorators provided by the appropriate registry.
Each plugin needs to register both its configuration schema and its strategy implementation.

```python
# Register the config class with its YAML type name
@execution_registry.register_config("my_strategy")
class MyStrategyConfig(ExecutionStrategyConfig):
    ...

# Register the implementation class with the same type name
@execution_registry.register_class("my_strategy")
class MyStrategy(ExecutionStrategy):
    ...
```

The string `"my_strategy"` becomes the YAML key used to reference this plugin in
suite definitions.

---

### Available Registries

Each plugin type has its own registry:

| Registry Name              | Plugin Type           | Decorators                     |
|----------------------------|------------------------|--------------------------------|
| `deployment_registry`      | DeploymentStrategy     | `@deployment_registry.register_class` |
| `execution_registry`       | ExecutionStrategy      | `@execution_registry.register_class`  |
| `monitoring_registry`      | MonitoringStrategy     | `@monitoring_registry.register_class` |
| `hook_registry`            | HookStrategy           | `@hook_registry.register_class`      |
| `reporting_registry`       | ReportingHookStrategy  | `@reporting_registry.register_class` |
| `configuration_registry`   | ConfigurationStrategy  | `@configuration_registry.register_class` |
| `test_step_action_registry`| TestStepAction         | `@test_step_action_registry.register_class` |

Each of these also supports `register_config(...)` for registering the
configuration schema.

---

### Plugin Discovery at Runtime

At test suite load time:

1. The YAML parser reads the `type` field under `deployment`, `execution`, etc.
2. The framework queries the appropriate registry using the type string.
3. It retrieves the registered config class and validates the YAML block.
4. It constructs the plugin using the parsed config.
5. The plugin is injected into the component or test lifecycle as needed.

---

### Best Practice

Always match the type string in `@register_config("type")` and
`@register_class("type")`. This string must be unique
**within the scope of the plugin type**.

Incorrect or missing registration will result in validation or runtime errors
when parsing a test suite.

---

## 5. Contexts and Hook Points

Plugins operate within well-defined **execution contexts**, which give them
access to framework utilities such as logging, telemetry, runtime state, and
framework events. They are also triggered by **hook points** - lifecycle phases
like `deploy`, `start`, `stop`, etc.

---

### Contexts

Each plugin receives a specific **context object** when invoked. These are
defined in `lib/core/context/` and contain relevant information for the plugin's
scope.

| Context Class                    | Used By                    | Description                                                                 |
|----------------------------------|----------------------------|-----------------------------------------------------------------------------|
| `BaseContext`                    | Base Class for all Context Types      | Provides core features: logging, telemetry, runtime info                    |
| `FrameworkElementContext`        | Inherited Class suite/step/scenario   | Base class for contexts created for Suite, Scenario, Steps main execution   |
| `SuiteContext`                   | Indirect access by any descendant ctx | Root CTX instance in the hierarchy, holds shared runtimes, telemetry, etc   |
| `ScendarioContext`               | Indirect access by any descendant ctx | Provides access to the test scenario defition                               |
| `StepContext`                    | Passed to All (non-hook) Strategies   | Provides access to the step, any related component, and access to ancestors |
| `ComponentHookContext`           | Passed to Component-level hooks       | Includes access to the component being manipulated                          |
| `FrameworkElementHookContext`    | Passed to Suite/scenario/step hooks   | Includes access to the element (suite, scenario, step) being manipulated    |

Context's are arranged in a logical hierarchy, with SuiteContext at the root
like so:

```shell
|_ SuiteContext(FrameworkElementContext)
    |_ FrameworkElementHookContext(BaseContext) # If hook installed on Suite
    |_ ScendarioContext(FrameworkElementContext)
        |_ FrameworkElementHookContext(BaseContext) # If hook installed on Scenario
        |_ StepContext(FrameworkElementContext)
            |_ FrameworkElementHookContext(BaseContext) # If hook installed on Step
            |_ ComponentHookContext(BaseContext)
```

Since the SuiteContext is available in all contexts, shared state (telemetry
data, shared clients, cli args, other shared runtime info) is typically stored there.

The BaseContext provides the following useful methods on all Context Types:

- get_components() - Retrieves component dictionary from the suite object
- get_component_by_name(name) - Retrieves named component from the suite object
- get_suite() - Retrieves root test suite object.
- get_metadata() - Returns the metadata dictionary for the context (merged with
    it's ancestors)
- record_event(event_name, timestamp_unix_nanos, **kwargs) - Records a telemetry
    event enriched with context metadata.
- get_logger(logger_name) - Returns a LoggerAdapter with context metadata injected.
- get_tracer(name, runtime_name) - Retrieves an OpenTelemetry tracer from the
    suite's telemetry runtime.
- get_meter(name, runtime_name) - Retrieves a telemetry meter from the suite's
    telemetry runtime.
- get_telemetry_client(runtime_name) - Retrieves telemetry client from the
    suite's telemetry runtime.

Your plugin should declare which contexts it supports via the
`PLUGIN_META.supported_contexts` list, generally either or both of ComponentHookContext
and FrameworkElementHookContext.

---

### Hook Points

Hook points define **when** a plugin is invoked. They are named phases in the
test lifecycle and come in 2 broad types:

- **Framework Element Hooks** (e.g. run on Suite, Scenario, Steps)
- **Component Strategy Hooks** (e.g. run on Component's deploy, start, stop phases)

Hooks are declared in YAML as `pre` or `post` lists for a phase. At runtime, all
applicable hooks are executed in order when the phase for the associated element
or component is invoked.

The following phases are defined:

- **RUN:** Used by all test elements (Suite, Scenario, Steps)
- **CONFIGURE:** Call configuration strategies to e.g. prepare manifests for
    deployment
- **DEPLOY:** Call a deployment strategy to e.g. deploy / start a
    process/container
- **START:** Call an execution strategy to e.g. start sending load
- **STOP:** Call an execution strategy to e.g. stop sending load
- **DESTROY:** Call a deployment strategy to e.g. stop a process/container
- **START_MONITORING:** Call a monitoring strategy to e.g. monitor a
    process / container
- **STOP_MONITORING:** Call a monitoring strategy to e.g. stop monitoring a
    process / container

```yaml
hooks:
  run:
    post:
      - run_command:
          command: echo "Done!"
```

Each hook plugin can define logic that will run during these lifecycle events -
from cleanup tasks to validations, logging, or side-effects.

---

### Defining Context Compatibility

Each plugin should declare its compatible contexts explicitly:

```python
PLUGIN_META = PluginMeta(
    supported_contexts=["ComponentHookContext", "FrameworkElementHookContext"],
    installs_hooks=[],
    yaml_example=...
)
```

At the moment, this is only for documentation, and no validation is done at runtime
(your plugin should check this).

---

## 6. Configuration Schema Definition

Each plugin requires a configuration schema that defines the parameters it accepts.
These schemas provide validation, type safety, and default values, enabling the
framework to parse and validate user-provided YAML configuration.

---

### Base Config Classes

Most plugin types have a corresponding base config class defined under `lib/core/strategies/`:

| Plugin Type         | Base Config Class          |
|---------------------|----------------------------|
| HookStrategy        | `HookStrategyConfig`       |
| ExecutionStrategy   | `ExecutionStrategyConfig`  |
| ReportingHookStrategy | `ReportingHookStrategyConfig` |
| DeploymentStrategy  | `DeploymentStrategyConfig` |
| MonitoringStrategy  | `MonitoringStrategyConfig` |
| ConfigurationStrategy | `ConfigurationStrategyConfig` |

Your config class should inherit from the appropriate base class.

---

### Defining Config Fields

Use Python type annotations to define config fields. For example:

```python
from typing import Optional
from ....core.strategies.hook_strategy import HookStrategyConfig

class RunCommandConfig(HookStrategyConfig):
    command: str
    timeout_seconds: Optional[int] = 30  # default value
```

- **Required fields** have no default and must be specified by the user.
- **Optional fields** can have default values.
- Supported types include standard Python types like `str`, `int`, `float`,
    `bool`, `Optional[...]`, and collections such as `List[str]`.

---

### Validation and Defaults

The framework uses these annotations to:

- Validate that required fields are present.
- Apply default values when omitted.
- Provide clear error messages on misconfiguration.

---

### Registering the Config Class

Register your config class with the corresponding registry using the
`@register_config("type_name")` decorator:

```python
from ....runner.registry import hook_registry

@hook_registry.register_config("run_command")
class RunCommandConfig(HookStrategyConfig):
    command: str
    timeout_seconds: Optional[int] = 30
```

This ties your config schema to the plugin type name, enabling automatic parsing
from YAML.

---

### Best Practices

- Keep config options minimal and focused.
- Document each field clearly in the class docstring.
- Use optional fields with sensible defaults where possible.
- Avoid complex nested objects unless necessary - keep configs declarative.

---

## 7. Plugin Class Implementation

The plugin class contains the logic that executes the behavior defined by your
strategy, hook, or action. It typically consumes the configuration class and
interacts with the framework through provided context objects.

---

### Base Classes

Choose the appropriate base class from `lib/core/strategies/` according to your
plugin type:

| Plugin Type         | Base Class             |
|---------------------|------------------------|
| HookStrategy        | `HookStrategy`          |
| ExecutionStrategy   | `ExecutionStrategy`     |
| ReportingHookStrategy | `ReportingHookStrategy` |
| DeploymentStrategy  | `DeploymentStrategy`    |
| MonitoringStrategy  | `MonitoringStrategy`    |
| ConfigurationStrategy | `ConfigurationStrategy` |

Your plugin class should inherit from the relevant base.

---

### Initialization

The plugin class constructor takes an instance of your config class:

```python
class RunCommandHook(HookStrategy):
    def __init__(self, config: RunCommandConfig):
        self.config = config
```

Store the config instance for use during execution.

---

### Implementing Plugin Logic

Define the main method(s) the framework calls to trigger plugin behavior.
This differs by plugin type:

| Plugin Type       | Typical Method(s) to Implement       | Purpose                            |
|-------------------|--------------------------------------|------------------------------------|
| HookStrategy          | execute(self, ctx: BaseContext)        | Run the hook logic           |
| ReportingHookStrategy | execute(self, ctx: BaseContext)        | Generate reports             |
| ExecutionStrategy     | start(self, component, ctx)            | Control execution lifecycle  |
| ExecutionStrategy     | stop(self, component, ctx)             | Control execution lifecycle  |
| DeploymentStrategy    | deploy(self, component, ctx)           | Handle deployment steps      |
| DeploymentStrategy    | destroy(self, component, ctx)          | Handle destroy steps         |
| MonitoringStrategy    | start_monitoring(self, component, ctx) | Start monitoring activities  |
| MonitoringStrategy    | stop_monitoring(self, component, ctx)  | Stop monitoring activities   |
| ConfigurationStrategy | configure(self, ctx)`                  | Manage configuration updates |

Use the `ctx` (context) argument to access utilities such as logging, component
state, or framework APIs.

---

### Example: HookStrategy execute Method

```python
def execute(self, ctx: BaseContext):
    logger = ctx.get_logger(__name__)
    logger.debug(f"Executing command: {self.config.command}")
    import subprocess

    subprocess.run([self.config.command], shell=True, check=True)
```

- Use context for logging and utility functions.
- Handle exceptions as appropriate or let them propagate.
- Keep the execution method focused on the core logic.

---

### Registering the Plugin Class

Use the appropriate registry decorator with your plugin's type name:

```python
from ....runner.registry import hook_registry

@hook_registry.register_class("run_command")
class RunCommandHook(HookStrategy):
    ...
```

This connects the implementation class to the framework's plugin system.

---

### Plugin Metadata

Define a `PLUGIN_META` class attribute of type `PluginMeta` to declare:

- Supported contexts.
- Installed default hooks.
- Example YAML usage.
- Any additional CLI arguments to install

Example:

```python
from ....runner.registry import PluginMeta
from ....core.context import FrameworkElementHookContext, ComponentHookContext

PLUGIN_META = PluginMeta(
    supported_contexts=[FrameworkElementHookContext.__name__, ComponentHookContext.__name__],
    installs_hooks=[],
    yaml_example="""
steps:
  - action:
      wait:
        delay_seconds: 10
    hooks:
      run:
        pre:
          - run_command:
              command: python somefile.py
"""
)
```

This metadata aids documentation generation and validation, as well as CLI
parser configuration.

### Custom CLI Flags

Some plugins may need to add extra CLI arguments. These are installed using the
cli_flags attribute on the implementation's PLUGIN_META (a list of CliFlag's
which are passed to the root argparse parser).

This example will add --docker.no-build to skip building images at runtime.

```python
PLUGIN_META = PluginMeta(
        ...
        cli_flags=[
            CliFlag(
                group="Docker Options",
                flag="--docker.no-build",
                dest="docker_no_build",
                help="Skip build of Docker containers.",
                action="store_true",
                default=False,
            ),
        ],
```

Plugins can access the root argparse args object through the Suite context.
This example shows how the 'build_docker_image' plugin accesses it and uses
it to decide whether to skip building:

```python
    def execute(self, ctx: BaseContext):
        # ...
        args = ctx.get_suite().get_runtime("args")
        if args.docker_no_build:
            ctx.status = ExecutionStatus.SKIPPED
            return
        # ...
```

---

## 8. Registering Plugins with the Framework Registry

To integrate your plugin into the framework, it must be registered with the
appropriate registry. This enables the framework to discover and instantiate
your plugin dynamically by its type name.

---

### Registry Types

The framework provides specialized registries for different plugin categories,
all accessible from `lib/runner/registry.py`. Common registries include:

| Registry Name          | Description                         | Typical Plugins Registered         |
|------------------------|-------------------------------------|------------------------------------|
| `hook_registry`        | Hook strategies                     | HookStrategy implementations       |
| `execution_registry`   | Execution strategies                | ExecutionStrategy implementations   |
| `deployment_registry`  | Deployment strategies               | DeploymentStrategy implementations  |
| `monitoring_registry`  | Monitoring strategies               | MonitoringStrategy implementations  |
| `reporting_registry`   | Reporting hooks                     | ReportingHookStrategy implementations|
| `configuration_registry` | Configuration strategies          | ConfigurationStrategy implementations|
| `test_step_action_registry` | Test step action implementations | Actions performed in test steps    |

---

### Registering Configuration Classes

Register the plugin's configuration class with the registry decorator `@register_config("type_name")`:

```python
@hook_registry.register_config("run_command")
class RunCommandConfig(HookStrategyConfig):
    command: str
```

This tells the framework how to parse and validate configuration data associated
with your plugin type.

---

### Registering Implementation Classes

Register the plugin implementation class with the registry decorator `@register_class("type_name")`:

```python
@hook_registry.register_class("run_command")
class RunCommandHook(HookStrategy):
    ...
```

This binds the type name `"run_command"` to your implementation class,
enabling dynamic plugin instantiation.

---

### Accessing Plugins from the Registry

At runtime, the framework uses these registries to lookup the classes associated
with a given type name:

```python
hook_cls = hook_registry.element["run_command"]
hook_config_cls = hook_registry.config["run_command"]
```

The `element` dictionary maps type names to implementation classes, while
`config` maps to config classes.

---

### Registry Best Practices

- Always register **both** your config class and implementation class under the
    same type name.
- Choose a clear, unique, and descriptive type name (usually snake_case).
- Keep registration decorators adjacent to their class definitions for clarity.
- Use consistent naming and organization reflecting the plugin's purpose.

---

### Example Summary

```python
from ....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from ....runner.registry import hook_registry

@hook_registry.register_config("run_command")
class RunCommandConfig(HookStrategyConfig):
    command: str

@hook_registry.register_class("run_command")
class RunCommandHook(HookStrategy):
    def __init__(self, config: RunCommandConfig):
        self.config = config

    def execute(self, ctx):
        #...
```

---

## 9. Logging and Error Handling

Reliable error handling and detailed logging are essential for maintaining and
debugging plugins within the Test Orchestration Framework. This section
describes how to implement error handling patterns and logging best practices
in your plugin code.

### 9.1 Error Handling Configuration

The framework supports an error handling schema via the `on_error` configuration
block, which can be applied to components or test steps to control retry
behavior and error propagation.

Example YAML snippet:

```yaml
on_error:
  retries: 2              # Number of retry attempts before failing
  retry_delay_seconds: 5  # Delay in seconds between retries
  continue: true          # Whether to continue execution after error/retries
```

- retries: Defines how many times the framework should retry the failed operation.
- retry_delay_seconds: Specifies the wait time between retries.
- continue: If true, the framework continues execution even if retries are
    exhausted; otherwise, the error causes failure.

Implementing your plugin to respect this schema enables smoother test execution
with configurable fault tolerance.

### 9.2 Logging Best Practices

Plugins should utilize the context's logger to record informative debug, info,
warning, and error messages. This facilitates troubleshooting and observability
during test runs.

- Obtain a logger instance via `ctx.get_logger(__name__)`.
- Use logger.debug() for verbose details useful during development or deep troubleshooting.
- Use logger.info() for high-level operational messages.
- Use logger.warning() and logger.error() for potential or actual issues.
- Avoid suppressing exceptions silently; log before raising to maintain traceability.

### 9.3 Example: Hook Plugin with Error Handling and Logging

Here is a simplified example from a hook plugin illustrating these concepts:

```python
def execute(self, ctx: BaseContext):
    """
    Execute the action to record an event

    Args:
        ctx (BaseContext): The execution context, providing utilities like logging.

    Raises:
        RuntimeError: If the context does not have an active span.
    """
    logger = ctx.get_logger(__name__)

    logger.debug(f"Recording custom event: {self.config.name}")
    if ctx and ctx.span and ctx.span.is_recording():
        ctx.record_event(self.config.name, **self.config.attributes)
        return
    logger.error(
        "The current context does not exist or does not have an active span."
    )
    raise RuntimeError(
        "The current context does not exist or does not have an active span."
    )
```

This example:

- Logs a debug message when starting the event recording.
- Checks for valid context and span before proceeding.
- Logs an error if the context is invalid and raises an exception to signal failure.

---

## 10. Testing and Validation

Proper testing ensures your plugins behave as expected and integrate seamlessly
with the framework. Plugin tests should be placed in the
`tests/impl/strategies/...` directory structure, mirroring the layout of the
corresponding implementation code in `lib/`.

### 10.1 Test Directory Layout

For example, if your plugin is located at:

```shell
lib/impl/strategies/hooks/raise_exception.py
```

Then the tests should reside in:

```shell
tests/impl/strategies/hooks/test_raise_exception.py
```

### 10.2 Example Test for a Hook Plugin

Below is an example test for the `RaiseExceptionHook` plugin demonstrating how to:

- Instantiate plugin config and class
- Mock the execution context and logger
- Verify that the plugin raises the expected exception with the correct message

```python
import pytest
from unittest.mock import MagicMock

from lib.impl.strategies.hooks.raise_exception import (
    RaiseExceptionHook,
    RaiseExceptionConfig,
)


def test_raise_exception_hook_raises_runtime_error():
    # Given
    message = "Intentional test failure"
    config = RaiseExceptionConfig(message=message)
    hook = RaiseExceptionHook(config=config)

    # Mock context with logger
    mock_ctx = MagicMock()
    mock_logger = MagicMock()
    mock_ctx.get_logger.return_value = mock_logger

    # When / Then
    with pytest.raises(RuntimeError) as exc_info:
        hook.execute(mock_ctx)

    # Assert the exception message
    assert str(exc_info.value) == message

    # Assert logger was called with debug
    mock_logger.debug.assert_called_once_with(
        f"Raising Exception per configuration: {message}"
    )


def test_raise_exception_hook_with_empty_message():
    config = RaiseExceptionConfig(message="")
    hook = RaiseExceptionHook(config=config)

    mock_ctx = MagicMock()
    mock_logger = MagicMock()
    mock_ctx.get_logger.return_value = mock_logger

    with pytest.raises(RuntimeError) as exc_info:
        hook.execute(mock_ctx)

    assert str(exc_info.value) == ""
    mock_logger.debug.assert_called_once_with(
        "Raising Exception per configuration: ")
```

11.3 Testing Recommendations

- Use pytest for concise and expressive test cases.
- Utilize unittest.mock.MagicMock to mock context, logger, and external dependencies.
- Cover both expected behavior and edge cases, including error scenarios.
- Follow the plugin interface contract strictly in tests.

By following these practices, your plugins will be robust, maintainable, and
reliable within the Test Orchestration Framework.

---

## 11. Extending the Framework

The Test Orchestration Framework can be extended by adding new **StepActions**,
which represent atomic operations executed during test steps. These actions can
control test flow, interact with components, or perform auxiliary tasks like
waiting or triggering hooks.

---

### 11.1 Adding a StepAction: Example `wait_action`

This example demonstrates how to implement a simple **wait** action that pauses
execution for a configured number of seconds.

---

#### Module Overview

```python
"""
Module defining a step action that introduces a delay during test execution.
"""
import time

from ...core.context import StepContext
from ...core.framework.step import StepActionConfig, StepAction
from ...runner.registry import step_action_registry, PluginMeta


ACTION_NAME = "wait"


@step_action_registry.register_config(ACTION_NAME)
class WaitActionConfig(StepActionConfig):
    """
    Configuration for the WaitAction.

    Attributes:
        delay_seconds (float): The number of seconds to pause during step execution.
            This should be a non-negative float representing the delay duration.
    """
    delay_seconds: float


@step_action_registry.register_class(ACTION_NAME)
class WaitAction(StepAction):
    """
    Step action that introduces a delay during test execution.

    This action simply waits for the duration specified in the configuration
    before proceeding.
    Useful for introducing timing gaps or waiting for external processes to settle.

    Attributes:
        config: Configuration containing the delay duration in seconds.
    """
    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Test Max Rate Logs
    steps:
    - name: Wait For Test
      action:
        wait:
          delay_seconds: 3
"""
    )

    def __init__(self, config: WaitActionConfig):
        self.config = config

    def execute(self, _ctx: StepContext):
        """
        Executes the wait action by sleeping for the configured number of seconds.

        Args:
            _ctx (StepContext): The execution context for the step (unused.
        """
        time.sleep(self.config.delay_seconds)
```

### 11.2 How to Extend

- Define a config class extending `StepActionConfig` describing your action's
    configurable parameters.
- Define an action class extending `StepAction` implementing the `execute()` method.
- Register both config and action classes in the `step_action_registry` using
    the same action name.
- Provide `PLUGIN_META` with metadata including supported contexts and YAML examples.
- Place the module under `lib/impl/actions/` or another appropriate subfolder
    mirroring framework layout.
- Add corresponding tests under `tests/impl/actions/`.

By following this pattern, you can extend the framework with any custom logic
encapsulated as step actions, making test flows more flexible and powerful.

---
