"""
This module defines abstractions and concrete implementations for managing,
querying, and exporting telemetry metric data based on OpenTelemetry SDK data structures.

Key components:

- MetricRow: TypedDict defining the schema of a single metric data row, including
  timestamp, metric name and type, value, and associated resource, scope, and metric attributes.

- MetricDataFrame: Subclass of pandas.DataFrame specialized for metric data, with schema
  validation and flexible querying capabilities based on metric properties and attributes.

- MetricDataBackend: Abstract base class defining the interface for backends that
  provide normalized metric data as MetricDataFrame objects. This design facilitates
  implementations that can fetch metrics from diverse sources, including both
  in-memory storage and remote metrics stores.

- MetricsRetriever: Abstract interface extending SignalRetriever for querying metrics
  with filtering on multiple attributes and time ranges.

- FrameworkMetricBackend: Thread-safe in-memory backend implementation that stores
  raw MetricsData objects and converts them to MetricDataFrame on demand, with caching.
  Serves as a local, efficient metric storage solution.

- FrameworkMetricsRetriever: Concrete MetricsRetriever implementation that queries metrics
  from a FrameworkMetricBackend.

- FrameworkMetricExporter: MetricExporter implementation that exports MetricsData
  to a FrameworkMetricBackend, supporting preferred aggregation and temporality.

Potential Use with Remote Metrics Stores:
-----------------------------------------
While this module primarily implements an in-memory backend, its design via the
MetricDataBackend and MetricsRetriever abstractions allows for easy extension to
remote or persistent metric storage solutions. For example, custom backends could be
implemented to query metrics from distributed monitoring systems, time-series
databases, or cloud telemetry services, returning data in the standardized
MetricDataFrame format for consistent querying and analysis by reporting plugins.
"""

import threading
from typing import List, Any, TypedDict, Dict, Union, get_type_hints, Optional, Callable

from abc import ABC, abstractmethod

import pandas as pd
from opentelemetry.sdk.metrics.export import (
    MetricsData,
    MetricExporter,
    MetricExportResult,
    AggregationTemporality,
    Sum,
    Gauge,
    Histogram,
    ExponentialHistogram,
)

from .signal_retriever import SignalRetriever


class MetricRow(TypedDict):
    """
    Represents a single row of telemetry metric data, a simplified version
    of an opentelemetry metric datapoint.

    Attributes:
        timestamp (pd.Timestamp): The time at which the metric was recorded.
        metric_name (str): The name of the metric.
        metric_type (str): The type of the metric (e.g., counter, gauge).
        value (Union[int, float, dict, None]): The recorded value of the metric.
            May be numeric, a structured value (e.g., histogram), or None.
        resource_attributes (Dict[str, Any]): Attributes describing the resource
            that emitted the metric (e.g., host, service name).
        scope_attributes (Dict[str, Any]): Attributes from the instrumentation scope
            (e.g., library name and version).
        metric_attributes (Dict[str, Any]): Custom attributes associated with
            the metric itself (e.g., labels or tags).
    """

    timestamp: pd.Timestamp
    metric_name: str
    metric_type: str
    value: Union[int, float, dict, None]
    resource_attributes: Dict[str, Any]
    scope_attributes: Dict[str, Any]
    metric_attributes: Dict[str, Any]


class MetricDataFrame(pd.DataFrame):
    """
    A pandas DataFrame subclass specialized for working with telemetry metric data.

    This class provides schema validation and convenient querying capabilities
    for data structured according to the MetricRow schema, a simplified version of
    a flattened otel metric datapoint.

    Properties:
        required_columns (list[str]): The list of required column names as defined
            by the MetricRow type hint.

    Methods:
        validate_schema():
            Ensures that all required columns from the MetricRow definition
            are present in the DataFrame. Raises a ValueError if any are missing.

        query_metrics(...):
            Returns a filtered MetricDataFrame based on provided criteria like
            metric name, type, time range, and specific resource/scope/metric
            attributes. An optional `where` callable allows for custom filtering
            logic.
    """

    @property
    def _constructor(self):
        return MetricDataFrame

    @property
    def required_columns(self) -> list[str]:
        return list(get_type_hints(MetricRow).keys())

    def validate_schema(self):
        missing = set(self.required_columns) - set(self.columns)
        if missing:
            raise ValueError(f"Missing required columns: {missing}")

    def query_metrics(
        self,
        metric_name: Optional[Union[str, list[str]]] = None,
        metric_type: Optional[str] = None,
        time_range: Optional[tuple[pd.Timestamp, pd.Timestamp]] = None,
        resource_attrs: Optional[Dict[str, Any]] = None,
        scope_attrs: Optional[Dict[str, Any]] = None,
        metric_attrs: Optional[Dict[str, Any]] = None,
        where: Optional[Callable[["MetricDataFrame"], "MetricDataFrame"]] = None,
    ) -> "MetricDataFrame":
        df = self.copy()

        if metric_name:
            if isinstance(metric_name, str):
                df = df[df["metric_name"] == metric_name]
            else:
                df = df[df["metric_name"].isin(metric_name)]

        if metric_type:
            df = df[df["metric_type"] == metric_type]

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

        if resource_attrs:
            df = df[df["resource_attributes"].apply(dict_filter(resource_attrs))]

        if scope_attrs:
            df = df[df["scope_attributes"].apply(dict_filter(scope_attrs))]

        if metric_attrs:
            df = df[df["metric_attributes"].apply(dict_filter(metric_attrs))]

        if where:
            df = where(df)

        return MetricDataFrame(df)


class MetricDataBackend(ABC):
    """
    Abstract interface for a backend that provides access to metric data.

    Implementations of this interface must define how to retrieve metric data
    as a normalized MetricDataFrame, which conforms to the MetricRow schema.

    Methods:
        get_metrics_df() -> MetricDataFrame:
            Retrieve the metric data in a structured DataFrame format.
    """

    @abstractmethod
    def get_metrics_df(self) -> MetricDataFrame:
        """Returns metrics data as a normalized DataFrame with MetricRow columns"""


class MetricsRetriever(SignalRetriever):
    """
    Abstract interface for querying telemetry metrics.

    Subclasses must implement the `query_metrics` method, which allows flexible
    filtering of metrics based on name, type, time range, and attribute values.
    This interface builds on the SignalRetriever base class and is specialized
    for metric data.

    Methods:
        query_metrics(...):
            Retrieve a MetricDataFrame filtered according to the provided criteria.
    """

    @abstractmethod
    def query_metrics(
        self,
        metric_name: Optional[Union[str, list[str]]] = None,
        metric_type: Optional[str] = None,
        time_range: Optional[tuple[pd.Timestamp, pd.Timestamp]] = None,
        resource_attrs: Optional[Dict[str, Any]] = None,
        scope_attrs: Optional[Dict[str, Any]] = None,
        metric_attrs: Optional[Dict[str, Any]] = None,
        where: Optional[Callable[["MetricDataFrame"], "MetricDataFrame"]] = None,
    ) -> "MetricDataFrame":
        """Returns a MetricDataFrame matching the specified query."""


class FrameworkMetricBackend(MetricDataBackend):
    """
    In-memory backend for storing and retrieving telemetry metrics in a thread-safe manner.

    This implementation of MetricDataBackend accumulates OpenTelemetry MetricsData
    objects in memory and exposes them as a normalized MetricDataFrame. It uses
    a cached DataFrame representation that is regenerated only when new data is added.

    Thread safety is ensured via a lock around mutation and read access.

    Attributes:
        _metrics (List[MetricsData]): Internal list storing raw otel MetricData objects
        _df_cache (MetricDataFrame | None): Cached DataFrame representation of the metrics.
        _lock (threading.Lock): Ensures thread-safe access to internal state.

    Methods:
        add(metric_data: MetricsData):
            Adds new metric data and invalidates the cached DataFrame.

        get_metrics_df() -> MetricDataFrame:
            Returns a copy of the cached metric DataFrame. If the cache is invalid,
            it regenerates the DataFrame from stored data.
    """

    def __init__(self):
        self._metrics: List[MetricsData] = []
        self._df_cache: MetricDataFrame | None = None
        self._lock = threading.Lock()

    def add(self, metric_data: MetricsData):
        """
        Add new metric data to the backend.

        This method appends the provided MetricsData object to the internal
        storage and invalidates the cached DataFrame, ensuring the next
        retrieval reflects the updated data.

        Thread-safe via an internal lock.

        Args:
            metric_data (MetricsData): The metric data to add.
        """
        with self._lock:
            self._metrics.append(metric_data)
            self._df_cache = None  # Invalidate cache

    def get_metrics_df(self) -> MetricDataFrame:
        """
        Retrieve the metrics as a MetricDataFrame.

        If the internal cached DataFrame is invalid or missing, this method
        regenerates it by processing the stored MetricsData objects.
        Returns a copy of the cached DataFrame to prevent external mutation.

        Thread-safe via an internal lock.

        Returns:
            MetricDataFrame: A DataFrame containing all stored metric data.
        """
        with self._lock:
            if self._df_cache is None:
                self._generate_dataframes()
            return self._df_cache.copy()

    def _generate_dataframes(self):
        """
        Convert stored MetricsData objects into a normalized MetricDataFrame.

        Iterates through all stored MetricsData, extracting and flattening metric
        data points along with their associated resource, scope, and metric attributes.
        Supports multiple metric data types including Sum, Gauge, Histogram, and
        ExponentialHistogram, converting each data point into a dictionary row.

        Each row contains:
            - timestamp: The time of the metric data point (in UTC).
            - metric_name: The name of the metric.
            - metric_type: The type of metric (e.g., "Sum", "Gauge", "Histogram", etc.).
            - value: The metric value or a dictionary of aggregated values depending on type.
            - resource_attributes: Attributes of the resource emitting the metric.
            - scope_attributes: Attributes of the instrumentation scope.
            - metric_attributes: Attributes specific to the metric data point.

        After collecting all rows, the method constructs a MetricDataFrame and validates
        that it conforms to the expected schema.

        This method should only be called internally and is not thread-safe on its own;
        callers should ensure synchronization.
        """
        rows = []
        for metrics_data in self._metrics:
            for resource_metrics in metrics_data.resource_metrics:
                resource_attrs = dict(resource_metrics.resource.attributes)

                for scope_metrics in resource_metrics.scope_metrics:
                    scope = scope_metrics.scope
                    scope_attrs = {
                        "scope_name": scope.name,
                        "scope_version": scope.version,
                    }

                    for metric in scope_metrics.metrics:
                        metric_name = metric.name
                        metric_data = metric.data

                        # Determine metric type and handle accordingly
                        if isinstance(metric_data, (Sum, Gauge)):
                            metric_type = (
                                "Sum" if isinstance(metric_data, Sum) else "Gauge"
                            )
                            for dp in metric_data.data_points:
                                ts = pd.to_datetime(
                                    dp.time_unix_nano, unit="ns", utc=True
                                )
                                attrs = dict(dp.attributes)
                                rows.append(
                                    {
                                        "timestamp": ts,
                                        "metric_name": metric_name,
                                        "metric_type": metric_type,
                                        "value": dp.value,
                                        "resource_attributes": resource_attrs,
                                        "scope_attributes": scope_attrs,
                                        "metric_attributes": attrs,
                                    }
                                )

                        elif isinstance(metric_data, Histogram):
                            metric_type = "Histogram"
                            for dp in metric_data.data_points:
                                ts = pd.to_datetime(
                                    dp.time_unix_nano, unit="ns", utc=True
                                )
                                attrs = dict(dp.attributes)
                                value = {
                                    "count": dp.count,
                                    "sum": dp.sum,
                                    "buckets": list(dp.bucket_counts),
                                    "boundaries": list(dp.explicit_bounds),
                                    "min": dp.min,
                                    "max": dp.max,
                                }
                                rows.append(
                                    {
                                        "timestamp": ts,
                                        "metric_name": metric_name,
                                        "metric_type": metric_type,
                                        "value": value,
                                        "resource_attributes": resource_attrs,
                                        "scope_attributes": scope_attrs,
                                        "metric_attributes": attrs,
                                    }
                                )

                        elif isinstance(metric_data, ExponentialHistogram):
                            metric_type = "ExponentialHistogram"
                            for dp in metric_data.data_points:
                                ts = pd.to_datetime(
                                    dp.time_unix_nano, unit="ns", utc=True
                                )
                                attrs = dict(dp.attributes)
                                value = {
                                    "count": dp.count,
                                    "sum": dp.sum,
                                    "scale": dp.scale,
                                    "zero_count": dp.zero_count,
                                    "positive": {
                                        "offset": dp.positive.offset,
                                        "bucket_counts": list(
                                            dp.positive.bucket_counts
                                        ),
                                    },
                                    "negative": {
                                        "offset": dp.negative.offset,
                                        "bucket_counts": list(
                                            dp.negative.bucket_counts
                                        ),
                                    },
                                    "min": dp.min,
                                    "max": dp.max,
                                }
                                rows.append(
                                    {
                                        "timestamp": ts,
                                        "metric_name": metric_name,
                                        "metric_type": metric_type,
                                        "value": value,
                                        "resource_attributes": resource_attrs,
                                        "scope_attributes": scope_attrs,
                                        "metric_attributes": attrs,
                                    }
                                )
                        else:
                            # Unknown metric type
                            continue
        self._df_cache = MetricDataFrame(rows)
        self._df_cache.validate_schema()


class FrameworkMetricExporter(MetricExporter):
    """
    A optentelemetry SDK MetricExporter implementation that exports metrics to an in-memory backend.

    This exporter collects OpenTelemetry MetricsData and stores them in a
    FrameworkMetricBackend instance. It supports preferred temporality and
    aggregation settings, delegating actual metric storage and querying to the backend.

    Thread safety is ensured by using a lock around modifications to the backend.

    Attributes:
        backend (FrameworkMetricBackend): The backend where metrics are stored.
        lock (threading.Lock): Ensures thread-safe access when exporting metrics.

    Args:
        backend (FrameworkMetricBackend): The metric storage backend.
        preferred_temporality (Optional[Dict[type, AggregationTemporality]]): Preferred
            aggregation temporality for each instrument type.
        preferred_aggregation (Optional[Dict[type, Aggregation]]): Preferred aggregation
            type for each instrument type.
    """

    def __init__(
        self,
        backend: FrameworkMetricBackend,
        preferred_temporality: dict[type, AggregationTemporality] | None = None,
        preferred_aggregation: (
            dict[type, "opentelemetry.sdk.metrics.view.Aggregation"] | None
        ) = None,
    ):
        super().__init__(preferred_temporality, preferred_aggregation)
        self.backend = backend
        self.lock = threading.Lock()

    def export(
        self, metrics_data, timeout_millis: float = 10_000, **kwargs
    ) -> MetricExportResult:
        """
        Export metrics data by adding it to the backend.

        Args:
            metrics_data: The metrics data to export.
            timeout_millis (float): Timeout for the export operation in milliseconds.
            **kwargs: Additional exporter-specific parameters.

        Returns:
            MetricExportResult: The result of the export operation, SUCCESS if accepted.
        """
        self.backend.add(metrics_data)
        return MetricExportResult.SUCCESS

    def force_flush(self, timeout_millis: float = 30_000):
        """
        No-op flush operation.

        Args:
            timeout_millis (float): Timeout for the flush operation in milliseconds.
        """

    def shutdown(self, timeout_millis: float = 30_000, **kwargs):
        """
        No-op shutdown operation.

        Args:
            timeout_millis (float): Timeout for the shutdown operation in milliseconds.
            **kwargs: Additional shutdown parameters.
        """


class FrameworkMetricsRetriever(MetricsRetriever):
    """
    MetricsRetriever implementation that queries metrics from a FrameworkMetricBackend.

    This class acts as a bridge between high-level metric querying interfaces and
    the underlying in-memory backend storing raw metric data. It exposes schema
    information and supports complex metric queries by delegating to the backend's
    MetricDataFrame.

    Attributes:
        backend (FrameworkMetricBackend): The backend providing stored metric data.
    """

    def __init__(self, backend: FrameworkMetricBackend):
        """
        Initialize with the given FrameworkMetricBackend.

        Args:
            backend (FrameworkMetricBackend): Backend to query metrics from.
        """
        self.backend = backend

    def get_schema(self) -> Dict[str, Any]:
        """
        Get the schema of the metric data as defined by MetricRow.

        Returns:
            Dict[str, Any]: Mapping of column names to types.
        """
        return get_type_hints(MetricRow)

    def query_metrics(
        self,
        metric_name: Optional[Union[str, list[str]]] = None,
        metric_type: Optional[str] = None,
        time_range: Optional[tuple[pd.Timestamp, pd.Timestamp]] = None,
        resource_attrs: Optional[Dict[str, Any]] = None,
        scope_attrs: Optional[Dict[str, Any]] = None,
        metric_attrs: Optional[Dict[str, Any]] = None,
        where: Optional[Callable[["MetricDataFrame"], "MetricDataFrame"]] = None,
    ) -> "MetricDataFrame":
        """
        Query metrics from the backend with optional filtering parameters.

        Args:
            metric_name (Optional[Union[str, list[str]]]): Metric name(s) to filter.
            metric_type (Optional[str]): Metric type to filter.
            time_range (Optional[tuple[pd.Timestamp, pd.Timestamp]]): Start and end
                timestamps to filter.
            resource_attrs (Optional[Dict[str, Any]]): Resource attributes to filter.
            scope_attrs (Optional[Dict[str, Any]]): Scope attributes to filter.
            metric_attrs (Optional[Dict[str, Any]]): Metric-specific attributes to filter.
            where (Optional[Callable[[MetricDataFrame], MetricDataFrame]]): Optional
                callable for additional custom filtering.

        Returns:
            MetricDataFrame: Filtered metric data.
        """
        return self.backend.get_metrics_df().query_metrics(
            metric_name=metric_name,
            metric_type=metric_type,
            time_range=time_range,
            resource_attrs=resource_attrs,
            scope_attrs=scope_attrs,
            metric_attrs=metric_attrs,
            where=where,
        )
