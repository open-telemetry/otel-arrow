"""Component-owned pod proxy access and transport-neutral scrape guarantees."""

from contextlib import ExitStack
import math
import threading
from types import SimpleNamespace
from unittest.mock import MagicMock, PropertyMock

import pytest
import requests
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader

from lib.core.component.component import ComponentTarget
from lib.core.telemetry.metric import FrameworkMetricBackend, FrameworkMetricExporter
from lib.impl.strategies.common.kubernetes import KubernetesPodIdentity
from lib.impl.component.managed_component import ManagedComponentConfiguration
from lib.impl.strategies.existing.kubernetes import KubernetesTargetAccess
from lib.impl.strategies.monitoring import prometheus
from lib.impl.strategies.monitoring.prometheus import (
    PrometheusMonitoringConfig,
    PrometheusMonitoringRuntime,
    PrometheusMonitoringStrategy,
    scrape_and_convert_metrics,
)
from lib.runner.factory import build_managed_component


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
                }
            }
        ),
    )
    component.monitoring = PrometheusMonitoringStrategy(
        PrometheusMonitoringConfig(target_endpoint="metrics")
    )
    ctx = MagicMock()
    cleanup = ExitStack()
    ctx.get_suite().context.cleanup = cleanup
    try:
        yield SimpleNamespace(
            component=component, ctx=ctx, cleanup=cleanup, proxy=proxy_api
        )
    finally:
        runtime = component.runtime.get(PrometheusMonitoringRuntime.type)
        if runtime is not None and runtime.stop_event is not None:
            runtime.stop_event.set()
            if runtime.thread is not None and runtime.thread.ident is not None:
                runtime.thread.join(timeout=5)
        cleanup.close()


@pytest.fixture
def scrape_monitor(adopted, monkeypatch):
    component, ctx = adopted.component, adopted.ctx
    component.adopt(ctx)
    targets = component.get_targets()
    backend = FrameworkMetricBackend()
    reader = PeriodicExportingMetricReader(
        FrameworkMetricExporter(backend), export_interval_millis=math.inf
    )
    provider = MeterProvider(metric_readers=[reader])
    meter = provider.get_meter("target-scrapes")
    runtime = PrometheusMonitoringRuntime(
        targets=targets,
        target_endpoint="metrics",
        stop_event=threading.Event(),
        scrape_successes={target.name: 0 for target in targets},
    )
    logger, calls = MagicMock(), []
    monkeypatch.setattr(prometheus, "instrument_cache", {})

    def run(outcomes, stop_on_wait=False):
        routes = {name: iter(values) for name, values in outcomes.items()}

        def fetch(**kwargs):
            name = kwargs["name"].split(":")[1]
            calls.append(name)
            result = next(routes[name])
            if isinstance(result, Exception):
                raise result
            return MagicMock(data=result.encode())

        def wait(_timeout):
            reader.collect()
            if stop_on_wait:
                runtime.stop_event.set()

        adopted.proxy.api.connect_get_namespaced_pod_proxy_with_path.side_effect = fetch
        monkeypatch.setattr(runtime.stop_event, "wait", MagicMock(side_effect=wait))
        prometheus.monitor(
            component_name="engine",
            endpoint=None,
            include=[],
            exclude=[],
            interval=5,
            count=max(len(values) for values in outcomes.values()),
            stop_event=runtime.stop_event,
            meter=meter,
            logger=logger,
            test_suite_context=MagicMock(),
            request_timeout=1,
            runtime=runtime,
            get_targets=component.get_targets,
        )
        reader.collect()
        return backend

    try:
        yield SimpleNamespace(
            run=run,
            runtime=runtime,
            logger=logger,
            calls=calls,
            api=adopted.proxy.api,
            component=component,
        )
    finally:
        provider.shutdown()


# Scenario: Two discovered pods expose HTTP or HTTPS metrics through pod proxy.
# Guarantees: Discovery, health checks, and raw-text fetches reuse one owned client.
@pytest.mark.parametrize("scheme", ["http", "https"])
def test_proxy_reuses_client_and_preserves_identity(adopted, scheme):
    component, ctx, proxy = adopted.component, adopted.ctx, adopted.proxy
    component.existing.config.endpoints["metrics"].scheme = scheme
    component.existing.config.endpoints["metrics"].path = "/api/v1/metrics"
    component.adopt(ctx)
    targets = component.get_targets(timeout=2)
    for target in targets:
        assert target.endpoints["metrics"].fetch(3) == "example_value 7\n"
    assert [target.attributes["k8s.pod.uid"] for target in targets] == [
        "uid-0",
        "uid-1",
    ]
    assert len(proxy.clients) == 1
    proxy.new_client.assert_called_once()
    kwargs = proxy.new_client.call_args.kwargs
    assert kwargs["config_file"] == "/caller/config"
    assert kwargs["context"] == "caller-context"
    assert kwargs["persist_config"] is False
    assert kwargs["client_configuration"].retries == 0
    assert all(
        call.args == (proxy.clients[0],) for call in proxy.core_api.call_args_list
    )
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
# Guarantees: Text is not SDK-stringified, and all response connections are released.
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


# Scenario: HTTP authorization/status or TLS verification fails at the API boundary.
# Guarantees: Errors surface immediately instead of becoming empty or retried samples.
@pytest.mark.parametrize("status", [0, 403, 404, 503])
def test_proxy_status_errors_are_explicit_and_fatal(adopted, status):
    from kubernetes.client.exceptions import ApiException

    adopted.component.adopt(adopted.ctx)
    endpoint = adopted.component.get_targets()[0].endpoints["metrics"]
    error = ApiException(status=status, reason="endpoint unavailable")
    adopted.proxy.api.connect_get_namespaced_pod_proxy_with_path.side_effect = error
    with pytest.raises(RuntimeError, match="Pod proxy request failed") as raised:
        endpoint.fetch(1)
    assert raised.value.__cause__ is error


# Scenario: urllib3 reports connection, timeout, or TLS failure directly or wrapped.
# Guarantees: Only transient errors reach the scraper's bounded per-target retry budget.
@pytest.mark.parametrize(
    "kind", ["connection", "protocol", "connect-timeout", "read-timeout", "tls"]
)
@pytest.mark.parametrize("wrapped", [False, True])
def test_proxy_classifies_transport_errors(adopted, kind, wrapped):
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


# Scenario: Reading a raw response body times out after response acquisition.
# Guarantees: The failure remains retryable and the acquired connection is released.
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


# Scenario: Discovery or target publication is interrupted after client creation.
# Guarantees: Startup closes its client once, including after suite cleanup is retried.
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


# Scenario: Re-adoption replaces endpoints while old cleanup remains queued.
# Guarantees: Old endpoints cannot close or reuse the replacement client.
def test_readoption_isolates_client_lifetimes(adopted):
    component, ctx = adopted.component, adopted.ctx
    component.adopt(ctx)
    old_endpoint = component.get_targets()[0].endpoints["metrics"]
    previous = component.runtime.get(KubernetesTargetAccess.type)
    component.adopt(ctx)
    previous.close()
    adopted.proxy.clients[0].close.assert_called_once()
    adopted.proxy.clients[1].close.assert_not_called()
    with pytest.raises(RuntimeError, match="released"):
        old_endpoint.fetch(1)
    assert component.get_targets()[0].endpoints["metrics"].fetch(1)
    adopted.cleanup.close()
    adopted.proxy.clients[1].close.assert_called_once()


# Scenario: A pod or container is replaced, or a terminal leftover joins the selector.
# Guarantees: Replacements fail identity validation while terminal pods are ignored.
@pytest.mark.parametrize("change", ["pod-uid", "container-id", "terminal"])
def test_proxy_validates_fixed_pod_and_container_identities(adopted, change):
    adopted.component.adopt(adopted.ctx)
    pods = [pod.model_copy(deep=True) for pod in adopted.proxy.pods]
    observed = [observed_pod(pod) for pod in pods]
    if change == "pod-uid":
        pods[0].uid = "replacement"
    elif change == "container-id":
        pods[0].containers["app"] = "containerd://replacement"
    else:
        observed.append(observed_pod(None, phase="Succeeded"))
    adopted.proxy.api.list_namespaced_pod.return_value.items = observed
    if change == "terminal":
        assert len(adopted.component.get_targets()) == 2
    else:
        with pytest.raises(ValueError, match="identities changed"):
            adopted.component.get_targets()


# Scenario: Monitoring is interrupted before or just after its worker starts.
# Guarantees: Suite cleanup joins the worker before closing component-owned access.
@pytest.mark.parametrize("error", [RuntimeError, SystemExit, KeyboardInterrupt])
@pytest.mark.parametrize("stage", ["before-thread", "after-thread"])
def test_interrupted_monitoring_stops_before_client_close(
    adopted, monkeypatch, error, stage
):
    component, ctx = adopted.component, adopted.ctx
    component.adopt(ctx)

    def worker(**kwargs):
        runtime = kwargs["runtime"]
        runtime.scrape_successes.update({target.name: 1 for target in runtime.targets})
        kwargs["stop_event"].wait()
        runtime.completed = True

    def create_thread(*args, **kwargs):
        thread = threading.Thread(*args, **kwargs)
        start = thread.start

        def interrupted():
            if stage == "after-thread":
                start()
            raise error("interrupted")

        monkeypatch.setattr(thread, "start", interrupted)
        return thread

    def close():
        runtime = component.runtime.get(PrometheusMonitoringRuntime.type)
        assert runtime.stop_event.is_set()
        assert runtime.thread.ident is None or not runtime.thread.is_alive()

    adopted.proxy.clients[0].close.side_effect = close
    monkeypatch.setattr(prometheus, "monitor", worker)
    monkeypatch.setattr(
        prometheus,
        "threading",
        SimpleNamespace(Event=threading.Event, Thread=create_thread),
    )
    with pytest.raises(error, match="interrupted"):
        with adopted.cleanup:
            component.start_monitoring(ctx)
    adopted.proxy.clients[0].close.assert_called_once()


# Scenario: A captured scraper fails even after the component runtime is replaced.
# Guarantees: Suite cleanup propagates original worker errors and still closes access.
@pytest.mark.parametrize("failure", ["scrape-error", "unexpected-exit"])
def test_suite_cleanup_validates_captured_scraper_health(adopted, monkeypatch, failure):
    error = OSError("scrape failed")

    def worker(**kwargs):
        if failure == "scrape-error":
            kwargs["runtime"].error = error
            kwargs["runtime"].completed = True

    monkeypatch.setattr(prometheus, "monitor", worker)
    adopted.component.adopt(adopted.ctx)
    adopted.component.start_monitoring(adopted.ctx)
    original = adopted.component.runtime.get(PrometheusMonitoringRuntime.type)
    original.thread.join(timeout=5)
    assert not original.thread.is_alive()
    adopted.component.set_runtime_data(
        PrometheusMonitoringRuntime.type, PrometheusMonitoringRuntime(completed=True)
    )
    message = "scrape failed" if failure == "scrape-error" else "exited unexpectedly"
    with pytest.raises(RuntimeError, match=message):
        adopted.cleanup.close()
    adopted.proxy.clients[0].close.assert_called_once()


# Scenario: A target times out twice, recovers, and times out beside a healthy target.
# Guarantees: Success resets only its own budget; failed fetches never replay gauges.
@pytest.mark.parametrize(
    "error",
    [requests.ReadTimeout, requests.ConnectionError, TimeoutError, ConnectionError],
)
def test_transient_scrape_failures_do_not_fabricate_samples(scrape_monitor, error):
    backend = scrape_monitor.run(
        {
            "pod-0": [
                error("slow"),
                error("slow"),
                "example_value 7\n",
                error("slow"),
                "example_value 9\n",
            ],
            "pod-1": ["example_value 11\n"] * 5,
        }
    )
    PrometheusMonitoringStrategy._check_runtime(scrape_monitor.runtime)
    assert scrape_monitor.logger.warning.call_count == 3
    assert scrape_monitor.logger.info.call_count == 2
    metrics = backend.get_metrics_df()
    samples = metrics[metrics.metric_name == "example_value"]
    for pod, values in [("pod-0", [7, 9]), ("pod-1", [11] * 5)]:
        assert [
            row.value
            for row in samples.itertuples()
            if row.metric_attributes["k8s.pod.name"] == pod
        ] == values
    counts = {
        (
            row.metric_attributes["k8s.pod.name"],
            row.metric_attributes["outcome"],
        ): row.value
        for row in metrics[
            metrics.metric_name == "prometheus.scrape.count"
        ].itertuples()
    }
    assert counts == {
        ("pod-0", "failure"): 3,
        ("pod-0", "success"): 2,
        ("pod-1", "success"): 5,
    }


# Scenario: Either target repeatedly times out while the other remains healthy.
# Guarantees: Three consecutive failures are fatal independently for each target.
@pytest.mark.parametrize("pod", ["pod-0", "pod-1"])
def test_three_consecutive_failures_are_fatal(scrape_monitor, pod):
    outcomes = {name: ["example_value 1\n"] * 3 for name in ("pod-0", "pod-1")}
    outcomes[pod] = [TimeoutError("slow") for _ in range(3)]
    scrape_monitor.run(outcomes)
    assert scrape_monitor.calls.count(pod) == 3
    with pytest.raises(RuntimeError, match=f"{pod!r} failed 3 consecutive attempts"):
        PrometheusMonitoringStrategy._check_runtime(scrape_monitor.runtime)


# Scenario: A proxy HTTP error, TLS error, or invalid metrics body occurs.
# Guarantees: Non-transient failures are fatal on their first attempt.
@pytest.mark.parametrize(
    "result",
    [RuntimeError("HTTP 403"), requests.exceptions.SSLError("TLS"), "value invalid\n"],
)
def test_non_transient_failures_are_not_retried(scrape_monitor, result):
    scrape_monitor.run({"pod-0": [result], "pod-1": ["example_value 1\n"]})
    assert scrape_monitor.calls == ["pod-0"]
    with pytest.raises(RuntimeError, match="Target Prometheus monitoring failed"):
        PrometheusMonitoringStrategy._check_runtime(scrape_monitor.runtime)


# Scenario: Target identity changes or discovery API access is denied.
# Guarantees: Validation failure is fatal before any metric fetch.
@pytest.mark.parametrize("failure", ["identity", "api", "new-targets"])
def test_validation_failures_are_immediately_fatal(scrape_monitor, failure):
    from kubernetes.client.exceptions import ApiException

    if failure == "identity":
        scrape_monitor.api.list_namespaced_pod.return_value.items = []
    elif failure == "api":
        scrape_monitor.api.list_namespaced_pod.side_effect = ApiException(status=403)
    else:
        scrape_monitor.component.set_targets([])
    scrape_monitor.run({"pod-0": ["example_value 1\n"], "pod-1": ["example_value 1\n"]})
    assert not scrape_monitor.calls
    with pytest.raises(RuntimeError):
        PrometheusMonitoringStrategy._check_runtime(scrape_monitor.runtime)


# Scenario: Monitoring stops during retry before one target has any successful fetch.
# Guarantees: Shutdown interrupts the interval but rejects missing coverage.
def test_retry_shutdown_requires_successful_coverage(scrape_monitor):
    scrape_monitor.run(
        {"pod-0": [TimeoutError("slow")] * 2, "pod-1": ["example_value 1\n"] * 2},
        stop_on_wait=True,
    )
    assert scrape_monitor.calls == ["pod-0", "pod-1"]
    with pytest.raises(RuntimeError, match="No successful.*pod-0"):
        PrometheusMonitoringStrategy._check_runtime(scrape_monitor.runtime)


# Scenario: A backend-neutral endpoint returns metrics with spoofed identity labels.
# Guarantees: Fetching preserves component identity without direct HTTP calls.
def test_generic_fetch_preserves_authoritative_identity(monkeypatch):
    endpoint = SimpleNamespace(
        fetch=MagicMock(return_value='example_value{instance="spoofed"} 42\n')
    )
    gauge = MagicMock()
    get = MagicMock(side_effect=AssertionError("unexpected direct HTTP request"))
    monkeypatch.setattr(prometheus.requests, "get", get)
    monkeypatch.setattr(prometheus, "get_instrument", lambda *args: gauge)
    scrape_and_convert_metrics(
        endpoint,
        "service",
        MagicMock(),
        request_timeout=2,
        attributes={"instance": "actual", "component_name": "wrong"},
    )
    endpoint.fetch.assert_called_once_with(2)
    gauge.set.assert_called_once_with(
        42, {"instance": "actual", "component_name": "service"}
    )


# Scenario: A named monitor has no targets or a target without the requested endpoint.
# Guarantees: Invalid selection fails before a monitoring thread is created.
@pytest.mark.parametrize(
    "targets",
    [[], [ComponentTarget(name="instance", endpoints={"other": "http://host"})]],
)
def test_target_monitor_rejects_missing_endpoints(targets, monkeypatch):
    component = build_managed_component("service", ManagedComponentConfiguration())
    component.set_targets(targets)
    strategy = PrometheusMonitoringStrategy(
        PrometheusMonitoringConfig(target_endpoint="metrics")
    )
    thread = MagicMock()
    monkeypatch.setattr(
        prometheus, "threading", SimpleNamespace(Thread=thread, Event=threading.Event)
    )
    with pytest.raises(ValueError, match="No targets|has no endpoint"):
        strategy.start(component, MagicMock())
    thread.assert_not_called()
