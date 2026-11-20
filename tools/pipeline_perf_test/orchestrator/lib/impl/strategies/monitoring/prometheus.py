import threading
from dataclasses import dataclass, field
from typing import ClassVar, List, Literal, Optional
from logging import LoggerAdapter
import requests
from pydantic import Field, PositiveFloat, model_validator
from opentelemetry import trace
from opentelemetry.sdk.metrics import Meter
from opentelemetry.trace import SpanKind
from prometheus_client import parser

from ....core.strategies.monitoring_strategy import (
    MonitoringStrategy,
    MonitoringStrategyConfig,
)
from ....runner.registry import monitoring_registry, PluginMeta
from ....core.component.component import Component
from ....core.context.framework_element_contexts import StepContext, ScenarioContext
from ..common.kubernetes import ComponentKubernetesRuntime
from ..common.kubernetes_port_forward import (
    KubernetesPortForwardConfig,
    KubernetesPortForwards,
)

STRATEGY_NAME = "prometheus"
MAX_CONSECUTIVE_SCRAPE_FAILURES = 3


@dataclass
class PrometheusMonitoringRuntime:
    """
    Runtime data holder for Prometheus monitoring strategy.

    This class stores the thread running the Prometheus metrics scraping loop
    and the threading event used to signal when to stop the monitoring process.

    Attributes:
        type (ClassVar[str]): Identifier for this runtime type, fixed as "prometheus_monitoring".
        thread (Optional[threading.Thread]): The thread instance executing the monitoring loop.
        stop_event (Optional[threading.Event]): Event used to signal the monitoring thread to stop.
    """

    type: ClassVar[Literal["prometheus_monitoring"]] = "prometheus_monitoring"
    thread: Optional[threading.Thread] = None
    stop_event: Optional[threading.Event] = None
    forwards: Optional[KubernetesPortForwards] = None
    error: Optional[Exception] = None
    completed: bool = False
    scrape_successes: dict[str, int] = field(default_factory=dict)


@monitoring_registry.register_config(STRATEGY_NAME)
class PrometheusMonitoringConfig(MonitoringStrategyConfig):
    """
    Configuration settings for the Prometheus monitoring strategy.

    Attributes:
        endpoint (str): The HTTP endpoint URL to scrape Prometheus metrics from.
        interval (Optional[PositiveFloat]): The polling interval in seconds between metric scrapes. Must be positive when set. Default is 1.0.
        count (Optional[int]): The number of times to scrape metrics before stopping. A value of 0 means unlimited. Default is 0.
        include (Optional[List[str]]): List of metric names to explicitly include. If empty, all metrics are included by default.
        exclude (Optional[List[str]]): List of metric names to exclude from scraping.
        request_timeout (PositiveFloat): Per-request timeout in seconds for scraping the
            endpoint. Bounds how long a single scrape can block so the monitoring thread stays
            responsive to the stop signal even when the target hangs. Must be a positive number;
            zero, negative, and null values are rejected so the bound cannot be disabled. Default is 10.0.
    """

    endpoint: Optional[str] = Field(default=None, min_length=1)
    kubernetes_port_forward: Optional[KubernetesPortForwardConfig] = None
    interval: Optional[PositiveFloat] = 1.0
    count: Optional[int] = 0
    include: Optional[List[str]] = []
    exclude: Optional[List[str]] = []
    request_timeout: PositiveFloat = 10.0

    @model_validator(mode="after")
    def validate_endpoint(self):
        """Require one endpoint source, preserving the existing URL syntax."""
        if (self.endpoint is None) == (self.kubernetes_port_forward is None):
            raise ValueError("Set exactly one of endpoint or kubernetes_port_forward")
        return self


@monitoring_registry.register_class(STRATEGY_NAME)
class PrometheusMonitoringStrategy(MonitoringStrategy):
    """
    Strategy for monitoring a prometheus endpoint for a component.

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
    monitoring:
      prometheus:
        endpoint: http://localhost:8888/metrics
        include:
          - otelcol_exporter_send_failed_log_records_total
          - otelcol_exporter_sent_log_records_total
          - otelcol_process_cpu_seconds_total
""",
    )

    def __init__(self, config: PrometheusMonitoringConfig):
        self.config = config

    def start(self, component: "Component", ctx: StepContext):
        """
        Start the monitoring process.

        This method initializes and starts the collection of monitoring data for the component.
        Args:
            component: The component instance to stop.
            ctx: The current execution context for the containing test step.
        """
        logger = ctx.get_logger(__name__)
        meter = ctx.get_meter(__name__)

        logger.debug(f"Starting prometheus monitoring for {component.name}...")
        previous = component.runtime.get(PrometheusMonitoringRuntime.type)
        if previous is not None and previous.thread is not None:
            if previous.thread.is_alive():
                raise RuntimeError("Prometheus monitoring is already running")
        monitoring_runtime = PrometheusMonitoringRuntime()
        component.set_runtime_data(PrometheusMonitoringRuntime.type, monitoring_runtime)
        monitoring_runtime.stop_event = threading.Event()
        ts = ctx.get_suite()
        test_suite_context = ts.context
        if self.config.kubernetes_port_forward is not None:
            target = component.runtime.get(ComponentKubernetesRuntime.type)
            if not isinstance(target, ComponentKubernetesRuntime):
                raise ValueError(
                    "Kubernetes Prometheus access requires existing.kubernetes adoption"
                )
            monitoring_runtime.forwards = KubernetesPortForwards(
                target,
                self.config.kubernetes_port_forward,
                self.config.request_timeout,
                logger,
            )
            ts.context.cleanup.callback(
                self._stop_runtime, monitoring_runtime, component.name, logger
            )
            monitoring_runtime.forwards.open()

        monitor_args = {
            "endpoint": self.config.endpoint,
            "component_name": component.name,
            "interval": self.config.interval,
            "count": self.config.count,
            "include": self.config.include,
            "exclude": self.config.exclude,
            "request_timeout": self.config.request_timeout,
            "stop_event": monitoring_runtime.stop_event,
            "meter": meter,
            "logger": logger,
            "test_suite_context": test_suite_context,
            "runtime": monitoring_runtime,
        }
        monitoring_runtime.thread = threading.Thread(
            target=monitor, kwargs=monitor_args, daemon=True
        )
        try:
            monitoring_runtime.thread.start()
        except RuntimeError:
            self._stop_runtime(monitoring_runtime, component.name, logger)
            raise
        component.set_runtime_data(PrometheusMonitoringRuntime.type, monitoring_runtime)

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
        monitoring_runtime: PrometheusMonitoringRuntime = (
            component.get_or_create_runtime(
                PrometheusMonitoringRuntime.type, PrometheusMonitoringRuntime
            )
        )

        if monitoring_runtime.stop_event is None:
            # Monitoring was never started (e.g. deploy failed and teardown still
            # runs the stop step); nothing to join.
            return

        self._stop_runtime(monitoring_runtime, component.name, logger)
        self.collect(component, ctx)

    def _stop_runtime(self, monitoring_runtime, component_name, logger):
        """Stop the worker before releasing its owned endpoint transports."""
        monitoring_runtime.stop_event.set()
        # Bound the join so a wedged scrape thread cannot hang the whole
        # orchestrator during teardown. The scrape itself is bounded by
        # request_timeout, so this is a defensive upper bound.
        join_timeout = self.join_timeout()
        if monitoring_runtime.forwards is not None:
            join_timeout += len(
                monitoring_runtime.forwards.endpoints
            ) * self.config.request_timeout + min(5.0, self.config.request_timeout)
        try:
            thread = monitoring_runtime.thread
            if thread is not None and thread.ident is not None:
                thread.join(timeout=join_timeout)
                if thread.is_alive():
                    if monitoring_runtime.forwards is not None:
                        raise RuntimeError(
                            "Kubernetes Prometheus monitoring did not stop"
                        )
                    logger.warning(
                        f"Monitoring thread for {component_name} did not stop within "
                        f"{join_timeout:.0f}s; abandoning it "
                        "(daemon thread will exit with the process)."
                    )
        finally:
            if monitoring_runtime.forwards is not None:
                monitoring_runtime.forwards.close()
        self._check_runtime(monitoring_runtime)

    def join_timeout(self) -> float:
        """
        Compute the join timeout.
        """
        req_time = self.config.request_timeout
        interval_time = self.config.interval or 1.0
        buffer = 5.0
        return req_time + interval_time + buffer

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
        runtime = _component.runtime.get(PrometheusMonitoringRuntime.type)
        self._check_runtime(runtime)
        return {}

    @staticmethod
    def _check_runtime(runtime: Optional[PrometheusMonitoringRuntime]) -> None:
        if runtime is not None and runtime.forwards is not None:
            if runtime.error is not None:
                raise RuntimeError(
                    f"Kubernetes Prometheus monitoring failed: {runtime.error}"
                ) from runtime.error
            if (
                runtime.thread is not None
                and runtime.thread.ident is not None
                and not runtime.thread.is_alive()
                and not runtime.completed
            ):
                raise RuntimeError(
                    "Kubernetes Prometheus monitoring exited unexpectedly"
                )
            if runtime.completed:
                missing = [
                    pod for pod, count in runtime.scrape_successes.items() if not count
                ]
                if missing:
                    raise RuntimeError(
                        f"No successful Prometheus scrapes for pods: {missing}"
                    )


def should_keep(
    name: str, include: list[str] = None, exclude: list[str] = None
) -> bool:
    """
    Determine whether a given name should be kept based on include and exclude lists.

    The function follows these rules:
    - If an `include` list is provided and non-empty, return True only if the name is in the `include` list.
    - Otherwise, if an `exclude` list is provided and non-empty, return True only if the name is NOT in the `exclude` list.
    - If neither list is provided or both are empty, return True.

    Args:
        name (str): The name to check.
        include (list[str], optional): List of names to include. Defaults to None.
        exclude (list[str], optional): List of names to exclude. Defaults to None.

    Returns:
        bool: True if the name should be kept, False otherwise.
    """
    if include and len(include) > 0:
        return name in include
    elif exclude and len(exclude) > 0:
        return name not in exclude
    return True


# Cache instruments by (metric_name, metric_type)
instrument_cache = {}


def get_instrument(meter: Meter, metric_name: str, metric_type: str):
    """
    Retrieve or create a metric instrument from the given meter, caching it for reuse.

    This function checks if an instrument identified by the tuple (metric_name, metric_type)
    already exists in the cache. If so, it returns the cached instrument. Otherwise, it
    creates a new instrument (currently always a gauge), caches it, and returns it.

    Args:
        meter (Meter): The OpenTelemetry Meter used to create instruments.
        metric_name (str): The name of the metric instrument.
        metric_type (str): The type of the metric instrument (e.g., "gauge", "counter").
                           Currently, this argument does not affect the created instrument
                           as all are treated as gauges.

    Returns:
        Instrument: The requested or newly created metric instrument.
    """
    key = (metric_name, metric_type)
    if key in instrument_cache:
        return instrument_cache[key]

    # TODO: Everything is a gauge for now
    instrument = meter.create_gauge(metric_name)

    instrument_cache[key] = instrument
    return instrument


def scrape_and_convert_metrics(
    endpoint: str,
    component_name: str,
    meter: Meter,
    include: list[str] = None,
    exclude: list[str] = None,
    request_timeout: Optional[float] = 10.0,
    attributes: Optional[dict[str, str]] = None,
):
    """
    Scrape metrics from a Prometheus endpoint, filter them, and record them to an OpenTelemetry meter.

    This function fetches metrics data from the specified HTTP endpoint in Prometheus text format,
    parses the metrics, applies optional inclusion/exclusion filters on metric names, and converts
    the metrics into OpenTelemetry gauge instruments. The metrics are annotated with the given
    component name as an additional label.

    Args:
        endpoint (str): The URL of the Prometheus metrics endpoint to scrape.
        component_name (str): The name of the component to add as a label to each metric.
        meter (Meter): The OpenTelemetry meter to record metrics into.
        include (list[str], optional): List of metric names to include. If specified, only metrics
            with names in this list will be recorded. Defaults to None (include all).
        exclude (list[str], optional): List of metric names to exclude. If specified and
            include is None, metrics with names in this list will be skipped. Defaults to None.
        request_timeout (float, optional): Per-request timeout in seconds passed to
            requests.get so a hung endpoint cannot block the scrape indefinitely. Defaults to 10.0.

    Raises:
        requests.HTTPError: If the HTTP GET request to the endpoint fails or returns a bad status.

    Notes:
        - The function uses the `should_keep` helper to decide which metrics to process based
          on include/exclude filters.
        - All metrics are recorded as gauges via the `get_instrument` function, regardless of
          their original type.
        - Each metric sample is tagged with the component name as a label named 'component_name'.
    """
    resp = requests.get(endpoint, timeout=request_timeout)
    resp.raise_for_status()
    metrics_text = resp.text

    for family in parser.text_string_to_metric_families(metrics_text):
        instrument = None
        for sample in family.samples:
            name, labels, value, *_ = sample
            labels.update(attributes or {})
            labels["component_name"] = component_name
            if not should_keep(name, include=include, exclude=exclude):
                continue
            # Record the metric
            if not instrument:
                instrument = get_instrument(meter, family.name, family.type)
            instrument.set(value, labels)


def monitor(
    component_name: str,
    endpoint: str,
    include: List[str],
    exclude: List[str],
    interval: float,
    count: float,
    stop_event: threading.Event,
    meter: Meter,
    logger: LoggerAdapter,
    test_suite_context: ScenarioContext,
    request_timeout: Optional[float] = 10.0,
    runtime: Optional[PrometheusMonitoringRuntime] = None,
):
    """
    Continuously scrape and record Prometheus metrics from a specified endpoint at regular intervals.

    This function runs a monitoring loop that fetches metrics from the given Prometheus
    endpoint, filters them based on inclusion/exclusion lists, and records them using
    OpenTelemetry metrics instruments. The monitoring runs until a stop event is set or
    a specified count of iterations has been completed.

    The function also creates and manages an OpenTelemetry tracing span to instrument
    the monitoring process.

    Args:
        component_name (str): The logical name of the component being monitored; added as a metric label and span attribute.
        endpoint (str): URL of the Prometheus metrics endpoint to scrape.
        include (List[str]): List of metric names to include. Metrics not in this list will be ignored. Can be empty to include all.
        exclude (List[str]): List of metric names to exclude. Metrics in this list will be skipped unless included explicitly.
        interval (float): Time in seconds to wait between scrapes.
        count (float): Number of times to scrape metrics; if zero or negative, runs indefinitely until stopped.
        stop_event (threading.Event): Threading event used to signal early termination of monitoring.
        meter (Meter): OpenTelemetry meter used to record metrics.
        logger (LoggerAdapter): Logger instance for error and debug messages.
        test_suite_context (ScenarioContext): Context providing tracing instrumentation and span.
        request_timeout (float): Per-request timeout in seconds for each scrape, keeping the loop
            responsive to `stop_event` even when the target endpoint hangs.

    Behavior:
        - Creates an OpenTelemetry producer span for the duration of monitoring.
        - On each iteration, scrapes metrics and records them, then sleeps for `interval` seconds.
        - Respects the `stop_event` to terminate monitoring early.
        - If `count` is set > 0, stops after that many scrapes.
        - Direct endpoints log scrape errors and continue until stopped.
        - Forwarded endpoints retry transient transport errors on the next interval,
          failing after three consecutive failures per pod. Other errors are fatal.

    Raises:
        Forwarded scrape errors are captured in the runtime and surfaced during
        collection or cleanup.
    """
    tracer = test_suite_context.get_tracer("prometheus_monitor")
    forwards = runtime.forwards if runtime is not None else None
    errors = (
        requests.RequestException,
        OSError,
        ValueError,
        TypeError,
        KeyError,
        RuntimeError,
        ArithmeticError,
        AttributeError,
    )
    if forwards is not None:
        from kubernetes.client.exceptions import ApiException
        from kubernetes.config.config_exception import ConfigException
        from urllib3.exceptions import HTTPError

        errors += (ApiException, ConfigException, HTTPError)
        scrape_count = meter.create_counter(
            "prometheus.scrape.count",
            unit="{scrape}",
            description="Per-pod Prometheus scrape attempts by outcome.",
        )
        consecutive_failures: dict[str, int] = {}
        runtime.scrape_successes = {
            target.attributes["k8s.pod.name"]: 0 for target in forwards.endpoints
        }

    ctx = trace.set_span_in_context(test_suite_context.span)
    with tracer.start_as_current_span(
        f"Prometheus Monitor: {endpoint}", context=ctx, kind=SpanKind.PRODUCER
    ) as span:
        if component_name:
            span.set_attribute("ctx.component", component_name)
        remaining = count if count > 0 else None
        while not stop_event.is_set() and (remaining is None or remaining > 0):
            try:
                if forwards is None:
                    scrape_and_convert_metrics(
                        endpoint,
                        component_name,
                        meter,
                        include=include,
                        exclude=exclude,
                        request_timeout=request_timeout,
                    )
                else:
                    forwards.validate()
                    for target in forwards.endpoints:
                        if stop_event.is_set():
                            break
                        attributes = {
                            **target.attributes,
                            "component_name": component_name,
                        }
                        pod_uid = target.attributes["k8s.pod.uid"]
                        pod_name = target.attributes["k8s.pod.name"]
                        try:
                            scrape_and_convert_metrics(
                                target.url,
                                component_name,
                                meter,
                                include=include,
                                exclude=exclude,
                                request_timeout=request_timeout,
                                attributes=target.attributes,
                            )
                        except errors as error:
                            scrape_count.add(1, {**attributes, "outcome": "failure"})
                            if not isinstance(
                                error, (requests.Timeout, requests.ConnectionError)
                            ) or isinstance(error, requests.exceptions.SSLError):
                                raise
                            failures = consecutive_failures.get(pod_uid, 0) + 1
                            consecutive_failures[pod_uid] = failures
                            if failures >= MAX_CONSECUTIVE_SCRAPE_FAILURES:
                                raise RuntimeError(
                                    f"Prometheus scrape for pod {pod_name!r} failed "
                                    f"{failures} consecutive attempts: {error}"
                                ) from error
                            logger.warning(
                                "Prometheus scrape for pod %r failed (%d/%d); "
                                "retrying next interval: %s",
                                pod_name,
                                failures,
                                MAX_CONSECUTIVE_SCRAPE_FAILURES,
                                error,
                            )
                        else:
                            runtime.scrape_successes[pod_name] += 1
                            scrape_count.add(1, {**attributes, "outcome": "success"})
                            if consecutive_failures.pop(pod_uid, 0):
                                logger.info(
                                    "Prometheus scrape for pod %r recovered", pod_name
                                )
                stop_event.wait(interval or 1.0)
                if remaining is not None:
                    remaining -= 1

            except errors as e:
                logger.error(
                    f"Error collecting stats for component {component_name}: {e}"
                )
                if forwards is not None:
                    runtime.error = e
                    stop_event.set()
                    break
                stop_event.wait(interval or 1.0)
    if runtime is not None:
        runtime.completed = True
