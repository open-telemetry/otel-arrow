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
from contextlib import nullcontext
from enum import Enum
from typing import Callable, Dict, List, Any, Optional

from opentelemetry.trace import Status, StatusCode

from ..errors.error_handler import handle_with_policy
from ..runtime.runtime import Runtime
from ..context.base import ExecutionStatus, BaseContext
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

    name: Optional[str] = None

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

    def _maybe_trace(
        self, ctx: TestStepContext, name: str, phase: HookableComponentPhase
    ):
        """
        Optionally creates a tracing context for a specific component lifecycle phase.

        Args:
            ctx (TestStepContext): The context object containing tracing and span information.
            name (str): The base name for the tracing span (e.g., component name or operation).
            phase (HookableComponentPhase): The lifecycle phase to include in the span name.

        Returns:
            ContextManager: A tracing context manager if tracing is active, otherwise a no-op context.
        """
        tracer = ctx.get_tracer("component")
        if tracer and ctx.span:
            return tracer.start_as_current_span(f"{name}: {phase.value}")
        return nullcontext()

    def _with_span(self, ctx: "BaseContext", name: str, callable_fn, *args, **kwargs):
        """
        Wrap a callable function execution within a named tracing span.

        This method attempts to create and start a tracing span with the given `name`
        using the tracer retrieved from the provided context. If tracing is active and
        the current span is recording, the callable function is executed within the
        scope of the span. Any exceptions raised during the callable's execution are
        recorded in the span, and the span status is set to error accordingly.

        If tracing is not active or the span is not recording, the callable is simply
        executed without tracing.

        Args:
            ctx (BaseContext): Context object providing tracing capabilities and
                the current active span.
            name (str): The name of the tracing span to create.
            callable_fn (Callable): The function to be executed within the span.
            *args: Positional arguments to pass to the callable function.
            **kwargs: Keyword arguments to pass to the callable function.

        Returns:
            Any: The return value of the callable function.

        Raises:
            Exception: Propagates any exceptions raised by the callable function after
                recording them in the tracing span.
        """
        tracer = ctx.get_tracer("component")
        if tracer and ctx.span and ctx.span.is_recording():
            with tracer.start_as_current_span(name) as span:
                try:
                    res = callable_fn(*args, **kwargs)
                    span.set_status(StatusCode.OK)
                    return res
                except Exception as e:
                    span.record_exception(e)
                    span.set_status(Status(StatusCode.ERROR, str(e)))
                    raise
        else:
            return callable_fn(*args, **kwargs)

    def _run_hooks(self, phase: HookableComponentPhase, ctx: TestStepContext):
        """
        Executes all hooks that are registered for a specified lifecycle phase.

        This method iterates through the list of hooks registered for the given phase and calls each hook function.

        Args:
            phase (HookableComponentPhase): The lifecycle phase during which to run the hooks (e.g., PRE_CONFIGURE, POST_CONFIGURE).
        """
        hooks = self._hooks.get(phase, [])
        if not hooks:
            return
        with self._maybe_trace(
            ctx, f"Run Component Hooks ({self.name})", phase
        ) as span:
            logger = ctx.get_logger(__name__)

            logger.debug(
                "Running %d component hooks for phase: %s", len(hooks), phase.value
            )
            for hook in hooks:
                hook_context = ComponentHookContext(
                    phase=phase,
                    name=f"{hook.__class__.__name__} ({phase.value})",
                    parent_ctx=ctx,
                )
                ctx.add_child_ctx(hook_context)
                with hook_context:
                    hook_logger = hook_context.get_logger(__name__)
                    try:
                        hook_logger.debug("Executing main hook logic...")
                        handle_with_policy(
                            hook_context,
                            lambda h=hook, hc=hook_context: h.execute(hc),
                            hook.config.on_error,
                        )
                        if hook_context.status == ExecutionStatus.RUNNING:
                            hook_context.status = ExecutionStatus.SUCCESS
                    except Exception as e:  # pylint: disable=broad-except
                        hook_context.status = ExecutionStatus.ERROR
                        hook_context.error = e
                        hook_logger.error(f"Hook failed: {e}")
                        span.set_status(
                            Status(StatusCode.ERROR, "Fatal Child Hook Failure")
                        )
                        raise
            span.set_status(StatusCode.OK)

    @abstractmethod
    def _configure(self, ctx: TestStepContext):
        """Abstract method for configuring the component."""

    def configure(self, ctx: TestStepContext):
        return self._with_span(
            ctx, f"Configure Component ({self.name})", self._configure, ctx
        )

    @abstractmethod
    def _deploy(self, ctx: TestStepContext):
        """Abstract method for deploying the component (spawn a process or start a container/deployment)."""

    def deploy(self, ctx: TestStepContext):
        return self._with_span(
            ctx, f"Deploy Component ({self.name})", self._deploy, ctx
        )

    @abstractmethod
    def _start(self, ctx: TestStepContext):
        """Abstract method for starting the component's execution behavior."""

    def start(self, ctx: TestStepContext):
        return self._with_span(ctx, f"Start Component ({self.name})", self._start, ctx)

    @abstractmethod
    def _stop(self, ctx: TestStepContext):
        """Abstract method for stopping the component's execution behavior."""

    def stop(self, ctx: TestStepContext):
        return self._with_span(ctx, f"Stop Component ({self.name})", self._stop, ctx)

    @abstractmethod
    def _destroy(self, ctx: TestStepContext):
        """Abstract method for destroying the component (e.g. kill process, stop/remove container).

        The specific signals (term/kill) and container cleanup (stop vs rm) will be dictated and
        configured by the strategy implementation and lifecycle hooks.
        """

    def destroy(self, ctx: TestStepContext):
        return self._with_span(
            ctx, f"Destroy Component ({self.name})", self._destroy, ctx
        )

    @abstractmethod
    def _start_monitoring(self, ctx: TestStepContext):
        """Abstract method to start monitoring the component."""

    def start_monitoring(self, ctx: TestStepContext):
        return self._with_span(
            ctx, f"Start Monitoring ({self.name})", self._start_monitoring, ctx
        )

    @abstractmethod
    def _stop_monitoring(self, ctx: TestStepContext):
        """Abstract method to stop monitoring the component."""

    def stop_monitoring(self, ctx: TestStepContext):
        return self._with_span(
            ctx, f"Stop Monitoring ({self.name})", self._stop_monitoring, ctx
        )

    @abstractmethod
    def _collect_monitoring_data(self, ctx: TestExecutionContext):
        """Abstract method to collect monitoring data for the component."""

    def collect_monitoring_data(self, ctx: TestExecutionContext):
        return self._with_span(
            ctx,
            f"Collect Monitoring Data ({self.name})",
            self._collect_monitoring_data,
            ctx,
        )
