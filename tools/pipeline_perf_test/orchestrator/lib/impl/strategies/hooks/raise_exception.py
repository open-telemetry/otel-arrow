"""
Hook strategy module for raising exceptions during execution.

This module defines a simple hook implementation used primarily for testing and validation
purposes. It provides a configurable mechanism (`RaiseExceptionHook`) to intentionally raise
an exception when invoked, based on the provided configuration.

Classes:
    - RaiseExceptionConfig: Configuration schema for the exception hook.
    - RaiseExceptionHook: Hook strategy that raises a RuntimeError during execution.

Typical use case:
    This hook is useful for testing pipeline failure handling, error reporting,
    or simulating failure scenarios in test frameworks.
"""
from typing import Optional

from ....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from ....core.context.base import BaseContext
from ....core.context import (ComponentHookContext, FrameworkElementHookContext)
from ....runner.registry import hook_registry, PluginMeta

HOOK_NAME = "raise_exception"


@hook_registry.register_config(HOOK_NAME)
class RaiseExceptionConfig(HookStrategyConfig):
    """
    Configuration class for the 'raise_exception' hook.

    Attributes:
        message (str): The optional message to include in the exception
    """

    message: Optional[str] = ""


@hook_registry.register_class(HOOK_NAME)
class RaiseExceptionHook(HookStrategy):
    """
    Hook strategy that raises an exception.

    This class is responsible for raising an exception when fired. Primarily for testing.
    """
    PLUGIN_META = PluginMeta(
        supported_contexts=[FrameworkElementHookContext.__name__, ComponentHookContext.__name__],
        installs_hooks=[],
        yaml_example="""
hooks:
  run:
    pre:
      - raise_exception:
            message: This is a test exception.
"""
    )

    def __init__(self, config: RaiseExceptionConfig):
        """
        Initialize the hook with its configuration.

        Args:
            config (RaiseExceptionConfig): Configuration object containing an exception message.
        """
        self.config = config

    def execute(self, ctx: BaseContext):
        """
        Execute the action to raise an exception

        Args:
            ctx (BaseContext): The execution context, providing utilities like logging.

        Raises:
            subprocess.CalledProcessError: If the command returns a non-zero exit code.
        """
        logger = ctx.get_logger(__name__)

        logger.debug(f"Raising Exception per configuration: {self.config.message}")
        raise RuntimeError(self.config.message)
