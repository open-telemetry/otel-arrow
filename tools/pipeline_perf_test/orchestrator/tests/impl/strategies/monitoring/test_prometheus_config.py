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
