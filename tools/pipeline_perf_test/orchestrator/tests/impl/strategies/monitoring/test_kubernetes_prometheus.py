"""Generic adopted-pod transport, identity, and scrape failure checks."""

from unittest.mock import MagicMock

import pytest

from lib.impl.strategies.common.kubernetes import (
    ComponentKubernetesRuntime,
    KubernetesPodIdentity,
)
from lib.impl.strategies.common.kubernetes_port_forward import (
    KubernetesPortForwardConfig,
    KubernetesPortForwards,
)
from lib.impl.strategies.monitoring.prometheus import scrape_and_convert_metrics


@pytest.fixture
def forwarding(monkeypatch):
    from kubernetes import client, config
    from lib.impl.strategies.common import kubernetes_port_forward as module

    pods = [
        KubernetesPodIdentity(
            name=f"pod-{index}",
            uid=f"uid-{index}",
            node_name=f"node-{index}",
            containers={"app": f"containerd://{index}"},
        )
        for index in range(2)
    ]
    target = ComponentKubernetesRuntime(
        namespace="telemetry",
        label_selector="app=external",
        kubeconfig="/caller/config",
        context="caller-context",
        pods=pods,
    )
    api = MagicMock()
    api.list_namespaced_pod.return_value.items = pods
    monkeypatch.setattr(config, "new_client_from_config", MagicMock())
    monkeypatch.setattr(client, "CoreV1Api", lambda _: api)
    monkeypatch.setattr(module, "pod_identity", lambda pod: pod)
    processes = []
    commands = []

    def start(command, **kwargs):
        commands.append(command)
        kwargs["stdout"].write(
            f"Forwarding from 127.0.0.1:{21000 + len(processes)} -> 12345\n"
        )
        kwargs["stdout"].flush()
        process = MagicMock(pid=1000 + len(processes))
        process.poll.return_value = None
        processes.append(process)
        return process

    monkeypatch.setattr(module.subprocess, "Popen", start)
    monkeypatch.setattr(module.requests, "get", MagicMock())
    terminate = MagicMock()
    monkeypatch.setattr(module, "wait_or_terminate_process_tree", terminate)
    forwards = KubernetesPortForwards(
        target, KubernetesPortForwardConfig(port=12345), 1, MagicMock()
    )
    yield forwards, api, processes, commands, terminate
    forwards.close()


def test_fans_out_preserving_adopted_identity_and_connection(forwarding):
    forwards, api, processes, commands, terminate = forwarding
    forwards.open()
    assert [endpoint.attributes["k8s.pod.uid"] for endpoint in forwards.endpoints] == [
        "uid-0",
        "uid-1",
    ]
    for index, command in enumerate(commands):
        assert command == [
            "kubectl",
            "--kubeconfig",
            "/caller/config",
            "--context",
            "caller-context",
            "-n",
            "telemetry",
            "port-forward",
            f"pod/pod-{index}",
            ":12345",
            "--address",
            "127.0.0.1",
        ]
    forwards.close()
    assert terminate.call_count == 2
    for process in processes:
        process.wait.assert_called_once_with(timeout=5)


def test_partial_startup_releases_only_owned_processes(forwarding, monkeypatch):
    forwards, _, processes, _, terminate = forwarding
    original = forwards._ready

    def ready(process, path, pod):
        if pod == "pod-1":
            raise TimeoutError("endpoint unavailable")
        return original(process, path, pod)

    monkeypatch.setattr(forwards, "_ready", ready)
    with pytest.raises(TimeoutError, match="endpoint unavailable"):
        forwards.open()
    assert terminate.call_count == len(processes) == 2


@pytest.mark.parametrize(
    "change", ["pod-uid", "container-id", "missing-pod", "dead-forward"]
)
def test_rejects_identity_or_transport_changes(forwarding, change):
    forwards, api, processes, _, _ = forwarding
    forwards.open()
    pods = list(forwards.target.pods)
    if change == "pod-uid":
        pods[0] = pods[0].model_copy(update={"uid": "replacement"})
    elif change == "container-id":
        pods[0] = pods[0].model_copy(update={"containers": {"app": "replacement"}})
    elif change == "missing-pod":
        pods.pop()
    else:
        processes[0].poll.return_value = 1
    api.list_namespaced_pod.return_value.items = pods
    with pytest.raises((ValueError, RuntimeError)):
        forwards.validate()


def test_transport_identity_overrides_scraped_identity(monkeypatch):
    from lib.impl.strategies.monitoring import prometheus

    response = MagicMock()
    response.text = 'example_value{k8s_pod="unrelated"} 42\n'
    monkeypatch.setattr(prometheus.requests, "get", lambda *args, **kwargs: response)
    gauge = MagicMock()
    monkeypatch.setattr(prometheus, "get_instrument", lambda *args: gauge)
    scrape_and_convert_metrics(
        "http://localhost/metrics",
        "service",
        MagicMock(),
        attributes={"k8s_pod": "adopted", "k8s.pod.uid": "uid"},
    )
    gauge.set.assert_called_once_with(
        42, {"k8s_pod": "adopted", "k8s.pod.uid": "uid", "component_name": "service"}
    )
