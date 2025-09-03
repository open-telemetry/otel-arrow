import pytest
from unittest.mock import Mock, patch

from lib.impl.strategies.hooks.ready_check_http import (
    ReadyCheckHttpHook,
    ReadyCheckHttpConfig,
)

from lib.core.context.base import BaseContext


class DummyContext(BaseContext):
    def get_logger(self, name=None):
        import logging

        logging.basicConfig(level=logging.DEBUG)
        return logging.getLogger(name or __name__)


@pytest.fixture
def base_config():
    return {"url": "https://example.com/health", "method": "GET", "timeout": 1}


@patch("lib.impl.strategies.hooks.ready_check_http.requests.request")
def test_success_on_first_try(mock_request, base_config):
    mock_response = Mock(status_code=200, text="OK")
    mock_request.return_value = mock_response

    config = ReadyCheckHttpConfig(expected_status=200, **base_config)
    hook = ReadyCheckHttpHook(config)

    hook.execute(DummyContext())
    mock_request.assert_called_once()


@patch("lib.impl.strategies.hooks.ready_check_http.requests.request")
def test_eventual_success_after_retries(mock_request, base_config):
    fail_response = Mock(status_code=503, text="Unavailable")
    success_response = Mock(status_code=200, text="OK")
    mock_request.side_effect = [fail_response, fail_response, success_response]

    config = ReadyCheckHttpConfig(
        expected_status=200, max_retries=5, retry_interval=0.01, **base_config
    )
    hook = ReadyCheckHttpHook(config)

    hook.execute(DummyContext())
    assert mock_request.call_count == 3


@patch("lib.impl.strategies.hooks.ready_check_http.requests.request")
def test_failure_after_max_retries(mock_request, base_config):
    mock_request.return_value = Mock(status_code=503)

    config = ReadyCheckHttpConfig(
        expected_status=200, max_retries=3, retry_interval=0.01, **base_config
    )
    hook = ReadyCheckHttpHook(config)

    with pytest.raises(RuntimeError, match="Ready check failed"):
        hook.execute(DummyContext())

    assert mock_request.call_count == 3


@patch("lib.impl.strategies.hooks.ready_check_http.requests.request")
def test_json_field_value_match_success(mock_request, base_config):
    mock_response = Mock(status_code=200)
    mock_response.json.return_value = {"status": "ready"}
    mock_request.return_value = mock_response

    config = ReadyCheckHttpConfig(
        expected_status=200,
        expected_json_field="status",
        expected_json_value="ready",
        **base_config
    )
    hook = ReadyCheckHttpHook(config)
    hook.execute(DummyContext())


@patch("lib.impl.strategies.hooks.ready_check_http.requests.request")
def test_json_field_value_mismatch(mock_request, base_config):
    mock_response = Mock(status_code=200)
    mock_response.json.return_value = {"status": "starting"}
    mock_request.return_value = mock_response

    config = ReadyCheckHttpConfig(
        expected_status=200,
        expected_json_field="status",
        expected_json_value="ready",
        max_retries=2,
        retry_interval=0.01,
        **base_config
    )
    hook = ReadyCheckHttpHook(config)
    with pytest.raises(RuntimeError):
        hook.execute(DummyContext())


@patch("lib.impl.strategies.hooks.ready_check_http.requests.request")
def test_text_substring_match(mock_request, base_config):
    mock_response = Mock(status_code=200, text="Service is ready")
    mock_response.json.return_value = {}
    mock_request.return_value = mock_response

    config = ReadyCheckHttpConfig(
        expected_status=200, expected_text_substring="ready", **base_config
    )
    hook = ReadyCheckHttpHook(config)
    hook.execute(DummyContext())


@patch("lib.impl.strategies.hooks.ready_check_http.requests.request")
def test_invalid_json_does_not_crash(mock_request, base_config):
    mock_response = Mock(status_code=200, text="Not JSON")
    mock_response.json.side_effect = ValueError("Invalid JSON")
    mock_request.return_value = mock_response

    config = ReadyCheckHttpConfig(
        expected_status=200,
        expected_json_field="status",
        expected_json_value="ready",
        max_retries=2,
        retry_interval=0.01,
        **base_config
    )
    hook = ReadyCheckHttpHook(config)
    with pytest.raises(RuntimeError):
        hook.execute(DummyContext())


@patch("lib.impl.strategies.hooks.ready_check_http.requests.request")
def test_all_conditions_must_pass(mock_request, base_config):
    mock_response = Mock(status_code=200, text="Healthy")
    mock_response.json.return_value = {"status": "not-ready"}
    mock_request.return_value = mock_response

    config = ReadyCheckHttpConfig(
        expected_status=200,
        expected_json_field="status",
        expected_json_value="ready",
        expected_text_substring="Healthy",
        max_retries=2,
        retry_interval=0.01,
        **base_config
    )
    hook = ReadyCheckHttpHook(config)
    with pytest.raises(RuntimeError):
        hook.execute(DummyContext())
