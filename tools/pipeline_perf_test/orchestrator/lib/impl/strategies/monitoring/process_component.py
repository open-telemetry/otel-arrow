"""
Process Component Monitoring Strategy Module.

This module implements a monitoring strategy for components deployed as operating system
processes. It integrates with the OpenTelemetry framework to periodically collect and
report resource usage metrics (CPU and memory) for a given process ID (PID) using the
`psutil` library.

The strategy is designed to work within a test automation framework where components are
deployed and monitored in real time. The implementation includes:

- A dataclass to hold runtime monitoring thread state (`ProcessComponentMonitoringRuntime`).
- A configuration class to control monitoring interval (`ProcessComponentMonitoringConfig`).
- A strategy class (`ProcessComponentMonitoringStrategy`) that defines how monitoring is
  started, stopped, and how data is collected.
- A `monitor` function that runs in a background thread, collecting CPU and memory stats
  from the target process at regular intervals, and pushing them to OpenTelemetry metrics.

The strategy is registered under the name `"process_component"` and is compatible with
step-level execution contexts.


Usage Example (YAML):
components:
  backend-service:
    deployment:
      process: ...
    monitoring:
      process_component:
        interval: 1.0
"""
import threading
import time
from dataclasses import dataclass, field
from logging import LoggerAdapter
from typing import ClassVar, Literal, Optional, TYPE_CHECKING

import psutil
from opentelemetry import trace
from opentelemetry.sdk.metrics import Meter
from opentelemetry.trace import SpanKind

from ....core.strategies.monitoring_strategy import MonitoringStrategyConfig
from ....core.component.component import Component
from ....core.context.framework_element_contexts import StepContext, ScenarioContext
from ..deployment.process import (
    ComponentProcessRuntime,
)
from ....core.strategies.monitoring_strategy import MonitoringStrategy
from ....runner.registry import monitoring_registry, PluginMeta

if TYPE_CHECKING:
    from ....impl.component.managed_component import ManagedComponent


STRATEGY_NAME = "process_component"


@dataclass
class ProcessComponentMonitoringRuntime:
    """
    Runtime state holder for Process component monitoring.

    This class encapsulates the runtime control data required to manage a monitoring
    thread associated with a Process-based component. It includes the thread reference
    and a stop event used to cleanly terminate monitoring when required.

    Attributes:
        type (ClassVar[str]): A constant string identifier for the monitoring runtime type.
                              Used for internal runtime data lookup and storage.
        thread (Optional[threading.Thread]): The thread in which Process stats monitoring runs.
        stop_event (threading.Event): An event used to signal the monitoring thread to stop.
    """

    type: ClassVar[Literal["process_component_monitoring"]] = (
        "process_component_monitoring"
    )
    thread: Optional[threading.Thread] = None
    stop_event: threading.Event = field(default_factory=threading.Event)


@monitoring_registry.register_config(STRATEGY_NAME)
class ProcessComponentMonitoringConfig(MonitoringStrategyConfig):
    """
    Configuration class for the Process component monitoring strategy.

    This configuration defines settings for how often resource usage stats
    (CPU, memory, network) should be collected from a running Process.

    Attributes:
        interval (Optional[float]): Time in seconds between each polling interval
                                    for metrics collection. Defaults to 1.0 second.
    """

    interval: Optional[float] = 1.0


@monitoring_registry.register_class(STRATEGY_NAME)
class ProcessComponentMonitoringStrategy(MonitoringStrategy):
    """
    Strategy for monitoring a Process via the python psutil library.

    Monitoring strategies define how to start, stop, and collect data from a component's monitoring
    system. Concrete implementations should specify how to track, log, and aggregate monitoring
    data for a given component.

    Methods:
        start(component, ctx): Begin the monitoring process for the component.
        stop(component, ctx): Stop the monitoring process.
        collect(component, ctx): Collect and return monitoring data as a dictionary.
    """

    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        yaml_example="""
components:
  backend-service:
    deployment:
      process: ...   # Component should be deployed via process strategy.
    monitoring:
      process_component:
        interval: 1
""",
    )

    def __init__(self, config: ProcessComponentMonitoringConfig):
        super().__init__(STRATEGY_NAME)
        self.config = config
        self.stop_event = None

    def start(self, component: "ManagedComponent", ctx: StepContext):
        """
        Start the monitoring process.

        This method initializes and starts the collection of monitoring data for the component.
        Args:
            component: The component instance to stop.
            ctx: The current execution context for the containing test step.
        """
        logger = ctx.get_logger(__name__)
        meter = ctx.get_meter(__name__)
        ts = ctx.get_suite()

        self.stop_event = threading.Event()
        test_suite_context = ts.context

        logger.debug(f"Starting monitoring for {component.name}...")
        monitoring_runtime: ProcessComponentMonitoringRuntime = (
            component.get_or_create_runtime(
                ProcessComponentMonitoringRuntime.type,
                ProcessComponentMonitoringRuntime,
            )
        )
        process_runtime: ComponentProcessRuntime = component.get_or_create_runtime(
            ComponentProcessRuntime.type, ComponentProcessRuntime
        )
        if not process_runtime.pid:
            logger.error(
                "Error getting process ID from the deployment runtime...is it running?"
            )
            raise RuntimeError("can't find process id in deployment runtime.")
        monitor_args = {
            "pid": process_runtime.pid,
            "component_name": component.name,
            "stop_event": self.stop_event,
            "meter": meter,
            "logger": logger,
            "test_suite_context": test_suite_context,
            "interval": self.config.interval,
        }
        monitoring_runtime.thread = threading.Thread(
            target=monitor, kwargs=monitor_args, daemon=True
        )
        monitoring_runtime.thread.start()
        component.set_runtime_data(
            ProcessComponentMonitoringRuntime.type, monitoring_runtime
        )

    def stop(self, component: Component, ctx: StepContext):
        """
        Stop the monitoring process.

        This method shuts down any active monitoring and ensures data collection is concluded.
        Args:
            component: The component instance to stop.
            ctx: The current execution context for the containing test step.
        """
        logger = ctx.get_logger(__name__)
        logger.debug(f"Stopping monitoring for {component.name}")
        monitoring_runtime: ProcessComponentMonitoringRuntime = (
            component.get_or_create_runtime(
                ProcessComponentMonitoringRuntime.type,
                ProcessComponentMonitoringRuntime,
            )
        )
        self.stop_event.set()
        monitoring_runtime.thread.join()

    def collect(self, _component: Component, _ctx: ScenarioContext) -> dict:
        """
        Collect and return monitoring data.

        This method aggregates and returns the collected monitoring data as a dictionary.

        Args:
            component: The component instance to stop.
            ctx: The current execution context for the containing test step.

        Returns:
            dict: A dictionary of collected monitoring data.
        """
        return {}


def get_resources(proc: psutil.Process):
    """Get current resource utilization for the process and it's children.

    Parameters:
        proc: the process to monitor, including any spawned children.

    Returns:
        A tuple of cpu and memory values for the process and all children.
    """
    children = proc.children(recursive=True)
    all_procs = [proc] + children
    cpu = 0.0
    memory = 0.0

    for p in all_procs:
        try:
            times = p.cpu_times()  # short interval
            cpu += times.user + times.system
            memory += p.memory_info().rss      # in bytes
        except psutil.NoSuchProcess:
            continue
    return cpu, memory


def monitor(
    pid: int,
    component_name: str,
    stop_event: threading.Event,
    meter: Meter,
    logger: LoggerAdapter,
    test_suite_context: ScenarioContext,
    interval: float = 1.0,
):
    """
    Monitors an OS process by PID, reporting CPU and memory usage periodically.

    Parameters:
        pid (int): PID of the process to monitor.
        component_name (str): Logical component name the process belongs to.
        stop_event (threading.Event): Event used to signal when to stop monitoring.
        meter (Meter): OpenTelemetry meter used to define and update metrics.
        logger (LoggerAdapter): Logger for diagnostics and error messages.
        test_suite_context: Provides tracing context and instrumentation.
        interval (float): Polling interval in seconds.
    """

    cpu_usage_gauge = meter.create_gauge(
        "process.cpu.usage",
        "{cpu}",
        "Process CPU usage, measured in CPUs (0 to number of logical CPUs)",
    )
    memory_usage_gauge = meter.create_gauge(
        "process.memory.usage", "By", "Memory usage of the process."
    )

    tracer = test_suite_context.get_tracer("process_monitor")
    ctx = trace.set_span_in_context(test_suite_context.span)

    try:
        proc = psutil.Process(pid)
        with tracer.start_as_current_span(
            f"Process Monitor: PID {pid}", context=ctx, kind=SpanKind.PRODUCER
        ) as span:
            span.set_attributes(
                {
                    "process.pid": pid,
                    "ctx.component_name": component_name,
                }
            )

            (prev_total_time, _prev_mem) = get_resources(proc)
            prev_time = time.time()

            while not stop_event.is_set():
                try:
                    time.sleep(interval)

                    (cur_total_time, mem_usage) = get_resources(proc)
                    cur_time = time.time()

                    delta_proc = cur_total_time - prev_total_time
                    delta_time = cur_time - prev_time

                    cpu_usage = delta_proc / delta_time


                    labels = {
                        "pid": str(pid),
                        "component_name": component_name,
                    }

                    cpu_usage_gauge.set(cpu_usage, labels)
                    memory_usage_gauge.set(mem_usage, labels)

                    logger.debug(
                        f"Monitored Process {component_name} PID {pid}: Cur. CPU (#Cores): {cpu_usage:.2f} Cur Mem (By): {mem_usage:.0f}"
                    )

                    prev_total_time = cur_total_time
                    prev_time = cur_time

                except psutil.NoSuchProcess:
                    logger.warning(f"Process PID {pid} exited. Monitoring stopped.")
                    break
                except Exception as e:
                    logger.error(f"Error monitoring process PID {pid}: {e}")
                    break

    except psutil.NoSuchProcess:
        logger.error(f"Process PID {pid} does not exist.")
    except Exception as e:
        logger.error(f"Could not start monitoring for PID {pid}: {e}")
