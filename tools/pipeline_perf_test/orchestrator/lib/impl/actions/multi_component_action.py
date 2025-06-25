"""
Module implementing a custom step action for executing a specific lifecycle phase
(e.g., setup, teardown) on one or more components within a test step context.

Typical YAML configuration example:

```yaml
tests:
  - name: Test Max Rate Logs
    steps:
      - name: Deploy All
        action:
          multi_component_action:
            phase: deploy
            targets:
              - load-generator
              - otel-collector
              - backend-service
```

This module defines two main classes:

- `MultiComponentActionConfig`: Configuration schema for the `MultiComponentAction`,
  specifying the lifecycle phase to execute and the target component names. If no targets
  are specified, the action applies to all components in the context.

- `MultiComponentAction`: A step action that, when executed, iterates over the target
  components and invokes the specified lifecycle method corresponding to the provided phase.

The action is registered with the test step action registry under the name
`"multi_component_action"`.

Typical usage involves including this action in a test step definition where one needs
to programmatically apply a specific phase method to one or more components.

Raises:
    ValueError: If a specified component does not implement the expected lifecycle method.
    AssertionError: If a referenced component is not a valid `Component` instance.
"""

from typing import Optional
from ...core.component import ComponentPhase, Component
from ...core.context import StepContext
from ...core.framework.step import StepActionConfig, StepAction
from ...runner.registry import step_action_registry, PluginMeta


ACTION_NAME = "multi_component_action"


@step_action_registry.register_config(ACTION_NAME)
class MultiComponentActionConfig(StepActionConfig):
    """
    Configuration class for the MultiComponentAction.

    Attributes:
        phase (ComponentPhase): The lifecycle phase (e.g., 'setup', 'teardown') to execute
            on the target components.
        targets (Optional[list[str]]): A list of component names to which the action should apply.
            If empty or not provided, the action will apply to all components in the context.
    """
    phase: ComponentPhase
    targets: Optional[list[str]] = []


@step_action_registry.register_class(ACTION_NAME)
class MultiComponentAction(StepAction):
    """
    Step action that executes a specified lifecycle phase on one or more components.

    This action retrieves the target components (either all in the context or a specified subset)
    and invokes the method corresponding to the configured lifecycle phase on each.

    Attributes:
        config (MultiComponentActionConfig): Configuration object specifying the phase and targets.
    """
    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Test Max Rate Logs
    steps:
      - name: Deploy All
        action:
          multi_component_action:
            phase: deploy
            targets:
              - load-generator
              - otel-collector
              - backend-service
"""
    )

    def __init__(self, config: MultiComponentActionConfig):
        """
        Initializes the action with a parsed configuration object.

        Args:
            config (MultiComponentActionConfig): Configuration with target name(s) and phase to execute.
        """
        self.config = config

    def execute(self, ctx: StepContext):
        """
        Executes the configured lifecycle phase on the target components within the given step context.

        Args:
            ctx (StepContext): The context for the current step execution, providing access
                to components and lifecycle control.

        Behavior:
            - If no specific targets are defined in the config, all components from the context
            are selected.
            - If targets are specified, it retrieves each by name and ensures they are valid Components.
            - For each target component:
                - Sets it as the current step component in the context.
                - Retrieves and calls the method corresponding to the configured lifecycle phase.

        Raises:
            AssertionError: If any of the specified targets is not a valid Component instance.
            ValueError: If a component does not implement the expected lifecycle method.
        """
        components = {}
        if not self.config.targets:
            components = ctx.get_components()
        else:
            for component_name in self.config.targets:
                components[component_name] = ctx.get_component_by_name(component_name)
                assert isinstance(
                    components[component_name], Component
                ), "Expected Component not found"

        for component_name, component in components.items():
            ctx.set_step_component(component)

            lifecycle_action = self.config.phase.value

            if hasattr(component, lifecycle_action):
                method = getattr(component, lifecycle_action).__get__(component)
            else:
                raise ValueError(
                    f"Component {component_name} does not have action {lifecycle_action}"
                )
            method(ctx)
