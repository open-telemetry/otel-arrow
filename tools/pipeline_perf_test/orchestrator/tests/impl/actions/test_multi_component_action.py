import types
import pytest
from unittest.mock import MagicMock

from lib.core.component import Component, ComponentPhase
from lib.core.context import StepContext
from lib.core.framework.step import StepAction
from lib.impl.actions.multi_component_action import (
    ACTION_NAME,
    MultiComponentAction,
    MultiComponentActionConfig,
)
from lib.runner.registry import step_action_registry


def test_multi_component_action_is_registered():
    assert ACTION_NAME in step_action_registry.element
    assert ACTION_NAME in step_action_registry.config
    assert step_action_registry.element[ACTION_NAME] is MultiComponentAction
    assert step_action_registry.config[ACTION_NAME] is MultiComponentActionConfig


def test_multi_component_action_instantiation():
    config = MultiComponentActionConfig(phase=ComponentPhase.START, targets=["comp1"])
    action = MultiComponentAction(config)
    assert isinstance(action, StepAction)
    assert action.config.targets == ["comp1"]
    assert action.config.phase == ComponentPhase.START


def test_execute_with_specified_targets_calls_correct_methods():
    # Dummy lifecycle method with tracking
    def start(self, ctx):
        start.called[self.name] = ctx

    start.called = {}

    comp1 = MagicMock(spec=Component)
    comp1.name = "comp1"
    comp1.start = types.MethodType(start, comp1)

    comp2 = MagicMock(spec=Component)
    comp2.name = "comp2"
    comp2.start = types.MethodType(start, comp2)

    # Context returns these components by name
    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_component_by_name.side_effect = lambda name: {
        "comp1": comp1,
        "comp2": comp2,
    }[name]

    config = MultiComponentActionConfig(
        phase=ComponentPhase.START, targets=["comp1", "comp2"]
    )
    action = MultiComponentAction(config)

    action.execute(mock_ctx)

    assert start.called["comp1"] == mock_ctx
    assert start.called["comp2"] == mock_ctx

    mock_ctx.set_step_component.assert_any_call(comp1)
    mock_ctx.set_step_component.assert_any_call(comp2)


def test_execute_with_all_components_calls_correct_methods():
    # Dummy method
    def start(self, ctx):
        start.called[self.name] = ctx

    start.called = {}

    comp1 = MagicMock(spec=Component)
    comp1.name = "comp1"
    comp1.start = types.MethodType(start, comp1)

    comp2 = MagicMock(spec=Component)
    comp2.name = "comp2"
    comp2.start = types.MethodType(start, comp2)

    # Context provides all components
    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_components.return_value = {"comp1": comp1, "comp2": comp2}

    config = MultiComponentActionConfig(phase=ComponentPhase.START, targets=None)
    action = MultiComponentAction(config)
    action.execute(mock_ctx)

    assert start.called["comp1"] == mock_ctx
    assert start.called["comp2"] == mock_ctx


def test_execute_raises_if_component_missing_method():
    comp = MagicMock(spec=Component)
    comp.name = "comp1"
    # Ensure 'stop' is not defined
    if hasattr(comp, "stop"):
        delattr(comp, "stop")

    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_component_by_name.return_value = comp

    config = MultiComponentActionConfig(phase=ComponentPhase.STOP, targets=["comp1"])
    action = MultiComponentAction(config)

    with pytest.raises(ValueError, match="does not have action stop"):
        action.execute(mock_ctx)


def test_execute_raises_if_target_not_a_component():
    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_component_by_name.return_value = object()  # Not a Component

    config = MultiComponentActionConfig(
        phase=ComponentPhase.START, targets=["bad-comp"]
    )
    action = MultiComponentAction(config)

    with pytest.raises(AssertionError, match="Expected Component not found"):
        action.execute(mock_ctx)


def test_multi_component_action_plugin_meta():
    assert MultiComponentAction.PLUGIN_META.yaml_example
    assert StepContext.__name__ in MultiComponentAction.PLUGIN_META.supported_contexts
