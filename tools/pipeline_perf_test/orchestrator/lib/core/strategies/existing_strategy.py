"""Read-only resource discovery for caller-owned components."""

from abc import abstractmethod
from typing import TYPE_CHECKING

from pydantic import ConfigDict

from .base import BaseStrategy, BaseStrategyConfig
from ..context.framework_element_contexts import StepContext

if TYPE_CHECKING:
    from ..component.component import Component


class ExistingStrategyConfig(BaseStrategyConfig):
    """Configuration for resolving a caller-owned component."""

    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)


class ExistingStrategy(BaseStrategy):
    """Resolve existing resource identity without taking lifecycle ownership."""

    @abstractmethod
    def resolve(self, component: "Component", ctx: StepContext):
        """Validate the resource and save its identity for monitoring."""
