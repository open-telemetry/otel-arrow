import pytest
from unittest.mock import Mock, patch
from lib.core.errors.error_handler import handle_with_policy, ExecutionStatus


@pytest.fixture
def ctx():
    mock_ctx = Mock()
    mock_ctx.get_logger.return_value = Mock()
    mock_ctx.status = None
    mock_ctx.error = None
    mock_ctx.span = None
    return mock_ctx


def test_func_succeeds_first_try(ctx):
    func = Mock(return_value="success")
    on_error = Mock(retries=3, continue_=False, retry_delay_seconds=0)

    result = handle_with_policy(ctx, func, on_error)

    assert result == "success"
    func.assert_called_once()


@patch("lib.core.errors.error_handler.time.sleep", return_value=None)
def test_func_retries_then_succeeds(mock_sleep, ctx):
    func = Mock(side_effect=[Exception("fail"), "success"])
    on_error = Mock(retries=3, continue_=False, retry_delay_seconds=0)

    result = handle_with_policy(ctx, func, on_error)

    assert result == "success"
    assert func.call_count == 2


def test_func_fails_exhausts_retries_and_raises(ctx):
    func = Mock(side_effect=Exception("fatal"))
    on_error = Mock(retries=2, continue_=False, retry_delay_seconds=0)

    with pytest.raises(Exception, match="fatal"):
        handle_with_policy(ctx, func, on_error)

    assert func.call_count == 3


def test_func_fails_continue_true(ctx):
    func = Mock(side_effect=Exception("nonfatal"))
    on_error = Mock(retries=2, continue_=True, retry_delay_seconds=0)

    handle_with_policy(ctx, func, on_error)

    assert ctx.status == ExecutionStatus.ERROR
    assert isinstance(ctx.error, Exception)


def test_span_status_updated(ctx):
    span = Mock()
    ctx.span = span
    func = Mock(side_effect=Exception("oops"))
    on_error = Mock(retries=1, continue_=True, retry_delay_seconds=0)

    handle_with_policy(ctx, func, on_error)

    span.set_status.assert_called_once()
