"""
Embedded OTLP/gRPC metrics sink for engine-pushed telemetry.

Components under test (e.g. the dataflow engine) can push their internal
metrics over OTLP instead of being scraped. This module provides:

- 'OtlpMetricsSink': A small gRPC server implementing the OTLP
  MetricsService. Each received data point is flattened into a row and kept
  in memory for later querying (e.g. by 'sql_report').
- 'StartOtlpMetricsSinkHook' ('start_otlp_metrics_sink'): Starts a sink and
  stores it on the suite runtime. Intended for suite-level 'run.pre' hooks.
- 'StopOtlpMetricsSinkHook' ('stop_otlp_metrics_sink'): Stops the sink.
  Intended for suite-level 'run.post' hooks.

Why push instead of scrape:
    Scraped counters are sampled at times chosen by the observer and it is
    difficult to guarantee that metrics which require precise accounting such
    as the total count of signals sent/received will line up perfectly across
    the loadgen/backend of a benchmark. Additionally the engine un-registers
    metrics as soon as a pipeline shutdown signal is received and so a scraper
    loses access to any final datapoints for signals that are processed during
    the shutdown sequence. For push-based metricst the engine performs a final
    flush during graceful shutdown, so summing pushed deltas yields exact
    whole-run totals.

Row schema (see 'PUSHED_METRIC_COLUMNS'):
    The rows are shaped to match the orchestrator's 'MetricRow' schema so they
    can be concatenated onto the framework 'metrics' table in 'sql_report'
    (see lib/core/telemetry/metric.py). The required 'MetricRow' columns are:

    timestamp         - pd.Timestamp (UTC) data point time_unix_nano (interval end)
    metric_name       - OTLP metric name, verbatim (e.g. 'logs.produced')
    metric_type       - 'Sum' | 'Gauge' | 'Histogram' | 'ExponentialHistogram'
                        (matched to the SDK backend's casing)
    value             - float (sum/gauge value; histogram/summary sum)
    resource_attributes - dict of OTLP resource attributes
    scope_attributes  - dict of OTLP scope attributes, plus 'scope_name' and
                        'scope_version' (consistent with the SDK backend)
    metric_attributes - dict of OTLP data point attributes

    Two extra columns carry push-specific information the push reports need:

    start_timestamp   - pd.Timestamp (UTC) start_time_unix_nano (NaT if 0)
    temporality       - 'delta' | 'cumulative' | None

    Keeping the engine's native OTLP metric names and attributes (no renaming)
    means a run collected via push is homogeneous: every row in 'metrics' uses
    OTLP names, and push reports select rows by metric name plus resource and
    scope attributes.
"""

import threading
from concurrent import futures
from typing import Any, Dict, List, Optional

import grpc
import pandas as pd
from opentelemetry.proto.collector.metrics.v1 import (
    metrics_service_pb2,
    metrics_service_pb2_grpc,
)
from opentelemetry.proto.common.v1.common_pb2 import AnyValue, KeyValue
from opentelemetry.proto.metrics.v1 import metrics_pb2

from ....core.context.base import BaseContext
from ....core.context import FrameworkElementHookContext
from ....core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from ....runner.registry import hook_registry, PluginMeta

START_HOOK_NAME = "start_otlp_metrics_sink"
STOP_HOOK_NAME = "stop_otlp_metrics_sink"

# Namespace used to store the sink on the suite runtime.
OTLP_METRICS_SINK_RUNTIME = "otlp_metrics_sink"

# Columns emitted per data point. Note: 'start_timestamp' and 'temporality' are
# push-specific fields.
PUSHED_METRIC_COLUMNS = [
    "timestamp",
    "metric_name",
    "metric_type",
    "value",
    "resource_attributes",
    "scope_attributes",
    "metric_attributes",

    # Push specific
    "start_timestamp",
    "temporality",
]

# OTLP metric type -> metric_type label, matching the casing the SDK-backed
# framework metrics use (see lib/core/telemetry/metric.py).
_METRIC_TYPE = {
    "sum": "Sum",
    "gauge": "Gauge",
    "histogram": "Histogram",
    "exponential_histogram": "ExponentialHistogram",
    "summary": "Summary",
}

_TEMPORALITY = {
    metrics_pb2.AGGREGATION_TEMPORALITY_DELTA: "delta",
    metrics_pb2.AGGREGATION_TEMPORALITY_CUMULATIVE: "cumulative",
}


def any_value_to_python(value: AnyValue) -> Any:
    """Convert an OTLP AnyValue into a plain python value."""
    kind = value.WhichOneof("value")
    if kind is None:
        return None
    if kind == "array_value":
        return [any_value_to_python(v) for v in value.array_value.values]
    if kind == "kvlist_value":
        return attributes_to_dict(value.kvlist_value.values)
    return getattr(value, kind)


def attributes_to_dict(attributes: List[KeyValue]) -> Dict[str, Any]:
    """Convert a repeated OTLP KeyValue field into a dict."""
    return {kv.key: any_value_to_python(kv.value) for kv in attributes}


def _ts(nanos: int) -> Optional[pd.Timestamp]:
    if not nanos:
        return None
    return pd.Timestamp(nanos, unit="ns", tz="UTC")


def flatten_export_request(
    request: metrics_service_pb2.ExportMetricsServiceRequest,
    received_at: pd.Timestamp,
) -> List[Dict[str, Any]]:
    """Flatten an OTLP metrics export request into one MetricRow-shaped row per
    data point.

    'received_at' is accepted for call-site compatibility but is not stored;
    rows are timestamped with the engine-assigned data point end time.
    """
    del received_at  # engine-assigned timestamps are authoritative
    rows: List[Dict[str, Any]] = []
    for resource_metrics in request.resource_metrics:
        resource_attrs = attributes_to_dict(resource_metrics.resource.attributes)
        for scope_metrics in resource_metrics.scope_metrics:
            scope = scope_metrics.scope
            # Fold the scope name/version into scope_attributes so the flattened
            # table exposes "scope_attributes.scope_name" like the SDK backend.
            scope_attrs = attributes_to_dict(scope.attributes)
            scope_attrs["scope_name"] = scope.name
            scope_attrs["scope_version"] = scope.version
            for metric in scope_metrics.metrics:
                base = {
                    "metric_name": metric.name,
                    "resource_attributes": resource_attrs,
                    "scope_attributes": scope_attrs,
                }
                data_kind = metric.WhichOneof("data")
                if data_kind in ("sum", "gauge"):
                    data = getattr(metric, data_kind)
                    temporality = (
                        _TEMPORALITY.get(data.aggregation_temporality)
                        if data_kind == "sum"
                        else None
                    )
                    for dp in data.data_points:
                        rows.append(
                            {
                                **base,
                                **_dp_base(dp),
                                "metric_type": _METRIC_TYPE[data_kind],
                                "temporality": temporality,
                                "value": _number_point(dp),
                            }
                        )
                elif data_kind in ("histogram", "exponential_histogram"):
                    data = getattr(metric, data_kind)
                    for dp in data.data_points:
                        rows.append(
                            {
                                **base,
                                **_dp_base(dp),
                                "metric_type": _METRIC_TYPE[data_kind],
                                "temporality": _TEMPORALITY.get(
                                    data.aggregation_temporality
                                ),
                                "value": _distribution_point(dp),
                            }
                        )
                elif data_kind == "summary":
                    for dp in metric.summary.data_points:
                        rows.append(
                            {
                                **base,
                                **_dp_base(dp),
                                "metric_type": _METRIC_TYPE[data_kind],
                                "temporality": None,
                                "value": _distribution_point(dp),
                            }
                        )
    return rows


def _dp_base(dp) -> Dict[str, Any]:
    # Fields shared by every data point kind: the interval timestamps and the
    # per-point attributes. Only the 'value' differs between number and
    # distribution points (see '_number_point'/'_distribution_point').
    return {
        "start_timestamp": _ts(dp.start_time_unix_nano),
        "timestamp": _ts(dp.time_unix_nano),
        "metric_attributes": attributes_to_dict(dp.attributes),
    }


def _number_point(dp: metrics_pb2.NumberDataPoint) -> Any:
    kind = dp.WhichOneof("value")
    if kind == "as_int":
        # Keep integer counters exact. Casting sint64 counts to float would lose
        # precision above 2^53.
        return int(dp.as_int)
    if kind == "as_double":
        return float(dp.as_double)
    return None


def _distribution_point(dp) -> Optional[float]:
    # 'sum' is optional on Histogram/ExponentialHistogram data points, so an
    # absent sum must map to NULL rather than a phantom 0.0. Summary data points
    # have a non-optional 'sum' (no field presence), so read it directly.
    if isinstance(dp, metrics_pb2.SummaryDataPoint):
        return float(dp.sum)
    return float(dp.sum) if dp.HasField("sum") else None


class _MetricsServicer(metrics_service_pb2_grpc.MetricsServiceServicer):
    def __init__(self, sink: "OtlpMetricsSink"):
        self._sink = sink

    def Export(self, request, context):  # noqa: N802 (grpc naming)
        self._sink.ingest(request)
        return metrics_service_pb2.ExportMetricsServiceResponse()


class OtlpMetricsSink:
    """In-memory OTLP/gRPC metrics receiver."""

    def __init__(self, endpoint: str = "0.0.0.0:14317", max_workers: int = 4):
        self.endpoint = endpoint
        self._max_workers = max_workers
        self._rows: List[Dict[str, Any]] = []
        self._lock = threading.Lock()
        self._server: Optional[grpc.Server] = None
        self.port: Optional[int] = None

    def start(self) -> int:
        """Start serving. Returns the bound port."""
        if self._server is not None:
            return self.port
        server = grpc.server(
            futures.ThreadPoolExecutor(max_workers=self._max_workers),
            options=[
                ("grpc.max_receive_message_length", 64 * 1024 * 1024),
                # grpcio enables SO_REUSEPORT by default, which would let a
                # second sink (e.g. a stale orchestrator) silently share the
                # port and split the pushed data. Fail to bind instead.
                ("grpc.so_reuseport", 0),
            ],
        )
        metrics_service_pb2_grpc.add_MetricsServiceServicer_to_server(
            _MetricsServicer(self), server
        )
        try:
            port = server.add_insecure_port(self.endpoint)
        except RuntimeError as e:
            raise RuntimeError(
                f"Failed to bind OTLP metrics sink to {self.endpoint}: {e}"
            ) from e
        if port == 0:
            raise RuntimeError(f"Failed to bind OTLP metrics sink to {self.endpoint}")
        server.start()
        self._server = server
        self.port = port
        return port

    def stop(self, grace: float = 2.0) -> None:
        """Stop serving, waiting up to 'grace' seconds for in-flight requests."""
        if self._server is None:
            return
        self._server.stop(grace).wait()
        self._server = None

    def ingest(self, request: metrics_service_pb2.ExportMetricsServiceRequest):
        """Flatten and store an export request."""
        rows = flatten_export_request(request, pd.Timestamp.now(tz="UTC"))
        with self._lock:
            self._rows.extend(rows)

    def row_count(self) -> int:
        with self._lock:
            return len(self._rows)

    def to_dataframe(self) -> pd.DataFrame:
        """Return a snapshot of all received data points."""
        with self._lock:
            rows = list(self._rows)
        return pd.DataFrame(rows, columns=PUSHED_METRIC_COLUMNS)


def get_otlp_metrics_sink(ctx: BaseContext) -> Optional[OtlpMetricsSink]:
    """Return the suite's OTLP metrics sink if one was started."""
    suite = ctx.get_suite()
    if suite is None:
        return None
    return suite.get_runtime(OTLP_METRICS_SINK_RUNTIME)


@hook_registry.register_config(START_HOOK_NAME)
class StartOtlpMetricsSinkConfig(HookStrategyConfig):
    """
    Configuration for the 'start_otlp_metrics_sink' hook.

    Attributes:
        endpoint: host:port to bind the OTLP/gRPC server to. Containers on a
            docker bridge network can reach it via 'host.docker.internal' when
            the container is started with
            'extra_hosts: {host.docker.internal: host-gateway}'.
    """

    endpoint: str = "0.0.0.0:14317"


@hook_registry.register_class(START_HOOK_NAME)
class StartOtlpMetricsSinkHook(HookStrategy):
    """Start an embedded OTLP/gRPC metrics sink and attach it to the suite."""

    PLUGIN_META = PluginMeta(
        supported_contexts=[FrameworkElementHookContext.__name__],
        installs_hooks=[],
        yaml_example="""
hooks:
  run:
    pre:
      - start_otlp_metrics_sink:
          endpoint: 0.0.0.0:14317
    post:
      - stop_otlp_metrics_sink: {}
""",
    )

    def __init__(self, config: StartOtlpMetricsSinkConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        logger = ctx.get_logger(__name__)
        suite = ctx.get_suite()
        existing = suite.get_runtime(OTLP_METRICS_SINK_RUNTIME)
        if existing is not None:
            logger.info("OTLP metrics sink already running on %s", existing.endpoint)
            return
        sink = OtlpMetricsSink(endpoint=self.config.endpoint)
        port = sink.start()
        suite.set_runtime_data(OTLP_METRICS_SINK_RUNTIME, sink)
        logger.info(
            "OTLP metrics sink listening on %s (port %s)", self.config.endpoint, port
        )


@hook_registry.register_config(STOP_HOOK_NAME)
class StopOtlpMetricsSinkConfig(HookStrategyConfig):
    """Configuration for the 'stop_otlp_metrics_sink' hook."""

    grace_seconds: float = 2.0


@hook_registry.register_class(STOP_HOOK_NAME)
class StopOtlpMetricsSinkHook(HookStrategy):
    """Stop the suite's embedded OTLP/gRPC metrics sink."""

    PLUGIN_META = PluginMeta(
        supported_contexts=[FrameworkElementHookContext.__name__],
        installs_hooks=[],
        yaml_example="""
hooks:
  run:
    post:
      - stop_otlp_metrics_sink: {}
""",
    )

    def __init__(self, config: StopOtlpMetricsSinkConfig):
        self.config = config

    def execute(self, ctx: BaseContext):
        sink = get_otlp_metrics_sink(ctx)
        if sink is None:
            return
        sink.stop(self.config.grace_seconds)
        ctx.get_logger(__name__).info(
            "OTLP metrics sink stopped after receiving %d data points",
            sink.row_count(),
        )
