import pytest
from unittest.mock import patch, Mock

from lib.impl.strategies.hooks.send_http_request import (
    SendHttpRequestHook,
    SendHttpRequestConfig,
)
from lib.core.context.base import BaseContext


class DummyContext(BaseContext):
    def get_logger(self, name=None):
        import logging

        logging.basicConfig(level=logging.DEBUG)
        return logging.getLogger(name or __name__)


@pytest.fixture
def base_config():
    return {
        "url": "https://example.com/notify",
        "method": "POST",
        "headers": {"Content-Type": "application/json"},
        "payload": {"event": "start"},
        "timeout": 5,
        "raise_for_status": True,
    }


@patch("lib.impl.strategies.hooks.send_http_request.requests.request")
def test_successful_request(mock_request, base_config):
    mock_response = Mock(status_code=200, text="Success")
    mock_request.return_value = mock_response

    config = SendHttpRequestConfig(**base_config)
    hook = SendHttpRequestHook(config)

    hook.execute(DummyContext())
    mock_request.assert_called_once_with(
        method="POST",
        url="https://example.com/notify",
        headers={"Content-Type": "application/json"},
        json={"event": "start"},
        timeout=5,
    )


@patch("lib.impl.strategies.hooks.send_http_request.requests.request")
def test_request_without_headers_or_payload(mock_request):
    mock_response = Mock(status_code=204, text="")
    mock_request.return_value = mock_response

    config = SendHttpRequestConfig(
        url="https://example.com/ping",
        method="GET",
        timeout=3,
    )
    hook = SendHttpRequestHook(config)

    hook.execute(DummyContext())

    mock_request.assert_called_once_with(
        method="GET",
        url="https://example.com/ping",
        headers=None,
        json=None,
        timeout=3,
    )


@patch("lib.impl.strategies.hooks.send_http_request.requests.request")
def test_raises_on_http_error(mock_request, base_config):
    mock_response = Mock(status_code=500, text="Internal Server Error")
    mock_response.raise_for_status.side_effect = Exception("500 Server Error")
    mock_request.return_value = mock_response

    config = SendHttpRequestConfig(**base_config)
    hook = SendHttpRequestHook(config)

    with pytest.raises(Exception, match="500 Server Error"):
        hook.execute(DummyContext())


@patch("lib.impl.strategies.hooks.send_http_request.requests.request")
def test_does_not_raise_when_turned_off(mock_request, base_config):
    mock_response = Mock(status_code=500, text="Internal Server Error")
    mock_response.raise_for_status.side_effect = Exception("Should not raise")
    mock_request.return_value = mock_response

    base_config["raise_for_status"] = False
    config = SendHttpRequestConfig(**base_config)
    hook = SendHttpRequestHook(config)

    # Should NOT raise
    hook.execute(DummyContext())


@patch("lib.impl.strategies.hooks.send_http_request.requests.request")
def test_network_error(mock_request, base_config):
    from requests.exceptions import Timeout

    mock_request.side_effect = Timeout("Request timed out")

    config = SendHttpRequestConfig(**base_config)
    hook = SendHttpRequestHook(config)

    with pytest.raises(Timeout, match="timed out"):
        hook.execute(DummyContext())
