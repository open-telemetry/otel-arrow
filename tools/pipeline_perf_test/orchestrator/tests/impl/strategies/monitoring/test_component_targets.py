"""Backend-neutral target monitoring and access ownership."""

from contextlib import ExitStack
from types import SimpleNamespace
import threading
from unittest.mock import DEFAULT, MagicMock

import pytest
import requests

from lib.core.component import ComponentTarget
from lib.core.strategies.existing_strategy import ExistingStrategy
from lib.runner.factory import build_managed_component
from lib.impl.component.managed_component import ManagedComponentConfiguration
from lib.impl.strategies.monitoring import prometheus
from lib.impl.strategies.monitoring.prometheus import (
    PrometheusMonitoringConfig,
    PrometheusMonitoringRuntime,
    PrometheusMonitoringStrategy,
)


@pytest.fixture
def scrape_targets(monkeypatch):
    def run(outcomes, validate=None):
        component = build_managed_component("service", ManagedComponentConfiguration())
        endpoints = {
            name: SimpleNamespace(fetch=MagicMock(side_effect=results))
            for name, results in outcomes.items()
        }
        component.set_targets(
            [
                ComponentTarget(
                    name,
                    {"metrics": endpoint},
                    {"component_name": "spoofed", "component.target.name": "spoofed"},
                )
                for name, endpoint in endpoints.items()
            ],
            validate,
        )
        runtime = PrometheusMonitoringRuntime(
            targets=component.get_targets(),
            target_endpoint="metrics",
            stop_event=threading.Event(),
        )
        monkeypatch.setattr(runtime.stop_event, "wait", lambda _timeout: None)
        gauge, meter, logger = MagicMock(), MagicMock(), MagicMock()
        monkeypatch.setattr(prometheus, "get_instrument", lambda *args: gauge)
        prometheus.monitor(
            component_name="service",
            endpoint=None,
            include=[],
            exclude=[],
            interval=1,
            count=max(len(results) for results in outcomes.values()),
            stop_event=runtime.stop_event,
            meter=meter,
            logger=logger,
            test_suite_context=MagicMock(),
            request_timeout=2,
            runtime=runtime,
            get_targets=component.get_targets,
        )
        return SimpleNamespace(
            runtime=runtime,
            endpoints=endpoints,
            gauge=gauge,
            meter=meter,
            logger=logger,
        )

    return run


# Scenario: Two generic endpoints return samples with forged component identity.
# Guarantees: Fetching uses bounded requests and distinct authoritative target labels.
def test_named_targets_preserve_identity_without_backend_dependencies(scrape_targets):
    result = scrape_targets(
        {"first": ['example{component_name="forged"} 42\n'], "second": ["example 7\n"]}
    )
    for endpoint in result.endpoints.values():
        endpoint.fetch.assert_called_once_with(2)
    assert [
        (call.args[0], call.args[1]) for call in result.gauge.set.call_args_list
    ] == [
        (42, {"component_name": "service", "component.target.name": "first"}),
        (7, {"component_name": "service", "component.target.name": "second"}),
    ]
    PrometheusMonitoringStrategy._check_runtime(result.runtime)


# Scenario: A target fails with a transport, HTTP, TLS, or malformed-response error.
# Guarantees: Named-target failures surface immediately without new retry policy.
@pytest.mark.parametrize(
    "error",
    [
        requests.ReadTimeout("slow"),
        ConnectionError("disconnected"),
        requests.HTTPError("403"),
        requests.exceptions.SSLError("TLS"),
        ValueError("bad"),
    ],
)
def test_target_failures_surface_immediately(scrape_targets, error):
    result = scrape_targets({"first": [error, "example 42\n"]})
    assert result.endpoints["first"].fetch.call_count == 1
    with pytest.raises(RuntimeError) as raised:
        PrometheusMonitoringStrategy._check_runtime(result.runtime)
    assert raised.value.__cause__ is error


# Scenario: An existing explicit URL times out, then returns filtered metrics.
# Guarantees: It logs and continues, preserving timeout, filtering and legacy labels.
def test_direct_url_preserves_scrape_behavior(monkeypatch):
    response = MagicMock(text='keep{instance="legacy"} 42\nskip 7\n')
    get = MagicMock(side_effect=[requests.ReadTimeout("slow"), response])
    monkeypatch.setattr(prometheus.requests, "get", get)
    gauge = MagicMock()
    monkeypatch.setattr(prometheus, "get_instrument", lambda *args: gauge)
    logger = MagicMock()
    stop = threading.Event()
    monkeypatch.setattr(stop, "wait", lambda _timeout: None)
    prometheus.monitor(
        component_name="service",
        endpoint="http://localhost/metrics",
        include=["keep"],
        exclude=["keep"],
        interval=1,
        count=1,
        stop_event=stop,
        meter=MagicMock(),
        logger=logger,
        test_suite_context=MagicMock(),
        request_timeout=2,
    )
    assert get.call_count == 2
    get.assert_called_with("http://localhost/metrics", timeout=2)
    logger.error.assert_called_once()
    gauge.set.assert_called_once_with(
        42, {"instance": "legacy", "component_name": "service"}
    )


# Scenario: Component validation fails before the next named-target scrape.
# Guarantees: Stale access cannot continue collecting samples.
def test_target_validation_failure_stops_scraping(scrape_targets):
    validate = MagicMock(side_effect=[None, ValueError("identity changed")])
    result = scrape_targets({"first": ["example 42\n"]}, validate)
    result.endpoints["first"].fetch.assert_not_called()
    with pytest.raises(RuntimeError, match="identity changed"):
        PrometheusMonitoringStrategy._check_runtime(result.runtime)


# Scenario: A named monitor selects no targets or an absent endpoint.
# Guarantees: Invalid selection fails before a monitoring thread is created.
@pytest.mark.parametrize(
    "targets", [[], [ComponentTarget("first", {"other": "http://localhost"})]]
)
def test_target_selection_is_validated_before_thread_start(targets, monkeypatch):
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


@pytest.fixture
def adopted_target(monkeypatch):
    component = build_managed_component("service", ManagedComponentConfiguration())
    existing = MagicMock(spec=ExistingStrategy)
    existing.resolve.side_effect = lambda *_args: component.set_targets(
        [ComponentTarget("first", {"metrics": "http://localhost/metrics"})]
    )
    component.existing = existing
    component.monitoring = PrometheusMonitoringStrategy(
        PrometheusMonitoringConfig(target_endpoint="metrics", interval=0.01)
    )
    started = threading.Event()
    runtimes = []

    def worker(**kwargs):
        runtime = kwargs["runtime"]
        runtimes.append(runtime)
        started.set()
        runtime.stop_event.wait()
        runtime.completed = True

    def release(*_args):
        assert all(not runtime.thread.is_alive() for runtime in runtimes)

    existing.release.side_effect = release
    monkeypatch.setattr(prometheus, "monitor", worker)
    ctx = MagicMock()
    ctx.get_suite().get_runtime.return_value = None
    with ExitStack() as cleanup:
        ctx.get_suite().context.cleanup = cleanup
        yield SimpleNamespace(
            component=component,
            existing=existing,
            ctx=ctx,
            cleanup=cleanup,
            started=started,
            runtimes=runtimes,
        )


# Scenario: Active target monitoring is destroyed, re-adopted, or left to suite cleanup.
# Guarantees: Workers stop before access release, and old callbacks cannot release new access.
@pytest.mark.parametrize("action", ["destroy", "readopt", "suite-exit"])
def test_component_owns_stop_before_release(adopted_target, action):
    fixture = adopted_target
    component, ctx = fixture.component, fixture.ctx
    component.adopt(ctx)
    component.start_monitoring(ctx)
    assert fixture.started.wait(timeout=5)
    if action == "destroy":
        component.destroy(ctx)
        assert not component.get_targets()
        with pytest.raises(ValueError, match="successful.*adopt"):
            component.start_monitoring(ctx)
    elif action == "readopt":
        component.adopt(ctx)
        fixture.existing.release.assert_called_once()
        fixture.started.clear()
        component.start_monitoring(ctx)
        assert fixture.started.wait(timeout=5)
    fixture.cleanup.close()
    assert fixture.existing.release.call_count == (2 if action == "readopt" else 1)
    assert all(not runtime.thread.is_alive() for runtime in fixture.runtimes)


# Scenario: Adoption interrupts after it may have allocated partial provider access.
# Guarantees: Suite exit invokes the captured provider's release even before adoption succeeds.
@pytest.mark.parametrize("error", [RuntimeError, SystemExit, KeyboardInterrupt])
def test_interrupted_adoption_releases_partial_access(adopted_target, error):
    fixture = adopted_target
    fixture.existing.resolve.side_effect = error("interrupted")
    with pytest.raises(error, match="interrupted"):
        fixture.component.adopt(fixture.ctx)
    fixture.cleanup.close()
    fixture.existing.release.assert_called_once()


# Scenario: Monitoring is replaced before explicit stop, destruction or re-adoption.
# Guarantees: Shutdown stops the original consumer before releasing its provider access.
@pytest.mark.parametrize("action", ["destroy", "adopt"])
@pytest.mark.parametrize("stop_first", [False, True])
def test_release_stops_original_monitor_after_strategy_replacement(
    adopted_target, action, stop_first
):
    fixture = adopted_target
    fixture.component.adopt(fixture.ctx)
    fixture.component.start_monitoring(fixture.ctx)
    assert fixture.started.wait(timeout=5)
    original = fixture.runtimes[0]
    replacement = MagicMock()
    fixture.component.monitoring = replacement
    try:
        if stop_first:
            fixture.component.stop_monitoring(fixture.ctx)
            assert not original.thread.is_alive()
        getattr(fixture.component, action)(fixture.ctx)
        replacement.stop.assert_not_called()
        fixture.existing.release.assert_called_once()
    finally:
        original.stop_event.set()
        original.thread.join(timeout=5)


# Scenario: The original monitor fails its first stop after its strategy is replaced.
# Guarantees: Explicit stop, destroy and re-adoption can retry before releasing access.
@pytest.mark.parametrize("retry", ["stop_monitoring", "destroy", "adopt"])
def test_shutdown_retries_original_stop(adopted_target, monkeypatch, retry):
    fixture = adopted_target
    component, ctx = fixture.component, fixture.ctx
    component.adopt(ctx)
    component.start_monitoring(ctx)
    assert fixture.started.wait(timeout=5)
    original = fixture.runtimes[0]
    stop = MagicMock(
        wraps=component.monitoring.stop,
        side_effect=[RuntimeError("stop failed"), DEFAULT],
    )
    monkeypatch.setattr(component.monitoring, "stop", stop)
    replacement = MagicMock()
    component.monitoring = replacement
    try:
        with pytest.raises(RuntimeError, match="stop failed"):
            component.stop_monitoring(ctx)
        assert original.thread.is_alive()
        fixture.existing.release.assert_not_called()
        getattr(component, retry)(ctx)
        assert not original.thread.is_alive()
        assert stop.call_count == 2
        replacement.stop.assert_not_called()
        if retry == "stop_monitoring":
            component.destroy(ctx)
        fixture.existing.release.assert_called_once()
    finally:
        original.stop_event.set()
        original.thread.join(timeout=5)


# Scenario: A replacement strategy tries to start before the original monitor stops.
# Guarantees: The active cleanup remains owned and the replacement never starts.
def test_monitoring_rejects_start_while_original_is_active(adopted_target):
    fixture = adopted_target
    component, ctx = fixture.component, fixture.ctx
    component.adopt(ctx)
    component.start_monitoring(ctx)
    assert fixture.started.wait(timeout=5)
    original = fixture.runtimes[0]
    replacement = MagicMock()
    component.monitoring = replacement
    try:
        with pytest.raises(RuntimeError, match="already running"):
            component.start_monitoring(ctx)
        replacement.start.assert_not_called()
        component.destroy(ctx)
        fixture.existing.release.assert_called_once()
        replacement.stop.assert_not_called()
    finally:
        original.stop_event.set()
        original.thread.join(timeout=5)


# Scenario: Runtime data is replaced while the original scraper is still active.
# Guarantees: Stop, destroy and re-adoption join the owned worker before access release.
@pytest.mark.parametrize("action", ["stop_monitoring", "destroy", "adopt"])
def test_shutdown_stops_original_worker_after_runtime_replacement(
    adopted_target, action
):
    fixture = adopted_target
    component, ctx = fixture.component, fixture.ctx
    component.adopt(ctx)
    component.start_monitoring(ctx)
    assert fixture.started.wait(timeout=5)
    original = fixture.runtimes[0]
    component.set_runtime_data(
        PrometheusMonitoringRuntime.type, PrometheusMonitoringRuntime(completed=True)
    )
    getattr(component, action)(ctx)
    assert original.stop_event.is_set()
    assert not original.thread.is_alive()
    fixture.cleanup.close()


# Scenario: A scraper fails, optionally after its component runtime is replaced.
# Guarantees: Captured cleanup surfaces that failure and releases only stopped access.
@pytest.mark.parametrize("replace_runtime", [False, True])
def test_cleanup_checks_the_original_scraper_runtime(adopted_target, replace_runtime):
    fixture = adopted_target
    fixture.component.adopt(fixture.ctx)
    fixture.component.start_monitoring(fixture.ctx)
    assert fixture.started.wait(timeout=5)
    original = fixture.runtimes[0]
    original.error = ValueError("scraper failed")
    original.stop_event.set()
    original.thread.join(timeout=5)
    assert not original.thread.is_alive()
    if replace_runtime:
        fixture.component.set_runtime_data(
            PrometheusMonitoringRuntime.type,
            PrometheusMonitoringRuntime(completed=True),
        )
    with pytest.raises(RuntimeError, match="scraper failed"):
        fixture.cleanup.close()
    fixture.existing.release.assert_called_once()
