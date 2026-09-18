from typing import Optional

import pytest

from lib.impl.strategies.monitoring.docker_component import (
    _calculate_cpu_usage,
    _docker_stats_interval_seconds,
)


def _stats_payload(
    *,
    cpu_total: int,
    precpu_total: int,
    read: str = "2026-09-18T12:00:02.500000000Z",
    preread: str = "2026-09-18T12:00:00.000000000Z",
    system_total: Optional[int] = None,
    presystem_total: Optional[int] = None,
) -> dict:
    payload = {
        "read": read,
        "preread": preread,
        "cpu_stats": {
            "cpu_usage": {
                "total_usage": cpu_total,
            },
        },
        "precpu_stats": {
            "cpu_usage": {
                "total_usage": precpu_total,
            },
        },
    }
    if system_total is not None and presystem_total is not None:
        payload["cpu_stats"].update(
            {
                "system_cpu_usage": system_total,
                "online_cpus": 4,
            }
        )
        payload["precpu_stats"].update(
            {
                "system_cpu_usage": presystem_total,
            }
        )
    return payload


# Scenario: Docker emits RFC3339Nano timestamps in stats responses.
# Guarantees: Nanosecond precision timestamps are accepted and measured in seconds.
def test_docker_stats_interval_uses_read_and_preread_timestamps():
    payload = _stats_payload(
        cpu_total=0,
        precpu_total=0,
        read="2026-09-18T12:00:02.123456789Z",
        preread="2026-09-18T12:00:00.023456789Z",
    )

    assert _docker_stats_interval_seconds(payload) == pytest.approx(2.1)


# Scenario: Windows Docker stats omit system_cpu_usage.
# Guarantees: CPU usage is normalized with the stats sample window, not poll cadence.
def test_calculate_cpu_usage_for_windows_uses_stats_sample_interval():
    payload = _stats_payload(
        cpu_total=60_000_000,
        precpu_total=10_000_000,
        read="2026-09-18T12:00:02.500000000Z",
        preread="2026-09-18T12:00:00.000000000Z",
    )

    assert _calculate_cpu_usage(payload) == pytest.approx(2.0)


# Scenario: Linux Docker stats include system_cpu_usage.
# Guarantees: Existing system-delta CPU normalization remains unchanged.
def test_calculate_cpu_usage_for_linux_uses_system_cpu_delta():
    payload = _stats_payload(
        cpu_total=30_000,
        precpu_total=10_000,
        system_total=250_000,
        presystem_total=50_000,
    )

    assert _calculate_cpu_usage(payload) == pytest.approx(0.4)
