from typing import Optional

from pydantic import BaseModel, Field

from ..errors.error_handler import OnErrorConfig


class BaseStrategyConfig(BaseModel):
    """Base model for all Strategy configs."""

    on_error: Optional[OnErrorConfig] = Field(default_factory=OnErrorConfig)
