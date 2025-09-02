"""
Module: base

This module defines foundational classes for implementing strategy-based behavior
across different subsystems. It includes a base configuration model and a base
abstract strategy class, which other strategy types can extend.

Classes:
    BaseStrategyConfig: Pydantic model that provides common configuration options for strategies.
    BaseStrategy: Abstract base class to be inherited by all concrete strategy implementations.
"""

from abc import ABC
from typing import Optional

from pydantic import BaseModel, Field

from ..errors.error_handler import OnErrorConfig


class BaseStrategyConfig(BaseModel):
    """
    Base model for all strategy configurations.

    This configuration class provides shared fields and behaviors that all
    strategy config models should inherit. It includes a default error handling
    configuration that can be used to control behavior when strategies fail.

    Attributes:
        on_error (Optional[OnErrorConfig]): Configuration specifying how errors should be handled
            during strategy execution. Defaults to a new OnErrorConfig instance.
    """

    on_error: Optional[OnErrorConfig] = Field(default_factory=OnErrorConfig)


class BaseStrategy(ABC):
    """
    Base strategy class to be inherited by all strategy implementations.

    This abstract base class serves as a common parent for all strategy types,
    providing a uniform interface and enabling polymorphic behavior.
    """
