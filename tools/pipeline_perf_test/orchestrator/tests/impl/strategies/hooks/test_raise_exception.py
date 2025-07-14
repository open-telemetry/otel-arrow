import pytest
from unittest.mock import MagicMock

from lib.impl.strategies.hooks.raise_exception import (
    RaiseExceptionHook,
    RaiseExceptionConfig,
)


def test_raise_exception_hook_raises_runtime_error():
    # Given
    message = "Intentional test failure"
    config = RaiseExceptionConfig(message=message)
    hook = RaiseExceptionHook(config=config)

    # Mock context with logger
    mock_ctx = MagicMock()
    mock_logger = MagicMock()
    mock_ctx.get_logger.return_value = mock_logger

    # When / Then
    with pytest.raises(RuntimeError) as exc_info:
        hook.execute(mock_ctx)

    # Assert the exception message
    assert str(exc_info.value) == message

    # Assert logger was called with debug
    mock_logger.debug.assert_called_once_with(
        f"Raising Exception per configuration: {message}"
    )


def test_raise_exception_hook_with_empty_message():
    config = RaiseExceptionConfig(message="")
    hook = RaiseExceptionHook(config=config)

    mock_ctx = MagicMock()
    mock_logger = MagicMock()
    mock_ctx.get_logger.return_value = mock_logger

    with pytest.raises(RuntimeError) as exc_info:
        hook.execute(mock_ctx)

    assert str(exc_info.value) == ""
    mock_logger.debug.assert_called_once_with("Raising Exception per configuration: ")
