"""
Process Perf Cycles Monitoring Strategy Module.

This module adds Linux perf-based CPU cycle monitoring for process-deployed
components. It periodically runs `perf stat` against a target PID and records
hardware counter deltas as OpenTelemetry gauges.

Notes:
- This strategy requires Linux and the `perf` binary to be available.
- Some environments (for example locked-down CI runners) may deny access to
  hardware counters. In that case the strategy records availability as 0 and
  keeps running without failing the test.
"""

import subprocess
import threading
from dataclasses import dataclass, field
from logging import LoggerAdapter
from typing import TYPE_CHECKING, ClassVar, Literal, Optional

from opentelemetry import trace
from opentelemetry.sdk.metrics import Meter
from opentelemetry.trace import SpanKind

from ....core.component.component import Component
from ....core.context.framework_element_contexts import ScenarioContext, StepContext
from ....core.strategies.monitoring_strategy import MonitoringStrategy
from ....core.strategies.monitoring_strategy import MonitoringStrategyConfig
from ....runner.registry import PluginMeta, monitoring_registry
from ..deployment.process import ComponentProcessRuntime
from .perf_support import ensure_perf_binary

if TYPE_CHECKING:
    from ....impl.component.managed_component import ManagedComponent


STRATEGY_NAME = "perf_cycles_process"


@dataclass
class PerfCyclesProcessMonitoringRuntime:
    """Runtime state holder for perf cycles process monitoring."""

    type: ClassVar[Literal["perf_cycles_process_monitoring"]] = (
        "perf_cycles_process_monitoring"
    )
    thread: Optional[threading.Thread] = None
    stop_event: threading.Event = field(default_factory=threading.Event)


@monitoring_registry.register_config(STRATEGY_NAME)
class PerfCyclesProcessMonitoringConfig(MonitoringStrategyConfig):
    """Configuration for process perf-cycle monitoring."""

    interval: Optional[float] = 1.0
    # Additional counters are best-effort and may not be available on all hosts.
    include_ref_cycles: Optional[bool] = True
    include_instructions: Optional[bool] = True


@monitoring_registry.register_class(STRATEGY_NAME)
class PerfCyclesProcessMonitoringStrategy(MonitoringStrategy):
    """Monitor process CPU cycles using Linux `perf stat`."""

    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
components:
  backend-service:
    deployment:
      process: ...
    monitoring:
      perf_cycles_process:
        interval: 1
        include_ref_cycles: true
        include_instructions: true
""",
    )

    def __init__(self, config: PerfCyclesProcessMonitoringConfig):
        super().__init__(STRATEGY_NAME)
        self.config = config
        self.stop_event = None

    def start(self, component: "ManagedComponent", ctx: StepContext):
        logger = ctx.get_logger(__name__)
        meter = ctx.get_meter(__name__)
        ts = ctx.get_suite()

        self.stop_event = threading.Event()
        test_suite_context = ts.context

        logger.debug("Starting perf cycles monitoring for %s...", component.name)
        monitoring_runtime: PerfCyclesProcessMonitoringRuntime = (
            component.get_or_create_runtime(
                PerfCyclesProcessMonitoringRuntime.type,
                PerfCyclesProcessMonitoringRuntime,
            )
        )
        process_runtime: ComponentProcessRuntime = component.get_or_create_runtime(
            ComponentProcessRuntime.type, ComponentProcessRuntime
        )
        if not process_runtime.pid:
            raise RuntimeError("can't find process id in deployment runtime.")

        monitor_args = {
            "pid": process_runtime.pid,
            "component_name": component.name,
            "stop_event": self.stop_event,
            "meter": meter,
            "logger": logger,
            "test_suite_context": test_suite_context,
            "interval": self.config.interval,
            "include_ref_cycles": bool(self.config.include_ref_cycles),
            "include_instructions": bool(self.config.include_instructions),
        }
        monitoring_runtime.thread = threading.Thread(
            target=monitor, kwargs=monitor_args, daemon=True
        )
        monitoring_runtime.thread.start()
        component.set_runtime_data(
            PerfCyclesProcessMonitoringRuntime.type, monitoring_runtime
        )

    def stop(self, component: Component, ctx: StepContext):
        logger = ctx.get_logger(__name__)
        logger.debug("Stopping perf cycles monitoring for %s", component.name)
        monitoring_runtime: PerfCyclesProcessMonitoringRuntime = (
            component.get_or_create_runtime(
                PerfCyclesProcessMonitoringRuntime.type,
                PerfCyclesProcessMonitoringRuntime,
            )
        )
        self.stop_event.set()
        if monitoring_runtime.thread:
            monitoring_runtime.thread.join()

    def collect(self, _component: Component, _ctx: ScenarioContext) -> dict:
        return {}


def _parse_perf_stat_output(stderr: str) -> dict[str, float]:
    """
    Parse `perf stat -x ';'` output into event->value mapping.

    Expected record format starts with:
      value;unit;event;...
    """
    metrics: dict[str, float] = {}
    for raw_line in stderr.splitlines():
        line = raw_line.strip()
        if not line or ";" not in line:
            continue
        parts = [p.strip() for p in line.split(";")]
        if len(parts) < 3:
            continue

        raw_value, event_name = parts[0], parts[2]
        # Unsupported values show as "<not supported>" / "<not counted>".
        if raw_value.startswith("<"):
            continue
        # `--no-big-num` removes thousand separators, but keep this robust.
        value_normalized = raw_value.replace(",", "")
        try:
            metrics[event_name] = float(value_normalized)
        except ValueError:
            continue

    return metrics


def _collect_perf_counters(
    pid: int,
    interval: float,
    events: list[str],
    logger: LoggerAdapter,
) -> dict[str, float]:
    perf_bin = ensure_perf_binary(logger)
    if not perf_bin:
        logger.debug("`perf` binary is not available on PATH")
        return {}

    command = [
        perf_bin,
        "stat",
        "-x",
        ";",
        "--no-big-num",
        "-e",
        ",".join(events),
        "-p",
        str(pid),
        "--",
        "sleep",
        str(interval),
    ]
    try:
        result = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
            timeout=max(interval + 3.0, 5.0),
        )
    except Exception as exc:
        logger.debug("failed to run perf stat for pid %s: %s", pid, exc)
        return {}

    metrics = _parse_perf_stat_output(result.stderr)
    if not metrics and result.stderr:
        logger.debug("perf stat returned no parseable metrics for pid %s", pid)
    return metrics


def monitor(
    pid: int,
    component_name: str,
    stop_event: threading.Event,
    meter: Meter,
    logger: LoggerAdapter,
    test_suite_context: ScenarioContext,
    interval: float = 1.0,
    include_ref_cycles: bool = True,
    include_instructions: bool = True,
):
    """Monitor Linux perf cycle counters for a process."""
    cycles_gauge = meter.create_gauge(
        "process.cpu.cycles",
        "{cycle}",
        "CPU cycles consumed by the process during the poll interval.",
    )
    ref_cycles_gauge = meter.create_gauge(
        "process.cpu.ref_cycles",
        "{cycle}",
        "Reference CPU cycles consumed by the process during the poll interval.",
    )
    instructions_gauge = meter.create_gauge(
        "process.cpu.instructions",
        "{instruction}",
        "Instructions retired by the process during the poll interval.",
    )
    perf_available_gauge = meter.create_gauge(
        "process.cpu.perf_available",
        "{bool}",
        "1 if perf counters were collected successfully in the interval, else 0.",
    )

    events = ["cycles"]
    if include_ref_cycles:
        events.append("ref-cycles")
    if include_instructions:
        events.append("instructions")

    tracer = test_suite_context.get_tracer("perf_cycles_process_monitor")
    ctx = trace.set_span_in_context(test_suite_context.span)

    with tracer.start_as_current_span(
        f"Perf Cycles Monitor: PID {pid}", context=ctx, kind=SpanKind.PRODUCER
    ) as span:
        span.set_attributes(
            {
                "process.pid": pid,
                "ctx.component_name": component_name,
                "monitor.events": ",".join(events),
            }
        )

        labels = {
            "pid": str(pid),
            "component_name": component_name,
        }
        while not stop_event.is_set():
            metrics = _collect_perf_counters(pid, interval, events, logger)
            if not metrics:
                perf_available_gauge.set(0.0, labels)
                continue

            perf_available_gauge.set(1.0, labels)

            cycles = metrics.get("cycles")
            ref_cycles = metrics.get("ref-cycles")
            instructions = metrics.get("instructions")

            if cycles is not None:
                cycles_gauge.set(cycles, labels)
            if ref_cycles is not None:
                ref_cycles_gauge.set(ref_cycles, labels)
            if instructions is not None:
                instructions_gauge.set(instructions, labels)

            logger.debug(
                "Perf counters %s pid=%s cycles=%s ref-cycles=%s instructions=%s",
                component_name,
                pid,
                f"{cycles:.0f}" if cycles is not None else "n/a",
                f"{ref_cycles:.0f}" if ref_cycles is not None else "n/a",
                f"{instructions:.0f}" if instructions is not None else "n/a",
            )
