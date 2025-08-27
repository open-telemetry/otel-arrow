import sys
import os
import time
import threading
from unittest.mock import MagicMock, patch

# Add root dir to sys.path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from loadgen import LoadGenerator  # noqa: E402


@patch("loadgen.grpc.insecure_channel")
@patch("loadgen.logs_service_pb2_grpc.LogsServiceStub")
def test_worker_thread_sends_logs(mock_stub_class, mock_channel):
    generator = LoadGenerator()

    # Mock stub and its Export method
    mock_stub = MagicMock()
    mock_stub.Export.return_value = None
    mock_stub_class.return_value = mock_stub

    # Set up args for the worker thread
    args = {
        "body_size": 10,
        "num_attributes": 1,
        "attribute_value_size": 5,
        "batch_size": 3,
        "threads": 1,
        "target_rate": None,
    }

    # Start the thread and stop it shortly after
    generator.stop_event.clear()
    thread = threading.Thread(target=generator.worker_thread, args=(0, args))
    thread.start()

    # Let the worker send at least once
    time.sleep(0.2)
    generator.stop_event.set()
    thread.join()

    # Verify Export was called
    assert mock_stub.Export.called
    assert generator.metrics["sent"] >= 3
    assert generator.metrics["bytes_sent"] > 0
    assert generator.metrics["failed"] == 0


@patch("loadgen.grpc.insecure_channel")
@patch("loadgen.logs_service_pb2_grpc.LogsServiceStub")
def test_worker_thread_handles_export_failure(mock_stub_class, mock_channel):
    generator = LoadGenerator()

    # Simulate gRPC Export failure
    mock_stub = MagicMock()
    mock_stub.Export.side_effect = Exception("gRPC failed")
    mock_stub_class.return_value = mock_stub

    args = {
        "body_size": 10,
        "num_attributes": 1,
        "attribute_value_size": 5,
        "batch_size": 2,
        "threads": 1,
        "target_rate": None,
    }

    generator.stop_event.clear()
    thread = threading.Thread(target=generator.worker_thread, args=(0, args))
    thread.start()

    time.sleep(0.2)
    generator.stop_event.set()
    thread.join()

    assert generator.metrics["failed"] >= 2


@patch("loadgen.grpc.insecure_channel")
@patch("loadgen.logs_service_pb2_grpc.LogsServiceStub")
def test_worker_thread_rate_limiting_and_late_batches(mock_stub_class, mock_channel):
    generator = LoadGenerator()

    # Mock stub with delay to simulate late sending
    def slow_export(request):
        time.sleep(0.3)  # delay > interval causes batch to be late
        return None

    mock_stub = MagicMock()
    mock_stub.Export.side_effect = slow_export
    mock_stub_class.return_value = mock_stub

    args = {
        "body_size": 5,
        "num_attributes": 1,
        "attribute_value_size": 5,
        "batch_size": 2,
        "threads": 1,
        "target_rate": 10,  # 10 logs/sec => batch every 0.2s
    }

    generator.stop_event.clear()
    thread = threading.Thread(target=generator.worker_thread, args=(0, args))
    thread.start()

    # Wait for a few iterations
    time.sleep(0.7)
    generator.stop_event.set()
    thread.join()

    # Because slow_export takes longer than allowed interval,
    # late_batches should increase
    assert generator.metrics["late_batches"] > 0
    assert generator.metrics["sent"] >= 2
