"""
Module: context

This module provides the BaseContext class which provides shared fields common to
different implementations of Contexts throughout the orchestrator.
"""

import logging
import datetime
import json
import traceback
from enum import Enum

from contextlib import AbstractContextManager
from dataclasses import dataclass, field
from typing import Dict, List, Optional, TYPE_CHECKING

from opentelemetry.trace import Span, Status, StatusCode

from ..telemetry.telemetry_runtime import TelemetryRuntime
from ..telemetry.test_event import TestEvent
from ..telemetry.telemetry_client import TelemetryClient

if TYPE_CHECKING:
    from opentelemetry.sdk.metrics import Meter
    from opentelemetry.sdk.trace import Tracer
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
    start_time: Optional[datetime.datetime] = None
    end_time: Optional[datetime.datetime] = None
    metadata: dict = field(default_factory=dict)

    start_event_type: TestEvent = field(init=False, default=TestEvent.SUITE_START)
    end_event_type: TestEvent = field(init=False, default=TestEvent.SUITE_END)

    span: Optional[Span] = field(default=None, init=False)
    span_cm: Optional[AbstractContextManager] = field(default=None, init=False)
    span_name: Optional[str] = None

    parent_ctx: Optional["BaseContext"] = None
    child_contexts: List["BaseContext"] = field(default_factory=list)

    def __post_init__(self):
        """
        Initializes default metadata after object creation.

        Sets the 'test.ctx.type' to the class name and 'test.ctx.name' to the object's name
        in the metadata dictionary, if they are not already defined.
        """
        self.metadata.setdefault("test.ctx.type", self.__class__.__name__)
        self.metadata.setdefault("test.ctx.name", self.name)

    def __enter__(self):
        """
        Enters the context for use in a 'with' statement.

        Starts the process or operation associated with this object
        and returns the object itself for use within the context block.
        """
        self.start()
        return self

    def __exit__(self, exc_type, exc_value, _traceback):
        """
        Exits the context, handling completion or exceptions.

        Updates the context status to ERROR if an exception occurred,
        or SUCCESS if the status was still RUNNING. Finalizes the context
        by calling the end() method.

        Args:
            exc_type: The exception type, if any occurred.
            exc_value: The exception instance.
            _traceback: The traceback object.
        """
        if exc_type:
            self.status = ExecutionStatus.ERROR
            self.error = exc_value
        elif self.status == ExecutionStatus.RUNNING:
            self.status = ExecutionStatus.SUCCESS
        self.end()

    def start(self):
        """
        Marks the beginning of the context or operation.

        - Sets the execution status to RUNNING.
        - Records the current UTC time as the start time.
        - Initializes a tracing span (if a tracer is available) using the object's class name
        and optional `name` attribute for the span name.
        - Attaches contextual attributes to the tracing span.
        - Records a start event with a precise timestamp in nanoseconds.

        This method is typically called at the beginning of a timed or monitored execution block,
        such as within a context manager (`__enter__`).
        """
        self.status = ExecutionStatus.RUNNING
        self.start_time = datetime.datetime.now(tz=datetime.timezone.utc)

        tracer = self.get_tracer("test-framework")
        if tracer:
            span_name = getattr(
                self,
                "span_name",
                f"{self.__class__.__name__} - {getattr(self, 'name', 'unnamed')}",
            )
            self.span_cm = tracer.start_as_current_span(span_name)
            self.span = self.span_cm.__enter__()
            attrs = self.merge_ctx_metadata()
            for k, v in attrs.items():
                self.span.set_attribute(k, v)

        timestamp_unix_nanos = int(self.start_time.timestamp() * 1_000_000_000)
        self._record_start_event(timestamp_unix_nanos)

    def end(self):
        """
        Marks the end of the context or operation and logs its duration.

        - Records the current UTC time as the end time.
        - Logs an end event with a precise timestamp in nanoseconds.
        - If a tracing span is active:
            - Sets the span's status based on the current execution status (SUCCESS, ERROR, etc.).
            - Closes the span context manager to finalize the trace.

        This method is typically called at the end of a monitored or timed execution block,
        such as within a context manager (`__exit__`).
        """
        self.end_time = datetime.datetime.now(tz=datetime.timezone.utc)
        timestamp_unix_nanos = int(self.end_time.timestamp() * 1_000_000_000)
        self._record_end_event(timestamp_unix_nanos)
        if self.span_cm and self.span:
            if self.status == ExecutionStatus.SUCCESS:
                self.span.set_status(StatusCode.OK)
            elif self.status == ExecutionStatus.ERROR:
                self.span.set_status(
                    Status(StatusCode.ERROR, str(self.error) or "Context failed")
                )
            else:
                # Optional: Handle PENDING or other states explicitly
                self.span.set_status(Status(StatusCode.UNSET))
            self.span_cm.__exit__(None, None, None)

    @property
    def duration(self) -> Optional[float]:
        """Duration of the context between start/stop calls or none if not run/stopped."""
        if self.start_time is not None and self.end_time is not None:
            return (self.end_time - self.start_time).total_seconds()
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
            return self.parent_ctx.get_component_by_name(name)
        raise NotImplementedError("This context does not support get_component_by_name")

    def get_test_suite(self) -> Optional["TestSuite"]:
        """Get the root test suite object from a given context."""
        if hasattr(self, "parent_ctx"):
            return self.parent_ctx.get_test_suite()
        raise NotImplementedError("This context does not support get_test_suite")

    def record_event(
        self, event_name: str, timestamp_unix_nanos: Optional[int] = None, **kwargs
    ):
        """
        Record an event, enriching it with context-specific metadata.
        """
        if not self.span or not self.span.is_recording():
            return
        if self.status:
            kwargs.setdefault("test.ctx.status", self.status.value)
        if self.error:
            kwargs.setdefault("test.ctx.error", str(self.error))
        if self.duration:
            kwargs.setdefault("test.ctx.duration", self.duration)
        kwargs = self.merge_ctx_metadata(**kwargs)
        self.span.add_event(event_name, kwargs, timestamp=timestamp_unix_nanos)

    def merge_ctx_metadata(self, **kwargs: dict):
        """Merge context metadata and status with the supplied arguments."""
        if self.error:
            kwargs.setdefault("test.ctx.error", str(self.error))
        for key, value in self.metadata.items():
            if value:
                kwargs.setdefault(key, value)
        return kwargs

    def get_logger(self, logger_name: str = __name__) -> logging.LoggerAdapter:
        """
        Returns a context-aware logger with enriched metadata.

        - Retrieves a base logger using the specified logger name.
        - Merges contextual metadata from the object.
        - Adds non-empty metadata fields as 'extra' context to the logger.
        - Returns a `LoggerAdapter` that injects this context into all log records.

        Args:
            logger_name (str): The name of the logger to retrieve. Defaults to the current module's name.

        Returns:
            logging.LoggerAdapter: A logger adapter that includes contextual metadata for structured logging.
        """
        base_logger = logging.getLogger(logger_name)
        extra = {}
        attrs = self.merge_ctx_metadata()
        for k, v in attrs.items():
            if v:
                extra[k] = v
        return logging.LoggerAdapter(base_logger, extra)

    def get_tracer(
        self, name="default", runtime_name: str = TelemetryRuntime.type
    ) -> Optional["Tracer"]:
        """
        Retrieves a tracer instance from the telemetry runtime.

        - Accesses the current test suite and retrieves the specified telemetry runtime.
        - Returns a tracer identified by the given name from that runtime.
        - If the telemetry runtime is not found, returns None.

        Args:
            name (str): The name of the tracer to retrieve. Defaults to "default".
            runtime_name (str): The name/type of the telemetry runtime to use. Defaults to the class-level `TelemetryRuntime.type`.

        Returns:
            Optional[Tracer]: A tracer instance for telemetry, or None if unavailable.
        """
        ts = self.get_test_suite()
        telemetry_runtime: TelemetryRuntime = ts.get_runtime(runtime_name)
        if not telemetry_runtime:
            return
        return telemetry_runtime.get_tracer(name)

    def get_meter(
        self, name="default", runtime_name: str = TelemetryRuntime.type
    ) -> Optional["Meter"]:
        """
        Retrieves a meter instance from the telemetry runtime.

        - Accesses the current test suite and obtains the specified telemetry runtime.
        - Returns a meter identified by the given name from that runtime.
        - If the telemetry runtime is not available, returns None.

        Args:
            name (str): The name of the meter to retrieve. Defaults to "default".
            runtime_name (str): The name/type of the telemetry runtime to use. Defaults to the class-level `TelemetryRuntime.type`.

        Returns:
            Optional[Meter]: A meter instance for recording metrics, or None if unavailable.
        """
        ts = self.get_test_suite()
        telemetry_runtime: TelemetryRuntime = ts.get_runtime(runtime_name)
        if not telemetry_runtime:
            return
        return telemetry_runtime.get_meter(name)

    def get_telemetry_client(
        self, runtime_name: str = TelemetryRuntime.type
    ) -> Optional[TelemetryClient]:
        """
        Retrieves the telemetry client from the specified telemetry runtime.

        - Accesses the current test suite to retrieve the telemetry runtime by name.
        - Returns the telemetry client associated with that runtime.
        - If the runtime is not found, returns None.

        Args:
            runtime_name (str): The name/type of the telemetry runtime to use.
                                Defaults to the class-level `TelemetryRuntime.type`.

        Returns:
            Optional[TelemetryClient]: The telemetry client instance, or None if unavailable.
        """
        ts = self.get_test_suite()
        telemetry_runtime: TelemetryRuntime = ts.get_runtime(runtime_name)
        if not telemetry_runtime:
            return
        return telemetry_runtime.get_client()

    def get_metadata(self) -> dict:
        """Return metadata specific to this context level (e.g. test_name, test_step)."""
        return self.metadata

    def _record_start_event(self, timestamp_unix_nanos: Optional[int]):
        """Hook to record a context-specific start event."""
        self.record_event(
            self.start_event_type.namespaced(), timestamp=timestamp_unix_nanos
        )

    def _record_end_event(self, timestamp_unix_nanos: Optional[int]):
        """Hook to record a context-specific end event."""
        self.record_event(
            self.end_event_type.namespaced(), timestamp=timestamp_unix_nanos
        )

    @staticmethod
    def _format_time(timestamp: Optional[datetime.datetime]) -> str:
        if timestamp is None:
            return "None"
        return timestamp.isoformat()

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
            "start_time": self._format_time(self.start_time),
            "end_time": self._format_time(self.end_time),
            "duration": self.duration,
            "metadata": self.metadata,
            "child_contexts": [child.to_dict() for child in self.child_contexts],
        }
