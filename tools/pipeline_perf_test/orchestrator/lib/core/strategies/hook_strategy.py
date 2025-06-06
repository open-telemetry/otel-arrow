"""
Module: hook_strategy

This module defines the `HookStrategy` abstract base class, which provides a unified
interface for implementing custom hooks that can be executed at various stages of
a testbed or component lifecycle.

Hook strategies encapsulate behavior that can be injected into specific points in the
execution flow, such as setup, teardown, or validation phases. This allows for modular
and reusable logic that interacts with the runtime context without modifying core
execution strategies.


Classes:
    HookStrategyConfig(BaseModel): Base model for Hook Strategy config, passed to strategy init.
    HookStrategy (ABC): Abstract interface for executing a hook with access to runtime context.
"""

from abc import ABC, abstractmethod

from ..context.base import BaseContext
from .base import BaseStrategyConfig


class HookStrategyConfig(BaseStrategyConfig):
    """Base model for Execution Strategy config, passed to strategy init."""


class HookStrategy(ABC):
    @abstractmethod
    def __init__(self, config: HookStrategyConfig):
        """All hook strategies must be initialized with a config object."""

    @abstractmethod
    def execute(self, ctx: BaseContext) -> None:
        """Execute the hook and pass it the current context."""
