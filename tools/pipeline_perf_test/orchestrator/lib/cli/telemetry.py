"""
Telemetry setup module for the pipeline performance test orchestrator CLI.

This module configures OpenTelemetry-based tracing and metrics instrumentation
for CLI execution. It includes logging customization, initialization of OTLP exporters
(for trace and metric data), and integration with framework telemetry systems.

Key components:
- Logging formatter with support for safe contextual keys.
- OTLP exporters for trace and metrics via gRPC.
- Custom framework exporters and retrievers.
- Creation of a `TelemetryRuntime` used by the orchestrator runtime.

Dependencies:
- OpenTelemetry SDK and OTLP exporters.
- Internal telemetry classes: TelemetryClient, Span/Metrics backends and exporters.
"""

import argparse
import logging

from opentelemetry import trace, metrics
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.exporter.otlp.proto.grpc.metric_exporter import OTLPMetricExporter
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.sdk.resources import Resource, SERVICE_NAME
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor, SimpleSpanProcessor

from ..core.telemetry.telemetry_runtime import TelemetryRuntime
from ..core.telemetry.telemetry_client import TelemetryClient
from ..core.telemetry.span import (
    FrameworkSpanBackend,
    FrameworkSpanExporter,
    FrameworkSpanRetriever,
)
from ..core.telemetry.metric import (
    FrameworkMetricBackend,
    FrameworkMetricExporter,
    FrameworkMetricsRetriever,
)

from ..core.telemetry.log import SpanAwareLogHandler
from .util import get_git_info

SERVICE = "pipeline-perf-test-orchestrator"
git_info = get_git_info()


def setup_logging(args: argparse.Namespace):
    """
    Configure application-wide logging behavior for CLI execution.

    This includes:
    - Stream handler with custom formatter that safely replaces `.` in context keys.
    - Inclusion of contextual fields (`test_ctx_name`, `test_ctx_type`) in log output.
    - Adjusts verbosity based on `--debug` flag in CLI args.

    Args:
        args (argparse.Namespace): Parsed command-line arguments with possible `debug` flag.
    """

    class SafeContextFormatter(logging.Formatter):
        def format(self, record):
            record_dict = record.__dict__.copy()
            for key in list(record_dict.keys()):
                if "." in key:
                    new_key = key.replace(".", "_")
                    # Only add if not already present to avoid overwriting
                    record_dict[new_key] = record_dict[key]
            record_dict.setdefault("test_ctx_name", "-")
            record_dict.setdefault("test_ctx_type", "-")
            original_dict = record.__dict__
            record.__dict__ = record_dict

            try:
                formatted = super().format(record)
            finally:
                record.__dict__ = original_dict  # Restore original dict

            return formatted

    formatter = SafeContextFormatter(
        fmt="[{asctime}] {levelname} | ctx.type={test_ctx_type} ctx.name={test_ctx_name} | {message}",
        style="{",
        datefmt="%Y-%m-%d %H:%M:%S",
    )

    handler = logging.StreamHandler()
    handler.setFormatter(formatter)

    logger = logging.getLogger()
    logger.setLevel(logging.INFO)

    lib_logger = logging.getLogger("lib")
    lib_logger.setLevel(logging.DEBUG if args.debug else logging.INFO)
    lib_logger.addHandler(handler)
    lib_logger.propagate = False

    # Add framework logging handler
    span_log_handler = SpanAwareLogHandler()
    lib_logger.addHandler(span_log_handler)


def build_telemetry_runtime(args: argparse.Namespace) -> TelemetryRuntime:
    """
    Constructs the framework telemetry runtime including tracing and metrics providers.

    The function configures:
    - OTLP span and metric exporters (to localhost collector)
    - Internal framework exporters for trace and metrics
    - Meter and tracer providers using shared service and git resource info

    Returns:
        TelemetryRuntime: Fully initialized telemetry runtime with access to providers and client.
    """
    resource = Resource.create({SERVICE_NAME: SERVICE, **git_info})

    # Setup general trace provider
    trace_provider = TracerProvider(resource=resource)
    if args.export_traces:
        otlp_span_exporter = OTLPSpanExporter(endpoint=args.otlp_endpoint)
        trace_provider.add_span_processor(BatchSpanProcessor(otlp_span_exporter))

    # Setup general metric exporter
    readers = []
    if args.export_metrics:
        otlp_metric_exporter = OTLPMetricExporter(endpoint=args.otlp_endpoint)
        otlp_reader = PeriodicExportingMetricReader(exporter=otlp_metric_exporter)
        readers.append(otlp_reader)

    # Add Framework tracing infrastructure
    fw_span_backend = FrameworkSpanBackend()
    fw_span_client = FrameworkSpanRetriever(backend=fw_span_backend)
    fw_spans = FrameworkSpanExporter(backend=fw_span_backend)
    trace_provider.add_span_processor(SimpleSpanProcessor(fw_spans))
    trace.set_tracer_provider(trace_provider)

    # Add Framework metrics infrastructure
    fw_metric_backend = FrameworkMetricBackend()
    fw_metric_client = FrameworkMetricsRetriever(backend=fw_metric_backend)
    fw_metrics = FrameworkMetricExporter(backend=fw_metric_backend)
    fw_reader = PeriodicExportingMetricReader(
        exporter=fw_metrics, export_interval_millis=100
    )
    readers.append(fw_reader)

    meter_provider = MeterProvider(metric_readers=readers, resource=resource)
    metrics.set_meter_provider(meter_provider)

    # Create the framework client for telemetry access
    fw_telemetry_client = TelemetryClient(
        metrics=fw_metric_client, spans=fw_span_client
    )

    return TelemetryRuntime(
        tracer_provider=trace_provider,
        meter_provider=meter_provider,
        telemetry_client=fw_telemetry_client,
    )


def setup_telemetry(args: argparse.Namespace) -> TelemetryRuntime:
    """Setup logging, metrics, tracing instrumentation for the framework and suite.

    Args:
        args: The command line arguments.

    returns:
        a Framework TelemetryRuntime for use in a Suite.
    """
    setup_logging(args)
    return build_telemetry_runtime(args)
