from unittest.mock import MagicMock

import duckdb
import grpc
import pandas as pd
from opentelemetry.proto.collector.metrics.v1 import (
    metrics_service_pb2,
    metrics_service_pb2_grpc,
)
from opentelemetry.proto.common.v1.common_pb2 import (
    AnyValue,
    InstrumentationScope,
    KeyValue,
)
from opentelemetry.proto.metrics.v1 import metrics_pb2
from opentelemetry.proto.resource.v1.resource_pb2 import Resource

from lib.impl.strategies.hooks.otlp_metrics_sink import (
    OTLP_METRICS_SINK_RUNTIME,
    OtlpMetricsSink,
    StartOtlpMetricsSinkConfig,
    StartOtlpMetricsSinkHook,
    StopOtlpMetricsSinkConfig,
    StopOtlpMetricsSinkHook,
    _distribution_point,
    _number_point,
    flatten_export_request,
)
from lib.impl.strategies.hooks.reporting.sql_report import (
    QueryConfig,
    SQLReportConfig,
    SQLReportDetails,
    SQLReportHook,
)


def _kv(key, value):
    if isinstance(value, str):
        return KeyValue(key=key, value=AnyValue(string_value=value))
    return KeyValue(key=key, value=AnyValue(int_value=value))


def _delta_request(service, test, scope_name, scope_attrs, name, points):
    """Build an export request with one delta monotonic sum.

    points: list of (start_ns, end_ns, value, attrs_dict)
    """
    data_points = [
        metrics_pb2.NumberDataPoint(
            start_time_unix_nano=start,
            time_unix_nano=end,
            as_int=value,
            attributes=[_kv(k, v) for k, v in attrs.items()],
        )
        for start, end, value, attrs in points
    ]
    return metrics_service_pb2.ExportMetricsServiceRequest(
        resource_metrics=[
            metrics_pb2.ResourceMetrics(
                resource=Resource(
                    attributes=[_kv("service.name", service), _kv("test.name", test)]
                ),
                scope_metrics=[
                    metrics_pb2.ScopeMetrics(
                        scope=InstrumentationScope(
                            name=scope_name,
                            attributes=[_kv(k, v) for k, v in scope_attrs.items()],
                        ),
                        metrics=[
                            metrics_pb2.Metric(
                                name=name,
                                sum=metrics_pb2.Sum(
                                    aggregation_temporality=metrics_pb2.AGGREGATION_TEMPORALITY_DELTA,
                                    is_monotonic=True,
                                    data_points=data_points,
                                ),
                            )
                        ],
                    )
                ],
            )
        ]
    )


# Scenario: An OTLP export request containing a delta monotonic sum with
#   resource, scope, and data point attributes is flattened.
# Guarantees: One MetricRow-shaped row per data point is produced carrying the
#   verbatim OTLP metric name, SDK-cased metric type, delta temporality,
#   engine-assigned start/end timestamps, numeric value, and the three
#   attribute levels as dicts, with the scope name folded into scope_attributes.
def test_flatten_export_request_delta_sum():
    request = _delta_request(
        "backend-service",
        "T1",
        "node.input",
        {"node.id": "perf"},
        "items",
        [(1_000, 2_000, 5, {"signal": "logs", "outcome": "success"})],
    )

    rows = flatten_export_request(request, pd.Timestamp.now(tz="UTC"))

    assert len(rows) == 1
    row = rows[0]
    assert row["metric_name"] == "items"
    assert row["metric_type"] == "Sum"
    assert row["temporality"] == "delta"
    assert row["value"] == 5.0
    assert row["start_timestamp"] == pd.Timestamp(1_000, unit="ns", tz="UTC")
    assert row["timestamp"] == pd.Timestamp(2_000, unit="ns", tz="UTC")
    assert row["resource_attributes"] == {
        "service.name": "backend-service",
        "test.name": "T1",
    }
    assert row["scope_attributes"]["node.id"] == "perf"
    assert row["scope_attributes"]["scope_name"] == "node.input"
    assert row["metric_attributes"] == {"signal": "logs", "outcome": "success"}


# Scenario: A NumberDataPoint carries an integer counter value larger than
#   2^53 (the exact-integer limit of IEEE-754 double).
# Guarantees: The flattened value is a Python int equal to the exported count,
#   so whole-run counter totals are not silently rounded by a float cast.
def test_number_point_preserves_large_integer_value():
    big = 2**53 + 1
    dp = metrics_pb2.NumberDataPoint(
        start_time_unix_nano=1,
        time_unix_nano=2,
        as_int=big,
    )

    row = _number_point(dp)

    assert row["value"] == big
    assert isinstance(row["value"], int)


# Scenario: A NumberDataPoint carries a floating point (as_double) value.
# Guarantees: The flattened value is a float equal to the exported value, so
#   gauge/double metrics are not coerced to int.
def test_number_point_preserves_double_value():
    dp = metrics_pb2.NumberDataPoint(
        start_time_unix_nano=1,
        time_unix_nano=2,
        as_double=1.5,
    )

    row = _number_point(dp)

    assert row["value"] == 1.5
    assert isinstance(row["value"], float)


# Scenario: A HistogramDataPoint is exported without its optional 'sum' field
#   set, versus one that explicitly sets sum to 0.0.
# Guarantees: An absent sum flattens to None (NULL) rather than a phantom 0.0,
#   while an explicit 0.0 is preserved, so aggregations do not count a missing
#   sum as a real zero.
def test_distribution_point_missing_sum_is_null():
    without_sum = metrics_pb2.HistogramDataPoint(
        start_time_unix_nano=1,
        time_unix_nano=2,
        count=3,
    )
    with_zero_sum = metrics_pb2.HistogramDataPoint(
        start_time_unix_nano=1,
        time_unix_nano=2,
        count=3,
        sum=0.0,
    )

    assert _distribution_point(without_sum)["value"] is None
    assert _distribution_point(with_zero_sum)["value"] == 0.0


# Scenario: A SummaryDataPoint (whose 'sum' has no field presence) is flattened.
# Guarantees: The summary sum is read directly without a presence check, so it
#   does not raise and yields the exported value.
def test_distribution_point_summary_sum_read_directly():
    dp = metrics_pb2.SummaryDataPoint(
        start_time_unix_nano=1,
        time_unix_nano=2,
        count=4,
        sum=12.5,
    )

    assert _distribution_point(dp)["value"] == 12.5


# Scenario: A real OTLP/gRPC client exports several delta batches to a running
#   OtlpMetricsSink bound to an ephemeral port.
# Guarantees: Every data point from every request is retained and the sum of
#   the delta values equals the total exported, which is the property the
#   push-based loss calculation relies on.
def test_sink_receives_grpc_exports_and_preserves_delta_totals():
    sink = OtlpMetricsSink(endpoint="127.0.0.1:0")
    port = sink.start()
    try:
        with grpc.insecure_channel(f"127.0.0.1:{port}") as channel:
            stub = metrics_service_pb2_grpc.MetricsServiceStub(channel)
            for i in range(3):
                stub.Export(
                    _delta_request(
                        "load-generator",
                        "T1",
                        "receiver.traffic_generator",
                        {"node.id": "receiver"},
                        "logs.produced",
                        [(i * 10, (i + 1) * 10, 100 + i, {})],
                    ),
                    timeout=5,
                )
    finally:
        sink.stop()

    df = sink.to_dataframe()
    assert sink.row_count() == 3
    assert df["value"].sum() == 100 + 101 + 102
    assert set(df["metric_name"]) == {"logs.produced"}


# Scenario: A second sink tries to bind the port of an already running sink
#   (e.g. a stale orchestrator process left running).
# Guarantees: The second bind fails loudly instead of silently sharing the
#   port via SO_REUSEPORT, which would split pushed data between processes
#   and corrupt whole-run totals.
def test_sink_refuses_to_share_port():
    first = OtlpMetricsSink(endpoint="127.0.0.1:0")
    port = first.start()
    try:
        second = OtlpMetricsSink(endpoint=f"127.0.0.1:{port}")
        try:
            second.start()
            raised = False
        except RuntimeError:
            raised = True
        finally:
            second.stop()
        assert raised
    finally:
        first.stop()


# Scenario: The start/stop hooks are executed against a suite context.
# Guarantees: The start hook stores a running sink on the suite runtime under
#   OTLP_METRICS_SINK_RUNTIME, a second start is a no-op that keeps the same
#   sink, and the stop hook shuts it down without error.
def test_start_stop_hooks_manage_suite_runtime():
    runtime = {}
    suite = MagicMock()
    suite.get_runtime.side_effect = runtime.get
    suite.set_runtime_data.side_effect = runtime.__setitem__
    ctx = MagicMock()
    ctx.get_suite.return_value = suite

    StartOtlpMetricsSinkHook(
        StartOtlpMetricsSinkConfig(endpoint="127.0.0.1:0")
    ).execute(ctx)
    sink = runtime[OTLP_METRICS_SINK_RUNTIME]
    assert isinstance(sink, OtlpMetricsSink)
    assert sink.port

    StartOtlpMetricsSinkHook(
        StartOtlpMetricsSinkConfig(endpoint="127.0.0.1:0")
    ).execute(ctx)
    assert runtime[OTLP_METRICS_SINK_RUNTIME] is sink

    StopOtlpMetricsSinkHook(StopOtlpMetricsSinkConfig()).execute(ctx)
    assert sink._server is None


def _empty_metric_frame():
    """A framework-style metrics frame (MetricRow columns) with no rows."""
    return pd.DataFrame(
        columns=[
            "timestamp",
            "metric_name",
            "metric_type",
            "value",
            "resource_attributes",
            "scope_attributes",
            "metric_attributes",
        ]
    )


def _empty_span_frame():
    """A spans/events frame with the columns flatten_columns expects."""
    return pd.DataFrame(columns=["name", "resource", "attributes"])


def _ctx_with_sink(sink):
    suite = MagicMock()
    suite.get_runtime.side_effect = lambda ns: (
        sink if ns == OTLP_METRICS_SINK_RUNTIME else None
    )
    ctx = MagicMock()
    ctx.get_suite.return_value = suite
    return ctx


# Scenario: sql_report builds its in-memory tables while a metrics sink holding
#   pushed OTLP rows is attached to the suite.
# Guarantees: Pushed rows are concatenated into the single 'metrics' table with
#   the engine's own OTLP names and attributes flattened into
#   '<level>_attributes.<key>' columns, so report SQL can filter by
#   service.name / test.name / node.id without a separate pushed table.
def test_sql_report_appends_pushed_rows_to_metrics_table():
    sink = OtlpMetricsSink(endpoint="127.0.0.1:0")
    sink.ingest(
        _delta_request(
            "backend-service",
            "T1",
            "node.input",
            {"node.id": "perf"},
            "items",
            [(1, 2, 7, {"signal": "logs"}), (2, 3, 8, {"signal": "logs"})],
        )
    )
    hook = SQLReportHook(
        SQLReportConfig(
            name="t",
            report_config=SQLReportDetails(
                queries=[QueryConfig(name="q", sql="SELECT 1")]
            ),
        )
    )

    hook.conn = duckdb.connect()
    hook._register_in_memory_tables(
        _empty_metric_frame(),
        _empty_span_frame(),
        _empty_span_frame(),
        _ctx_with_sink(sink),
    )
    total = hook.conn.execute("""
        SELECT SUM(value) FROM metrics
        WHERE metric_name = 'items'
          AND "resource_attributes.service.name" = 'backend-service'
          AND "resource_attributes.test.name" = 'T1'
          AND "scope_attributes.node.id" = 'perf'
          AND "scope_attributes.scope_name" = 'node.input'
          AND "metric_attributes.signal" = 'logs'
        """).fetchone()[0]
    assert total == 15


# Scenario: sql_report builds its in-memory tables when no metrics sink was
#   started for the suite (e.g. a scrape-based run).
# Guarantees: The 'metrics' table is registered from the framework rows alone;
#   the absence of a sink neither errors nor injects pushed rows.
def test_sql_report_without_sink_registers_only_framework_metrics():
    hook = SQLReportHook(
        SQLReportConfig(
            name="t",
            report_config=SQLReportDetails(
                queries=[QueryConfig(name="q", sql="SELECT 1")]
            ),
        )
    )
    hook.conn = duckdb.connect()
    hook._register_in_memory_tables(
        _empty_metric_frame(),
        _empty_span_frame(),
        _empty_span_frame(),
        _ctx_with_sink(None),
    )
    assert hook.conn.execute("SELECT COUNT(*) FROM metrics").fetchone()[0] == 0
