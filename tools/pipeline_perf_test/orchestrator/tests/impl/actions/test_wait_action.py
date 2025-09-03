import pytest
from unittest.mock import MagicMock, patch

from lib.core.context import StepContext
from lib.core.framework.step import StepAction
from lib.impl.actions.wait_action import WaitAction, WaitActionConfig, ACTION_NAME
from lib.runner.registry import step_action_registry


def test_wait_action_is_registered():
    # Ensure the WaitAction and its config are registered correctly
    assert ACTION_NAME in step_action_registry.element
    assert ACTION_NAME in step_action_registry.config

    assert step_action_registry.element[ACTION_NAME] is WaitAction
    assert step_action_registry.config[ACTION_NAME] is WaitActionConfig


def test_wait_action_instantiation():
    # Ensure WaitAction can be instantiated with a config
    config = WaitActionConfig(delay_seconds=1.5)
    action = WaitAction(config)
    assert isinstance(action, StepAction)
    assert action.config.delay_seconds == 1.5


@patch("lib.impl.actions.wait_action.time.sleep", autospec=True)
def test_wait_action_execute_calls_sleep(mock_sleep):
    # Ensure that time.sleep is called with the correct delay
    config = WaitActionConfig(delay_seconds=2.0)
    action = WaitAction(config)

    mock_ctx = MagicMock(spec=StepContext)
    action.execute(mock_ctx)

    mock_sleep.assert_called_once_with(2.0)


def test_wait_action_plugin_meta():
    # Validate the metadata structure is intact
    assert WaitAction.PLUGIN_META.yaml_example
    assert StepContext.__name__ in WaitAction.PLUGIN_META.supported_contexts
