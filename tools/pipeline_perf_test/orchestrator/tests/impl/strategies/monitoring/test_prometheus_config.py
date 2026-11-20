"""Validation tests for the Prometheus monitoring strategy config."""

import pytest
from pydantic import ValidationError

from lib.impl.strategies.monitoring.prometheus import PrometheusMonitoringConfig


# Scenario: a prometheus config is created with a non-positive request_timeout.
# Guarantees: zero and negative scrape timeouts are rejected so the bound cannot
# be disabled or made to fail every scrape.
@pytest.mark.parametrize("bad_timeout", [0, -1, -0.5])
def test_rejects_non_positive_request_timeout(bad_timeout):
    with pytest.raises(ValidationError):
        PrometheusMonitoringConfig(
            endpoint="http://localhost:9090/metrics", request_timeout=bad_timeout
        )


# Scenario: a prometheus config is created with a non-positive interval.
# Guarantees: zero and negative polling intervals are rejected.
@pytest.mark.parametrize("bad_interval", [0, -1, -0.5])
def test_rejects_non_positive_interval(bad_interval):
    with pytest.raises(ValidationError):
        PrometheusMonitoringConfig(
            endpoint="http://localhost:9090/metrics", interval=bad_interval
        )


# Scenario: a prometheus config is created with valid positive timeouts and the
# default (None/omitted) interval.
# Guarantees: valid values are accepted and None interval remains allowed.
def test_accepts_valid_values():
    cfg = PrometheusMonitoringConfig(
        endpoint="http://localhost:9090/metrics",
        request_timeout=2.5,
        interval=0.5,
    )
    assert cfg.request_timeout == 2.5
    assert cfg.interval == 0.5
    # None interval is still permitted.
    assert (
        PrometheusMonitoringConfig(
            endpoint="http://localhost:9090/metrics", interval=None
        ).interval
        is None
    )


# Scenario: A config omits or mixes endpoint sources, or specifies an invalid tunnel.
# Guarantees: Monitoring requires one unambiguous and valid endpoint source.
@pytest.mark.parametrize(
    "config",
    [
        {},
        {
            "endpoint": "http://localhost/metrics",
            "kubernetes_port_forward": {"port": 80},
        },
        {"kubernetes_port_forward": {"port": 0}},
        {"kubernetes_port_forward": {"port": 65536}},
        {"kubernetes_port_forward": {"port": 80, "path": "metrics"}},
        {"kubernetes_port_forward": {"port": 80, "startup_timeout": 0}},
    ],
)
def test_rejects_invalid_endpoint_sources(config):
    with pytest.raises(ValidationError):
        PrometheusMonitoringConfig(**config)


# Scenario: A caller selects forwarded pod access instead of a direct URL.
# Guarantees: The config retains the chosen remote port and metrics path.
def test_accepts_adopted_pod_endpoint():
    config = PrometheusMonitoringConfig(
        kubernetes_port_forward={"port": 12345, "path": "/custom/metrics"}
    )
    assert config.endpoint is None
    assert config.kubernetes_port_forward.port == 12345
