"""Generic adopted-pod transport, identity, and scrape failure checks."""

from unittest.mock import MagicMock
from types import SimpleNamespace
from contextlib import ExitStack
import math
import threading

import pytest
import requests
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader

from lib.core.telemetry.metric import FrameworkMetricBackend, FrameworkMetricExporter
from lib.impl.strategies.common.kubernetes import (
    ComponentKubernetesRuntime,
    KubernetesPodIdentity,
)
from lib.impl.strategies.common.kubernetes_port_forward import (
    KubernetesPortForwardConfig,
    KubernetesPortForwards,
)
from lib.impl.component.managed_component import ManagedComponentConfiguration
from lib.runner.factory import build_managed_component
from lib.impl.strategies.monitoring import prometheus
from lib.impl.strategies.monitoring.prometheus import (
    PrometheusMonitoringConfig,
    PrometheusMonitoringRuntime,
    PrometheusMonitoringStrategy,
    scrape_and_convert_metrics,
)


def observed_pod(identity, phase="Running"):
    return SimpleNamespace(identity=identity, status=SimpleNamespace(phase=phase))


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
    api.list_namespaced_pod.return_value.items = [observed_pod(pod) for pod in pods]
    monkeypatch.setattr(config, "new_client_from_config", MagicMock())
    monkeypatch.setattr(client, "CoreV1Api", lambda _: api)
    monkeypatch.setattr(module, "pod_identity", lambda pod: pod.identity)
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


@pytest.fixture
def forwarded_monitor(forwarding, monkeypatch):
    forwards, _, processes, _, terminate = forwarding
    component = build_managed_component("engine", ManagedComponentConfiguration())
    component.set_runtime_data(ComponentKubernetesRuntime.type, forwards.target)
    strategy = PrometheusMonitoringStrategy(
        PrometheusMonitoringConfig(kubernetes_port_forward={"port": 12345})
    )
    ctx = MagicMock()
    cleanup = ExitStack()
    ctx.get_suite().context.cleanup = cleanup
    monkeypatch.setattr(prometheus, "KubernetesPortForwards", lambda *args: forwards)
    try:
        yield strategy, component, ctx, cleanup, processes, terminate
    finally:
        runtime = component.runtime.get(PrometheusMonitoringRuntime.type)
        if runtime is not None and runtime.stop_event is not None:
            runtime.stop_event.set()
            if runtime.thread is not None and runtime.thread.ident is not None:
                runtime.thread.join(timeout=5)
        cleanup.close()


@pytest.fixture
def scrape_monitor(forwarding, monkeypatch):
    forwards, api, processes, _, _ = forwarding
    forwards.open()
    backend = FrameworkMetricBackend()
    reader = PeriodicExportingMetricReader(
        FrameworkMetricExporter(backend), export_interval_millis=math.inf
    )
    provider = MeterProvider(metric_readers=[reader])
    meter = provider.get_meter("transient-scrapes")
    stop_event = threading.Event()
    runtime = PrometheusMonitoringRuntime(forwards=forwards, stop_event=stop_event)
    logger = MagicMock()
    calls = []
    monkeypatch.setattr(prometheus, "instrument_cache", {})

    def run(outcomes, stop_on_wait=False):
        routes = {
            endpoint.url: (
                endpoint.attributes["k8s.pod.name"],
                iter(outcomes[endpoint.attributes["k8s.pod.name"]]),
            )
            for endpoint in forwards.endpoints
        }

        def get(url, timeout):
            pod, responses = routes[url]
            calls.append(pod)
            response = next(responses)
            if isinstance(response, Exception):
                raise response
            return SimpleNamespace(text=response, raise_for_status=lambda: None)

        def wait(_timeout):
            reader.collect()
            if stop_on_wait:
                stop_event.set()

        monkeypatch.setattr(prometheus.requests, "get", get)
        monkeypatch.setattr(stop_event, "wait", MagicMock(side_effect=wait))
        prometheus.monitor(
            component_name="engine",
            endpoint=None,
            include=[],
            exclude=[],
            interval=5,
            count=max(len(values) for values in outcomes.values()),
            stop_event=stop_event,
            meter=meter,
            logger=logger,
            test_suite_context=MagicMock(),
            request_timeout=1,
            runtime=runtime,
        )
        reader.collect()
        return backend

    try:
        yield SimpleNamespace(
            run=run,
            runtime=runtime,
            logger=logger,
            calls=calls,
            api=api,
            processes=processes,
        )
    finally:
        provider.shutdown()


# Scenario: One pod times out twice, recovers, and times out again beside a healthy pod.
# Guarantees: Only that pod's successes reset its budget, and gaps never replay gauges.
@pytest.mark.parametrize(
    "error", [requests.ReadTimeout, requests.ConnectTimeout, requests.ConnectionError]
)
def test_transient_scrape_failures_recover_without_fabricated_samples(
    scrape_monitor, error
):
    backend = scrape_monitor.run(
        {
            "pod-0": [
                error("slow endpoint"),
                error("slow endpoint"),
                "example_value 7\n",
                error("slow endpoint"),
                "example_value 9\n",
            ],
            "pod-1": ["example_value 11\n"] * 5,
        }
    )
    metrics = backend.get_metrics_df()
    PrometheusMonitoringStrategy._check_runtime(scrape_monitor.runtime)
    assert scrape_monitor.runtime.error is None
    assert not scrape_monitor.runtime.stop_event.is_set()
    assert scrape_monitor.logger.warning.call_count == 3
    assert scrape_monitor.logger.info.call_count == 2
    assert scrape_monitor.runtime.stop_event.wait.call_count == 5

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


# Scenario: One endpoint keeps timing out while the other endpoint remains healthy.
# Guarantees: The third consecutive failure is fatal regardless of endpoint order.
@pytest.mark.parametrize("pod", ["pod-0", "pod-1"])
def test_three_consecutive_scrape_failures_are_fatal_per_pod(scrape_monitor, pod):
    errors = [requests.ReadTimeout("slow endpoint") for _ in range(3)]
    outcomes = {name: ["example_value 1\n"] * 3 for name in ["pod-0", "pod-1"]}
    outcomes[pod] = errors
    scrape_monitor.run(outcomes)
    runtime = scrape_monitor.runtime
    assert runtime.stop_event.is_set()
    assert runtime.error.__cause__ is errors[-1]
    with pytest.raises(RuntimeError, match=f"{pod!r} failed 3 consecutive attempts"):
        PrometheusMonitoringStrategy._check_runtime(runtime)
    assert scrape_monitor.calls.count(pod) == 3
    assert runtime.stop_event.wait.call_count == 2


# Scenario: A forwarded scrape has an HTTP, TLS configuration, or parsing error.
# Guarantees: Non-transient failures remain fatal on the first attempt.
@pytest.mark.parametrize(
    "response",
    [
        requests.HTTPError("404"),
        requests.exceptions.SSLError("invalid certificate"),
        "example_value invalid\n",
    ],
)
def test_non_transient_scrape_failures_are_not_retried(scrape_monitor, response):
    scrape_monitor.run({"pod-0": [response], "pod-1": ["example_value 1\n"]})
    assert scrape_monitor.runtime.error is not None
    assert scrape_monitor.runtime.stop_event.is_set()
    assert scrape_monitor.calls == ["pod-0"]
    scrape_monitor.runtime.stop_event.wait.assert_not_called()
    with pytest.raises(RuntimeError, match="Kubernetes Prometheus monitoring failed"):
        PrometheusMonitoringStrategy._check_runtime(scrape_monitor.runtime)


# Scenario: Identity validation detects a replaced pod or a dead owned tunnel.
# Guarantees: Scrape retry tolerance cannot hide invalid measurement targets.
@pytest.mark.parametrize("failure", ["identity", "tunnel"])
def test_validation_failures_remain_immediately_fatal(scrape_monitor, failure):
    if failure == "identity":
        scrape_monitor.api.list_namespaced_pod.return_value.items = []
    else:
        scrape_monitor.processes[0].poll.return_value = 1
    scrape_monitor.run({"pod-0": ["example_value 1\n"], "pod-1": ["example_value 1\n"]})
    assert scrape_monitor.runtime.error is not None
    assert scrape_monitor.runtime.stop_event.is_set()
    assert not scrape_monitor.calls
    with pytest.raises(RuntimeError, match="Kubernetes Prometheus monitoring failed"):
        PrometheusMonitoringStrategy._check_runtime(scrape_monitor.runtime)


# Scenario: Monitoring stops after one timeout before any pod-0 sample succeeds.
# Guarantees: Shutdown interrupts the retry interval, but zero coverage cannot pass.
def test_stopping_during_retry_does_not_hide_missing_coverage(scrape_monitor):
    scrape_monitor.run(
        {
            "pod-0": [requests.ReadTimeout("slow endpoint")] * 2,
            "pod-1": ["example_value 1\n"] * 2,
        },
        stop_on_wait=True,
    )
    assert scrape_monitor.calls == ["pod-0", "pod-1"]
    scrape_monitor.runtime.stop_event.wait.assert_called_once_with(5)
    assert scrape_monitor.runtime.error is None
    with pytest.raises(RuntimeError, match="No successful.*pod-0"):
        PrometheusMonitoringStrategy._check_runtime(scrape_monitor.runtime)


# Scenario: Startup fails or is interrupted after tunnels open or a worker starts.
# Guarantees: Early registered cleanup signals workers and releases every owned tunnel.
@pytest.mark.parametrize("error", [RuntimeError, SystemExit, KeyboardInterrupt])
@pytest.mark.parametrize("stage", ["open", "before-thread", "after-thread"])
def test_interrupted_startup_releases_transports_and_worker(
    forwarded_monitor, monkeypatch, error, stage
):
    strategy, component, ctx, cleanup, processes, terminate = forwarded_monitor

    def worker(**kwargs):
        kwargs["stop_event"].wait()
        kwargs["runtime"].completed = True

    monkeypatch.setattr(prometheus, "monitor", worker)
    if stage == "open":
        from lib.impl.strategies.common import kubernetes_port_forward

        original_open = kubernetes_port_forward.KubernetesPortForwards.open

        def interrupted_open(forwards):
            original_open(forwards)
            raise error("startup interrupted")

        monkeypatch.setattr(
            kubernetes_port_forward.KubernetesPortForwards, "open", interrupted_open
        )
    else:

        def create_thread(*args, **kwargs):
            thread = threading.Thread(*args, **kwargs)
            original_start = thread.start

            def interrupted_start():
                if stage == "after-thread":
                    original_start()
                raise error("startup interrupted")

            monkeypatch.setattr(thread, "start", interrupted_start)
            return thread

        monkeypatch.setattr(
            prometheus,
            "threading",
            SimpleNamespace(Event=threading.Event, Thread=create_thread),
        )
    with pytest.raises(error, match="startup interrupted"):
        with cleanup:
            strategy.start(component, ctx)
    runtime = component.runtime.get(PrometheusMonitoringRuntime.type)
    assert runtime.stop_event.is_set()
    assert runtime.thread is None or not runtime.thread.is_alive()
    assert terminate.call_count == len(processes) == 2


# Scenario: Suite cleanup is the only shutdown after a forwarded scraper exits.
# Guarantees: Captured worker failures propagate even if the component runtime changes.
@pytest.mark.parametrize("failure", ["scrape-error", "unexpected-exit"])
def test_suite_cleanup_validates_captured_scraper_health(
    forwarded_monitor, monkeypatch, failure
):
    strategy, component, ctx, cleanup, processes, terminate = forwarded_monitor
    error = OSError("scrape failed")

    def worker(**kwargs):
        if failure == "scrape-error":
            kwargs["runtime"].error = error
            kwargs["runtime"].completed = True

    monkeypatch.setattr(prometheus, "monitor", worker)
    strategy.start(component, ctx)
    original = component.runtime.get(PrometheusMonitoringRuntime.type)
    original.thread.join(timeout=5)
    assert not original.thread.is_alive()
    component.set_runtime_data(
        PrometheusMonitoringRuntime.type, PrometheusMonitoringRuntime(completed=True)
    )
    message = "scrape failed" if failure == "scrape-error" else "exited unexpectedly"
    with pytest.raises(RuntimeError, match=message) as raised:
        cleanup.close()
    if failure == "scrape-error":
        assert raised.value.__cause__ is error
    assert terminate.call_count == len(processes) == 2


# Scenario: Two adopted pods need independent forwarded metrics endpoints.
# Guarantees: Tunnels retain caller credentials and replica identity, then close.
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


# Scenario: The second pod endpoint fails readiness after its tunnel starts.
# Guarantees: Partial startup releases every owned helper without touching pods.
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


# Scenario: An adopted pod changes identity, disappears, or stops running.
# Guarantees: Monitoring fails rather than silently switching or losing targets.
@pytest.mark.parametrize(
    "change", ["pod-uid", "container-id", "missing-pod", "dead-forward", "terminal-pod"]
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
    elif change == "dead-forward":
        processes[0].poll.return_value = 1
    current = [observed_pod(pod) for pod in pods]
    if change == "terminal-pod":
        current[0].status.phase = "Failed"
    api.list_namespaced_pod.return_value.items = current
    with pytest.raises((ValueError, RuntimeError)):
        forwards.validate()


# Scenario: Completed or failed pods remain beside the adopted live pods.
# Guarantees: Starting and validating owned tunnels ignores terminal leftovers.
@pytest.mark.parametrize("phase", ["Succeeded", "Failed"])
def test_terminal_selector_matches_do_not_break_forwarding(forwarding, phase):
    forwards, api, _, _, _ = forwarding
    api.list_namespaced_pod.return_value.items.append(observed_pod(None, phase))
    forwards.open()
    forwards.validate()
    assert len(forwards.endpoints) == 2


# Scenario: Scraped labels conflict with the adopted target's authoritative identity.
# Guarantees: Recorded samples retain the actual pod and logical component identity.
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
