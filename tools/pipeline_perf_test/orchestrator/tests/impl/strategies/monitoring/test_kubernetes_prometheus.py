"""Kubernetes pod-proxy transport, identity and access ownership."""

from contextlib import ExitStack
import threading
from types import SimpleNamespace
from unittest.mock import MagicMock, PropertyMock

import pytest

from lib.runner.factory import build_managed_component
from lib.impl.component.managed_component import ManagedComponentConfiguration
from lib.impl.strategies.common.kubernetes import KubernetesPodIdentity
from lib.impl.strategies.existing.kubernetes import KubernetesTargetAccess
from lib.impl.strategies.monitoring import prometheus
from lib.impl.strategies.monitoring.prometheus import (
    PrometheusMonitoringRuntime,
    PrometheusMonitoringStrategy,
)


def observed_pod(identity, phase="Running"):
    return SimpleNamespace(identity=identity, status=SimpleNamespace(phase=phase))


@pytest.fixture
def proxy_api(monkeypatch):
    from kubernetes import client, config
    from lib.impl.strategies.common import kubernetes
    from lib.impl.strategies.existing import kubernetes as existing

    pods = [
        KubernetesPodIdentity(
            name=f"pod-{index}",
            uid=f"uid-{index}",
            node_name=f"node-{index}",
            containers={"app": f"containerd://{index}"},
        )
        for index in range(2)
    ]
    api = MagicMock()
    api.list_namespaced_pod.return_value.items = [observed_pod(pod) for pod in pods]
    api.connect_get_namespaced_pod_proxy_with_path.side_effect = (
        lambda **kwargs: MagicMock(data=b"example_value 7\n")
    )
    clients = []

    def create_client(**kwargs):
        value = MagicMock()
        clients.append(value)
        return value

    new_client = MagicMock(side_effect=create_client)
    core_api = MagicMock(return_value=api)
    monkeypatch.setattr(config, "new_client_from_config", new_client)
    monkeypatch.setattr(client, "CoreV1Api", core_api)
    monkeypatch.setattr(kubernetes, "pod_identity", lambda pod: pod.identity)
    monkeypatch.setattr(existing, "pod_identity", lambda pod: pod.identity)
    return SimpleNamespace(
        api=api, clients=clients, new_client=new_client, core_api=core_api, pods=pods
    )


@pytest.fixture
def adopted(proxy_api):
    component = build_managed_component(
        "engine",
        ManagedComponentConfiguration.model_validate(
            {
                "existing": {
                    "kubernetes": {
                        "namespace": "telemetry",
                        "label_selector": "app=external",
                        "kubeconfig": "/caller/config",
                        "context": "caller-context",
                        "endpoints": {"metrics": {"port": 12345}},
                    }
                },
                "monitoring": {"prometheus": {"target_endpoint": "metrics"}},
            }
        ),
    )
    ctx = MagicMock()
    ctx.get_suite().get_runtime.return_value = None
    with ExitStack() as cleanup:
        ctx.get_suite().context.cleanup = cleanup
        yield SimpleNamespace(
            component=component, ctx=ctx, cleanup=cleanup, proxy=proxy_api
        )


# Scenario: Two discovered pods expose HTTP or HTTPS metrics through pod proxy.
# Guarantees: Discovery, validation and raw-text fetches reuse one owned client.
@pytest.mark.parametrize("scheme", ["http", "https"])
def test_proxy_reuses_client_and_preserves_identity(adopted, scheme):
    component, ctx, proxy = adopted.component, adopted.ctx, adopted.proxy
    component.existing.config.endpoints["metrics"].scheme = scheme
    component.existing.config.endpoints["metrics"].path = "/api/v1/metrics"
    component.adopt(ctx)
    targets = component.get_targets(timeout=2)
    for target in targets:
        assert target.endpoints["metrics"].fetch(3) == "example_value 7\n"
    assert [dict(target.attributes) for target in targets] == [
        {
            "k8s.namespace.name": "telemetry",
            "k8s.pod.name": f"pod-{index}",
            "k8s.pod.uid": f"uid-{index}",
            "k8s.node.name": f"node-{index}",
        }
        for index in range(2)
    ]
    proxy.new_client.assert_called_once()
    kwargs = proxy.new_client.call_args.kwargs
    assert kwargs["config_file"] == "/caller/config"
    assert kwargs["context"] == "caller-context"
    assert kwargs["persist_config"] is False
    assert kwargs["client_configuration"].retries == 0
    assert all(
        call.args == (proxy.clients[0],) for call in proxy.core_api.call_args_list
    )
    assert proxy.api.list_namespaced_pod.call_args.kwargs["_request_timeout"] == (2, 2)
    for index, call in enumerate(
        proxy.api.connect_get_namespaced_pod_proxy_with_path.call_args_list
    ):
        assert call.kwargs == {
            "name": f"{scheme}:pod-{index}:12345",
            "namespace": "telemetry",
            "path": "api/v1/metrics",
            "_request_timeout": 3,
            "_preload_content": False,
        }
    component.destroy(ctx)
    adopted.cleanup.close()
    proxy.clients[0].close.assert_called_once()
    with pytest.raises(RuntimeError, match="released"):
        targets[0].endpoints["metrics"].fetch(1)


# Scenario: A proxy response contains literal newlines or invalid UTF-8 bytes.
# Guarantees: The endpoint preserves text and releases responses even on decode failure.
@pytest.mark.parametrize("body", [b"example_value 7\nother_value 9\n", b"\xff"])
def test_proxy_consumes_and_releases_raw_response(adopted, body):
    adopted.component.adopt(adopted.ctx)
    endpoint = adopted.component.get_targets()[0].endpoints["metrics"]
    response = MagicMock(data=body)
    proxy = adopted.proxy.api.connect_get_namespaced_pod_proxy_with_path
    proxy.side_effect = None
    proxy.return_value = response
    if body == b"\xff":
        with pytest.raises(UnicodeDecodeError):
            endpoint.fetch(1)
    else:
        assert endpoint.fetch(1) == body.decode()
    response.close.assert_called_once()
    response.release_conn.assert_called_once()


# Scenario: The API denies proxy access or returns an endpoint/status failure.
# Guarantees: The original error surfaces without an empty result or application retry.
@pytest.mark.parametrize("status", [0, 403, 404, 503])
def test_proxy_status_errors_surface_immediately(adopted, status):
    from kubernetes.client.exceptions import ApiException

    adopted.component.adopt(adopted.ctx)
    endpoint = adopted.component.get_targets()[0].endpoints["metrics"]
    error = ApiException(status=status, reason="endpoint unavailable")
    proxy = adopted.proxy.api.connect_get_namespaced_pod_proxy_with_path
    proxy.side_effect = error
    with pytest.raises(RuntimeError, match="Pod proxy request failed") as raised:
        endpoint.fetch(1)
    assert raised.value.__cause__ is error
    proxy.assert_called_once()


# Scenario: urllib3 reports connection, timeout or TLS failure directly or wrapped.
# Guarantees: Transport errors cross the generic endpoint boundary with their cause.
@pytest.mark.parametrize(
    "kind", ["connection", "protocol", "connect-timeout", "read-timeout", "tls"]
)
@pytest.mark.parametrize("wrapped", [False, True])
def test_proxy_translates_transport_errors(adopted, kind, wrapped):
    from urllib3.exceptions import (
        ConnectTimeoutError,
        MaxRetryError,
        NewConnectionError,
        ProtocolError,
        ReadTimeoutError,
        SSLError,
    )

    errors = {
        "connection": (NewConnectionError(None, "refused"), ConnectionError),
        "protocol": (ProtocolError("disconnected"), ConnectionError),
        "connect-timeout": (ConnectTimeoutError("slow"), TimeoutError),
        "read-timeout": (ReadTimeoutError(None, "/proxy", "slow"), TimeoutError),
        "tls": (SSLError("verification failed"), RuntimeError),
    }
    error, expected = errors[kind]
    if wrapped:
        error = MaxRetryError(None, "/proxy", error)
    adopted.component.adopt(adopted.ctx)
    endpoint = adopted.component.get_targets()[0].endpoints["metrics"]
    adopted.proxy.api.connect_get_namespaced_pod_proxy_with_path.side_effect = error
    with pytest.raises(expected) as raised:
        endpoint.fetch(1)
    assert raised.value.__cause__ is error


# Scenario: Reading a response times out after response acquisition.
# Guarantees: Timeout propagation still closes the response and releases its connection.
def test_proxy_releases_response_after_body_timeout(adopted):
    from urllib3.exceptions import ReadTimeoutError

    adopted.component.adopt(adopted.ctx)
    endpoint = adopted.component.get_targets()[0].endpoints["metrics"]
    response = MagicMock()
    type(response).data = PropertyMock(
        side_effect=ReadTimeoutError(None, "/proxy", "slow")
    )
    proxy = adopted.proxy.api.connect_get_namespaced_pod_proxy_with_path
    proxy.side_effect = None
    proxy.return_value = response
    with pytest.raises(TimeoutError):
        endpoint.fetch(1)
    response.close.assert_called_once()
    response.release_conn.assert_called_once()


# Scenario: Discovery or target publication interrupts after client creation.
# Guarantees: Startup closes its client once, including when suite cleanup follows.
@pytest.mark.parametrize("error", [RuntimeError, SystemExit, KeyboardInterrupt])
@pytest.mark.parametrize("stage", ["discovery", "publication"])
def test_interrupted_adoption_closes_owned_client(adopted, monkeypatch, error, stage):
    if stage == "discovery":
        adopted.proxy.api.list_namespaced_pod.side_effect = error("interrupted")
    else:
        original = adopted.component.set_targets

        def publish(targets, *args):
            if targets:
                raise error("interrupted")
            return original(targets, *args)

        monkeypatch.setattr(adopted.component, "set_targets", publish)
    with pytest.raises(error, match="interrupted"):
        adopted.component.adopt(adopted.ctx)
    adopted.cleanup.close()
    adopted.proxy.clients[0].close.assert_called_once()


# Scenario: A selected pod/container changes, disappears or gains a terminal sibling.
# Guarantees: Live identity changes require re-adoption; order and terminal pods do not.
@pytest.mark.parametrize(
    "change",
    ["pod-uid", "container-id", "node", "missing", "extra", "terminal", "order"],
)
def test_proxy_validates_fixed_pod_and_container_identities(adopted, change):
    adopted.component.adopt(adopted.ctx)
    pods = [pod.model_copy(deep=True) for pod in adopted.proxy.pods]
    observed = [observed_pod(pod) for pod in pods]
    if change == "pod-uid":
        pods[0].uid = "replacement"
    elif change == "container-id":
        pods[0].containers["app"] = "containerd://replacement"
    elif change == "node":
        pods[0].node_name = "replacement"
    elif change == "missing":
        observed.pop()
    elif change == "extra":
        extra = pods[0].model_copy(update={"name": "new-pod"})
        observed.append(observed_pod(extra))
    elif change == "terminal":
        observed.append(observed_pod(None, phase="Succeeded"))
    else:
        observed.reverse()
    adopted.proxy.api.list_namespaced_pod.return_value.items = observed
    if change in ("terminal", "order"):
        assert len(adopted.component.get_targets()) == 2
    else:
        with pytest.raises(ValueError, match="identities changed"):
            adopted.component.get_targets()


# Scenario: The API denies validation or refreshed client authentication fails.
# Guarantees: Target access propagates a contextual error with the backend cause.
@pytest.mark.parametrize("failure", ["forbidden", "authentication"])
def test_proxy_validation_surfaces_backend_failure(adopted, failure):
    from kubernetes.client.exceptions import ApiException
    from kubernetes.config.config_exception import ConfigException

    adopted.component.adopt(adopted.ctx)
    error = (
        ApiException(status=403)
        if failure == "forbidden"
        else ConfigException("authentication failed")
    )
    adopted.proxy.api.list_namespaced_pod.side_effect = error
    with pytest.raises(RuntimeError, match="Could not validate") as raised:
        adopted.component.get_targets()
    assert raised.value.__cause__ is error
    adopted.proxy.api.connect_get_namespaced_pod_proxy_with_path.assert_not_called()


# Scenario: An adopted Kubernetes component is destroyed, re-adopted or cleaned up.
# Guarantees: Active scrapers stop before their client closes; old access stays closed.
@pytest.mark.parametrize("action", ["destroy", "readopt", "suite-exit"])
def test_monitoring_stops_before_owned_client_closes(adopted, monkeypatch, action):
    component, ctx, proxy = adopted.component, adopted.ctx, adopted.proxy

    def worker(**kwargs):
        kwargs["stop_event"].wait()
        kwargs["runtime"].completed = True

    monkeypatch.setattr(prometheus, "monitor", worker)
    component.adopt(ctx)
    old_endpoint = component.get_targets()[0].endpoints["metrics"]
    previous = component.runtime.get(KubernetesTargetAccess.type)
    component.start_monitoring(ctx)
    runtime = component.runtime.get(PrometheusMonitoringRuntime.type)

    def close():
        assert runtime.stop_event.is_set()
        assert not runtime.thread.is_alive()

    proxy.clients[0].close.side_effect = close
    if action == "destroy":
        component.destroy(ctx)
    elif action == "readopt":
        component.adopt(ctx)
        previous.close()
        proxy.clients[1].close.assert_not_called()
        assert component.get_targets()[0].endpoints["metrics"].fetch(1)
    adopted.cleanup.close()
    for value in proxy.clients:
        value.close.assert_called_once()
    with pytest.raises(RuntimeError, match="released"):
        old_endpoint.fetch(1)


# Scenario: Monitoring cannot confirm that a worker stopped during suite cleanup.
# Guarantees: Kubernetes access remains open rather than closing beneath a consumer.
def test_failed_monitoring_stop_keeps_client_open(adopted, monkeypatch):
    component, ctx, proxy = adopted.component, adopted.ctx, adopted.proxy
    component.adopt(ctx)
    thread = MagicMock()
    thread.is_alive.return_value = True
    monkeypatch.setattr(
        prometheus,
        "threading",
        SimpleNamespace(Event=threading.Event, Thread=MagicMock(return_value=thread)),
    )
    component.start_monitoring(ctx)
    with pytest.raises(RuntimeError, match="monitoring"):
        adopted.cleanup.close()
    proxy.clients[0].close.assert_not_called()
    thread.is_alive.return_value = False
    component.stop_monitoring(ctx)
    component.destroy(ctx)
    proxy.clients[0].close.assert_called_once()


# Scenario: Generic Prometheus monitoring consumes the adopted pod endpoints.
# Guarantees: Each exported sample carries its authoritative pod and component identity.
def test_proxy_monitoring_preserves_per_pod_metric_identity(adopted, monkeypatch):
    component = adopted.component
    component.adopt(adopted.ctx)
    runtime = PrometheusMonitoringRuntime(
        targets=component.get_targets(),
        target_endpoint="metrics",
        stop_event=threading.Event(),
    )
    monkeypatch.setattr(runtime.stop_event, "wait", lambda _timeout: None)
    gauge = MagicMock()
    monkeypatch.setattr(prometheus, "get_instrument", lambda *args: gauge)
    prometheus.monitor(
        component_name=component.name,
        endpoint=None,
        include=[],
        exclude=[],
        interval=1,
        count=1,
        stop_event=runtime.stop_event,
        meter=MagicMock(),
        logger=MagicMock(),
        test_suite_context=MagicMock(),
        request_timeout=2,
        runtime=runtime,
        get_targets=component.get_targets,
    )
    PrometheusMonitoringStrategy._check_runtime(runtime)
    assert [call.args for call in gauge.set.call_args_list] == [
        (
            7,
            {
                "component_name": "engine",
                "component.target.name": f"pod-{index}",
                "k8s.namespace.name": "telemetry",
                "k8s.pod.name": f"pod-{index}",
                "k8s.pod.uid": f"uid-{index}",
                "k8s.node.name": f"node-{index}",
            },
        )
        for index in range(2)
    ]
