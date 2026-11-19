"""Kubernetes source measurements, identity stability, and monitor failures."""

from copy import deepcopy
from datetime import datetime, timedelta, timezone
from unittest.mock import MagicMock, Mock, patch

from kubernetes import client
from kubernetes.client.exceptions import ApiException
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from pydantic import ValidationError
import pytest

from lib.impl import strategies  # noqa: F401
from lib.core.telemetry.metric import FrameworkMetricBackend, FrameworkMetricExporter
from lib.impl.component.managed_component import ManagedComponentConfiguration
from lib.impl.strategies.common.kubernetes import (
    ComponentKubernetesRuntime,
    pod_identity,
)
from lib.impl.strategies.monitoring.kubernetes_component import (
    KubernetesComponentMonitoringConfig,
    KubernetesComponentMonitoringStrategy,
    KubernetesMonitoringRuntime,
    read_samples,
)
from lib.runner.factory import build_managed_component

MODULE = "lib.impl.strategies.monitoring.kubernetes_component"


@pytest.fixture
def metrics_api():
    def status(name):
        return client.V1ContainerStatus(
            name=name,
            image="example",
            image_id="sha256:example",
            container_id=f"containerd://{name}",
            ready=True,
            restart_count=0,
            state=client.V1ContainerState(running=client.V1ContainerStateRunning()),
        )

    pod = client.V1Pod(
        metadata=client.V1ObjectMeta(name="engine-0", uid="uid-engine"),
        spec=client.V1PodSpec(
            node_name="node-0",
            containers=[client.V1Container(name="engine")],
            init_containers=[
                client.V1Container(name="setup"),
                client.V1Container(name="sidecar", restart_policy="Always"),
            ],
        ),
        status=client.V1PodStatus(
            phase="Running",
            conditions=[client.V1PodCondition(type="Ready", status="True")],
            container_statuses=[status("engine")],
            init_container_statuses=[status("sidecar")],
        ),
    )
    target = ComponentKubernetesRuntime(
        namespace="telemetry",
        label_selector="app=engine",
        context="benchmark",
        kubeconfig="/caller/kubeconfig",
        pods=[pod_identity(pod)],
    )
    response = {
        "items": [
            {
                "metadata": {"name": "engine-0", "namespace": "telemetry"},
                "timestamp": (
                    datetime.now(timezone.utc) - timedelta(seconds=2)
                ).isoformat(),
                "window": "1m6.318s",
                "containers": [
                    {"name": name, "usage": {"cpu": "250m", "memory": "8Mi"}}
                    for name in ("engine", "sidecar")
                ],
            }
        ]
    }
    core = Mock(spec_set=["list_namespaced_pod"])
    core.list_namespaced_pod.return_value = client.V1PodList(items=[pod])
    metrics = Mock(spec_set=["list_namespaced_custom_object"])
    metrics.list_namespaced_custom_object.return_value = response
    with (
        patch("kubernetes.config.new_client_from_config") as connection,
        patch("kubernetes.client.CoreV1Api", return_value=core),
        patch("kubernetes.client.CustomObjectsApi", return_value=metrics),
    ):
        yield target, pod, response, core, metrics, connection


@pytest.mark.parametrize(
    "cpu,memory,cores,bytes_",
    [
        ("621641n", "8928Ki", 0.000621641, 8928 * 1024),
        ("250m", "8Mi", 0.25, 8 * 1024**2),
        ("2", "1Gi", 2, 1024**3),
        ("0", "1M", 0, 1000000),
    ],
)
def test_resource_samples_use_base_units_and_include_native_sidecars(
    metrics_api, cpu, memory, cores, bytes_
):
    target, _, response, core, metrics, connection = metrics_api
    response["items"][0]["containers"][0]["usage"] = {"cpu": cpu, "memory": memory}
    samples = read_samples(target, KubernetesComponentMonitoringConfig())
    assert [sample.container for sample in samples] == ["engine", "sidecar"]
    assert samples[0].cpu_cores == pytest.approx(cores)
    assert samples[0].memory_bytes == bytes_
    assert samples[0].source_timestamp == response["items"][0]["timestamp"]
    assert samples[0].window_seconds == pytest.approx(66.318)
    connection.assert_called_once_with(
        config_file="/caller/kubeconfig", context="benchmark", persist_config=False
    )
    metrics.list_namespaced_custom_object.assert_called_once_with(
        group="metrics.k8s.io",
        version="v1beta1",
        namespace="telemetry",
        plural="pods",
        label_selector="app=engine",
        _request_timeout=(5.0, 10),
    )
    core.list_namespaced_pod.assert_called_once_with(
        namespace="telemetry", label_selector="app=engine", _request_timeout=(5.0, 10)
    )


# Scenario: An invalid batch is returned after a previously accepted source window.
# Guarantees: Cached or older timestamps cannot bypass data and identity validation.
@pytest.mark.parametrize(
    "invalid",
    [
        "missing-pod",
        "duplicate-pod",
        "missing-sidecar",
        "duplicate-container",
        "missing-memory",
        "negative-cpu",
        "nonfinite-memory",
        "stale",
        "future",
        "naive-time",
        "invalid-window",
        "wrong-namespace",
        "replaced-pod",
        "restarted-container",
        "unready-pod",
    ],
)
def test_invalid_batches_fail_without_recording_partial_measurements(
    metrics_api, invalid
):
    target, pod, response, _, _, _ = metrics_api
    item = response["items"][0]
    runtime = KubernetesMonitoringRuntime(
        timestamps={"engine-0": datetime.fromisoformat(item["timestamp"])}
    )
    if invalid == "missing-pod":
        response["items"] = []
    elif invalid == "duplicate-pod":
        response["items"].append(deepcopy(item))
    elif invalid == "missing-sidecar":
        item["containers"].pop()
    elif invalid == "duplicate-container":
        item["containers"].append(deepcopy(item["containers"][0]))
    elif invalid == "missing-memory":
        del item["containers"][1]["usage"]["memory"]
    elif invalid == "negative-cpu":
        item["containers"][1]["usage"]["cpu"] = "-1m"
    elif invalid == "nonfinite-memory":
        item["containers"][1]["usage"]["memory"] = "NaN"
    elif invalid in ("stale", "future"):
        offset = -1000 if invalid == "stale" else 60
        item["timestamp"] = (
            datetime.now(timezone.utc) + timedelta(seconds=offset)
        ).isoformat()
    elif invalid == "naive-time":
        item["timestamp"] = "2026-01-01T00:00:00"
    elif invalid == "invalid-window":
        item["window"] = "0s"
    elif invalid == "wrong-namespace":
        item["metadata"]["namespace"] = "other"
    elif invalid == "replaced-pod":
        pod.metadata.uid = "new-uid"
    elif invalid == "restarted-container":
        pod.status.container_statuses[0].container_id = "containerd://restarted"
    elif invalid == "unready-pod":
        pod.status.conditions[0].status = "False"
    strategy = KubernetesComponentMonitoringStrategy(
        KubernetesComponentMonitoringConfig()
    )
    cpu, memory = Mock(), Mock()
    with pytest.raises((ValueError, ArithmeticError)):
        strategy._poll(
            Mock(name="engine"),
            target,
            runtime,
            cpu,
            memory,
            MagicMock(),
        )
    cpu.set.assert_not_called()
    memory.set.assert_not_called()


# Scenario: Duplicate and older windows are followed by a replay and a newer window.
# Guarantees: New windows record all containers; old windows cannot lower the watermark.
def test_cached_and_older_source_samples_are_skipped_until_newer_data(metrics_api):
    target, _, response, _, _, _ = metrics_api
    strategy = KubernetesComponentMonitoringStrategy(
        KubernetesComponentMonitoringConfig()
    )
    runtime = KubernetesMonitoringRuntime()
    component = Mock()
    component.name = "engine"
    cpu, memory = Mock(), Mock()
    ctx = MagicMock()
    strategy._poll(component, target, runtime, cpu, memory, ctx)
    strategy._poll(component, target, runtime, cpu, memory, ctx)
    assert cpu.set.call_count == memory.set.call_count == 2
    ctx.get_logger.return_value.warning.assert_not_called()
    attributes = cpu.set.call_args.args[1]
    assert attributes["sample.timestamp"] == response["items"][0]["timestamp"]
    assert attributes["sample.window_seconds"] == pytest.approx(66.318)
    assert attributes["container_id"] == "containerd://sidecar"
    assert attributes["k8s.pod.uid"] == "uid-engine"

    timestamp = datetime.fromisoformat(response["items"][0]["timestamp"])
    response["items"][0]["timestamp"] = (timestamp - timedelta(seconds=1)).isoformat()
    strategy._poll(component, target, runtime, cpu, memory, ctx)
    assert cpu.set.call_count == memory.set.call_count == 2
    assert runtime.timestamps == {"engine-0": timestamp}
    ctx.get_logger.return_value.warning.assert_called_once()
    warning = ctx.get_logger.return_value.warning.call_args.args[0]
    assert "engine-0" in warning
    assert response["items"][0]["timestamp"] in warning
    assert timestamp.isoformat() in warning

    response["items"][0]["timestamp"] = timestamp.isoformat()
    strategy._poll(component, target, runtime, cpu, memory, ctx)
    assert cpu.set.call_count == memory.set.call_count == 2
    ctx.get_logger.return_value.warning.assert_called_once()

    response["items"][0]["timestamp"] = (timestamp + timedelta(seconds=1)).isoformat()
    strategy._poll(component, target, runtime, cpu, memory, ctx)
    assert cpu.set.call_count == memory.set.call_count == 4
    assert runtime.timestamps == {"engine-0": timestamp + timedelta(seconds=1)}


# Scenario: One pod's timestamp regresses while another advances, then it recovers.
# Guarantees: Pods advance independently and record regular and sidecar data together.
def test_mixed_pod_windows_keep_independent_watermarks_and_complete_containers(
    metrics_api,
):
    target, pod, response, core, _, _ = metrics_api
    other_pod = deepcopy(pod)
    other_pod.metadata.name = "engine-1"
    other_pod.metadata.uid = "uid-other"
    target.pods.append(pod_identity(other_pod))
    core.list_namespaced_pod.return_value.items.append(other_pod)
    other_metrics = deepcopy(response["items"][0])
    other_metrics["metadata"]["name"] = "engine-1"
    response["items"].append(other_metrics)
    timestamp = datetime.fromisoformat(response["items"][0]["timestamp"])
    newer = timestamp + timedelta(seconds=1)
    strategy = KubernetesComponentMonitoringStrategy(
        KubernetesComponentMonitoringConfig()
    )
    runtime = KubernetesMonitoringRuntime()
    component = Mock()
    component.name = "engine"
    cpu, memory = Mock(), Mock()
    ctx = MagicMock()
    strategy._poll(component, target, runtime, cpu, memory, ctx)
    assert cpu.set.call_count == memory.set.call_count == 4

    response["items"][0]["timestamp"] = (timestamp - timedelta(seconds=1)).isoformat()
    other_metrics["timestamp"] = newer.isoformat()
    strategy._poll(component, target, runtime, cpu, memory, ctx)
    assert cpu.set.call_count == memory.set.call_count == 6
    assert runtime.timestamps == {"engine-0": timestamp, "engine-1": newer}
    for gauge in (cpu, memory):
        attributes = [call.args[1] for call in gauge.set.call_args_list[-2:]]
        assert {item["k8s.pod.name"] for item in attributes} == {"engine-1"}
        assert {item["container_name"] for item in attributes} == {"engine", "sidecar"}
        assert {item["sample.timestamp"] for item in attributes} == {newer.isoformat()}
    ctx.get_logger.return_value.warning.assert_called_once()

    response["items"][0]["timestamp"] = newer.isoformat()
    strategy._poll(component, target, runtime, cpu, memory, ctx)
    assert cpu.set.call_count == memory.set.call_count == 8
    assert runtime.timestamps == {"engine-0": newer, "engine-1": newer}
    for gauge in (cpu, memory):
        attributes = [call.args[1] for call in gauge.set.call_args_list[-2:]]
        assert {item["k8s.pod.name"] for item in attributes} == {"engine-0"}
        assert {item["container_name"] for item in attributes} == {"engine", "sidecar"}


# Scenario: The metrics API returns the same source window across two collections.
# Guarantees: Existing metric storage receives one CPU/memory row per container.
def test_unique_source_samples_reach_existing_metric_storage(metrics_api):
    target, _, _, _, _, _ = metrics_api
    backend = FrameworkMetricBackend()
    reader = PeriodicExportingMetricReader(
        FrameworkMetricExporter(backend), export_interval_millis=600000
    )
    provider = MeterProvider(metric_readers=[reader])
    try:
        meter = provider.get_meter(MODULE)
        cpu = meter.create_gauge("container.cpu.usage", unit="{cpu}")
        memory = meter.create_gauge("container.memory.working_set", unit="By")
        strategy = KubernetesComponentMonitoringStrategy(
            KubernetesComponentMonitoringConfig()
        )
        runtime = KubernetesMonitoringRuntime()
        component = Mock()
        component.name = "engine"
        ctx = MagicMock()
        strategy._poll(component, target, runtime, cpu, memory, ctx)
        provider.force_flush()
        strategy._poll(component, target, runtime, cpu, memory, ctx)
        provider.force_flush()
        rows = backend.get_metrics_df()
        assert len(rows) == 4
        assert set(rows["metric_name"]) == {
            "container.cpu.usage",
            "container.memory.working_set",
        }
        assert {attrs["container_name"] for attrs in rows["metric_attributes"]} == {
            "engine",
            "sidecar",
        }
        assert all("sample.timestamp" in attrs for attrs in rows["metric_attributes"])
    finally:
        provider.shutdown()


def test_start_requires_adopted_kubernetes_identity():
    component = build_managed_component(
        "endpoint-only", ManagedComponentConfiguration()
    )
    strategy = KubernetesComponentMonitoringStrategy(
        KubernetesComponentMonitoringConfig()
    )
    with pytest.raises(ValueError, match="requires existing.kubernetes adoption"):
        strategy.start(component, MagicMock())


def test_background_errors_are_logged_and_propagate_on_stop(metrics_api):
    target, _, response, _, metrics, _ = metrics_api
    component = build_managed_component("engine", ManagedComponentConfiguration())
    component.set_runtime_data(ComponentKubernetesRuntime.type, target)
    error = ApiException(status=403, reason="Forbidden")
    metrics.list_namespaced_custom_object.side_effect = [response, error]
    strategy = KubernetesComponentMonitoringStrategy(
        KubernetesComponentMonitoringConfig(interval=0.01)
    )
    ctx = MagicMock()
    strategy.start(component, ctx)
    runtime = component.runtime.get(KubernetesMonitoringRuntime.type)
    assert runtime.stop_event.wait(timeout=5)
    with pytest.raises(RuntimeError, match="Forbidden") as raised:
        strategy.stop(component, ctx)
    assert raised.value.__cause__ is error
    ctx.get_logger.return_value.exception.assert_called_once()
    assert not runtime.thread.is_alive()


def test_initial_api_failure_propagates_before_thread_start(metrics_api):
    target, _, _, _, metrics, _ = metrics_api
    component = build_managed_component("engine", ManagedComponentConfiguration())
    component.set_runtime_data(ComponentKubernetesRuntime.type, target)
    metrics.list_namespaced_custom_object.side_effect = ApiException(status=503)
    strategy = KubernetesComponentMonitoringStrategy(
        KubernetesComponentMonitoringConfig()
    )
    with pytest.raises(ApiException):
        strategy.start(component, MagicMock())
    assert component.runtime.get(KubernetesMonitoringRuntime.type).thread is None
    strategy.stop(component, MagicMock())


def test_stop_interrupts_long_poll_wait_without_more_api_calls(metrics_api):
    target, _, _, _, metrics, _ = metrics_api
    component = build_managed_component("engine", ManagedComponentConfiguration())
    component.set_runtime_data(ComponentKubernetesRuntime.type, target)
    strategy = KubernetesComponentMonitoringStrategy(
        KubernetesComponentMonitoringConfig(interval=3600)
    )
    strategy.start(component, MagicMock())
    strategy.stop(component, MagicMock())
    assert metrics.list_namespaced_custom_object.call_count == 1
    assert not component.runtime.get(KubernetesMonitoringRuntime.type).thread.is_alive()


@pytest.mark.parametrize(
    "alive,message",
    [(True, "did not stop in time"), (False, "exited unexpectedly")],
)
def test_stuck_or_unexpectedly_exited_monitor_cannot_report_success(alive, message):
    component = build_managed_component("engine", ManagedComponentConfiguration())
    thread = Mock()
    thread.is_alive.return_value = alive
    component.set_runtime_data(
        KubernetesMonitoringRuntime.type, KubernetesMonitoringRuntime(thread=thread)
    )
    strategy = KubernetesComponentMonitoringStrategy(
        KubernetesComponentMonitoringConfig()
    )
    with pytest.raises(RuntimeError, match=message):
        strategy.stop(component, MagicMock())
    assert 0 < thread.join.call_args.kwargs["timeout"] < 60


@pytest.mark.parametrize("field", ["interval", "max_age_seconds", "request_timeout"])
@pytest.mark.parametrize("value", [0, -1, None, float("nan"), float("inf")])
def test_invalid_collection_bounds_are_rejected(field, value):
    with pytest.raises(ValidationError):
        KubernetesComponentMonitoringConfig(**{field: value})
