import pytest
import types
from unittest.mock import MagicMock

from lib.core.component import Component, ComponentPhase
from lib.core.context import StepContext
from lib.core.framework.step import StepAction
from lib.impl.actions.component_action import (
    ComponentAction,
    ComponentActionConfig,
    ACTION_NAME,
)
from lib.runner.registry import step_action_registry


def test_component_action_is_registered():
    # Confirm action and config are registered
    assert ACTION_NAME in step_action_registry.element
    assert ACTION_NAME in step_action_registry.config

    assert step_action_registry.element[ACTION_NAME] is ComponentAction
    assert step_action_registry.config[ACTION_NAME] is ComponentActionConfig


def test_component_action_instantiation():
    config = ComponentActionConfig(target="load-generator", phase=ComponentPhase.START)
    action = ComponentAction(config)
    assert isinstance(action, StepAction)
    assert action.config.target == "load-generator"
    assert action.config.phase == ComponentPhase.START


def test_component_action_execute_calls_phase_method():
    # Dummy method with tracking
    def dummy_start(self, ctx):
        dummy_start.called = True
        dummy_start.called_with = ctx

    dummy_start.called = False
    dummy_start.called_with = None

    # Component with real method (not a mock)
    mock_component = MagicMock(spec=Component)
    mock_component.start = types.MethodType(dummy_start, mock_component)

    # Context
    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_component_by_name.return_value = mock_component

    config = ComponentActionConfig(target="load-generator", phase=ComponentPhase.START)
    action = ComponentAction(config)

    action.execute(mock_ctx)

    # Assertions
    assert dummy_start.called is True
    assert dummy_start.called_with == mock_ctx
    mock_ctx.set_step_component.assert_called_once_with(mock_component)


def test_component_action_raises_if_component_missing_method():
    mock_component = MagicMock(spec=Component)
    # Explicitly ensure it doesn't have a 'stop' attribute
    if hasattr(mock_component, "stop"):
        delattr(mock_component, "stop")

    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_component_by_name.return_value = mock_component

    config = ComponentActionConfig(target="foo", phase=ComponentPhase.STOP)
    action = ComponentAction(config)

    with pytest.raises(ValueError, match="does not have action stop"):
        action.execute(mock_ctx)


def test_component_action_raises_if_component_invalid():
    # Component is not of type Component
    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_component_by_name.return_value = object()  # Invalid type

    config = ComponentActionConfig(target="foo", phase=ComponentPhase.START)
    action = ComponentAction(config)

    with pytest.raises(AssertionError, match="Expected Component not found"):
        action.execute(mock_ctx)


def test_component_action_plugin_meta():
    assert ComponentAction.PLUGIN_META.yaml_example
    assert StepContext.__name__ in ComponentAction.PLUGIN_META.supported_contexts
