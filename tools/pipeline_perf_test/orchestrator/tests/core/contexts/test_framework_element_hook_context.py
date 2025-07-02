from unittest.mock import MagicMock

from lib.core.context.framework_element_hook_context import FrameworkElementHookContext


def test_test_element_hook_context_post_init_merges_metadata_and_sets_fields():
    parent_ctx = MagicMock()
    parent_ctx.metadata = {"from_parent": "yes"}

    phase = MagicMock()
    phase.value = "BEFORE"

    ctx = FrameworkElementHookContext(
        name="hook-A",
        metadata={"from_self": "ok"},
        parent_ctx=parent_ctx,
        phase=phase,
    )

    # Metadata should be merged
    assert ctx.metadata["from_parent"] == "yes"
    assert ctx.metadata["from_self"] == "ok"
    assert ctx.metadata["test.ctx.phase"] == "BEFORE"

    # Event types and span name
    assert ctx.start_event_type.name == "HOOK_START"
    assert ctx.end_event_type.name == "HOOK_END"
    assert ctx.span_name == "Run Framework Hook hook-A BEFORE"


def test_test_element_hook_context_post_init_without_parent_ctx():
    phase = MagicMock()
    phase.value = "AFTER"

    ctx = FrameworkElementHookContext(
        name="hook-B",
        metadata={"only": "me"},
        parent_ctx=None,
        phase=phase,
    )

    assert ctx.metadata["only"] == "me"
    assert ctx.metadata["test.ctx.phase"] == "AFTER"
    assert ctx.span_name == "Run Framework Hook hook-B AFTER"


def test_get_framework_element_returns_parents_element():
    test_element = MagicMock()
    parent_ctx = MagicMock()
    parent_ctx.get_framework_element.return_value = test_element

    ctx = FrameworkElementHookContext(
        name="hook-C",
        metadata={},
        parent_ctx=parent_ctx,
        phase=MagicMock(value="DURING"),
    )

    assert ctx.get_framework_element() == test_element
    parent_ctx.get_framework_element.assert_called_once()


def test_get_framework_element_returns_none_if_no_parent_ctx():
    ctx = FrameworkElementHookContext(
        name="hook-D",
        metadata={},
        parent_ctx=None,
        phase=MagicMock(value="AFTER"),
    )

    assert ctx.get_framework_element() is None
