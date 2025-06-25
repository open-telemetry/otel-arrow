"""
Docker Component Monitoring Strategy Module

This module provides a monitoring strategy implementation that collects real-time
resource usage metrics (CPU, memory, and network I/O) from Docker containers
associated with managed components. It uses the Docker Python SDK to poll
container statistics and reports metrics via OpenTelemetry.

Classes:
    - DockerComponentMonitoringRuntime: Runtime state for monitoring, including thread and stop event.
    - DockerComponentMonitoringConfig: Configuration for monitoring intervals.
    - DockerComponentMonitoringStrategy: Monitoring strategy class that integrates with the
      component lifecycle to collect and expose metrics.

Functions:
    - monitor: Background thread function that periodically fetches container stats and logs/records them.

Features:
    - Polls container statistics at a configurable interval (default 1s).
    - Calculates normalized CPU usage and total memory usage.
    - Uses OpenTelemetry's Meter API to expose metrics as gauges.
    - Traces the monitoring lifecycle using OpenTelemetry spans.
    - Gracefully shuts down using a threading stop event.

Usage:
    This strategy is registered under the name `"docker_component"` in the monitoring
    registry and can be applied to any component deployed via Docker. It is designed to run
    transparently alongside test or scenario execution.

    Typical Yaml definition:

```yaml
components:
  backend-service:
    deployment:
      docker: ...
    monitoring:
      docker_component:
        interval: 1
```

Note:
    Monitoring occurs in a separate thread and is daemonized, meaning it will not block
    shutdown. However, proper cleanup via the `stop()` method is expected to avoid leaks.
"""
import threading
import time
from dataclasses import dataclass, field
from logging import LoggerAdapter
from typing import ClassVar, Literal, Optional, TYPE_CHECKING

import docker
from docker.errors import APIError
from opentelemetry import trace
from opentelemetry.sdk.metrics import Meter
from opentelemetry.trace import SpanKind

from ....core.strategies.monitoring_strategy import MonitoringStrategyConfig
from ....core.component.component import Component
from ....core.context.framework_element_contexts import StepContext, ScenarioContext
from ..common.docker import (
    ComponentDockerRuntime,
    get_or_create_docker_client,
)
from ....core.strategies.monitoring_strategy import MonitoringStrategy
from ....runner.registry import monitoring_registry, PluginMeta

if TYPE_CHECKING:
    from ....impl.component.managed_component import ManagedComponent


STRATEGY_NAME = "docker_component"


@dataclass
class DockerComponentMonitoringRuntime:
    """
    Runtime state holder for Docker component monitoring.

    This class encapsulates the runtime control data required to manage a monitoring
    thread associated with a Docker-based component. It includes the thread reference
    and a stop event used to cleanly terminate monitoring when required.

    Attributes:
        type (ClassVar[str]): A constant string identifier for the monitoring runtime type.
                              Used for internal runtime data lookup and storage.
        thread (Optional[threading.Thread]): The thread in which Docker stats monitoring runs.
        stop_event (threading.Event): An event used to signal the monitoring thread to stop.
    """
    type: ClassVar[Literal["docker_component_monitoring"]] = (
        "docker_component_monitoring"
    )
    thread: Optional[threading.Thread] = None
    stop_event: threading.Event = field(default_factory=threading.Event)


@monitoring_registry.register_config(STRATEGY_NAME)
class DockerComponentMonitoringConfig(MonitoringStrategyConfig):
    """
    Configuration class for the Docker component monitoring strategy.

    This configuration defines settings for how often resource usage stats
    (CPU, memory, network) should be collected from a running Docker container.

    Attributes:
        interval (Optional[float]): Time in seconds between each polling interval
                                    for metrics collection. Defaults to 1.0 second.
    """
    interval: Optional[float] = 1.0


@monitoring_registry.register_class(STRATEGY_NAME)
class DockerComponentMonitoringStrategy(MonitoringStrategy):
    """
    Strategy for monitoring a docker container resource mentrics via the python docker API client.

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
      docker: ...   # Component should be deployed via docker strategy.
    monitoring:
      docker_component:
        interval: 1
"""
    )
    def __init__(self, config: DockerComponentMonitoringConfig):
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
        client = get_or_create_docker_client(ctx)
        logger = ctx.get_logger(__name__)
        meter = ctx.get_meter(__name__)
        ts = ctx.get_suite()

        self.stop_event = threading.Event()
        test_suite_context = ts.context

        logger.debug(f"Starting monitoring for {component.name}...")
        monitoring_runtime: DockerComponentMonitoringRuntime = (
            component.get_or_create_runtime(
                DockerComponentMonitoringRuntime.type, DockerComponentMonitoringRuntime
            )
        )
        docker_runtime: ComponentDockerRuntime = component.get_or_create_runtime(
            ComponentDockerRuntime.type, ComponentDockerRuntime
        )
        if not docker_runtime.container_id:
            logger.error(
                "Error getting container ID from the docker deployment runtime...is it running?"
            )
            raise RuntimeError("can't find docker container id in deployment runtime.")
        monitor_args = {
            "container_id": docker_runtime.container_id,
            "component_name": component.name,
            "stop_event": self.stop_event,
            "client": client,
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
            DockerComponentMonitoringRuntime.type, monitoring_runtime
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
        monitoring_runtime: DockerComponentMonitoringRuntime = (
            component.get_or_create_runtime(
                DockerComponentMonitoringRuntime.type, DockerComponentMonitoringRuntime
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


def monitor(
    container_id: str,
    component_name: str,
    client: docker.DockerClient,
    stop_event: threading.Event,
    meter: Meter,
    logger: LoggerAdapter,
    test_suite_context: ScenarioContext,
    interval: float = 1.0,
):
    """
    Periodically monitors a Docker container's CPU and memory usage, updating
    metrics and logging performance statistics until a stop event is triggered.

    This function is intended to be run in a separate thread and will
    continuously poll the container's stats endpoint at the specified interval.
    It calculates CPU usage based on Docker's stats, measures memory usage,
    and updates both via OpenTelemetry meters. All collected statistics are
    recorded in a `ProcessStats` object.

    Parameters:
        container_id (str): The ID of the Docker container to monitor.
        component_name (str): Logical component name the container belongs to.
        client (docker.DockerClient): Docker client used to interact with the Docker API.
        stats (ProcessStats): Object to record and analyze collected statistics.
        stop_event (threading.Event): Event used to signal when to stop monitoring.
        meter (Meter): OpenTelemetry meter used to define and update metrics.
        logger (LoggerAdapter): Logger for diagnostic and error messages.
        test_suite_context (TestExecutionContext): Provides tracing context and instrumentation.
        interval (float): Polling interval in seconds (default is 1.0).

    Notes:
        - CPU usage is normalized to the number of available CPUs.
        - Memory usage is reported in bytes.
        - The function respects the provided stop_event and exits gracefully.
        - If any API or unexpected error occurs, it logs the error and stops monitoring.
    """
    cpu_usage_gauge = meter.create_gauge(
        "container.cpu.usage",
        "{cpu}",
        "Container's CPU usage, measured in cpus. Range from 0 to the number of allocatable CPUs",
    )
    memory_usage_gauge = meter.create_gauge(
        "container.memory.usage", "By", "Memory usage of the container."
    )
    network_rx_gauge = meter.create_gauge(
        "container.network.rx", "By", "Received network traffic in bytes"
    )
    network_tx_gauge = meter.create_gauge(
        "container.network.tx", "By", "Transmitted network traffic in bytes"
    )
    try:
        container = client.containers.get(container_id)
        container_name = container.name
    except APIError as e:
        logger.error(f"Could not retrieve container {container_id}: {e}")
        return

    tracer = test_suite_context.get_tracer("docker_monitor")
    ctx = trace.set_span_in_context(test_suite_context.span)
    with tracer.start_as_current_span(
        f"Docker Monitor: {container_name}", context=ctx, kind=SpanKind.PRODUCER
    ) as span:
        span.set_attributes(
            {
                "container.name": container_name,
                "container.id": container_id,
                "ctx.component_name": component_name,
            }
        )
        while not stop_event.is_set():
            try:
                start = time.time()
                stat_data = container.stats(stream=False)
                labels = {
                    "container_id": container_id[:12],
                    "container_name": container_name,
                    "component_name": component_name,
                }

                # Network usage calculation
                networks = stat_data.get("networks", {})
                rx_bytes = sum(net["rx_bytes"] for net in networks.values())
                tx_bytes = sum(net["tx_bytes"] for net in networks.values())
                network_rx_gauge.set(rx_bytes, labels)
                network_tx_gauge.set(tx_bytes, labels)

                # CPU usage calculation
                cpu_stats = stat_data["cpu_stats"]
                precpu_stats = stat_data["precpu_stats"]
                cpu_delta = (
                    cpu_stats["cpu_usage"]["total_usage"]
                    - precpu_stats["cpu_usage"]["total_usage"]
                )
                system_delta = (
                    cpu_stats["system_cpu_usage"] - precpu_stats["system_cpu_usage"]
                )

                cpu_usage = 0.0
                if system_delta > 0.0 and cpu_delta > 0.0:
                    num_cpus = (
                        len(cpu_stats["cpu_usage"].get("percpu_usage", []))
                        or cpu_stats["online_cpus"]
                    )
                    cpu_usage = (cpu_delta / system_delta) * num_cpus

                # Memory usage in Bytes
                mem_usage = stat_data["memory_stats"]["usage"]
                cpu_usage_gauge.set(cpu_usage, labels)
                memory_usage_gauge.set(mem_usage, labels)
                # stats.add_sample(cpu_usage, mem_usage)
                poll_duration = time.time() - start
                logger.debug(
                    f"Monitored Container {container_name} ({container_id[:12]}) Cur. CPU (#Cores): {cpu_usage:.2f} Cur Mem (By): {mem_usage:.2f} Took: {poll_duration:.2f}"
                )
                if poll_duration < interval:
                    time.sleep(interval - poll_duration)

            except APIError as e:
                logger.error(
                    f"Error collecting stats for container {container_name} ({container_id[:12]}): {e}"
                )
                break
            except Exception as e:
                logger.error(
                    f"Unexpected error while monitoring {container_name} ({container_id[:12]}): {e}"
                )
                break
