"""Read container resource usage from Kubernetes metrics-server."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
import math
import re
import threading
from typing import ClassVar, Optional

from opentelemetry.metrics import _Gauge
from pydantic import ConfigDict, PositiveFloat

from ....core.component.component import Component
from ....core.context.framework_element_contexts import StepContext, ScenarioContext
from ....core.strategies.monitoring_strategy import (
    MonitoringStrategy,
    MonitoringStrategyConfig,
)
from ....runner.registry import monitoring_registry, PluginMeta
from ..common.kubernetes import (
    ComponentKubernetesRuntime,
    nonterminal_pods,
    pod_identity,
)

STRATEGY_NAME = "kubernetes_component"
_DURATION_PART = re.compile(r"(\d+(?:\.\d+)?)(ns|us|ms|s|m|h)")
_DURATION_UNITS = {"ns": 1e-9, "us": 1e-6, "ms": 1e-3, "s": 1, "m": 60, "h": 3600}


@monitoring_registry.register_config(STRATEGY_NAME)
class KubernetesComponentMonitoringConfig(MonitoringStrategyConfig):
    """Poll adopted containers, rejecting stale or incomplete resource samples."""

    model_config = ConfigDict(extra="forbid", allow_inf_nan=False)

    interval: PositiveFloat = 5
    max_age_seconds: PositiveFloat = 120
    request_timeout: PositiveFloat = 10


@dataclass(frozen=True)
class ContainerResourceSample:
    """CPU usage and working set from one Kubernetes source observation."""

    pod: str
    container: str
    source_timestamp: str
    timestamp: datetime
    window_seconds: float
    cpu_cores: float
    memory_bytes: float


@dataclass
class KubernetesMonitoringRuntime:
    """Thread state and source timestamps already recorded for each pod."""

    type: ClassVar[str] = "kubernetes_component_monitoring"
    stop_event: threading.Event = field(default_factory=threading.Event)
    thread: Optional[threading.Thread] = None
    error: Optional[Exception] = None
    completed: bool = False
    timestamps: dict[str, datetime] = field(default_factory=dict)


def window_seconds(value: str) -> float:
    """Parse the positive Go duration reported by the Kubernetes metrics API."""
    parts = list(_DURATION_PART.finditer(value))
    if not parts or "".join(part[0] for part in parts) != value:
        raise ValueError(f"Invalid Kubernetes metrics window: {value!r}")
    seconds = sum(float(part[1]) * _DURATION_UNITS[part[2]] for part in parts)
    if not math.isfinite(seconds) or seconds <= 0:
        raise ValueError(f"Invalid Kubernetes metrics window: {value!r}")
    return seconds


def resource_quantity(value: str) -> float:
    """Convert Kubernetes quantities to finite, nonnegative base units."""
    from kubernetes.utils.quantity import parse_quantity

    number = float(parse_quantity(value))
    if not math.isfinite(number) or number < 0:
        raise ValueError(f"Invalid Kubernetes resource quantity: {value!r}")
    return number


def read_samples(
    target: ComponentKubernetesRuntime,
    config: KubernetesComponentMonitoringConfig,
) -> list[ContainerResourceSample]:
    """Read a complete sample batch and verify the adopted identities are stable."""
    from kubernetes import client, config as kube_config

    timeout = (min(5.0, config.request_timeout), config.request_timeout)
    with kube_config.new_client_from_config(
        config_file=target.kubeconfig,
        context=target.context,
        persist_config=False,
    ) as api_client:
        metrics = client.CustomObjectsApi(api_client).list_namespaced_custom_object(
            group="metrics.k8s.io",
            version="v1beta1",
            namespace=target.namespace,
            plural="pods",
            label_selector=target.label_selector,
            _request_timeout=timeout,
        )
        current = client.CoreV1Api(api_client).list_namespaced_pod(
            namespace=target.namespace,
            label_selector=target.label_selector,
            _request_timeout=timeout,
        )
    expected = {pod.name: pod for pod in target.pods}
    live_pods = nonterminal_pods(current.items)
    identities = [pod_identity(pod) for pod in live_pods]
    if (
        len(identities) != len(expected)
        or {pod.name: pod for pod in identities} != expected
    ):
        raise ValueError("Kubernetes pod/container identities changed; adopt again")

    now = datetime.now(timezone.utc)
    samples = []
    observed = set()
    terminal_names = {pod.metadata.name for pod in current.items} - set(expected)
    for item in metrics["items"]:
        name = item["metadata"]["name"]
        if name in terminal_names:
            continue
        if name not in expected or name in observed:
            raise ValueError(f"Unexpected or duplicate Kubernetes metrics pod {name!r}")
        observed.add(name)
        if item["metadata"].get("namespace") != target.namespace:
            raise ValueError(
                f"Kubernetes metrics pod {name!r} has a different namespace"
            )
        uid = item["metadata"].get("uid")
        if uid is not None and uid != expected[name].uid:
            raise ValueError(f"Kubernetes metrics pod {name!r} has a different UID")
        source_timestamp = item["timestamp"]
        timestamp = datetime.fromisoformat(source_timestamp.replace("Z", "+00:00"))
        if timestamp.tzinfo is None:
            raise ValueError(
                f"Kubernetes metrics pod {name!r} has no timestamp timezone"
            )
        age = (now - timestamp).total_seconds()
        if not -5 <= age <= config.max_age_seconds:
            raise ValueError(
                f"Stale or future Kubernetes metrics for {name!r}: {age:.1f}s"
            )
        window = window_seconds(item["window"])
        containers = item["containers"]
        names = [container["name"] for container in containers]
        if len(set(names)) != len(names) or set(names) != set(
            expected[name].containers
        ):
            raise ValueError(
                f"Incomplete or unexpected metrics containers for {name!r}"
            )
        for container in containers:
            usage = container["usage"]
            if "cpu" not in usage or "memory" not in usage:
                raise ValueError(f"Missing CPU or memory metrics for pod {name!r}")
            samples.append(
                ContainerResourceSample(
                    pod=name,
                    container=container["name"],
                    source_timestamp=source_timestamp,
                    timestamp=timestamp,
                    window_seconds=window,
                    cpu_cores=resource_quantity(usage["cpu"]),
                    memory_bytes=resource_quantity(usage["memory"]),
                )
            )
    if observed != set(expected):
        raise ValueError(
            f"Missing Kubernetes metrics pods: {sorted(set(expected) - observed)}"
        )
    return samples


@monitoring_registry.register_class(STRATEGY_NAME)
class KubernetesComponentMonitoringStrategy(MonitoringStrategy):
    """Monitor adopted containers using Kubernetes metrics-server without a proxy."""

    PLUGIN_META = PluginMeta(
        supported_contexts=[StepContext.__name__],
        installs_hooks=[],
        notes=(
            "Requires existing.kubernetes, successful adoption, and metrics-server. "
            "Uses the adoption kubeconfig; no engine metrics endpoint is required."
        ),
        yaml_example="""
components:
  existing-service:
    existing:
      kubernetes:
        namespace: telemetry
        label_selector: app=my-service
    monitoring:
      kubernetes_component:
        interval: 5
        max_age_seconds: 120
""",
    )

    def __init__(self, config: KubernetesComponentMonitoringConfig) -> None:
        self.config = config

    def start(self, component: Component, ctx: StepContext) -> None:
        """Validate initial samples synchronously, then poll in a bounded thread."""
        target = component.runtime.get(ComponentKubernetesRuntime.type)
        if not isinstance(target, ComponentKubernetesRuntime):
            raise ValueError(
                "Kubernetes monitoring requires existing.kubernetes adoption"
            )
        previous = component.runtime.get(KubernetesMonitoringRuntime.type)
        if previous is not None and previous.thread is not None:
            if previous.thread.is_alive():
                raise RuntimeError("Kubernetes monitoring is already running")
        runtime = KubernetesMonitoringRuntime()
        component.set_runtime_data(KubernetesMonitoringRuntime.type, runtime)
        meter = ctx.get_meter(__name__)
        if meter is None:
            raise RuntimeError(
                "Kubernetes resource monitoring requires a telemetry meter"
            )
        cpu = meter.create_gauge(
            "container.cpu.usage", unit="{cpu}", description="CPU usage in cores."
        )
        memory = meter.create_gauge(
            "container.memory.working_set",
            unit="By",
            description="Container working set reported by Kubernetes metrics-server.",
        )
        self._poll(component, target, runtime, cpu, memory, ctx)
        runtime.thread = threading.Thread(
            target=self._monitor,
            args=(component, target, runtime, cpu, memory, ctx),
            daemon=True,
        )
        ctx.get_suite().context.cleanup.callback(self._stop_runtime, runtime)
        runtime.thread.start()

    def _poll(
        self,
        component: Component,
        target: ComponentKubernetesRuntime,
        runtime: KubernetesMonitoringRuntime,
        cpu: _Gauge,
        memory: _Gauge,
        ctx: StepContext,
    ) -> None:
        samples = read_samples(target, self.config)
        source_timestamps = {sample.pod: sample.timestamp for sample in samples}
        new_timestamps = {}
        for pod, timestamp in source_timestamps.items():
            previous = runtime.timestamps.get(pod)
            if previous is None or timestamp > previous:
                new_timestamps[pod] = timestamp
            elif timestamp < previous:
                ctx.get_logger(__name__).warning(
                    f"Ignoring older Kubernetes metrics for {pod!r}: "
                    f"source timestamp {timestamp.isoformat()} precedes "
                    f"last recorded timestamp {previous.isoformat()}"
                )
        observed_at = datetime.now(timezone.utc).isoformat()
        identities = {pod.name: pod for pod in target.pods}
        for sample in samples:
            if sample.pod not in new_timestamps:
                continue
            pod = identities[sample.pod]
            attributes = {
                "component_name": component.name,
                "k8s.namespace.name": target.namespace,
                "k8s.pod.name": pod.name,
                "k8s.pod.uid": pod.uid,
                "k8s.node.name": pod.node_name,
                "container_name": sample.container,
                "container_id": pod.containers[sample.container],
                "sample.timestamp": sample.source_timestamp,
                "sample.window_seconds": sample.window_seconds,
                "sample.observed_at": observed_at,
            }
            cpu.set(sample.cpu_cores, attributes)
            memory.set(sample.memory_bytes, attributes)
        runtime.timestamps.update(new_timestamps)

    def _monitor(
        self,
        component: Component,
        target: ComponentKubernetesRuntime,
        runtime: KubernetesMonitoringRuntime,
        cpu: _Gauge,
        memory: _Gauge,
        ctx: StepContext,
    ) -> None:
        from kubernetes.client.exceptions import ApiException
        from kubernetes.config.config_exception import ConfigException
        from urllib3.exceptions import HTTPError

        try:
            while not runtime.stop_event.wait(self.config.interval):
                self._poll(component, target, runtime, cpu, memory, ctx)
        except (
            ApiException,
            ConfigException,
            HTTPError,
            OSError,
            ValueError,
            KeyError,
            TypeError,
            AttributeError,
            ArithmeticError,
        ) as error:
            runtime.error = error
            runtime.stop_event.set()
            ctx.get_logger(__name__).exception("Kubernetes resource monitoring failed")
        runtime.completed = True

    def stop(self, component: Component, ctx: StepContext) -> None:
        """Stop polling and propagate background measurement errors to the scenario."""
        runtime = component.runtime.get(KubernetesMonitoringRuntime.type)
        if runtime is None or runtime.thread is None:
            return
        self._stop_runtime(runtime)

    def _stop_runtime(self, runtime: KubernetesMonitoringRuntime) -> None:
        """Stop the captured worker, including when a suite skips normal shutdown."""
        runtime.stop_event.set()
        timeout = (
            2 * (min(5.0, self.config.request_timeout) + self.config.request_timeout)
            + 5
        )
        thread = runtime.thread
        if thread is not None and thread.ident is not None:
            thread.join(timeout=timeout)
            if thread.is_alive():
                raise RuntimeError(
                    "Kubernetes resource monitoring did not stop in time"
                )
        self._check_runtime(runtime)

    def collect(
        self, component: Component, _ctx: ScenarioContext | StepContext
    ) -> dict:
        """Check measurement health; samples use the existing telemetry backend."""
        runtime = component.runtime.get(KubernetesMonitoringRuntime.type)
        self._check_runtime(runtime)
        return {}

    @staticmethod
    def _check_runtime(runtime: Optional[KubernetesMonitoringRuntime]) -> None:
        if runtime is not None and runtime.error is not None:
            raise RuntimeError(
                f"Kubernetes resource monitoring failed: {runtime.error}"
            ) from runtime.error
        if (
            runtime is not None
            and runtime.thread is not None
            and runtime.thread.ident is not None
            and not runtime.thread.is_alive()
            and not runtime.completed
        ):
            raise RuntimeError("Kubernetes resource monitoring exited unexpectedly")
