"""
Module: context

This module provides the BaseContext class which provides shared fields common to
different implementations of Contexts throughout the orchestrator.
"""

import datetime
import json
import time
import traceback
from enum import Enum

from dataclasses import dataclass, field
from typing import Dict, List, Optional, TYPE_CHECKING


if TYPE_CHECKING:
    from ..test_framework.test_suite import TestSuite
    from ..component.component import Component


class ExecutionStatus(str, Enum):
    """This class represents the execution status of a context at the current point in time"""

    SUCCESS = "success"
    FAILURE = "failure"
    ERROR = "error"
    SKIPPED = "skipped"
    PENDING = "pending"
    RUNNING = "running"
    CANCELLED = "cancelled"
    TIMEOUT = "timeout"
    INCOMPLETE = "incomplete"


@dataclass
class BaseContext:
    """
    Base context class which includes common timing, status, metadata fields.
    """

    name: Optional[str] = None
    status: Optional[ExecutionStatus] = ExecutionStatus.PENDING
    error: Optional[Exception] = None
    start_time: Optional[float] = None
    end_time: Optional[float] = None
    metadata: dict = field(default_factory=dict)

    parent_ctx: Optional["BaseContext"] = None
    child_contexts: List["BaseContext"] = field(default_factory=list)

    @property
    def duration(self) -> Optional[float]:
        """Duration of the context between start/stop calls or none if not run/stopped."""
        if self.start_time is not None and self.end_time is not None:
            return self.end_time - self.start_time
        return None

    def add_child_ctx(self, ctx: "BaseContext"):
        """Add a context to the list of children on the calling context and set the parent on the child.

        Args:
            ctx: The context to append to the list of children.
        """
        self.child_contexts.append(ctx)
        ctx.parent_ctx = self

    def get_components(self) -> Dict[str, "Component"]:
        """Get the components from the root context.

        Returns: The dict of components, indexed by component name.
        """
        if hasattr(self, "parent_ctx"):
            return self.parent_ctx.get_components()
        raise NotImplementedError("This context does not support get_components")

    def get_component_by_name(self, name: str) -> Optional["Component"]:
        """Get a component instance by the name of the component (from the root context).

        Args:
            name: The name of the component to return

        Returns: The named component or none if not found.
        """
        if hasattr(self, "parent_ctx"):
            return self.parent_ctx.get_component(name)
        raise NotImplementedError("This context does not support get_component_by_name")

    def get_test_suite(self) -> Optional["TestSuite"]:
        """Get the root test suite object from a given context."""
        if hasattr(self, "parent_ctx"):
            return self.parent_ctx.get_test_suite()
        raise NotImplementedError("This context does not support get_test_suite")

    def log(self, message: str):
        """
        Logs a message both to the logger and stores it in the context log list.

        Args:
            message: the message to log
        """
        msg_dict = {"ctx_type": self.__class__.__name__, "message": message}
        if self.name:
            msg_dict["name"] = self.name
        if self.status:
            msg_dict["status"] = self.status
        if self.duration:
            msg_dict["duration"] = f"{self.duration:.4f}"

        log_entry = f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] {json.dumps(msg_dict)}"
        print(log_entry)

    def start(self):
        """Mark start of context (e.g., for timing)."""
        self.log("Context Starting...")
        self.status = ExecutionStatus.RUNNING
        self.start_time = time.time()

    def end(self):
        """Mark end of context and log duration."""
        self.end_time = time.time()
        self.log("Context Ended")

    @staticmethod
    def _format_time(timestamp: Optional[float]) -> str:
        if timestamp is None:
            return "None"
        return datetime.datetime.fromtimestamp(timestamp).isoformat()

    def summary_string(self, indent: int = 2) -> str:
        """
        Summarize the key information about the context.

        Returns: a string representation of the context", including metadata,
            timing, status, and any error message.
        """
        pad = " " * indent
        error_str = (
            "".join(
                traceback.format_exception_only(type(self.error), self.error)
            ).strip()
            if self.error
            else "None"
        )
        duration_str = f"{self.duration:.4f}" if self.duration is not None else "None"

        return (
            f"{pad}Name: {self.name}\n"
            f"{pad}Status: {self.status.value}\n"
            f"{pad}Error: {error_str}\n"
            f"{pad}Start Time: {self._format_time(self.start_time)}\n"
            f"{pad}End Time: {self._format_time(self.end_time)}\n"
            f"{pad}Duration: {duration_str}\n"
            f"{pad}Metadata: {json.dumps(self.metadata, indent=2)}"
        )

    def to_dict(self) -> dict:
        """
        Serialize the BaseContext instance into a dictionary.

        Returns:
            dict: A dictionary representation of the context, including metadata,
                timing, status, error message (as string), and recursively
                serialized parent and child contexts.
        """
        return {
            "name": self.name,
            "status": self.status.value if self.status else None,
            "error": str(self.error) if self.error else None,
            "start_time": self.start_time,
            "end_time": self.end_time,
            "duration": self.duration,
            "metadata": self.metadata,
            "child_contexts": [child.to_dict() for child in self.child_contexts],
        }
