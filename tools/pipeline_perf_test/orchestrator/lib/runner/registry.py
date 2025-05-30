"""
registry.py

This module defines a flexible registry system for managing the mapping between
string-based type identifiers and their corresponding element and configuration classes.
It supports registration and lookup of components such as strategies, actions, and their
configuration schemas.

Core Concepts:
- ElementRegistry: A base registry that maintains mappings for both element classes
  and their associated configuration classes using decorators.
- StrategyRegistry: A specialization of ElementRegistry for registering various strategy types.
- TestStepActionRegistry: A specialization of ElementRegistry for test step actions.

Usage:
- Register classes using the `@register_class("type_name")` decorator.
- Register config schemas using the `@register_config("type_name")` decorator.
- Retrieve registered classes from `element` or `config` dictionaries.

Defined Registries:
- deployment_registry: For deployment strategies.
- monitoring_registry: For monitoring strategies.
- reporting_registry: For reporting strategies.
- configuration_registry: For configuration strategies.
- execution_registry: For execution strategies.
- hook_registry: For hook strategies.
- test_step_action_registry: For test step action implementations.

This module centralizes and standardizes how extensible elements are registered and
resolved dynamically by type name throughout the framework.
"""

from typing import Dict, Type, Callable, TypeVar


T = TypeVar("T")
ConfigT = TypeVar("ConfigT")


class ElementRegistry:
    """
    Registry for mapping element and configuration type names to their classes.

    Attributes:
        element (Dict[str, Type]): Maps element type names to element classes.
        config (Dict[str, Type]): Maps element type names to configuration classes.
    """

    def __init__(self):
        """
        Initialize empty registries for elements and configs.
        """
        self.element: Dict[str, Type] = {}
        self.config: Dict[str, Type] = {}

    def register_class(self, type_name: str) -> Callable[[Type[T]], Type[T]]:
        """
        Decorator to register a element class under a given type name.

        Args:
            type_name (str): The identifier to register the element class under.

        Returns:
            Callable[[Type[T]], Type[T]]: A decorator that registers the class and returns it unchanged.
        """

        def decorator(cls: Type[T]) -> Type[T]:
            self.element[type_name] = cls
            return cls

        return decorator

    def register_config(
        self, type_name: str
    ) -> Callable[[Type[ConfigT]], Type[ConfigT]]:
        """
        Decorator to register a configuration class under a given type name.

        Args:
            type_name (str): The identifier to register the config class under.

        Returns:
            Callable[[Type[ConfigT]], Type[ConfigT]]: A decorator that registers the class and returns it unchanged.
        """

        def decorator(cls: Type[ConfigT]) -> Type[ConfigT]:
            self.config[type_name] = cls
            return cls

        return decorator


class StrategyRegistry(ElementRegistry):
    """
    Registry for mapping strategy and configuration type names to their classes.
    """


class TestStepActionRegistry(ElementRegistry):
    """
    Registry for mapping strategy and configuration type names to their classes.
    """


# Domain-specific registries for different strategy categories
deployment_registry = StrategyRegistry()
monitoring_registry = StrategyRegistry()
reporting_registry = StrategyRegistry()
configuration_registry = StrategyRegistry()
execution_registry = StrategyRegistry()
hook_registry = StrategyRegistry()
test_step_action_registry = TestStepActionRegistry()
