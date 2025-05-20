"""
Module: lifecycle_component

This module defines the abstract base class `LifecycleComponent`, which serves as a blueprint for components
in a load generation test orchestrator. It provides hooks for various lifecycle phases such as configuration,
deployment, starting, stopping, and destruction, allowing for custom behavior during these phases.

Components derived from `LifecycleComponent` must implement the lifecycle methods (`configure`, `deploy`,
`start`, `stop`, and `destroy`) and can register hooks to be executed at specific points in the lifecycle.

Classes:
    LifecyclePhase: An enumeration of the different phases in the lifecycle of a component.
    LifecycleComponent: An abstract base class that defines the structure for components with lifecycle hooks.
"""

from abc import ABC, abstractmethod
from collections import defaultdict
from dataclasses import dataclass
from enum import Enum
from typing import Callable, Dict, List, Optional, Any

from ..context.base import BaseContext


@dataclass
class LifecycleHookContext(BaseContext):
    """
    Holds state for a test hook execution.
    """

    component: Optional["LifecycleComponent"] = None
    phase: Optional["LifecyclePhase"] = None


class LifecyclePhase(Enum):
    """
    Enum representing the various phases in the lifecycle of a component.

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


class LifecycleComponent(ABC):
    """
    Abstract base class for components within a load generation test orchestrator.

    This class provides a mechanism for registering and executing hooks at various lifecycle phases, allowing
    subclasses to define specific behaviors during phases such as configuration, deployment, starting, stopping,
    and destruction. Subclasses are required to implement the lifecycle methods (`configure`, `deploy`, `start`,
    `stop`, and `destroy`).

    Components can register hooks that will be executed during specific lifecycle phases. Hooks are callable
    functions that are executed when a particular phase occurs, enabling custom actions at various points in the
    lifecycle.

    Attributes:
        _hooks (Dict[LifecyclePhase, List[Callable]]): A registry of hooks for each lifecycle phase,
                                                       where the key is the phase and the value is a list of
                                                       callable functions to execute during that phase.

    Methods:
        add_hook(phase, hook): Registers a hook function to be executed during a specified lifecycle phase.
        _run_hooks(phase): Executes all hooks that have been registered for a specified lifecycle phase.
        configure(): Abstract method to be implemented by subclasses for configuring the component.
        deploy(): Abstract method to be implemented by subclasses for deploying the component.
        start(): Abstract method to be implemented by subclasses for starting the component.
        stop(): Abstract method to be implemented by subclasses for stopping the component.
        destroy(): Abstract method to be implemented by subclasses for destroying the component.
        start_monitoring(): Abstract method to be implemented by subclasses to start monitoring the component.
        stop_monitoring(): Abstract method to be implemented by subclasses to stop monitoring the component.
        collect_monitoring_data(): Abstract method to be implemented by subclasses to collect monitoring data for the component.
    """

    def __init__(self):
        """
        Initializes the LifecycleComponent instance by setting up an empty hook registry.

        The hook registry maps lifecycle phases to lists of hook functions (callables). Hooks
        can be added to different phases, and when those phases are triggered, the corresponding hooks will
        be executed.
        """
        self._hooks: Dict[
            LifecyclePhase, List[Callable[[LifecycleHookContext], Any]]
        ] = defaultdict(list)

    def add_hook(
        self, phase: LifecyclePhase, hook: Callable[[LifecycleHookContext], Any]
    ):
        """
        Registers a hook to be executed during a specific lifecycle phase.

        Hooks allow you to define custom behavior during various lifecycle phases, such as configuring
        the component, deploying it, starting or stopping it, and more. Each hook is a callable function.

        Args:
            phase (LifecyclePhase): The lifecycle phase during which the hook should be executed.
                                     Example phases are "pre_configure", "post_configure", "pre_deploy", etc.
            hook (Callable): A function to be executed during the specified lifecycle phase.

        Example:
            component.add_hook(LifecyclePhase.PRE_DEPLOY, lambda: print("Preparing deployment..."))
        """
        self._hooks[phase].append(hook)

    def _run_hooks(self, phase: LifecyclePhase):
        """
        Executes all hooks that are registered for a specified lifecycle phase.

        This method iterates through the list of hooks registered for the given phase and calls each hook function.

        Args:
            phase (LifecyclePhase): The lifecycle phase during which to run the hooks (e.g., PRE_CONFIGURE, POST_CONFIGURE).
        """
        hook_context = LifecycleHookContext(
            component=self,
            phase=phase,
        )
        for hook in self._hooks.get(phase, []):
            hook(hook_context)

    @abstractmethod
    def configure(self):
        """Abstract method for configuring the component."""

    @abstractmethod
    def deploy(self):
        """Abstract method for deploying the component."""

    @abstractmethod
    def start(self):
        """Abstract method for starting the component."""

    @abstractmethod
    def stop(self):
        """Abstract method for stopping the component."""

    @abstractmethod
    def destroy(self):
        """Abstract method for destroying the component."""

    @abstractmethod
    def start_monitoring(self):
        """Abstract method to start monitoring the component."""

    @abstractmethod
    def stop_monitoring(self):
        """Abstract method to stop monitoring the component."""

    @abstractmethod
    def collect_monitoring_data(self):
        """Abstract method to collect monitoring data for the component."""
