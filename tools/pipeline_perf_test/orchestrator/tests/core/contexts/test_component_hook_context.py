from unittest.mock import MagicMock
from types import SimpleNamespace

from lib.core.context.component_hook_context import ComponentHookContext


def test_component_hook_context_post_init_merges_metadata():
    parent_component = MagicMock()
    parent_component.name = "MyComponent"

    parent_ctx = MagicMock()
    parent_ctx.metadata = {"parent": "value"}
    parent_ctx.step = SimpleNamespace(component=parent_component)

    ctx = ComponentHookContext(
        name="hook1",
        metadata={"local": "meta"},
        parent_ctx=parent_ctx,
        phase=MagicMock(value="BEFORE"),
    )

    # Event types set
    assert ctx.start_event_type.name == "HOOK_START"
    assert ctx.end_event_type.name == "HOOK_END"

    # Metadata is merged with correct component name and phase
    assert ctx.metadata["parent"] == "value"
    assert ctx.metadata["local"] == "meta"
    assert ctx.metadata["test.ctx.component"] == "MyComponent"
    assert ctx.metadata["test.ctx.phase"] == "BEFORE"

    # Span name is derived from name
    assert ctx.span_name == "Run Component Hook: hook1"


def test_component_hook_context_post_init_no_parent_ctx():
    ctx = ComponentHookContext(
        name="hook2",
        metadata={"only": "local"},
        parent_ctx=None,
        phase=MagicMock(value="AFTER"),
    )

    assert ctx.metadata == {
        "only": "local",
        "test.ctx.phase": "AFTER",
        "test.ctx.name": "hook2",
        "test.ctx.type": "ComponentHookContext",
    }
    assert ctx.span_name == "Run Component Hook: hook2"


def test_get_step_component_returns_component():
    component = MagicMock()
    parent_ctx = MagicMock()
    parent_ctx.step.component = component

    ctx = ComponentHookContext(
        name="hook", phase=MagicMock(value="PHASE"), parent_ctx=parent_ctx
    )
    assert ctx.get_step_component() is component


def test_get_step_component_raises_without_parent_ctx():
    ctx = ComponentHookContext(
        name="hook", phase=MagicMock(value="PHASE"), parent_ctx=None
    )
    try:
        ctx.get_step_component()
    except RuntimeError as e:
        assert "parent_ctx must be set" in str(e)
    else:
        assert False, "Expected RuntimeError not raised"
