import pytest
from unittest.mock import Mock

from lib.core.context.base import BaseContext
from lib.core.context.framework_element_contexts import (
    StepContext,
    SuiteContext,
    ScenarioContext,
)
from lib.core.framework import Step, Scenario, Suite
from lib.core.strategies.hook_strategy import HookStrategy, HookStrategyConfig
from lib.core.component.component import Component


class DummyComponent(Component):
    def _configure(self, ctx):
        pass

    def _deploy(self, ctx):
        pass

    def _start(self, ctx):
        pass

    def _stop(self, ctx):
        pass

    def _destroy(self, ctx):
        pass

    def _start_monitoring(self, ctx):
        pass

    def _stop_monitoring(self, ctx):
        pass

    def _collect_monitoring_data(self, ctx):
        pass


class DummyHookStrategy(HookStrategy):
    def __init__(self, config=None, on_execute=None):
        self.config = config or HookStrategyConfig()
        self.was_called = False
        self.on_execute = on_execute

    def execute(self, ctx: BaseContext):
        self.was_called = True
        if self.on_execute:
            self.on_execute(ctx)


@pytest.fixture
def base_ctx():
    return BaseContext()


@pytest.fixture
def dummy_component():
    return DummyComponent()


@pytest.fixture
def dummy_hook():
    return DummyHookStrategy(HookStrategyConfig())


@pytest.fixture
def fake_test_step():
    return Step(name="FakeTestStep", action=Mock())


@pytest.fixture
def fake_test_step_factory():
    def _create_fake_test_Step():
        return Step(name="FakeTestStep", action=Mock())

    return _create_fake_test_Step


@pytest.fixture
def fake_test(fake_test_step):
    return Scenario(name="FakeTest", steps=[fake_test_step])


@pytest.fixture
def fake_test_suite(fake_test):
    ts = Suite(name="FakeTestSuite", tests=[fake_test], components={})
    ts.context = SuiteContext(name="FakeTestSuiteContext", suite=ts)
    return ts


@pytest.fixture
def step_context(fake_test_step):
    ctx = StepContext(name="FakeTestStepContext", step=fake_test_step)
    return ctx
