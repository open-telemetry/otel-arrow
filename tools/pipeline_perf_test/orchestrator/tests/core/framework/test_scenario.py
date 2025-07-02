import pytest

from unittest.mock import patch
from lib.core.framework.element import HookableTestPhase
from lib.core.framework import Scenario
from lib.core.context.base import ExecutionStatus
from lib.core.context.framework_element_contexts import ScenarioContext


def test_scenario_runs_all_steps_in_order(fake_test_step_factory, fake_test_suite):
    # Mock steps
    step1 = fake_test_step_factory()
    step1.name = "Step 1"
    step2 = fake_test_step_factory()
    step2.name = "Step 2"

    # Patch StepContext and required constants
    scenario = Scenario(name="Test Scenario", steps=[step1, step2])

    ctx = ScenarioContext(
        name="Dummy", scenario_definition=scenario, parent_ctx=fake_test_suite.context
    )
    scenario.run(ctx)

    # Check steps run
    step1.action.execute.assert_called_once()
    step2.action.execute.assert_called_once()

    # Step context added
    assert len(ctx.child_contexts) == 2

    # Status should be set to SUCCESS if still RUNNING
    for child in ctx.child_contexts:
        assert child.status == ExecutionStatus.SUCCESS
    assert ctx.child_contexts[0].get_framework_element() == step1
    assert ctx.child_contexts[1].get_framework_element() == step2
    assert ctx.child_contexts[0].start_time < ctx.child_contexts[1].start_time


def test_step_raises_exception_and_is_handled(
    fake_test_step_factory, fake_test_suite, caplog
):
    failing_step = fake_test_step_factory()
    failing_step.name = "Failing Step"
    failing_step.action.execute.side_effect = RuntimeError("Boom")

    scenario = Scenario(name="Test", steps=[failing_step])
    ctx = ScenarioContext(
        name="TestCtx", scenario_definition=scenario, parent_ctx=fake_test_suite.context
    )

    with pytest.raises(RuntimeError):
        scenario.run(ctx)

    assert ctx.child_contexts[0].status == ExecutionStatus.ERROR
    assert isinstance(ctx.child_contexts[0].error, RuntimeError)
    assert "Boom" in caplog.text


def test_context_type_assertion():
    from lib.core.framework import Scenario

    scenario = Scenario(name="InvalidCtxScenario", steps=[])

    class DummyCtx:
        def get_logger(self, *_):
            return lambda *a, **kw: None  # dummy logger

    with pytest.raises(AssertionError, match="Expected ScenarioContext"):
        scenario.run(DummyCtx())


def test_hooks_are_called(fake_test_step_factory, fake_test_suite):
    step = fake_test_step_factory()
    scenario = Scenario(name="HookTest", steps=[step])
    ctx = ScenarioContext(
        name="ctx", scenario_definition=scenario, parent_ctx=fake_test_suite.context
    )

    with patch.object(scenario, "_run_hooks") as mock_hooks:
        scenario.run(ctx)

    mock_hooks.assert_any_call(HookableTestPhase.PRE_RUN, ctx)
    mock_hooks.assert_any_call(HookableTestPhase.POST_RUN, ctx)
    assert mock_hooks.call_count == 2


def test_step_status_defaults_to_success(fake_test_step_factory, fake_test_suite):
    step = fake_test_step_factory()
    step.name = "NoStatusStep"

    # step.run does not set step_ctx.status
    scenario = Scenario(name="StatusDefault", steps=[step])
    ctx = ScenarioContext(
        name="ctx", scenario_definition=scenario, parent_ctx=fake_test_suite.context
    )

    scenario.run(ctx)

    assert ctx.child_contexts[0].status == ExecutionStatus.SUCCESS


def test_step_context_is_used(fake_test_step_factory, fake_test_suite):
    step = fake_test_step_factory()
    scenario = Scenario(name="CtxTest", steps=[step])
    ctx = ScenarioContext(
        name="ctx", scenario_definition=scenario, parent_ctx=fake_test_suite.context
    )

    with patch("lib.core.context.StepContext.__enter__") as enter_mock, patch(
        "lib.core.context.StepContext.__exit__"
    ) as exit_mock:
        scenario.run(ctx)

    enter_mock.assert_called()
    exit_mock.assert_called()
