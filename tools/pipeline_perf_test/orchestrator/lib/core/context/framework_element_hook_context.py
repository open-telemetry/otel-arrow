"""
Module: framework_element_hook_context

This module defines data structures and enums that support lifecycle hooks for test framework elements.
These hooks enable actions to be performed automatically before or after key phases of a test's execution.

Hookable lifecycle phases are defined using the `HookableTestPhase` enum, allowing for consistent
integration points such as before a test runs or after it completes.

The `FrameworkElementHookContext` class extends the base context and provides structured access to information
about the current test phase and its associated framework element. This context object is passed to hook
strategies for introspection and action.

Typical use cases include:
    - Injecting setup logic before a test runs.
    - Executing cleanup or result validation after a test finishes.
    - Dynamically altering behavior based on the test phase or test element metadata.

Enums:
    HookableTestPhase: Defines supported lifecycle phases where hooks can be executed.

Classes:
    FrameworkElementHookContext(BaseContext): Context object passed to lifecycle hooks, containing phase
                                           and optional reference to the test framework element.
"""

from dataclasses import dataclass
from enum import Enum
from typing import Optional, TYPE_CHECKING

from ..telemetry.framework_event import FrameworkEvent
from .base import BaseContext
from .framework_element_contexts import FrameworkElementContext

if TYPE_CHECKING:
    from ..framework.element import FrameworkLifecyclePhase, FrameworkElement


class HookableTestPhase(Enum):
    """This class represents the phases of a test framework element which can accept hooks."""

    PRE_RUN = "pre_run"
    POST_RUN = "post_run"


@dataclass
class FrameworkElementHookContext(BaseContext):
    """
    Context object used during test lifecycle hooks to provide information
    about the current phase and its associated test framework element.

    This context is passed to hook strategies executed during specific
    lifecycle phases (e.g., pre-run, post-run). It allows the hook to
    access the broader test context and the test element involved in the phase.

    Attributes:
        parent_ctx (Optional[FrameworkElementContext]): Reference to the parent test element's context.
        phase (Optional[TestLifecyclePhase]): The lifecycle phase during which the hook is executed.

    Methods:
        get_test_element() -> Optional[FrameworkElement]:
            Returns the test element associated with the current context, if available.
    """

    parent_ctx: Optional["FrameworkElementContext"] = None
    phase: Optional["FrameworkLifecyclePhase"] = None

    def __post_init__(self):
        """
        Performs additional initialization after object creation.
        """
        super().__post_init__()
        self.start_event_type = FrameworkEvent.HOOK_START
        self.end_event_type = FrameworkEvent.HOOK_END
        if self.parent_ctx and hasattr(self.parent_ctx, "metadata"):
            merged_metadata = {**self.parent_ctx.metadata, **self.metadata}
            self.metadata = merged_metadata
        self.metadata["test.ctx.phase"] = self.phase.value
        self.span_name = f"Run Framework Hook {self.name} {self.phase.value}"

    def get_framework_element(self) -> Optional["FrameworkElement"]:
        """Returns the test element associated with the current context, if available."""
        if self.parent_ctx is None:
            return None
        return self.parent_ctx.get_framework_element()
