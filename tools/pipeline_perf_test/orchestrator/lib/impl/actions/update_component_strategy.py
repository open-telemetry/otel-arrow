"""
Module defining a step action for dynamically updating the strategy configuration
of a managed component during test execution.

This action enables partial or full reconfiguration of a component's internal strategies,
such as deployment, monitoring, execution, or general configuration. It supports dynamic
overrides to simulate different runtime conditions, adjust resources, or test configuration changes.

Classes:
    - UpdateComponentStrategyConfig: Defines the configurable fields that can be updated
      for a target component. Supports partial updates of deployment, monitoring, execution,
      and configuration blocks.
    - UpdateComponentStrategyAction: Applies the update to the specified component by merging
      the partial configuration into its existing configuration and rebuilding the affected strategies.

Functions:
    - update_model: Recursively merges a dictionary of updates into a Pydantic model, handling
      nested models and wrapped configuration structures (e.g., ConfigurableWrapper).

Usage:
    This action can be used in a test step YAML like:

        steps:
          - name: Reconfigure Otel Collector
            action:
              update_component_strategy:
                target: otel-collector
                deployment:
                  docker:
                    volumes:
                      - ./configs/test.yaml:/etc/config.yaml:ro

Raises:
    - AssertionError: If the target component is missing or not a ManagedComponent.
    - RuntimeError: If no applicable strategy was updated.
"""
from pydantic import BaseModel
from typing import Mapping, Optional

from ...core.context import StepContext
from ...core.framework.step import StepActionConfig, StepAction
from ...impl.component.managed_component import (
    ManagedComponent,
    ManagedComponentConfiguration,
)
from ...impl.strategies.monitoring.composite_monitoring_strategy import (
    CompositeMonitoringStrategy,
)
from ...runner.registry import step_action_registry, PluginMeta
from ...runner.wrappers import ConfigurableWrapper


ACTION_NAME = "update_component_strategy"


@step_action_registry.register_config(ACTION_NAME)
class UpdateComponentStrategyConfig(StepActionConfig):
    """
    Configuration schema for the UpdateComponentStrategyAction.

    Attributes:
        target (str): The name of the component to update. This must refer to a valid
            ManagedComponent present in the current step context.
        configuration (Optional[dict]): Partial update for the component's base configuration.
        deployment (Optional[dict]): Partial update for the component's deployment strategy.
        execution (Optional[dict]): Partial update for the component's execution strategy.
        monitoring (Optional[dict]): Partial update for the component's monitoring strategy.
    """
    target: str
    configuration: Optional[dict] = None
    deployment: Optional[dict] = None
    execution: Optional[dict] = None
    monitoring: Optional[dict] = None


@step_action_registry.register_class(ACTION_NAME)
class UpdateComponentStrategyAction(StepAction):
    """
    Step action that applies updates to a strategy configuration of a managed component.

    This action merges partial updates into the existing component configuration and
    rebuilds relevant strategies such as deployment, monitoring, execution, or configuration.

    Attributes:
        config (UpdateComponentStrategyConfig): The configuration specifying the target
            component and the partial updates to apply.
    """
    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Test Max Rate Logs
    steps:
      - name: Reconfigure Otel Collector Docker Volume
        action:
          update_component_strategy:
            target: otel-collector
            deployment:
              docker:
                volumes:
                  - ./configs/test_batch_sizes/component_configs/collector-config-batch-10k.yaml:/etc/otel/collector-config.yaml:ro
"""
    )

    def __init__(self, config: UpdateComponentStrategyConfig):
        """
        Initializes the UpdateComponentStrategyAction with the provided configuration.

        Args:
            config (UpdateComponentStrategyConfig): The update configuration containing
                the target component name and optional update dictionaries.
        """
        self.config = config

    def execute(self, ctx: StepContext):
        """
        Executes the update of the target managed component's strategy configuration.

        Steps performed:
        - Retrieves the target component by name from the step context and asserts it
            is a ManagedComponent.
        - Sets the component as the current step component in the context.
        - Dumps the partial update dictionary from the config, excluding the target name.
        - Merges the partial update into the component's existing configuration using
            the recursive `update_model` function.
        - Checks which parts of the configuration have been updated (deployment,
            monitoring, configuration, execution) and rebuilds/replaces the component's
            strategies accordingly.
        - If no strategy was successfully updated, logs an error and raises a RuntimeError.
        - Finally, persists the updated configuration back into the component.

        Args:
            ctx (StepContext): The current step execution context providing access
                to components and logging.

        Raises:
            AssertionError: If the target component is not found or is of an incompatible type.
            RuntimeError: If the update did not result in any strategy replacement.
        """
        logger = ctx.get_logger(__name__)
        component = ctx.get_component_by_name(self.config.target)
        assert isinstance(
            component, ManagedComponent
        ), "Component not found, or incompatible type"
        ctx.set_step_component(component)
        base_config: ManagedComponentConfiguration = component.component_config

        # Partial update as dict (assumes UpdateComponentStrategyConfig is a Pydantic model)
        partial_update_dict = self.config.model_dump(
            exclude={"target"}, exclude_none=True
        )

        # Merge the partial update into the existing config
        updated_config = update_model(base_config, partial_update_dict)

        updated = False
        # If a new deployment is defined, rebuild and replace the strategy
        if "deployment" in partial_update_dict:
            new_deployment_strategy = updated_config.deployment.build_element()
            updated = component.replace_strategy(new_deployment_strategy)

        # Optionally, handle other fields like 'monitoring', 'execution', etc.
        if "monitoring" in partial_update_dict:
            new_monitoring_strategy = CompositeMonitoringStrategy(
                strategies=[
                    strat.build_element()
                    for _, strat in (updated_config.monitoring or {}).items()
                ]
            )
            updated = component.replace_strategy(new_monitoring_strategy)

        if "configuration" in partial_update_dict:
            updated = component.replace_strategy(
                updated_config.configuration.build_element()
            )

        if "execution" in partial_update_dict:
            updated = component.replace_strategy(
                updated_config.execution.build_element()
            )

        if not updated:
            logger.error("Failed to update component with strategy.")
            raise RuntimeError("Failed to update component with strategy.")

        # Persist the updated config back into the component
        component.component_config = updated_config


def update_model(model: BaseModel, update: dict) -> BaseModel:
    """
    Recursively updates a Pydantic model with values from a given update dictionary.

    This function is capable of handling:
      - Nested Pydantic models
      - Wrapped configurations using ConfigurableWrapper
      - Partial updates that only modify specific fields
      - Merging deep structures without overwriting unrelated fields

    Args:
        model (BaseModel): The original Pydantic model to be updated.
        update (dict): A dictionary containing partial updates to apply.

    Returns:
        BaseModel: A new model instance with the updates applied.

    Behavior:
        - For fields that are `ConfigurableWrapper` instances, it recursively updates
          their internal `config` based on the type key.
        - For fields that are nested Pydantic models, it applies recursive updates.
        - For all other fields, values from the update dictionary are applied directly.

    Example:
        updated_model = update_model(existing_model, {"execution": {"threads": 4}})
    """
    data = model.model_dump()

    for k, v in update.items():
        current_value = getattr(model, k, None)

        if isinstance(current_value, ConfigurableWrapper):
            # We're updating the .config of the wrapper
            updated_config = update_model(
                current_value.config, v.get(current_value.element_type)
            )
            data[k] = current_value.model_copy(update={"config": updated_config})

        elif isinstance(current_value, BaseModel) and isinstance(v, Mapping):
            data[k] = update_model(current_value, v)

        else:
            # New field or direct value (fallback)
            data[k] = v

    return model.model_copy(update=data)
