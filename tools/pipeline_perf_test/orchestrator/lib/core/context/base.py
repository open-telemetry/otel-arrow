"""
Module: context

This module provides the BaseContext class which provides shared fields common to
different implementations of Contexts throughout the orchestrator.
"""
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class BaseContext:
    """
    Base context class which includes common timing, status, metadata fields.
    """
    status: Optional[str] = field(init=False)
    error: Optional[Exception] = field(init=False)
    start_time: Optional[float] = field(init=False)
    end_time: Optional[float] = field(init=False)
    metadata: dict = field(default_factory=dict, init=False)

    # Prevent initialization ordering errors for non-default fields in inheriting class.
    def __post_init__(self):
        self.status = None
        self.error = None
        self.start_time = None
        self.end_time = None
