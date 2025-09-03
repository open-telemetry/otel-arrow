import pytest
from unittest.mock import MagicMock

from lib.core.context import StepContext
from lib.core.framework.step import StepAction
from lib.impl.actions.no_op_action import NoOpAction, NoOpActionConfig, ACTION_NAME
from lib.runner.registry import step_action_registry


def test_noop_action_is_registered():
    # Ensure the NoOpAction and its config are registered correctly
    assert ACTION_NAME in step_action_registry.element
    assert ACTION_NAME in step_action_registry.config

    assert step_action_registry.element[ACTION_NAME] is NoOpAction
    assert step_action_registry.config[ACTION_NAME] is NoOpActionConfig


def test_noop_action_instantiation():
    # Ensure NoOpAction can be instantiated without error
    config = NoOpActionConfig()
    action = NoOpAction(config)
    assert isinstance(action, StepAction)
    assert action.config == config


def test_noop_action_execute_does_nothing():
    # Execute the action and assert no exceptions or side effects
    config = NoOpActionConfig()
    action = NoOpAction(config)

    mock_ctx = MagicMock(spec=StepContext)
    action.execute(mock_ctx)  # Should not raise or return anything


def test_noop_action_plugin_meta():
    # Validate the metadata structure is intact
    assert NoOpAction.PLUGIN_META.yaml_example
    assert StepContext.__name__ in NoOpAction.PLUGIN_META.supported_contexts
