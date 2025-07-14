import pytest
from unittest.mock import MagicMock
from lib.core.telemetry.telemetry_runtime import TelemetryRuntime


@pytest.fixture
def runtime():
    tracer_provider = MagicMock()
    meter_provider = MagicMock()
    telemetry_client = MagicMock()
    return TelemetryRuntime(
        tracer_provider=tracer_provider,
        meter_provider=meter_provider,
        telemetry_client=telemetry_client,
    )


def test_get_tracer(runtime):
    tracer = MagicMock()
    runtime.tracer_provider.get_tracer.return_value = tracer

    result = runtime.get_tracer("test-tracer")
    runtime.tracer_provider.get_tracer.assert_called_once_with("test-tracer")
    assert result is tracer


def test_get_meter(runtime):
    meter = MagicMock()
    runtime.meter_provider.get_meter.return_value = meter

    result = runtime.get_meter("test-meter")
    runtime.meter_provider.get_meter.assert_called_once_with("test-meter")
    assert result is meter


def test_get_client(runtime):
    assert runtime.get_client() is runtime.telemetry_client
