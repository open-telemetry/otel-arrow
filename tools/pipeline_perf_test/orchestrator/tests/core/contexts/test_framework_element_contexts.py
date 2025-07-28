from typing import Dict, Optional
from dataclasses import dataclass

import pytest
from unittest.mock import MagicMock

from lib.core.context.framework_element_contexts import (
    FrameworkElementContext,
    SuiteContext,
    FrameworkEvent,
    ScenarioContext,
    StepContext,
)


class DummyContext(FrameworkElementContext):
    def get_framework_element(self):
        return None


@dataclass
class DummyParentContext:
    metadata: Dict[str, str]


@dataclass
class DummyScenario:
    name: str


class DummyComponent:
    pass


class DummySuite:
    pass


@dataclass
class DummyStep:
    name: str
    component: Optional[DummyComponent] = None

    def set_component(self, component: DummyComponent):
        self.component = component


# Base Framework Element Cases
def test_get_suite_delegates_to_parent():
    parent = MagicMock()
    context = DummyContext()
    context.parent_ctx = parent
    context.get_suite()
    parent.get_suite.assert_called_once()


def test_get_suite_raises_without_parent():
    context = DummyContext()
    context.parent_ctx = None
    with pytest.raises(RuntimeError, match="SuiteContext.suite must be set"):
        context.get_suite()


def test_get_component_by_name_delegates():
    parent = MagicMock()
    context = DummyContext()
    context.parent_ctx = parent
    name = "component_x"
    context.get_component_by_name(name)
    parent.get_component_by_name.assert_called_once_with(name)


def test_get_component_by_name_raises_without_parent():
    context = DummyContext()
    context.parent_ctx = None
    with pytest.raises(RuntimeError, match="Can't get parent context"):
        context.get_component_by_name("my_component")


# Suite Test Cases
def test_post_init_sets_metadata_and_span_name():
    context = SuiteContext(name="MySuite")
    assert context.start_event_type == FrameworkEvent.SUITE_START
    assert context.end_event_type == FrameworkEvent.SUITE_END
    assert context.metadata["test.suite"] == "MySuite"
    assert context.span_name == "Run Test Suite: MySuite"


def test_get_framework_element_returns_suite():
    suite = DummySuite()
    context = SuiteContext(name="suite", suite=suite)
    assert context.get_framework_element() is suite


def test_add_and_get_component():
    component = DummyComponent()
    context = SuiteContext(name="suite")
    context.add_component("my_component", component)
    assert context.get_components()["my_component"] is component
    assert context.get_component_by_name("my_component") is component


def test_get_component_by_name_returns_none_if_missing():
    context = SuiteContext(name="suite")
    assert context.get_component_by_name("missing") is None


def test_get_suite_returns_suite():
    suite = DummySuite()
    context = SuiteContext(name="suite", suite=suite)
    assert context.get_suite() is suite


def test_get_suite_raises_if_missing():
    context = SuiteContext(name="suite", suite=None)
    with pytest.raises(RuntimeError, match="SuiteContext.suite must be set"):
        context.get_suite()


# Scenario Cases
def test_post_init_sets_event_types_and_span_name():
    scenario = DummyScenario(name="MyScenario")
    ctx = ScenarioContext(name="TestCtx", scenario_definition=scenario)
    ctx.metadata = {}
    ctx.__post_init__()

    assert ctx.start_event_type == FrameworkEvent.TEST_START
    assert ctx.end_event_type == FrameworkEvent.TEST_END
    assert ctx.metadata["test.name"] == "MyScenario"
    assert ctx.span_name == "Run Test: MyScenario"


def test_post_init_merges_metadata_from_parent():
    parent = DummyParentContext(metadata={"env": "staging", "team": "QA"})
    scenario = DummyScenario(name="AuthScenario")
    ctx = ScenarioContext(
        name="ChildCtx", scenario_definition=scenario, parent_ctx=parent
    )
    ctx.metadata = {"priority": "high"}
    ctx.__post_init__()

    assert ctx.metadata["env"] == "staging"
    assert ctx.metadata["team"] == "QA"
    assert ctx.metadata["priority"] == "high"
    assert ctx.metadata["test.name"] == "AuthScenario"


def test_get_framework_element_returns_scenario_definition():
    scenario = DummyScenario(name="ScenarioX")
    ctx = ScenarioContext(name="Ctx", scenario_definition=scenario)
    assert ctx.get_framework_element() == scenario


# Step Cases
def test_post_init_sets_event_and_metadata():
    step = DummyStep(name="Login Step")
    ctx = StepContext(name="step_ctx", step=step)
    ctx.metadata = {}
    ctx.__post_init__()

    assert ctx.start_event_type == FrameworkEvent.STEP_START
    assert ctx.end_event_type == FrameworkEvent.STEP_END
    assert ctx.metadata["test.step"] == "Login Step"
    assert ctx.span_name == "Run Test Step: Login Step"


def test_post_init_merges_metadata_from_parent():
    parent = DummyParentContext(metadata={"env": "dev", "test.name": "LoginTest"})
    step = DummyStep(name="Step 1")
    ctx = StepContext(name="step_ctx", parent_ctx=parent, step=step)
    ctx.metadata = {"custom": "value"}
    ctx.__post_init__()

    assert ctx.metadata["env"] == "dev"
    assert ctx.metadata["test.name"] == "LoginTest"
    assert ctx.metadata["custom"] == "value"
    assert ctx.metadata["test.step"] == "Step 1"


def test_get_framework_element_returns_step():
    step = DummyStep(name="S1")
    ctx = StepContext(name="ctx", step=step)
    assert ctx.get_framework_element() is step


def test_get_step_component_returns_correct_component():
    component = DummyComponent()
    step = DummyStep(name="S2", component=component)
    ctx = StepContext(name="ctx", step=step)
    assert ctx.get_step_component() is component


def test_set_step_component_updates_component():
    component = DummyComponent()
    step = DummyStep(name="S3")
    ctx = StepContext(name="ctx", step=step)
    ctx.set_step_component(component)
    assert step.component is component
