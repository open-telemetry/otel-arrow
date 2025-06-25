"""
CLI argument parser for the Test Orchestration Framework.

This module defines the core command-line interface used to configure
and run test suites. It includes standard options like config paths and
telemetry/export settings, and supports dynamic extension via plugin hooks.
"""
import argparse
from .plugin_api import apply_argument_hooks

def build_parser() -> argparse.ArgumentParser:
    """
    Build and return the CLI argument parser.

    Returns:
        argparse.ArgumentParser: A configured argument parser for the test runner.

    Plugin Hooks:
        Additional arguments may be added dynamically via plugins using
        `apply_argument_hooks()` (run at plugin registration time).

    Example:
        parser = build_parser()
        args = parser.parse_args()
    """
    parser = argparse.ArgumentParser(description="Test Orchestration Framework CLI")

    # Standard/core arguments
    parser.add_argument(
        "--config", "-c", required=True, help="Path to test suite YAML config."
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable debug mode (verbose output, etc.)."
    )
    otlp = parser.add_argument_group("OTLP Export")
    otlp.add_argument("--otlp-endpoint", type=str, default="http://localhost:4317",
                        help="OTLP exporter endpoint (e.g., http://localhost:4317)")
    otlp.add_argument("--export-traces", action="store_true",
                        help="Enable OpenTelemetry tracing to external otlp endpoint")
    otlp.add_argument("--export-metrics", action="store_true",
                        help="Enable OpenTelemetry metrics export to external otlp endpoint")
    apply_argument_hooks(parser)

    return parser
