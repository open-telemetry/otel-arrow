"""
Hook strategy module for recording custom telemetry events.

This module defines the `record_event` hook, which emits structured events to the
currently active telemetry span during pipeline execution. It is useful for marking
important points or metadata in the trace.

Classes:
    - RecordEventConfig: Configuration schema defining the event name and attributes.
    - RecordEventHook: Hook strategy that records an event to the context's active span.

Use case:
    Enables test authors or strategy developers to insert custom telemetry markers
    into a pipeline execution flow, which can later be analyzed or visualized.
"""

from typing import Optional

from pydantic import Field

from ....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from ....core.context.base import BaseContext
from ....core.context import ComponentHookContext, FrameworkElementHookContext
from ....runner.registry import hook_registry, PluginMeta

HOOK_NAME = "record_event"


@hook_registry.register_config(HOOK_NAME)
class RecordEventConfig(HookStrategyConfig):
    """
    Configuration class for the 'record_event' hook.

    Attributes:
        name: str - the name of the event to emit
        attributes: Optional[dict] - attributes to record with the event
    """

    name: str
    attributes: Optional[dict] = Field(default_factory=dict)


@hook_registry.register_class(HOOK_NAME)
class RecordEventHook(HookStrategy):
    """
    Hook strategy that records an event to the context's current span.

    This hook allows users to mark significant events or timestamps during test execution
    or pipeline steps by recording them as events on the active trace/span. These events
    are useful for correlating application behavior, measuring durations between steps,
    or annotating traces for observability and debugging purposes.

    Typical usage:
        - Marking the start and end of an observation window.
        - Logging test lifecycle events in distributed tracing systems.
        - Recording custom milestones in execution telemetry.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[
            FrameworkElementHookContext.__name__,
            ComponentHookContext.__name__,
        ],
        installs_hooks=[],
        yaml_example="""
tests:
  - name: Test Max Rate Logs
    steps:
      - name: Mark 10s Observation Window
        action:
          wait:
            delay_seconds: 10
        hooks:
          run:
            pre:
              - record_event:
                  name: observation_start
            post:
              - record_event:
                  name: observation_stop
""",
    )

    def __init__(self, config: RecordEventConfig):
        """
        Initialize the hook with its configuration.

        Args:
            config (RecordEventConfig): Configuration object containing event config.
        """
        self.config = config

    def execute(self, ctx: BaseContext):
        """
        Execute the action to record an event

        Args:
            ctx (BaseContext): The execution context, providing utilities like logging.

        Raises:
            RuntimeError if the context doesn't have an active span.
        """
        logger = ctx.get_logger(__name__)

        logger.debug(f"Recording custom event: {self.config.name}")
        if ctx and ctx.span and ctx.span.is_recording():
            ctx.record_event(self.config.name, **self.config.attributes)
            return
        raise RuntimeError(
            "The current context does not exist or does not have an active span."
        )
