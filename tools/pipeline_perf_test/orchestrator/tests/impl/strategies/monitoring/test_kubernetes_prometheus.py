"""Kubernetes forwarding transport, identity and access ownership."""

from contextlib import ExitStack
import threading
from types import SimpleNamespace
from unittest.mock import MagicMock, PropertyMock

import pytest
import requests

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


def text_response(body=b"example_value 7\n"):
    response = MagicMock(content=body)
    response.__enter__.return_value = response

    def close(*_args):
        response.close()
        return False

    response.__exit__.side_effect = close
    return response


@pytest.fixture
def forward_api(monkeypatch):
    from kubernetes import client, config
    from lib.impl.strategies.common import kubernetes, kubernetes_port_forward
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
    clients, forwards = [], []

    def create_client(**_kwargs):
        value = MagicMock()
        clients.append(value)
        return value

    def create_forward(**_kwargs):
        forward = MagicMock()
        forward.closed = False
        local_port = 40000 + len(forwards)

        def url(scheme, path):
            if forward.closed:
                raise RuntimeError("Port-forward has been released; adopt again")
            return f"{scheme}://127.0.0.1:{local_port}{path}"

        forward.url.side_effect = url
        forward.close.side_effect = lambda: setattr(forward, "closed", True)
        forwards.append(forward)
        return forward

    new_client = MagicMock(side_effect=create_client)
    core_api = MagicMock(return_value=api)
    factory = MagicMock(side_effect=create_forward)
    get = MagicMock(side_effect=lambda *_args, **_kwargs: text_response())
    monkeypatch.setattr(config, "new_client_from_config", new_client)
    monkeypatch.setattr(client, "CoreV1Api", core_api)
    monkeypatch.setattr(kubernetes, "pod_identity", lambda pod: pod.identity)
    monkeypatch.setattr(existing, "pod_identity", lambda pod: pod.identity)
    monkeypatch.setattr(existing, "KubernetesPortForward", factory)
    monkeypatch.setattr(
        kubernetes_port_forward,
        "requests",
        SimpleNamespace(get=get, exceptions=requests.exceptions),
    )
    return SimpleNamespace(
        api=api,
        clients=clients,
        forwards=forwards,
        factory=factory,
        get=get,
        new_client=new_client,
        core_api=core_api,
        pods=pods,
    )


@pytest.fixture
def adopted(forward_api):
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
            component=component, ctx=ctx, cleanup=cleanup, backend=forward_api
        )


# Scenario: Two pods publish HTTP or HTTPS endpoints through owned local tunnels.
# Guarantees: Identity uses one client; forwarding uses the selected kubeconfig/context.
@pytest.mark.parametrize("scheme", ["http", "https"])
def test_forwarding_preserves_identity_and_selected_credentials(adopted, scheme):
    component, ctx, backend = adopted.component, adopted.ctx, adopted.backend
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
    backend.new_client.assert_called_once()
    kwargs = backend.new_client.call_args.kwargs
    assert kwargs["config_file"] == "/caller/config"
    assert kwargs["context"] == "caller-context"
    assert kwargs["persist_config"] is False
    assert kwargs["client_configuration"].retries == 0
    assert all(
        call.args == (backend.clients[0],) for call in backend.core_api.call_args_list
    )
    assert backend.api.list_namespaced_pod.call_args.kwargs["_request_timeout"] == (
        2,
        2,
    )
    for index, call in enumerate(backend.factory.call_args_list):
        assert call.kwargs["namespace"] == "telemetry"
        assert call.kwargs["pod"] == f"pod-{index}"
        assert call.kwargs["port"] == 12345
        assert call.kwargs["kubeconfig"] == "/caller/config"
        assert call.kwargs["context"] == "caller-context"
        backend.forwards[index].start.assert_called_once()
        assert backend.get.call_args_list[index].args == (
            f"{scheme}://127.0.0.1:{40000 + index}/api/v1/metrics",
        )
        assert backend.get.call_args_list[index].kwargs == {
            "timeout": 3,
            "proxies": {"http": "", "https": ""},
        }
    component.destroy(ctx)
    adopted.cleanup.close()
    backend.clients[0].close.assert_called_once()
    for forward in backend.forwards:
        forward.close.assert_called_once()
    with pytest.raises(RuntimeError, match="released"):
        targets[0].endpoints["metrics"].fetch(1)


# Scenario: Two named endpoints share a pod port but use different HTTP paths.
# Guarantees: The provider starts only one tunnel per pod/port.
def test_named_endpoints_share_forwarding_per_pod_port(adopted):
    component = adopted.component
    config = component.existing.config.endpoints["metrics"]
    component.existing.config.endpoints["health"] = config.model_copy(
        update={"path": "/health"}
    )
    component.adopt(adopted.ctx)
    for target in component.get_targets():
        assert target.endpoints["metrics"].forward is target.endpoints["health"].forward
        assert target.endpoints["health"].fetch(1)
    assert adopted.backend.factory.call_count == 2


# Scenario: Forwarded responses contain literal newlines or invalid UTF-8 bytes.
# Guarantees: Raw text is preserved and responses close even when decoding fails.
@pytest.mark.parametrize("body", [b"example_value 7\nother_value 9\n", b"\xff"])
def test_forwarding_consumes_and_closes_raw_response(adopted, body):
    adopted.component.adopt(adopted.ctx)
    endpoint = adopted.component.get_targets()[0].endpoints["metrics"]
    response = text_response(body)
    adopted.backend.get.side_effect = None
    adopted.backend.get.return_value = response
    if body == b"\xff":
        with pytest.raises(UnicodeDecodeError):
            endpoint.fetch(1)
    else:
        assert endpoint.fetch(1) == body.decode()
    response.close.assert_called_once()


# Scenario: An endpoint rejects a request or transport fails during collection.
# Guarantees: Failures surface with their cause, without a retry or empty sample.
@pytest.mark.parametrize(
    "error,expected",
    [
        (requests.HTTPError("403 Forbidden"), RuntimeError),
        (requests.HTTPError("503 Unavailable"), RuntimeError),
        (requests.ConnectionError("refused"), ConnectionError),
        (requests.ConnectTimeout("slow"), TimeoutError),
        (requests.ReadTimeout("slow"), TimeoutError),
        (requests.exceptions.SSLError("verification failed"), RuntimeError),
    ],
)
def test_forwarding_surfaces_request_errors(adopted, error, expected):
    adopted.component.adopt(adopted.ctx)
    endpoint = adopted.component.get_targets()[0].endpoints["metrics"]
    response = text_response()
    adopted.backend.get.side_effect = None
    adopted.backend.get.return_value = response
    response.raise_for_status.side_effect = error
    with pytest.raises(expected) as raised:
        endpoint.fetch(1)
    assert raised.value.__cause__ is error
    adopted.backend.get.assert_called_once()
    response.close.assert_called_once()


# Scenario: Reading the forwarded HTTP response body times out.
# Guarantees: The response closes and the timeout crosses the endpoint boundary.
def test_forwarding_closes_response_after_body_timeout(adopted):
    adopted.component.adopt(adopted.ctx)
    response = text_response()
    type(response).content = PropertyMock(side_effect=requests.ReadTimeout("slow"))
    adopted.backend.get.side_effect = None
    adopted.backend.get.return_value = response
    endpoint = adopted.component.get_targets()[0].endpoints["metrics"]
    with pytest.raises(TimeoutError):
        endpoint.fetch(1)
    response.close.assert_called_once()


# Scenario: Discovery, tunnel startup or target publication interrupts adoption.
# Guarantees: Partial helpers and the owned client close, including on cancellation.
@pytest.mark.parametrize("error", [RuntimeError, SystemExit, KeyboardInterrupt])
@pytest.mark.parametrize("stage", ["discovery", "forwarding", "publication"])
def test_interrupted_adoption_closes_owned_access(adopted, monkeypatch, error, stage):
    if stage == "discovery":
        adopted.backend.api.list_namespaced_pod.side_effect = error("interrupted")
    elif stage == "forwarding":
        original = adopted.backend.factory.side_effect

        def create(**kwargs):
            forward = original(**kwargs)
            if len(adopted.backend.forwards) == 2:
                forward.start.side_effect = error("interrupted")
            return forward

        adopted.backend.factory.side_effect = create
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
    adopted.backend.clients[0].close.assert_called_once()
    for forward in adopted.backend.forwards:
        forward.close.assert_called_once()


# Scenario: A selected pod/container changes, disappears or gains a terminal sibling.
# Guarantees: Live identity changes require re-adoption; order and terminal pods do not.
@pytest.mark.parametrize(
    "change",
    ["pod-uid", "container-id", "node", "missing", "extra", "terminal", "order"],
)
def test_forwarding_validates_fixed_pod_and_container_identities(adopted, change):
    adopted.component.adopt(adopted.ctx)
    pods = [pod.model_copy(deep=True) for pod in adopted.backend.pods]
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
        observed.append(observed_pod(pods[0].model_copy(update={"name": "new-pod"})))
    elif change == "terminal":
        observed.append(observed_pod(None, phase="Succeeded"))
    else:
        observed.reverse()
    adopted.backend.api.list_namespaced_pod.return_value.items = observed
    if change in ("terminal", "order"):
        assert len(adopted.component.get_targets()) == 2
    else:
        with pytest.raises(ValueError, match="identities changed"):
            adopted.component.get_targets()


# Scenario: The API denies identity validation or authentication refresh fails.
# Guarantees: The contextual backend error surfaces before endpoint retrieval.
@pytest.mark.parametrize("failure", ["forbidden", "authentication"])
def test_forwarding_validation_surfaces_backend_failure(adopted, failure):
    from kubernetes.client.exceptions import ApiException
    from kubernetes.config.config_exception import ConfigException

    adopted.component.adopt(adopted.ctx)
    error = (
        ApiException(status=403)
        if failure == "forbidden"
        else ConfigException("authentication failed")
    )
    adopted.backend.api.list_namespaced_pod.side_effect = error
    with pytest.raises(RuntimeError, match="Could not validate") as raised:
        adopted.component.get_targets()
    assert raised.value.__cause__ is error
    adopted.backend.get.assert_not_called()


# Scenario: An active component is destroyed, re-adopted or cleaned up.
# Guarantees: Scrapers stop before tunnels/client close, and old endpoints stay closed.
@pytest.mark.parametrize("action", ["destroy", "readopt", "suite-exit"])
def test_monitoring_stops_before_owned_forwarding_closes(adopted, monkeypatch, action):
    component, ctx, backend = adopted.component, adopted.ctx, adopted.backend

    def worker(**kwargs):
        kwargs["stop_event"].wait()
        kwargs["runtime"].completed = True

    monkeypatch.setattr(prometheus, "monitor", worker)
    component.adopt(ctx)
    old_endpoint = component.get_targets()[0].endpoints["metrics"]
    previous = component.runtime.get(KubernetesTargetAccess.type)
    old_forwards = list(backend.forwards)
    component.start_monitoring(ctx)
    runtime = component.runtime.get(PrometheusMonitoringRuntime.type)

    def close_forward(forward):
        assert runtime.stop_event.is_set()
        assert not runtime.thread.is_alive()
        forward.closed = True

    for forward in old_forwards:
        forward.close.side_effect = lambda forward=forward: close_forward(forward)

    def close_client():
        assert all(forward.closed for forward in old_forwards)

    backend.clients[0].close.side_effect = close_client
    if action == "destroy":
        component.destroy(ctx)
    elif action == "readopt":
        component.adopt(ctx)
        previous.close()
        backend.clients[1].close.assert_not_called()
        assert component.get_targets()[0].endpoints["metrics"].fetch(1)
    adopted.cleanup.close()
    for value in backend.clients:
        value.close.assert_called_once()
    for forward in backend.forwards:
        forward.close.assert_called_once()
    with pytest.raises(RuntimeError, match="released"):
        old_endpoint.fetch(1)


# Scenario: Monitoring cannot confirm that its worker stopped during suite cleanup.
# Guarantees: Forwarding and API access remain open beneath the active consumer.
def test_failed_monitoring_stop_keeps_forwarding_open(adopted, monkeypatch):
    component, ctx, backend = adopted.component, adopted.ctx, adopted.backend
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
    backend.clients[0].close.assert_not_called()
    for forward in backend.forwards:
        forward.close.assert_not_called()
    thread.is_alive.return_value = False
    component.stop_monitoring(ctx)
    component.destroy(ctx)
    backend.clients[0].close.assert_called_once()


# Scenario: A tunnel cannot close on the first release attempt.
# Guarantees: Its ownership and API access remain available for a later cleanup.
def test_failed_forwarding_close_retains_access_for_retry(adopted):
    adopted.component.adopt(adopted.ctx)
    access = adopted.component.runtime.get(KubernetesTargetAccess.type)
    forward = adopted.backend.forwards[0]
    close = forward.close.side_effect
    forward.close.side_effect = RuntimeError("forwarding still running")
    with pytest.raises(RuntimeError, match="still running"):
        access.close()
    adopted.backend.clients[0].close.assert_not_called()
    assert access.forwards
    forward.close.side_effect = close
    access.close()
    assert not access.forwards
    adopted.backend.clients[0].close.assert_called_once()


# Scenario: Generic Prometheus consumes adopted forwarding endpoints.
# Guarantees: Exported samples retain authoritative pod and component identity.
def test_forwarding_monitoring_preserves_per_pod_metric_identity(adopted, monkeypatch):
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
