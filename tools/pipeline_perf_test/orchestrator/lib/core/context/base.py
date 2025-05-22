"""
Module: context

This module provides the BaseContext class which provides shared fields common to
different implementations of Contexts throughout the orchestrator.
"""

import json
import time

from dataclasses import dataclass, field
from typing import Dict, List, Optional, TYPE_CHECKING


if TYPE_CHECKING:
    from ..component.lifecycle_component import LifecycleComponent


@dataclass
class BaseContext:
    """
    Base context class which includes common timing, status, metadata fields.
    """

    name: Optional[str] = None
    status: Optional[str] = None
    error: Optional[Exception] = None
    start_time: Optional[float] = None
    end_time: Optional[float] = None
    metadata: dict = field(default_factory=dict)

    parent_ctx: Optional["BaseContext"] = None
    child_contexts: List["BaseContext"] = field(default_factory=list)

    def add_child_ctx(self, ctx: "BaseContext"):
        """Add a context to the list of children on the calling context and set the parent on the child.

        Args:
            ctx: The context to append to the list of children.
        """
        self.child_contexts.append(ctx)
        ctx.parent_ctx = self

    def get_components(self) -> Dict[str, "LifecycleComponent"]:
        """Get the components from the root context.

        Returns: The dict of components, indexed by component name.
        """
        if hasattr(self, "parent_ctx"):
            return self.parent_ctx.get_components()
        raise NotImplementedError("This context does not support get_components")

    def get_component_by_name(self, name: str) -> Optional["LifecycleComponent"]:
        """Get a component instance by the name of the component (from the root context).

        Args:
            name: The name of the component to return

        Returns: The named component or none if not found.
        """
        if hasattr(self, "parent_ctx"):
            return self.parent_ctx.get_component(name)
        raise NotImplementedError("This context does not support get_component_by_name")

    def get_client(
        self, name: str, client_factory: Optional[callable] = None
    ) -> object:
        """
        Retrieve a client from the root context, creating it if it does not exist.

        name: The name of the client.
        client_factory: A factory function to create the client if it does not exist.

        return: The client object.
        """
        if hasattr(self, "parent_ctx"):
            return self.parent_ctx.get_client(name, client_factory)
        raise NotImplementedError("This context does not support get_client")

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
        duration = (
            self.end_time - self.start_time
            if self.start_time and self.end_time
            else None
        )
        if duration:
            msg_dict["duration"] = f"{duration:.4f}"

        log_entry = f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] {json.dumps(msg_dict)}"
        print(log_entry)

    def start(self):
        """Mark start of context (e.g., for timing)."""
        self.log("Context Starting...")
        self.start_time = time.time()

    def end(self):
        """Mark end of context and log duration."""
        self.end_time = time.time()
        self.log("Context Ended")
