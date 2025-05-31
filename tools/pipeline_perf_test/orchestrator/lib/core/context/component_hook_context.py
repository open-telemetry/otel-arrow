"""
Module: component_hook_context

This module defines context structures and enumerations related to hook execution
during the lifecycle of a test component. It supports component-level lifecycle hooks
that can be triggered before or after specific phases such as deployment, configuration,
start, stop, and monitoring.

The `HookableComponentPhase` enum enumerates all supported lifecycle stages at which
hooks may be registered and executed. This provides fine-grained control for extending
behavior across the component's lifecycle.

The `ComponentHookContext` class provides structured context during the execution of a
hook, including access to the test step and component under orchestration. It allows
strategies and plugins to introspect and interact with the current hook execution.

Typical use cases include:
    - Injecting behavior before a component is deployed (e.g., validation or mocking).
    - Collecting diagnostics after a component has been stopped.
    - Triggering monitoring setup during component startup.

Enums:
    HookableComponentPhase: Enumerates all phases of the component lifecycle that support hook execution.

Classes:
    ComponentHookContext(BaseContext): Context object passed to component lifecycle hooks, giving access
                                       to the parent test step and the associated component instance.
"""

from dataclasses import dataclass
from enum import Enum
from typing import Optional, TYPE_CHECKING

from .base import BaseContext
from .test_contexts import TestStepContext

if TYPE_CHECKING:
    from ..component.component import Component


class HookableComponentPhase(Enum):
    """
    Enum representing the various phases in the lifecycle of a component which support hooks.

    These phases correspond to different stages of the component's lifecycle, where hooks can be registered
    and executed to perform actions before or after a phase is executed. These phases help manage the
    orchestration of components during test execution.

    Phases include:
        - PRE_CONFIGURE, POST_CONFIGURE
        - PRE_DEPLOY, POST_DEPLOY
        - PRE_START, POST_START
        - PRE_STOP, POST_STOP
        - PRE_DESTROY, POST_DESTROY
        - PRE_START_MONITORING, POST_START_MONITORING
        - PRE_STOP_MONITORING, POST_STOP_MONITORING
    """

    PRE_CONFIGURE = "pre_configure"
    POST_CONFIGURE = "post_configure"
    PRE_DEPLOY = "pre_deploy"
    POST_DEPLOY = "post_deploy"
    PRE_START = "pre_start"
    POST_START = "post_start"
    PRE_STOP = "pre_stop"
    POST_STOP = "post_stop"
    PRE_DESTROY = "pre_destroy"
    POST_DESTROY = "post_destroy"
    PRE_START_MONITORING = "pre_start_monitoring"
    POST_START_MONITORING = "post_start_monitoring"
    PRE_STOP_MONITORING = "pre_stop_monitoring"
    POST_STOP_MONITORING = "post_stop_monitoring"


@dataclass
class ComponentHookContext(BaseContext):
    """
    Holds state for a component hook execution.
    """

    parent_ctx: Optional["TestStepContext"] = None
    phase: Optional["HookableComponentPhase"] = None

    def get_step_component(self) -> Optional["Component"]:
        """Fetches the component instance on which this hook is firing.

        Returns: the component instance or none.
        """
        if self.parent_ctx is None:
            raise RuntimeError(
                "LifecycleHookContext.parent_ctx must be set to access the step component."
            )
        return self.parent_ctx.step.component
