"""
Module: component

This module defines the abstract base class `Component`, which serves as a blueprint for components
managed by the orchestrator. It provides hooks for various lifecycle phases such as configuration,
deployment, starting, stopping, and destruction, allowing for custom behavior during these phases.

Components derived from `Component` must implement the lifecycle methods (`configure`, `deploy`,
`start`, `stop`, and `destroy`) and can register hooks to be executed at specific points in the lifecycle.

Classes:
    ComponentPhase: An enumeration of the different phases in the lifecycle of a component.
    Component: An abstract base class that defines the structure for components with lifecycle hooks.
"""

from abc import ABC, abstractmethod
from collections import defaultdict
from enum import Enum
from typing import Callable, Dict, List, Any

from ..runtime.runtime import Runtime
from ..context.base import ExecutionStatus
from ..context.test_contexts import TestStepContext, TestExecutionContext
from ..strategies.hook_strategy import HookStrategy
from ..context.component_hook_context import (
    HookableComponentPhase,
    ComponentHookContext,
)


class ComponentPhase(Enum):
    """
    Enum representing the various primary phases in the lifecycle of a component.

    These phases help manage the orchestration of components during test execution.

    Phases include:
        - CONFIGURE        (call configuration strategies to e.g. prepare manifests for deployment)
        - DEPLOY           (call a deployment strategy to e.g. deploy / start a process/container)
        - START            (call an execution strategy to e.g. start sending load)
        - STOP             (call an execution strategy to e.g. stop sending load)
        - DESTROY          (call a deployment strategy to e.g. stop a process/container)
        - START_MONITORING (call a monitoring strategy to e.g. monitor a process / container)
        - STOP_MONITORING  (call a monitoring strategy to e.g. stop monitoring a process / container)
    """

    CONFIGURE = "configure"
    DEPLOY = "deploy"
    START = "start"
    STOP = "stop"
    DESTROY = "destroy"
    START_MONITORING = "start_monitoring"
    STOP_MONITORING = "stop_monitoring"


class Component(ABC):
    """
    Abstract base class for components within the test orchestrator.

    This class provides a mechanism for registering and executing hooks at various lifecycle phases, allowing
    subclasses to define specific behaviors during phases such as configuration, deployment, starting, stopping,
    and destruction. Subclasses are required to implement the lifecycle methods (`configure`, `deploy`, `start`,
    `stop`, and `destroy`).

    Components can register hooks that will be executed during specific lifecycle phases. Hooks are callable
    functions that are executed when a particular phase occurs, enabling custom actions at various points in the
    lifecycle.

    Attributes:
        _hooks (Dict[HookableComponentPhase, List[HookStrategy]]): A registry of hooks for each lifecycle phase,
                                                       where the key is the phase and the value is a list of
                                                       HookStrategy functions to execute during that phase.

    Methods:
        add_hook(phase, hook): Registers a hook function to be executed during a specified lifecycle phase.
        _run_hooks(phase): Executes all hooks that have been registered for a specified lifecycle phase.
        configure(): Abstract method to be implemented by subclasses for configuring the component.
        deploy(): Abstract method to be implemented by subclasses for deploying the component (e.g. spawn process, start container).
        start(): Abstract method to be implemented by subclasses for starting the component's execution behavior (e.g. send load).
        stop(): Abstract method to be implemented by subclasses for stopping the component's execution behavior (e.g. stop load).
        destroy(): Abstract method to be implemented by subclasses for destroying the component (e.g. kill process, stop/remove container).
        start_monitoring(): Abstract method to be implemented by subclasses to start monitoring the component.
        stop_monitoring(): Abstract method to be implemented by subclasses to stop monitoring the component.
        collect_monitoring_data(): Abstract method to be implemented by subclasses to collect monitoring data for the component.
    """

    def __init__(self):
        """
        Initializes the Component instance by setting up an empty hook registry.

        The hook registry maps lifecycle phases to lists of hook functions (callables). Hooks
        can be added to different phases, and when those phases are triggered, the corresponding hooks will
        be executed.
        """
        self._hooks: Dict[HookableComponentPhase, List[HookStrategy]] = defaultdict(
            list
        )
        self.runtime: Runtime = Runtime()

    def get_or_create_runtime(self, namespace: str, factory: Callable[[], Any]) -> Any:
        """Get an existing runtime data structure or initialize a new one.

        Args:
            namespace: The namespace to get/create data for.
            factory: The initialization method if no namespace data exists.
        """
        return self.runtime.get_or_create(namespace, factory)

    def set_runtime_data(self, namespace: str, data: Any):
        """Set the data value on the component's runtime with the specified namespace.

        Args:
            namespace: The namespace to set the data value on.
            data: The data to set.
        """
        self.runtime.set(namespace, data)

    def add_hook(self, phase: HookableComponentPhase, hook: HookStrategy):
        """
        Registers a hook to be executed during a specific lifecycle phase.

        Hooks allow you to define custom behavior during various lifecycle phases, such as configuring
        the component, deploying it, starting or stopping it, and more. Each hook is a callable function.

        Args:
            phase (HookableComponentPhase): The lifecycle phase during which the hook should be executed.
                                     Example phases are "pre_configure", "post_configure", "pre_deploy", etc.
            hook (Callable): A function to be executed during the specified lifecycle phase.

        Example:
            component.add_hook(HookableComponentPhase.PRE_DEPLOY, lambda: print("Preparing deployment..."))
        """
        self._hooks[phase].append(hook)

    def _run_hooks(self, phase: HookableComponentPhase, ctx: TestStepContext):
        """
        Executes all hooks that are registered for a specified lifecycle phase.

        This method iterates through the list of hooks registered for the given phase and calls each hook function.

        Args:
            phase (HookableComponentPhase): The lifecycle phase during which to run the hooks (e.g., PRE_CONFIGURE, POST_CONFIGURE).
        """
        ctx.log(f"Running hooks for phase: {phase.value}")
        for hook in self._hooks.get(phase, []):
            hook_context = ComponentHookContext(
                phase=phase, name=f"{hook.__class__.__name__} ({phase.value})"
            )
            ctx.add_child_ctx(hook_context)
            try:
                hook_context.start()
                hook.execute(hook_context)
                if hook_context.status == ExecutionStatus.RUNNING:
                    hook_context.status = ExecutionStatus.SUCCESS
            except Exception as e:  # pylint: disable=broad-except
                hook_context.status = ExecutionStatus.ERROR
                hook_context.error = e
                hook_context.log(f"Hook failed: {e}")
                break
            finally:
                hook_context.end()

    @abstractmethod
    def configure(self, ctx: TestStepContext):
        """Abstract method for configuring the component."""

    @abstractmethod
    def deploy(self, ctx: TestStepContext):
        """Abstract method for deploying the component (spawn a process or start a container/deployment)."""

    @abstractmethod
    def start(self, ctx: TestStepContext):
        """Abstract method for starting the component's execution behavior."""

    @abstractmethod
    def stop(self, ctx: TestStepContext):
        """Abstract method for stopping the component's execution behavior."""

    @abstractmethod
    def destroy(self, ctx: TestStepContext):
        """Abstract method for destroying the component (e.g. kill process, stop/remove container).

        The specific signals (term/kill) and container cleanup (stop vs rm) will be dictated and
        configured by the strategy implementation and lifecycle hooks.
        """

    @abstractmethod
    def start_monitoring(self, ctx: TestStepContext):
        """Abstract method to start monitoring the component."""

    @abstractmethod
    def stop_monitoring(self, ctx: TestStepContext):
        """Abstract method to stop monitoring the component."""

    @abstractmethod
    def collect_monitoring_data(self, ctx: TestExecutionContext):
        """Abstract method to collect monitoring data for the component."""
