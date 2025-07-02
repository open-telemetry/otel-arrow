import pytest
from lib.core.component.component import Component
from lib.core.component import ComponentPhase
from lib.core.context.base import ExecutionStatus


def test_component_is_abstract():
    with pytest.raises(TypeError):
        Component()


def test_add_and_run_hook(dummy_component, step_context, dummy_hook):
    phase = ComponentPhase.CONFIGURE
    dummy_component.add_hook(phase, dummy_hook)
    dummy_component._run_hooks(phase, step_context)

    assert dummy_hook.was_called is True


def test_hooks_are_registered_correctly(dummy_component, step_context, dummy_hook):
    phase = ComponentPhase.START
    called = []

    dummy_hook.on_execute = lambda ctx: called.append(ctx)

    dummy_component.add_hook(phase, dummy_hook)
    dummy_component.add_hook(phase, dummy_hook)
    dummy_component.add_hook(phase, dummy_hook)
    dummy_component._run_hooks(phase, step_context)

    assert dummy_hook.was_called is True
    assert called == [
        step_context.child_contexts[0],
        step_context.child_contexts[1],
        step_context.child_contexts[2],
    ]


def test_run_hooks_no_registered_hooks_does_nothing(dummy_component, step_context):
    phase = ComponentPhase.START

    dummy_component._run_hooks(phase, step_context)  # Should not raise or log
    # If needed, check nothing was added to context
    assert step_context.child_contexts == []


def test_hook_success_sets_child_context_success(
    dummy_hook, dummy_component, step_context
):
    dummy_hook.on_execute = lambda ctx: None
    phase = ComponentPhase.START
    dummy_component.add_hook(phase, dummy_hook)

    dummy_component._run_hooks(phase, step_context)

    child_ctx = step_context.child_contexts[0]
    assert child_ctx.status == ExecutionStatus.SUCCESS


def test_incomplete_subclass_raises_typeerror():
    class IncompleteComponent(Component):
        def _configure(self):
            pass

    with pytest.raises(TypeError):
        IncompleteComponent()


def test_hook_exception_sets_context_and_raises(
    dummy_component, step_context, dummy_hook
):
    def faulty(ctx):
        raise RuntimeError("boom")

    dummy_hook.on_execute = faulty
    phase = ComponentPhase.START
    dummy_component.add_hook(phase, dummy_hook)

    with pytest.raises(RuntimeError, match="boom"):
        dummy_component._run_hooks(phase, step_context)

    child_ctx = step_context.child_contexts[0]
    assert child_ctx.status == ExecutionStatus.ERROR
    assert isinstance(child_ctx.error, RuntimeError)


def test_hook_with_ignore_policy_does_not_raise(
    dummy_component, step_context, dummy_hook
):
    def faulty(ctx):
        raise RuntimeError("fail softly")

    dummy_hook.on_execute = faulty
    dummy_hook.config.on_error.continue_ = True

    phase = ComponentPhase.START
    dummy_component.add_hook(phase, dummy_hook)

    dummy_component._run_hooks(phase, step_context)

    child_ctx = step_context.child_contexts[0]
    assert child_ctx.status == ExecutionStatus.ERROR
    assert isinstance(child_ctx.error, RuntimeError)
