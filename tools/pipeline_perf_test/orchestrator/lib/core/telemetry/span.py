"""
This module defines abstractions and concrete implementations for managing,
querying, and exporting telemetry span and span event data based on OpenTelemetry SDK data structures.

Key components:

- SpanRow: TypedDict defining the schema of a single span data row, including
  span identifiers, timing, status, kind, attributes, and associated resource attributes.

- SpanEventRow: TypedDict defining the schema of a single span event data row,
  capturing event name, timestamp, attributes, and associated span ID.

- SpanDataFrame and SpanEventDataFrame: Subclasses of pandas.DataFrame specialized
  for spans and span events respectively, with schema validation and flexible querying
  capabilities based on span properties, events, and associated attributes.

- SpanDataBackend: Abstract base class defining the interface for backends that
  provide normalized span and span event data as SpanDataFrame and SpanEventDataFrame
  objects. This facilitates implementations that can fetch spans from various sources,
  including in-memory stores and remote tracing backends.

- SpanRetriever: Abstract interface extending SignalRetriever for querying spans
  and span events with rich filtering on multiple attributes, time ranges, durations,
  and custom predicates.

- FrameworkSpanBackend: Thread-safe in-memory backend implementation that stores
  raw ReadableSpan objects and converts them to SpanDataFrame and SpanEventDataFrame
  on demand, with caching. Provides efficient local span storage and retrieval.

- FrameworkSpanRetriever: Concrete SpanRetriever implementation that queries spans
  and events from a FrameworkSpanBackend.

- FrameworkSpanExporter: SpanExporter implementation that exports ReadableSpan objects
  into a FrameworkSpanBackend.

Potential Use with Remote Trace Stores:
---------------------------------------
While this module primarily implements an in-memory backend, the abstraction
via SpanDataBackend and SpanRetriever allows for easy extension to remote or
persistent tracing systems. Custom backends could be built to fetch spans and
events from distributed tracing platforms, log aggregators, or cloud-based
trace storage, returning data in the standardized SpanDataFrame and SpanEventDataFrame
formats to enable consistent querying and analysis by monitoring and diagnostics tools.
"""

from typing import List, Any, TypedDict, Dict, Optional, Union, Callable, get_type_hints
from abc import ABC, abstractmethod
import pandas as pd
import threading

from opentelemetry.sdk.trace import ReadableSpan
from opentelemetry.sdk.trace.export import SpanExporter, SpanExportResult
from .signal_retriever import SignalRetriever


class SpanRow(TypedDict):
    """
    Represents a single row of telemetry span data, a simplified version
    of an opentelemetry span.

    Attributes:
        name (str): The name of the span.
        trace_id (str): The identifier of the trace to which this span belongs.
        span_id (str): The unique identifier of this span.
        parent_id (str | None): The span_id of the parent span, or None if root span.
        start_time (pd.Timestamp): The timestamp when the span started.
        end_time (pd.Timestamp): The timestamp when the span ended.
        duration_ms (float): Duration of the span in milliseconds.
        status_code (str): The status code indicating the span's outcome (e.g., OK, ERROR).
        kind (str): The kind/type of span (e.g., SERVER, CLIENT).
        attributes (Dict[str, Any]): Key-value pairs of span-specific attributes.
        resource (Dict[str, Any]): Key-value pairs of resource-level attributes associated with the span.
    """

    name: str
    trace_id: str
    span_id: str
    parent_id: str | None
    start_time: pd.Timestamp
    end_time: pd.Timestamp
    duration_ms: float
    status_code: str
    kind: str
    attributes: Dict[str, Any]
    resource: Dict[str, Any]


class SpanEventRow(TypedDict):
    """
    TypedDict representing a single event within a span.

    Attributes:
        name (str): The name of the event.
        span_id (str): The identifier of the span this event belongs to.
        timestamp (pd.Timestamp): The timestamp when the event occurred.
        attributes (Dict[str, Any]): Key-value pairs of event-specific attributes.
    """

    name: str
    span_id: str
    timestamp: pd.Timestamp
    attributes: Dict[str, Any]


class SpanEventDataFrame(pd.DataFrame):
    """
    A pandas DataFrame subclass specialized for span event data.

    Provides schema validation and flexible querying capabilities for
    telemetry span events based on event properties and attributes.
    """

    @property
    def _constructor(self):
        """
        Ensures that DataFrame operations return a SpanEventDataFrame instance
        rather than a generic pandas DataFrame.
        """
        return SpanEventDataFrame

    @property
    def required_columns(self) -> list[str]:
        """
        List of required column names as defined by the SpanEventRow schema.

        Returns:
            List of column names that must be present in the DataFrame.
        """
        return list(get_type_hints(SpanEventRow).keys())

    def validate_schema(self):
        """
        Validates that all required columns defined in required_columns are present.

        Raises:
            ValueError: If any required column is missing.
        """
        missing = set(self.required_columns) - set(self.columns)
        if missing:
            raise ValueError(f"Missing required columns: {missing}")

    def query_span_events(
        self,
        name: Optional[Union[str, list[str]]] = None,
        span_id: Optional[str] = None,
        time_range: Optional[tuple[pd.Timestamp, pd.Timestamp]] = None,
        attributes: Optional[Dict[str, Any]] = None,
        where: Optional[Callable[["SpanEventDataFrame"], "SpanEventDataFrame"]] = None,
    ) -> "SpanEventDataFrame":
        """
        Query span events based on various filtering criteria.

        Args:
            name: Event name or list of event names to filter.
            span_id: Span ID to filter events belonging to a specific span.
            time_range: Tuple of (start, end) timestamps to filter events by occurrence time.
            attributes: Dictionary of attribute key-value pairs to filter events.
            where: Optional callable for additional custom filtering on the DataFrame.

        Returns:
            SpanEventDataFrame: Filtered subset of span events matching criteria.
        """
        df = self.copy()

        if name:
            if isinstance(name, str):
                df = df[df["name"] == name]
            else:
                df = df[df["name"].isin(name)]

        if span_id:
            df = df[df["span_id"] == span_id]

        if time_range:

            def ensure_utc(ts: pd.Timestamp) -> pd.Timestamp:
                ts = pd.Timestamp(ts)
                return (
                    ts.tz_localize("UTC") if ts.tzinfo is None else ts.tz_convert("UTC")
                )

            start, end = time_range
            start = ensure_utc(start)
            end = ensure_utc(end)
            df = df[(df["timestamp"] >= start) & (df["timestamp"] <= end)]

        def dict_filter(attr_filter: Dict[str, Any]):
            return lambda d: isinstance(d, dict) and all(
                d.get(k) == v for k, v in attr_filter.items()
            )

        if attributes:
            df = df[df["attributes"].apply(dict_filter(attributes))]

        if where:
            df = where(df)

        return SpanEventDataFrame(df)


class SpanDataFrame(pd.DataFrame):
    """
    A pandas DataFrame subclass specialized for telemetry span data.

    Provides schema validation, duration calculation, and flexible querying
    capabilities based on span properties and attributes.
    """

    @property
    def _constructor(self):
        """
        Ensures that pandas DataFrame operations return a SpanDataFrame instance
        instead of a generic DataFrame.
        """
        return SpanDataFrame

    @property
    def required_columns(self) -> list[str]:
        """
        Returns the list of required column names as defined by the SpanRow schema.

        Returns:
            List[str]: List of required column names for the span DataFrame.
        """
        return list(get_type_hints(SpanRow).keys())

    def validate_schema(self):
        """
        Validates that all required columns are present in the DataFrame.

        Raises:
            ValueError: If any required columns are missing.
        """
        missing = set(self.required_columns) - set(self.columns)
        if missing:
            raise ValueError(f"Missing required columns: {missing}")

    def with_durations(self) -> "SpanDataFrame":
        """
        Calculates the duration in milliseconds for each span and adds
        it as a 'duration_ms' column.

        Returns:
            SpanDataFrame: The DataFrame with the added 'duration_ms' column.
        """
        self["duration_ms"] = (
            self["end_time"] - self["start_time"]
        ).dt.total_seconds() * 1000
        return self

    def query_spans(
        self,
        name: Optional[Union[str, list[str]]] = None,
        trace_id: Optional[str] = None,
        span_id: Optional[str] = None,
        parent_id: Optional[str] = None,
        time_range: Optional[tuple[pd.Timestamp, pd.Timestamp]] = None,
        duration_range: Optional[tuple[float, float]] = None,
        status_code: Optional[str] = None,
        kind: Optional[str] = None,
        attributes: Optional[Dict[str, Any]] = None,
        resource: Optional[Dict[str, Any]] = None,
        where: Optional[Callable[["SpanDataFrame"], "SpanDataFrame"]] = None,
    ) -> "SpanDataFrame":
        """
        Query spans filtering by multiple optional criteria.

        Args:
            name: Span name or list of span names to filter by.
            trace_id: Trace identifier to filter spans from a specific trace.
            span_id: Span identifier to filter a specific span.
            parent_id: Parent span ID to filter child spans.
            time_range: Tuple of (start, end) timestamps to filter spans by start time.
            duration_range: Tuple of (min, max) duration in milliseconds to filter spans.
            status_code: Status code string to filter spans by their outcome.
            kind: Span kind/type (e.g., SERVER, CLIENT) to filter spans.
            attributes: Dictionary of attribute key-value pairs to filter spans.
            resource: Dictionary of resource-level attributes to filter spans.
            where: Optional callable for additional custom filtering on the DataFrame.

        Returns:
            SpanDataFrame: Filtered spans matching the criteria.
        """
        df = self.copy()

        if name:
            if isinstance(name, str):
                df = df[df["name"] == name]
            else:
                df = df[df["name"].isin(name)]

        if trace_id:
            df = df[df["trace_id"] == trace_id]

        if span_id:
            df = df[df["span_id"] == span_id]

        if parent_id:
            df = df[df["parent_id"] == parent_id]

        if time_range:

            def ensure_utc(ts: pd.Timestamp) -> pd.Timestamp:
                ts = pd.Timestamp(ts)
                return (
                    ts.tz_localize("UTC") if ts.tzinfo is None else ts.tz_convert("UTC")
                )

            start, end = time_range
            start = ensure_utc(start)
            end = ensure_utc(end)
            df = df[(df["start_time"] >= start) & (df["end_time"] <= end)]

        if duration_range:
            dur_min, dur_max = duration_range
            df = df[(df["duration_ms"] >= dur_min) & (df["duration_ms"] <= dur_max)]

        if status_code:
            df = df[df["status_code"] == status_code]

        if kind:
            df = df[df["kind"] == kind]

        def dict_filter(attr_filter: Dict[str, Any]):
            return lambda d: isinstance(d, dict) and all(
                d.get(k) == v for k, v in attr_filter.items()
            )

        if attributes:
            df = df[df["attributes"].apply(dict_filter(attributes))]

        if resource:
            df = df[df["resource"].apply(dict_filter(resource))]

        if where:
            df = where(df)

        return SpanDataFrame(df)


class SpanDataBackend(ABC):
    """
    Abstract base class defining the interface for span data backends.

    Implementations should provide access to span and span event data
    as normalized pandas DataFrames with defined schemas.
    """

    @abstractmethod
    def get_spans_df(self) -> SpanDataFrame:
        """
        Retrieve span data as a normalized DataFrame.

        Returns:
            SpanDataFrame: DataFrame containing spans with columns
            defined by the SpanRow schema.
        """

    @abstractmethod
    def get_events_df(self) -> SpanEventDataFrame:
        """
        Retrieve span event data as a normalized DataFrame.

        Returns:
            SpanEventDataFrame: DataFrame containing span events with columns
            defined by the SpanEventRow schema.
        """


class SpanRetriever(SignalRetriever):
    """
    Abstract interface for querying span and span event telemetry data.

    Implementations should provide methods to query spans and span events
    with flexible filtering options.
    """

    @abstractmethod
    def query_spans(
        self,
        name: Optional[Union[str, list[str]]] = None,
        trace_id: Optional[str] = None,
        span_id: Optional[str] = None,
        parent_id: Optional[str] = None,
        time_range: Optional[tuple[pd.Timestamp, pd.Timestamp]] = None,
        duration_range: Optional[tuple[float, float]] = None,
        status_code: Optional[str] = None,
        kind: Optional[str] = None,
        attributes: Optional[Dict[str, Any]] = None,
        resource: Optional[Dict[str, Any]] = None,
        where: Optional[Callable[["SpanDataFrame"], "SpanDataFrame"]] = None,
    ) -> "SpanDataFrame":
        """
        Query spans based on multiple optional filtering criteria.

        Args:
            name: Span name or list of names to filter.
            trace_id: Filter by trace identifier.
            span_id: Filter by specific span identifier.
            parent_id: Filter by parent span identifier.
            time_range: Tuple (start, end) timestamps to filter spans by start time.
            duration_range: Tuple (min, max) duration in milliseconds to filter spans.
            status_code: Filter by span status code.
            kind: Filter by span kind/type.
            attributes: Span attribute key-value pairs to filter on.
            resource: Resource-level attribute key-value pairs to filter on.
            where: Optional callable for custom additional filtering.

        Returns:
            SpanDataFrame: DataFrame containing spans matching the filters.
        """

    @abstractmethod
    def query_span_events(
        self,
        name: Optional[Union[str, list[str]]] = None,
        span_id: Optional[str] = None,
        time_range: Optional[tuple[pd.Timestamp, pd.Timestamp]] = None,
        attributes: Optional[Dict[str, Any]] = None,
        where: Optional[Callable[["SpanEventDataFrame"], "SpanEventDataFrame"]] = None,
    ) -> "SpanEventDataFrame":
        """
        Query span events based on optional filtering criteria.

        Args:
            name: Event name or list of names to filter.
            span_id: Filter events by span identifier.
            time_range: Tuple (start, end) timestamps to filter events by time.
            attributes: Event attribute key-value pairs to filter on.
            where: Optional callable for custom additional filtering.

        Returns:
            SpanEventDataFrame: DataFrame containing span events matching the filters.
        """


class FrameworkSpanBackend(SpanDataBackend):
    """
    Thread-safe in-memory backend for storing and querying span data.

    This backend collects spans (ReadableSpan instances), caches
    normalized pandas DataFrames for spans and span events, and
    provides access methods for those DataFrames.

    The caches are invalidated on new data addition to ensure
    up-to-date queries.
    """

    def __init__(self):
        """
        Initialize the backend with empty storage and thread lock.
        """
        self._spans: List[ReadableSpan] = []
        self._df_cache: SpanDataFrame | None = None
        self._events_df_cache: pd.DataFrame | None = None
        self._lock = threading.Lock()

    def add(self, span: ReadableSpan):
        """
        Add a new span to the backend.

        Args:
            span: A ReadableSpan instance to add.

        The internal caches for span and event DataFrames are invalidated
        upon addition to reflect the new data.
        """
        with self._lock:
            self._spans.append(span)
            self._df_cache = None  # Invalidate cache
            self._events_df_cache = None  # Invalidate cache

    def get_spans_df(self) -> SpanDataFrame:
        """
        Retrieve a copy of the cached span DataFrame.

        If the cache is empty or invalidated, regenerate the DataFrame
        from stored spans.

        Returns:
            SpanDataFrame: DataFrame containing all stored spans.
        """
        with self._lock:
            if self._df_cache is None:
                self._generate_dataframes()
            return self._df_cache.copy()

    def get_events_df(self) -> SpanEventDataFrame:
        """
        Retrieve a copy of the cached span event DataFrame.

        If the cache is empty or invalidated, regenerate the DataFrame
        from stored spans' events.

        Returns:
            SpanEventDataFrame: DataFrame containing all stored span events.
        """
        with self._lock:
            if self._events_df_cache is None:
                self._generate_dataframes()
            return self._events_df_cache.copy()

    def _generate_dataframes(self):
        """
        Regenerate the span and span event DataFrames from the stored spans.

        This method extracts relevant data from each span and its events,
        builds row dictionaries, and constructs validated SpanDataFrame
        and SpanEventDataFrame instances.
        """
        rows = [self._span_to_row(span) for span in self._spans]
        new_df = SpanDataFrame(rows)
        self._df_cache = new_df
        self._df_cache.validate_schema()

        event_rows = []
        for span in self._spans:
            for event in span.events:
                event_rows.append(
                    {
                        "span_id": f"{span.context.span_id:016x}",
                        "timestamp": event.timestamp,
                        "name": event.name,
                        "attributes": dict(event.attributes),
                    }
                )

        self._events_df_cache = SpanEventDataFrame(event_rows)
        self._events_df_cache.validate_schema()

    def _span_to_row(self, span: ReadableSpan) -> Dict[str, Any]:
        """
        Helper method to convert a ReadableSpan instance into a dictionary
        matching the SpanRow schema.

        Args:
            span: The ReadableSpan to convert.

        Returns:
            Dict[str, Any]: A dictionary representing the span suitable
            for DataFrame construction.
        """
        context = span.get_span_context()
        parent_id = span.parent.span_id if span.parent else None

        start_time = span.start_time
        end_time = span.end_time
        duration_ms = (end_time - start_time) * 1000

        return {
            "name": span.name,
            "trace_id": f"{context.trace_id:032x}",
            "span_id": f"{context.span_id:016x}",
            "parent_id": f"{parent_id:016x}" if parent_id else None,
            "start_time": start_time,
            "end_time": end_time,
            "duration_ms": round(duration_ms, 2),
            "status_code": span.status.status_code.name,
            "kind": str(span.kind.name),
            "attributes": dict(span.attributes),
            "resource": dict(span.resource.attributes),
        }


class FrameworkSpanExporter(SpanExporter):
    """
    SpanExporter implementation that exports opentelemetry spans into a FrameworkSpanBackend.

    This exporter collects incoming spans and adds them to the provided backend,
    which manages storage and querying of span data.
    """

    def __init__(self, backend: FrameworkSpanBackend):
        """
        Initialize the exporter with a FrameworkSpanBackend instance.

        Args:
            backend: The FrameworkSpanBackend to which spans will be exported.
        """
        self.backend = backend

    def export(self, spans: List[ReadableSpan]) -> SpanExportResult:
        """
        Export a list of spans by adding each to the backend.

        Args:
            spans: List of ReadableSpan instances to export.

        Returns:
            SpanExportResult: Always returns SUCCESS after adding spans.
        """
        for span in spans:
            self.backend.add(span)
        return SpanExportResult.SUCCESS


class FrameworkSpanRetriever(SpanRetriever):
    """
    SpanRetriever implementation that queries spans and span events
    from a FrameworkSpanBackend.

    This class provides filtering and querying capabilities over the
    spans and span events stored in the backend, returning dataframes
    matching the specified criteria.
    """

    def __init__(self, backend: FrameworkSpanBackend):
        """
        Initialize the retriever with a FrameworkSpanBackend.

        Args:
            backend: The backend instance to query spans and events from.
        """
        self.backend = backend

    def get_schema(self) -> Dict[str, Any]:
        """
        Return the schema of the SpanRow as a dict of field names and types.

        Returns:
            Dict[str, Any]: The schema/type hints of SpanRow fields.
        """
        return get_type_hints(SpanRow)

    def query_spans(
        self,
        name: Optional[Union[str, list[str]]] = None,
        trace_id: Optional[str] = None,
        span_id: Optional[str] = None,
        parent_id: Optional[str] = None,
        time_range: Optional[tuple[pd.Timestamp, pd.Timestamp]] = None,
        duration_range: Optional[tuple[float, float]] = None,
        status_code: Optional[str] = None,
        kind: Optional[str] = None,
        attributes: Optional[Dict[str, Any]] = None,
        resource: Optional[Dict[str, Any]] = None,
        where: Optional[Callable[["SpanDataFrame"], "SpanDataFrame"]] = None,
    ) -> "SpanDataFrame":
        """
        Query spans from the backend, filtered by the provided criteria.

        Args:
            name: Filter by span name(s).
            trace_id: Filter by trace ID.
            span_id: Filter by span ID.
            parent_id: Filter by parent span ID.
            time_range: Tuple of start and end timestamps to filter spans.
            duration_range: Tuple of min and max duration (ms) to filter spans.
            status_code: Filter by span status code.
            kind: Filter by span kind.
            attributes: Filter by span attributes.
            resource: Filter by resource attributes.
            where: Optional callable for further filtering on the DataFrame.

        Returns:
            SpanDataFrame: DataFrame containing matching spans.
        """
        return self.backend.get_spans_df().query_spans(
            name=name,
            trace_id=trace_id,
            span_id=span_id,
            parent_id=parent_id,
            time_range=time_range,
            duration_range=duration_range,
            status_code=status_code,
            kind=kind,
            attributes=attributes,
            resource=resource,
            where=where,
        )

    def query_span_events(
        self,
        name: Optional[Union[str, list[str]]] = None,
        span_id: Optional[str] = None,
        time_range: Optional[tuple[pd.Timestamp, pd.Timestamp]] = None,
        attributes: Optional[Dict[str, Any]] = None,
        where: Optional[Callable[["SpanEventDataFrame"], "SpanEventDataFrame"]] = None,
    ) -> "SpanEventDataFrame":
        """
        Query span events from the backend, filtered by the provided criteria.

        Args:
            name: Filter by event name(s).
            span_id: Filter by associated span ID.
            time_range: Tuple of start and end timestamps to filter events.
            attributes: Filter by event attributes.
            where: Optional callable for further filtering on the DataFrame.

        Returns:
            SpanEventDataFrame: DataFrame containing matching span events.
        """
        return self.backend.get_events_df().query_span_events(
            name=name,
            span_id=span_id,
            time_range=time_range,
            attributes=attributes,
            where=where,
        )
