import pytest
from collections import defaultdict
from contextlib import nullcontext
from unittest.mock import Mock, patch, MagicMock
from lib.core.framework import FrameworkElement
from lib.core.strategies.hook_strategy import HookStrategy
from lib.core.context import (
    HookableTestPhase,
    FrameworkElementContext,
    FrameworkElementHookContext,
    ExecutionStatus,
)


class ConcreteFrameworkElement(FrameworkElement):
    def run(self):
        pass


def test_add_hook():
    # Arrange
    element = ConcreteFrameworkElement()
    mock_phase = Mock(spec=HookableTestPhase)
    mock_hook = Mock(spec=HookStrategy)

    # Act
    element.add_hook(mock_phase, mock_hook)

    # Assert
    assert mock_hook in element._hooks[mock_phase]
    assert isinstance(element._hooks, defaultdict)


def test_maybe_trace_with_tracer_and_span():
    # Arrange
    element = ConcreteFrameworkElement()
    mock_ctx = Mock(spec=FrameworkElementContext)
    mock_tracer = Mock()
    mock_span_ctx = Mock()

    mock_ctx.get_tracer.return_value = mock_tracer
    mock_ctx.span = True
    mock_phase = Mock(spec=HookableTestPhase)
    mock_phase.value = "pre_run"

    mock_tracer.start_as_current_span.return_value = mock_span_ctx

    # Act
    result = element._maybe_trace(mock_ctx, "MyElement", mock_phase)

    # Assert
    mock_ctx.get_tracer.assert_called_once_with("test-framework")
    mock_tracer.start_as_current_span.assert_called_once_with("MyElement: pre_run")
    assert result == mock_span_ctx


def test_maybe_trace_without_tracer():
    # Arrange
    element = ConcreteFrameworkElement()
    mock_ctx = Mock(spec=FrameworkElementContext)
    mock_ctx.get_tracer.return_value = None
    mock_ctx.span = True

    mock_phase = Mock(spec=HookableTestPhase)
    mock_phase.value = "pre_run"

    # Act
    result = element._maybe_trace(mock_ctx, "MyElement", mock_phase)

    # Assert
    assert isinstance(result, nullcontext().__class__)


def test_maybe_trace_without_span():
    # Arrange
    element = ConcreteFrameworkElement()
    mock_ctx = Mock(spec=FrameworkElementContext)
    mock_ctx.get_tracer.return_value = Mock()
    mock_ctx.span = False
    mock_ctx.metadata = {}

    mock_phase = Mock(spec=HookableTestPhase)
    mock_phase.value = "post_run"

    # Act
    result = element._maybe_trace(mock_ctx, "ElementName", mock_phase)

    # Assert
    assert isinstance(result, nullcontext().__class__)


def test_run_hooks_no_hooks_does_nothing():
    # Arrange
    element = ConcreteFrameworkElement()
    mock_phase = Mock(spec=HookableTestPhase)
    mock_phase.value = "pre_run"
    mock_ctx = Mock(spec=FrameworkElementContext)

    element._hooks = {}
    element._run_hooks(mock_phase, mock_ctx)


def test_get_or_create_runtime_delegates_to_runtime():
    # Arrange
    element = ConcreteFrameworkElement()
    mock_runtime = Mock()
    element.runtime = mock_runtime

    namespace = "my_namespace"
    factory = Mock()
    expected_value = object()
    mock_runtime.get_or_create.return_value = expected_value

    # Act
    result = element.get_or_create_runtime(namespace, factory)

    # Assert
    mock_runtime.get_or_create.assert_called_once_with(namespace, factory)
    assert result == expected_value
