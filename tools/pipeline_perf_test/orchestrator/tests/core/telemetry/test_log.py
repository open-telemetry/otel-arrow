import logging
import pytest
from unittest.mock import MagicMock, patch
from lib.core.telemetry.log import (
    SpanAwareLogHandler,
)  # replace with actual module name


@pytest.fixture
def log_record():
    record = logging.LogRecord(
        name="test_logger",
        level=logging.INFO,
        pathname=__file__,
        lineno=10,
        msg="This is a test log",
        args=(),
        exc_info=None,
    )
    record.__dict__["test.custom_key"] = "custom_value"
    return record


def test_emit_with_active_recording_span(log_record):
    mock_span = MagicMock()
    mock_span.is_recording.return_value = True

    with patch("lib.core.telemetry.log.get_current_span", return_value=mock_span):
        handler = SpanAwareLogHandler()
        handler.emit(log_record)

        mock_span.add_event.assert_called_once()
        event_name, attributes = (
            mock_span.add_event.call_args[1]["name"],
            mock_span.add_event.call_args[1]["attributes"],
        )

        assert event_name == "log"
        assert attributes["log.severity"] == "INFO"
        assert attributes["log.message"] == "This is a test log"
        assert attributes["logger.name"] == "test_logger"
        assert attributes["test.custom_key"] == "custom_value"


def test_emit_with_no_span(log_record):
    with patch("lib.core.telemetry.log.get_current_span", return_value=None):
        handler = SpanAwareLogHandler()
        # Should not raise
        handler.emit(log_record)


def test_emit_with_non_recording_span(log_record):
    mock_span = MagicMock()
    mock_span.is_recording.return_value = False

    with patch("lib.core.telemetry.log.get_current_span", return_value=mock_span):
        handler = SpanAwareLogHandler()
        handler.emit(log_record)
        mock_span.add_event.assert_not_called()
