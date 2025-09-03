"""
wrappers.py

This module provides generic wrapper classes for building element instances
from configuration objects and type identifiers.

Each wrapper class is parameterized by specific configuration and element types,
and relies on a corresponding registry to resolve and instantiate the correct
element implementation.

Wrappers serve as a bridge between raw configuration data and fully constructed
element objects, facilitating consistent parsing, validation, and instantiation
across different element domains such as test actions, hooks, monitoring, reporting,
configuration, and execution.
"""

from typing import Type, Dict, Generic, TypeVar
from pydantic import BaseModel, model_validator

from ..core.strategies.deployment_strategy import (
    DeploymentStrategyConfig,
    DeploymentStrategy,
)
from ..core.strategies.monitoring_strategy import (
    MonitoringStrategyConfig,
    MonitoringStrategy,
)
from ..core.strategies.hook_strategy import HookStrategyConfig, HookStrategy
from ..core.strategies.execution_strategy import (
    ExecutionStrategy,
    ExecutionStrategyConfig,
)
from ..core.strategies.configuration_strategy import (
    ConfigurationStrategy,
    ConfigurationStrategyConfig,
)
from ..core.framework.step import StepActionConfig, StepAction
from .registry import (
    deployment_registry,
    monitoring_registry,
    configuration_registry,
    execution_registry,
    hook_registry,
    step_action_registry,
)

ConfigType = TypeVar("ConfigType", bound=BaseModel)  # pylint: disable=invalid-name
ElementType = TypeVar("ElementType")  # pylint: disable=invalid-name


class ConfigurableWrapper(BaseModel, Generic[ConfigType, ElementType]):
    """
    Generic wrapper for building a instances from a type and config.

    This class serves as a base for defining wrappers that take a
    type identifier and a corresponding config object, and resolve
    the actual class using a registry defined in subclasses.

    Type Args:
        ConfigType: The type of configuration model required by the strategy.
        ElementType: The class type being constructed.

    Attributes:
        element_type (str): The key identifying which implementation to use.
        config (ConfigType): The configuration object used to initialize the element.
        _registry (Dict[str, Type[ElementType]]): A registry mapping element type
            strings to concrete element classes. Must be defined in subclasses.
    """

    element_type: str
    config: ConfigType

    # Registry must be defined in subclasses
    _registry: Dict[str, Type[ElementType]] = {}

    def build_element(self) -> ElementType:
        """
        Instantiate the element based on the element type and config.

        Returns:
            ElementType: An instance of the resolved element class, initialized
            with the provided configuration.

        Raises:
            ValueError: If no element class is registered for the given element type.
        """
        element_cls = self._registry.get(self.element_type)
        if not element_cls:
            raise ValueError(
                f"No element class registered for type '{self.element_type}'"
            )
        return element_cls(self.config)


class HookWrapper(ConfigurableWrapper[HookStrategyConfig, HookStrategy]):
    """
    Wrapper class for hook strategies, specialized with hook-specific config and strategy types.

    This class uses the `hook_registry` to map strategy type strings to
    concrete hook strategy classes and configurations.

    Attributes:
        _registry (Dict[str, Type[HookStrategy]]): Registry mapping hook strategy type
            names to their corresponding classes, sourced from `hook_registry.strategy`.
    """

    _registry = hook_registry.element

    @model_validator(mode="before")
    @classmethod
    def parse_keyed_strategy(cls, data: Dict) -> Dict:
        """
        Validate and parse input data into a normalized dictionary containing
        `strategy_type` and `config` keys.

        This validator supports two input formats:
        - A dictionary already containing 'strategy_type' and 'config' keys.
        - A single-key dictionary where the key is the strategy type and the value
          is the config data dictionary.

        Args:
            data (Dict): Input data to parse, expected to follow one of the formats above.

        Returns:
            Dict: A dictionary with keys:
                - 'strategy_type': The resolved strategy type string.
                - 'config': An instantiated configuration object for that strategy.

        Raises:
            ValueError: If input is not a dict with exactly one strategy key,
                        or if the strategy type is unknown (not in `hook_registry.config`).
        """
        if isinstance(data, dict) and "strategy_type" in data and "config" in data:
            return data
        if not isinstance(data, dict) or len(data) != 1:
            raise ValueError("Hook must have exactly one strategy key")
        strategy_type, config_data = next(iter(data.items()))
        config_cls = hook_registry.config.get(strategy_type)
        if not config_cls:
            raise ValueError(f"Unknown hook strategy: '{strategy_type}'")
        config = config_cls(**config_data)
        return {
            "element_type": strategy_type,
            "config": config,
        }


class DeploymentWrapper(
    ConfigurableWrapper[DeploymentStrategyConfig, DeploymentStrategy]
):
    _registry = deployment_registry.element

    @model_validator(mode="before")
    @classmethod
    def parse_keyed_strategy(cls, data: Dict) -> Dict:
        if isinstance(data, dict) and "strategy_type" in data and "config" in data:
            return data
        if not isinstance(data, dict) or len(data) != 1:
            raise ValueError("Deployment must have exactly one strategy key")
        element_type, config_data = next(iter(data.items()))
        config_cls = deployment_registry.config.get(element_type)
        if not config_cls:
            raise ValueError(f"Unknown deployment strategy: '{element_type}'")
        config = config_cls(**config_data)
        return {
            "element_type": element_type,
            "config": config,
        }


class MonitoringWrapper(
    ConfigurableWrapper[MonitoringStrategyConfig, MonitoringStrategy]
):
    """
    Wrapper class for monitoring strategies, specialized with monitoring-specific config and strategy types.

    Attributes:
        _registry (Dict[str, Type[MonitoringStrategy]]): Registry mapping monitoring
            strategy type strings to their corresponding classes, sourced from
            `monitoring_registry.strategy`.
    """

    _registry = monitoring_registry.element


class ConfigurationWrapper(
    ConfigurableWrapper[ConfigurationStrategyConfig, ConfigurationStrategy]
):
    """
    Wrapper class for configuration strategies, specialized with configuration-specific config and strategy types.

    Attributes:
        _registry (Dict[str, Type[ConfigurationStrategy]]): Registry mapping configuration
            strategy type strings to their corresponding classes, sourced from
            `configuration_registry.strategy`.
    """

    _registry = configuration_registry.element


class ExecutionWrapper(ConfigurableWrapper[ExecutionStrategyConfig, ExecutionStrategy]):
    """
    Wrapper class for execution strategies, specialized with execution-specific config and strategy types.

    Attributes:
        _registry (Dict[str, Type[ExecutionStrategy]]): Registry mapping execution
            strategy type strings to their corresponding classes, sourced from
            `execution_registry.strategy`.
    """

    _registry = execution_registry.element

    @model_validator(mode="before")
    @classmethod
    def parse_keyed_strategy(cls, data: Dict) -> Dict:
        if isinstance(data, dict) and "strategy_type" in data and "config" in data:
            return data
        if not isinstance(data, dict) or len(data) != 1:
            raise ValueError("Deployment must have exactly one strategy key")
        element_type, config_data = next(iter(data.items()))
        config_cls = execution_registry.config.get(element_type)
        if not config_cls:
            raise ValueError(f"Unknown execution strategy: '{element_type}'")
        config = config_cls(**config_data)
        return {
            "element_type": element_type,
            "config": config,
        }


class TestStepActionWrapper(ConfigurableWrapper[StepActionConfig, StepAction]):
    """
    Wrapper class for test step actions, specialized with execution-specific config and types.

    Attributes:
        _registry (Dict[str, Type[TestStepAction]]): Registry mapping execution
            strategy type strings to their corresponding classes, sourced from
            `execution_registry.strategy`.
    """

    _registry = step_action_registry.element
