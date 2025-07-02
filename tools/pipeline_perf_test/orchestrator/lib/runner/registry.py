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

from dataclasses import dataclass
from typing import Dict, Type, Callable, TypeVar, Optional, Any

from ..cli.plugin_api import register_argument_hook, get_or_create_arg_group


@dataclass
class CliFlag:
    flag: str  # e.g. "--docker.build-containers"
    dest: str  # e.g. docker_build_containers
    help: str
    group: str  # e.g. "Docker Options"
    action: Optional[str] = None  # e.g. "store_true"
    default: Optional[Any] = None
    required: bool = False
    metavar: Optional[str] = None


@dataclass
class ReportMeta:
    """Structured metadata for report plugins."""

    supported_aggregations: Optional[list[str]]
    sample_output: Optional[dict[str, str]] = None


@dataclass
class PluginMeta:
    """Structured metadata for registered plugins."""

    supported_contexts: Optional[list[str]]
    yaml_example: Optional[str]
    installs_hooks: list[str]
    notes: Optional[str] = None
    cli_flags: Optional[list[CliFlag]] = None
    report_meta: Optional[ReportMeta] = None


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

            plugin_meta = getattr(cls, "PLUGIN_META", None)
            if plugin_meta and getattr(plugin_meta, "cli_flags", None):

                def add_args(parser):
                    for flag in plugin_meta.cli_flags:
                        group = get_or_create_arg_group(parser, group_name=flag.group)

                        kwargs = {
                            "dest": flag.dest,
                            "help": flag.help,
                            "default": flag.default,
                            "required": flag.required,
                        }
                        if flag.action:
                            kwargs["action"] = flag.action
                        if flag.metavar:
                            kwargs["metavar"] = flag.metavar

                        group.add_argument(flag.flag, **kwargs)

                register_argument_hook(add_args)

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


class StepActionRegistry(ElementRegistry):
    """
    Registry for mapping strategy and configuration type names to their classes.
    """


# Domain-specific registries for different strategy categories
deployment_registry = StrategyRegistry()
monitoring_registry = StrategyRegistry()
configuration_registry = StrategyRegistry()
execution_registry = StrategyRegistry()
hook_registry = StrategyRegistry()
step_action_registry = StepActionRegistry()
report_writer_registry = StrategyRegistry()
report_formatter_registry = StrategyRegistry()
