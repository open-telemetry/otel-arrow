"""
Module: component_action

This module defines a step action for executing a single lifecycle method
on a single test component within the orchestrator framework. It enables steps
in a test scenario to trigger specific component behaviors (such as starting monitoring or deploying a service) based on the defined
phase.

The action is registered under the name `"component_action"` in the test step action registry,
and is configured via `ComponentActionConfig`.

Typical YAML configuration example:

```yaml
tests:
  - name: Test Max Rate Logs
    steps:
    - name: Start Load Generator
      action:
        component_action:
          phase: start
          target: load-generator
```

Key Classes:

ComponentActionConfig: Defines the configuration schema for the action, including the
target component name and the phase to invoke.

ComponentAction: Retrieves the specified component from the step context and invokes
the method corresponding to the given phase.
"""
from ...core.component import ComponentPhase, Component
from ...core.context import StepContext
from ...core.framework.step import StepActionConfig, StepAction
from ...runner.registry import test_step_action_registry, PluginMeta


ACTION_NAME = "component_action"


@test_step_action_registry.register_config(ACTION_NAME)
class ComponentActionConfig(StepActionConfig):
    """
    Configuration schema for the 'component_action' test step action.

    This config specifies which component the action targets and what lifecycle
    phase (i.e., method) should be executed on that component.

    Attributes:
        target (str): The name of the component to invoke the action on.
                      This should match the name of a registered Component
                      in the current test context.

        phase (ComponentPhase): The lifecycle phase or action to invoke.
                                Must correspond to a valid method defined on
    """
    target: str
    phase: ComponentPhase


@test_step_action_registry.register_class(ACTION_NAME)
class ComponentAction(StepAction):
    """
    Step action implementation for executing a lifecycle phase on a named component.

    This class is executed at runtime during a test step, and it:

    - Resolves the component by name from the test context.
    - Verifies the component supports the requested lifecycle phase.
    - Executes the corresponding method on the component, passing in the current context.
    """
    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Test Max Rate Logs
    steps:
    - name: Start Load Generator
      action:
        component_action:
          phase: start
          target: load-generator
"""
    )

    def __init__(self, config: ComponentActionConfig):
        """
        Initializes the action with a parsed configuration object.

        Args:
            config (ComponentActionConfig): Configuration with target name and phase to execute.
        """
        self.config = config

    def execute(self, ctx: StepContext):
        """
        Executes the specified lifecycle method on the target component.

        Steps:
          1. Fetches the component by name from the test context.
          2. Confirms the object is a valid Component instance.
          3. Records the component in the context (for traceability or downstream steps).
          4. Looks up the method corresponding to the lifecycle phase.
          5. Calls the method with the current step context.

        Raises:
            ValueError: If the component does not implement the specified lifecycle method.
        """
        component = ctx.get_component_by_name(self.config.target)
        assert isinstance(component, Component), "Expected Component not found"
        ctx.set_step_component(component)

        lifecycle_action = self.config.phase.value

        if hasattr(component, lifecycle_action):
            method = getattr(component, lifecycle_action).__get__(component)
        else:
            raise ValueError(
                f"Component {self.config.target} does not have action {lifecycle_action}"
            )
        method(ctx)
