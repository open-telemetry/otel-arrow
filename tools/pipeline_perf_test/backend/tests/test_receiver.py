import signal
import sys
import pytest
from opentelemetry.proto.collector.logs.v1 import logs_service_pb2
from opentelemetry.proto.logs.v1 import logs_pb2
from unittest import mock
from unittest.mock import AsyncMock

from backend import (
    FakeLogsExporter,
    metrics,
    prom_metrics,
    received_logs_lock,
    handle_signal,
)

from backend import app


@pytest.fixture(autouse=True)
def flask_context():
    with app.app_context():
        yield


@pytest.mark.asyncio
async def test_export_increments_received_logs():
    # Setup fake request with 2 ResourceLogs, each with 1 ScopeLog with 3 and 5 records
    request = logs_service_pb2.ExportLogsServiceRequest(
        resource_logs=[
            logs_pb2.ResourceLogs(
                scope_logs=[logs_pb2.ScopeLogs(log_records=[{}, {}, {}])]
            ),
            logs_pb2.ResourceLogs(
                scope_logs=[logs_pb2.ScopeLogs(log_records=[{}, {}, {}, {}, {}])]
            ),
        ]
    )

    # Clear state before test
    async with received_logs_lock:
        import backend
        backend.received_logs = 0

    exporter = FakeLogsExporter()
    response = await exporter.Export(request, context=AsyncMock())

    # Confirm response type and new value
    from backend import received_logs as new_logs

    assert isinstance(response, logs_service_pb2.ExportLogsServiceResponse)
    assert new_logs == 8


@pytest.mark.asyncio
async def test_metrics_returns_correct_log_count(monkeypatch):
    async with received_logs_lock:
        import backend
        backend.received_logs = 42

    response = await metrics()
    data = response.get_json()
    assert data["received_logs"] == 42


@pytest.mark.asyncio
async def test_prom_metrics_returns_prometheus_format():
    async with received_logs_lock:
        import backend
        backend.received_logs = 1000

    response = await prom_metrics()
    assert response == "received_logs 1000"


def test_handle_signal_calls_exit(monkeypatch):
    monkeypatch.setattr(sys, "exit", mock.Mock())

    handle_signal(signal.SIGINT, None)

    sys.exit.assert_called_once_with(0)
