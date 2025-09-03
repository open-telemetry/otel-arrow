import pytest
from unittest.mock import MagicMock, patch

from pydantic import BaseModel

from lib.core.context import StepContext
from lib.core.framework.step import StepAction
from lib.impl.actions.update_component_strategy import (
    UpdateComponentStrategyAction,
    UpdateComponentStrategyConfig,
    update_model,
    ACTION_NAME,
)
from lib.runner.registry import step_action_registry
from lib.impl.component.managed_component import ManagedComponent
from lib.runner.wrappers import ConfigurableWrapper


def test_update_component_strategy_is_registered():
    # Ensure action and config are registered
    assert ACTION_NAME in step_action_registry.element
    assert ACTION_NAME in step_action_registry.config

    assert step_action_registry.element[ACTION_NAME] is UpdateComponentStrategyAction
    assert step_action_registry.config[ACTION_NAME] is UpdateComponentStrategyConfig


def test_update_component_strategy_instantiation():
    config = UpdateComponentStrategyConfig(
        target="my-component", deployment={"docker": {"image": "my/image"}}
    )
    action = UpdateComponentStrategyAction(config)
    assert isinstance(action, StepAction)
    assert action.config.target == "my-component"


def make_mock_wrapper(config_dict):
    config = MagicMock()
    config.model_copy.return_value = config  # simulate Pydantic's copy/update
    wrapper = MagicMock(spec=ConfigurableWrapper)
    wrapper.config = config
    wrapper.model_copy.return_value = wrapper
    wrapper.element_type = list(config_dict.keys())[0]
    return wrapper


@patch("lib.impl.actions.update_component_strategy.CompositeMonitoringStrategy")
def test_update_component_strategy_execute_success(mock_monitoring_strategy):
    # Create mock component with strategy wrappers
    mock_component = MagicMock(spec=ManagedComponent)
    mock_config = MagicMock()

    deployment_wrapper = make_mock_wrapper({"docker": {"image": "new"}})
    monitoring_wrapper = make_mock_wrapper({"composite": {"enabled": True}})
    execution_wrapper = make_mock_wrapper({"shell": {"cmd": "run"}})
    configuration_wrapper = make_mock_wrapper({"config": {"opt": "val"}})

    mock_config.deployment = deployment_wrapper
    mock_config.monitoring = monitoring_wrapper
    mock_config.execution = execution_wrapper
    mock_config.configuration = configuration_wrapper

    mock_component.component_config = mock_config
    mock_component.replace_strategy.return_value = True

    # Mock context
    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_component_by_name.return_value = mock_component

    config = UpdateComponentStrategyConfig(
        target="otel-collector",
        deployment={"docker": {"image": "new"}},
        monitoring={"composite": {"enabled": True}},
        execution={"shell": {"cmd": "run"}},
        configuration={"config": {"opt": "val"}},
    )

    action = UpdateComponentStrategyAction(config)
    action.execute(mock_ctx)

    # Assert replace_strategy was called multiple times
    assert mock_component.replace_strategy.call_count >= 1
    mock_ctx.set_step_component.assert_called_once_with(mock_component)


def test_update_component_strategy_raises_on_invalid_component():
    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_component_by_name.return_value = None  # Simulate missing component

    config = UpdateComponentStrategyConfig(target="missing")
    action = UpdateComponentStrategyAction(config)

    with pytest.raises(AssertionError):
        action.execute(mock_ctx)


def test_update_component_strategy_raises_on_no_strategy_updated():
    # Setup mock component
    mock_component = MagicMock(spec=ManagedComponent)
    mock_config = MagicMock()

    deployment_wrapper = make_mock_wrapper({"docker": {"image": "fake"}})
    mock_config.deployment = deployment_wrapper

    mock_component.component_config = mock_config
    mock_component.replace_strategy.return_value = False  # Simulate failure

    mock_ctx = MagicMock(spec=StepContext)
    mock_ctx.get_component_by_name.return_value = mock_component

    config = UpdateComponentStrategyConfig(
        target="otel-collector", deployment={"docker": {"image": "fake"}}
    )

    action = UpdateComponentStrategyAction(config)

    with pytest.raises(RuntimeError, match="Failed to update component"):
        action.execute(mock_ctx)


def test_update_model_merges_config():
    # Step 1: Mock inner config (Pydantic model)
    mock_inner_config = MagicMock(spec=BaseModel)
    mock_inner_config.model_copy.return_value = "updated_inner_config"

    # Step 2: Mock ConfigurableWrapper with element_type and config
    mock_wrapper = MagicMock(spec=ConfigurableWrapper)
    mock_wrapper.config = mock_inner_config
    mock_wrapper.element_type = "dummy"
    mock_wrapper.model_copy.return_value = "updated_wrapper"

    # Step 3: Mock the overall model holding the wrapper
    model = MagicMock()
    model.model_copy.return_value = "updated_model"
    setattr(model, "dummy", mock_wrapper)

    # Step 4: Construct update dict
    update_dict = {"dummy": {"dummy": {"x": 42}}}

    result = update_model(model, update_dict)

    # Verify correct calls
    mock_inner_config.model_copy.assert_called_once_with(update={"x": 42})
    mock_wrapper.model_copy.assert_called_once_with(
        update={"config": "updated_inner_config"}
    )
    model.model_copy.assert_called_once_with(update={"dummy": "updated_wrapper"})

    assert result == "updated_model"


def test_update_component_strategy_plugin_meta():
    assert UpdateComponentStrategyAction.PLUGIN_META.yaml_example
    assert (
        StepContext.__name__
        in UpdateComponentStrategyAction.PLUGIN_META.supported_contexts
    )
