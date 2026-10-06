from unittest.mock import Mock

import pytest

from lib.core.context.base import BaseContext, ExecutionStatus
from lib.core.context.framework_element_contexts import SuiteContext
from lib.core.framework import Scenario, Step, Suite
from lib.core.framework.element import HookableTestPhase
from lib.core.strategies.hook_strategy import HookStrategy, HookStrategyConfig


class _RecordingHook(HookStrategy):
    """Hook that records whether it ran and invokes an optional callback."""

    def __init__(self, on_execute=None):
        self.config = HookStrategyConfig()
        self.was_called = False
        self.on_execute = on_execute

    def execute(self, ctx: BaseContext):
        self.was_called = True
        if self.on_execute:
            self.on_execute(ctx)


def _suite_with_test(test: Scenario) -> Suite:
    suite = Suite(name="Suite", tests=[test], components={})
    suite.context = SuiteContext(name="SuiteCtx", suite=suite)
    return suite


# Scenario: A suite's single test fails after suite-level PRE_RUN hooks have run
#   and a POST_RUN (teardown) hook is registered.
# Guarantees: The POST_RUN hooks still execute despite the test failure (so a
#   receiver/resource started in PRE_RUN is torn down), and the original test
#   error -- not a cleanup outcome -- propagates to the caller.
def test_post_run_hooks_execute_when_a_test_fails():
    failing_action = Mock()
    failing_action.execute.side_effect = RuntimeError("Boom")
    failing_step = Step(name="FailingStep", action=failing_action)
    test = Scenario(name="FailingTest", steps=[failing_step])

    suite = _suite_with_test(test)
    post_hook = _RecordingHook()
    suite.add_hook(HookableTestPhase.POST_RUN, post_hook)

    with pytest.raises(RuntimeError, match="Boom"):
        suite.run()

    assert post_hook.was_called
    assert suite.context.status == ExecutionStatus.ERROR


# Scenario: A suite whose test fails has both a PRE_RUN and a POST_RUN hook.
# Guarantees: The PRE_RUN hook runs before the failing test and the POST_RUN
#   hook still runs afterward, confirming teardown is paired with setup even on
#   the failure path.
def test_pre_and_post_run_hooks_both_execute_on_failure():
    failing_action = Mock()
    failing_action.execute.side_effect = RuntimeError("Boom")
    test = Scenario(
        name="FailingTest", steps=[Step(name="FailingStep", action=failing_action)]
    )

    suite = _suite_with_test(test)
    order = []
    pre_hook = _RecordingHook(on_execute=lambda _ctx: order.append("pre"))
    post_hook = _RecordingHook(on_execute=lambda _ctx: order.append("post"))
    suite.add_hook(HookableTestPhase.PRE_RUN, pre_hook)
    suite.add_hook(HookableTestPhase.POST_RUN, post_hook)

    with pytest.raises(RuntimeError, match="Boom"):
        suite.run()

    assert order == ["pre", "post"]


# Scenario: Every test in a suite passes with a POST_RUN hook registered.
# Guarantees: The POST_RUN hooks run and the suite is marked SUCCESS on the
#   normal (non-error) path, so the try/finally teardown change does not alter
#   success behavior.
def test_post_run_hooks_execute_and_suite_succeeds_on_success():
    test = Scenario(name="OkTest", steps=[Step(name="OkStep", action=Mock())])

    suite = _suite_with_test(test)
    post_hook = _RecordingHook()
    suite.add_hook(HookableTestPhase.POST_RUN, post_hook)

    suite.run()

    assert post_hook.was_called
    assert suite.context.status == ExecutionStatus.SUCCESS


# Scenario: A suite-level PRE_RUN hook raises before any test runs, with a
#   POST_RUN hook registered.
# Guarantees: When setup itself fails, POST_RUN hooks are not run (nothing was
#   set up to tear down) and the PRE_RUN error propagates.
def test_post_run_hooks_skipped_when_pre_run_fails():
    suite = _suite_with_test(
        Scenario(name="OkTest", steps=[Step(name="OkStep", action=Mock())])
    )

    def _boom(_ctx):
        raise RuntimeError("PreBoom")

    pre_hook = _RecordingHook(on_execute=_boom)
    post_hook = _RecordingHook()
    suite.add_hook(HookableTestPhase.PRE_RUN, pre_hook)
    suite.add_hook(HookableTestPhase.POST_RUN, post_hook)

    with pytest.raises(RuntimeError, match="PreBoom"):
        suite.run()

    assert not post_hook.was_called
