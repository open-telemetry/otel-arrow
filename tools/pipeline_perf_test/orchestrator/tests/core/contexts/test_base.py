import logging
import pytest
import datetime

from unittest import mock
from lib.core.component import Component
from lib.core.context.base import BaseContext, ExecutionStatus
from lib.core.telemetry.framework_event import FrameworkEvent
from opentelemetry.trace import StatusCode, Status


class DummySpan:
    pass


class DummyContextManager:
    def __enter__(self):
        return "entered"

    def __exit__(self, exc_type, exc_val, exc_tb):
        pass


def test_base_context_initialization_defaults(base_ctx):
    assert base_ctx.status == ExecutionStatus.PENDING
    assert base_ctx.metadata["test.ctx.type"] == "BaseContext"
    assert base_ctx.metadata["test.ctx.name"] is None
    assert base_ctx.child_contexts == []
    assert base_ctx.start_event_type == FrameworkEvent.SUITE_START
    assert base_ctx.end_event_type == FrameworkEvent.SUITE_END
    assert base_ctx.span is None
    assert base_ctx.span_cm is None
    assert base_ctx.parent_ctx is None


def test_base_context_with_name_and_metadata():
    ctx = BaseContext(name="MyContext", metadata={"custom": "value"})
    assert ctx.name == "MyContext"
    assert ctx.metadata["custom"] == "value"
    assert ctx.metadata["test.ctx.type"] == "BaseContext"
    assert ctx.metadata["test.ctx.name"] == "MyContext"


def test_metadata_preserves_existing_values():
    ctx = BaseContext(
        name="ShouldBeOverridden",
        metadata={"test.ctx.type": "CustomType", "test.ctx.name": "CustomName"},
    )
    assert ctx.metadata["test.ctx.type"] == "CustomType"
    assert ctx.metadata["test.ctx.name"] == "CustomName"


def test_child_contexts_are_not_shared():
    ctx1 = BaseContext()
    ctx2 = BaseContext()
    ctx1.child_contexts.append("child1")
    assert ctx1.child_contexts == ["child1"]
    assert ctx2.child_contexts == []


def test_metadata_are_not_shared():
    ctx1 = BaseContext()
    ctx2 = BaseContext()
    ctx1.metadata["foo"] = "bar"
    assert "foo" not in ctx2.metadata


def test_can_initialize_with_all_optional_fields():
    now = datetime.datetime.now(tz=datetime.UTC)
    error = ValueError("Test error")
    ctx = BaseContext(
        name="Test",
        status=ExecutionStatus.FAILURE,
        error=error,
        start_time=now,
        end_time=now,
        span_name="my-span",
        parent_ctx=BaseContext(name="parent"),
    )
    assert ctx.name == "Test"
    assert ctx.status == ExecutionStatus.FAILURE
    assert ctx.error == error
    assert ctx.start_time == now
    assert ctx.end_time == now
    assert ctx.span_name == "my-span"
    assert ctx.parent_ctx.name == "parent"


def test_assigning_span_and_context_manager():
    ctx = BaseContext()
    ctx.span = DummySpan()
    ctx.span_cm = DummyContextManager()
    assert isinstance(ctx.span, DummySpan)
    assert isinstance(ctx.span_cm, DummyContextManager)


# Tests for with BaseContext (__enter__ and __exit__)


def test_context_manager_success():
    with BaseContext(name="test") as ctx:
        assert ctx.start_time is not None
    assert ctx.end_time is not None
    assert ctx.status == ExecutionStatus.SUCCESS
    assert ctx.error is None


def test_context_manager_exception():
    try:
        with BaseContext(name="fail") as ctx:
            assert ctx.start_time is not None
            raise ValueError("Oops")
    except ValueError:
        pass
    assert ctx.end_time is not None
    assert ctx.status == ExecutionStatus.ERROR
    assert isinstance(ctx.error, ValueError)


def test_context_manager_does_not_override_custom_status():
    class PreSetContext(BaseContext):
        def start(self):
            self.started = True
            self.status = ExecutionStatus.SKIPPED  # Not RUNNING

    with PreSetContext(name="skip") as ctx:
        pass
    assert ctx.status == ExecutionStatus.SKIPPED  # Not overridden


# BaseContext.start() tests


def test_start_sets_status_and_time(base_ctx):
    base_ctx.start()

    assert base_ctx.status == ExecutionStatus.RUNNING
    assert base_ctx.start_time is not None
    assert base_ctx.start_time.tzinfo is not None
    assert isinstance(base_ctx.start_time, datetime.datetime)


@mock.patch("lib.core.context.base.BaseContext.get_logger")
def test_start_handles_missing_tracer(mock_get_logger):
    ctx = BaseContext()

    # Make get_tracer raise a RuntimeError
    with mock.patch.object(ctx, "get_tracer", side_effect=RuntimeError("no tracer")):
        with mock.patch.object(ctx, "_record_start_event") as mock_event:
            ctx.start()

    mock_get_logger.return_value.warning.assert_called_once()
    mock_event.assert_called_once()
    assert ctx.status == ExecutionStatus.RUNNING
    assert ctx.span is None
    assert ctx.span_cm is None


def test_start_with_tracer_and_span():
    class TestableContext(BaseContext):
        def get_tracer(self, service_name):
            return self.mock_tracer

        def merge_ctx_metadata(self):
            return {"foo": "bar", "baz": "qux"}

        def _record_start_event(self, timestamp_nanos):
            self.recorded_timestamp = timestamp_nanos

    mock_span = mock.MagicMock()
    mock_cm = mock.MagicMock()
    mock_cm.__enter__.return_value = mock_span

    mock_tracer = mock.MagicMock()
    mock_tracer.start_as_current_span.return_value = mock_cm

    ctx = TestableContext(name="ctx")
    ctx.mock_tracer = mock_tracer

    ctx.start()

    # Assert span and span_cm are assigned correctly
    assert ctx.span == mock_span
    assert ctx.span_cm == mock_cm

    # span got attributes set
    mock_span.set_attribute.assert_any_call("foo", "bar")
    mock_span.set_attribute.assert_any_call("baz", "qux")

    # Tracer used correct span name
    mock_tracer.start_as_current_span.assert_called_once()
    args, kwargs = mock_tracer.start_as_current_span.call_args
    assert "ctx" in args[0]


# BaseContext.end() tests


def test_end_sets_end_time_and_calls_record_event():
    ctx = BaseContext()
    ctx.status = ExecutionStatus.SUCCESS

    with mock.patch.object(ctx, "record_event") as mock_record_event:
        ctx.end()

    assert ctx.end_time is not None
    assert ctx.end_time.tzinfo is not None
    mock_record_event.assert_called_once()
    timestamp = mock_record_event.call_args.kwargs["timestamp"]
    assert isinstance(timestamp, int)
    assert timestamp > 0


def test_end_sets_span_status_success():
    mock_span = mock.Mock()
    mock_cm = mock.MagicMock()

    ctx = BaseContext()
    ctx.status = ExecutionStatus.SUCCESS
    ctx.span = mock_span
    ctx.span_cm = mock_cm

    ctx.end()

    mock_span.set_status.assert_called_once_with(StatusCode.OK)
    mock_cm.__exit__.assert_called_once_with(None, None, None)


def test_end_sets_span_status_error():
    mock_span = mock.Mock()
    mock_cm = mock.MagicMock()

    ctx = BaseContext()
    ctx.status = ExecutionStatus.ERROR
    ctx.error = ValueError("fail!")
    ctx.span = mock_span
    ctx.span_cm = mock_cm

    ctx.end()

    mock_span.set_status.assert_called_once()
    status_arg = mock_span.set_status.call_args[0][0]
    assert isinstance(status_arg, Status)
    assert status_arg.status_code == StatusCode.ERROR
    assert "fail" in status_arg.description
    mock_cm.__exit__.assert_called_once_with(None, None, None)


def test_end_sets_span_status_unset():
    mock_span = mock.Mock()
    mock_cm = mock.MagicMock()

    ctx = BaseContext()
    ctx.status = ExecutionStatus.PENDING  # not SUCCESS or ERROR
    ctx.span = mock_span
    ctx.span_cm = mock_cm

    ctx.end()

    mock_span.set_status.assert_called_once()
    status_arg = mock_span.set_status.call_args[0][0]
    assert isinstance(status_arg, Status)
    assert status_arg.status_code == StatusCode.UNSET
    mock_cm.__exit__.assert_called_once_with(None, None, None)


def test_end_without_span_does_not_crash():
    ctx = BaseContext()
    ctx.status = ExecutionStatus.SUCCESS
    ctx.span = None
    ctx.span_cm = None

    # Should not raise
    ctx.end()


def test_duration_returns_seconds():
    ctx = BaseContext()
    now = datetime.datetime.now(datetime.timezone.utc)
    later = now + datetime.timedelta(seconds=3.5)
    ctx.start_time = now
    ctx.end_time = later

    assert ctx.duration == 3.5


def test_duration_returns_none_if_start_missing():
    ctx = BaseContext()
    ctx.end_time = datetime.datetime.now(datetime.timezone.utc)

    assert ctx.duration is None


def test_duration_returns_none_if_end_missing():
    ctx = BaseContext()
    ctx.start_time = datetime.datetime.now(datetime.timezone.utc)

    assert ctx.duration is None


def test_add_child_ctx_sets_bidirectional_link():
    parent = BaseContext(name="parent")
    child = BaseContext(name="child")

    parent.add_child_ctx(child)

    assert child in parent.child_contexts
    assert child.parent_ctx is parent
    assert parent.name == "parent"
    assert child.name == "child"


def test_add_multiple_children():
    parent = BaseContext(name="parent")
    children = [BaseContext(name=f"child-{i}") for i in range(3)]

    for child in children:
        parent.add_child_ctx(child)

    assert len(parent.child_contexts) == 3
    for child in children:
        assert child.parent_ctx is parent


def test_get_components_delegates_to_parent(dummy_component):
    class RootContext(BaseContext):
        def get_components(self) -> dict:
            return {"db": dummy_component, "api": dummy_component}

    root = RootContext()
    child = BaseContext(parent_ctx=root)

    components = child.get_components()

    assert isinstance(components, dict)
    assert "db" in components
    assert "api" in components


def test_get_components_raises_if_no_parent():
    ctx = BaseContext()

    with pytest.raises(NotImplementedError, match="does not support get_components"):
        ctx.get_components()


def test_get_component_by_name_returns_component(dummy_component):
    class RootContext(BaseContext):
        components = {"logger": dummy_component, "db": dummy_component}

        def get_component_by_name(self, name):
            return self.components.get(name)

    root = RootContext()
    child = BaseContext(parent_ctx=root)

    comp = child.get_component_by_name("db")
    assert isinstance(comp, Component)


def test_get_component_by_name_returns_none_if_not_found(dummy_component):
    class RootContext(BaseContext):
        components = {"logger": dummy_component, "db": dummy_component}

        def get_component_by_name(self, name):
            return self.components.get(name)

    root = RootContext()
    child = BaseContext(parent_ctx=root)

    assert child.get_component_by_name("cache") is None


def test_get_component_by_name_raises_if_no_root():
    ctx = BaseContext()

    with pytest.raises(NotImplementedError):
        ctx.get_component_by_name("anything")


def test_get_suite_delegates_to_parent(fake_test_suite):
    class SuiteContext(BaseContext):
        def get_suite(self):
            return fake_test_suite

    root = SuiteContext()
    child = BaseContext(parent_ctx=root)

    suite = child.get_suite()

    assert isinstance(suite, fake_test_suite.__class__)
    assert suite.name == "FakeTestSuite"


def test_get_suite_raises_without_override():
    ctx = BaseContext()

    with pytest.raises(NotImplementedError, match="does not support get_suite"):
        ctx.get_suite()


def test_nested_contexts_delegate_properly(fake_test_suite):
    class RootContext(BaseContext):
        def get_suite(self):
            return fake_test_suite

    root = RootContext()
    mid = BaseContext(parent_ctx=root)
    leaf = BaseContext(parent_ctx=mid)

    suite = leaf.get_suite()

    assert suite.name == "FakeTestSuite"


def test_returns_if_span_none():
    ctx = BaseContext()
    ctx.span = None
    assert ctx.record_event("my_event") is None


def test_returns_if_span_not_recording():
    ctx = BaseContext()
    ctx.span = mock.MagicMock()
    ctx.span.is_recording.return_value = False
    ctx.record_event("my_event")
    ctx.span.add_event.assert_not_called()


def test_merges_metadata():
    ctx = BaseContext()
    ctx.span = mock.MagicMock()
    ctx.span.is_recording.return_value = True
    ctx.error = ValueError("fail")
    ctx.metadata = {"meta1": "v1"}
    ctx.status = None

    ctx.record_event("event")

    _, kwargs = ctx.span.add_event.call_args[0]
    assert kwargs.get("test.ctx.error") == "fail"
    assert kwargs.get("meta1") == "v1"


def test_passes_timestamp():
    ctx = BaseContext()
    ctx.span = mock.MagicMock()
    ctx.span.is_recording.return_value = True
    ctx.merge_ctx_metadata = lambda **kwargs: kwargs

    ts = 999
    ctx.record_event("event", timestamp_unix_nanos=ts)

    # timestamp is passed as a keyword argument
    assert ctx.span.add_event.call_args.kwargs["timestamp"] == ts


def test_merge_ctx_metadata_adds_error_and_metadata():
    ctx = BaseContext()
    ctx.error = ValueError("fail")
    ctx.metadata = {
        "key1": "value1",
        "key2": None,  # falsy, should be ignored
        "key3": "",  # falsy, should be ignored
        "key4": 0,  # falsy, should be ignored
        "key5": "value5",
    }

    kwargs = {"existing": "keep"}
    merged = ctx.merge_ctx_metadata(**kwargs)

    # error string added
    assert merged["test.ctx.error"] == "fail"
    # metadata with truthy values added
    assert merged["key1"] == "value1"
    assert merged["key5"] == "value5"
    # falsy metadata keys ignored
    assert "key2" not in merged
    assert "key3" not in merged
    assert "key4" not in merged
    # original keys preserved
    assert merged["existing"] == "keep"


def test_merge_ctx_metadata_does_not_overwrite_existing_error():
    ctx = BaseContext()
    ctx.error = ValueError("fail")
    ctx.metadata = {"test.ctx.error": "already set"}

    kwargs = {"test.ctx.error": "present"}
    merged = ctx.merge_ctx_metadata(**kwargs)

    # existing 'test.ctx.error' is NOT overwritten by ctx.error because of setdefault
    assert merged["test.ctx.error"] == "present"


def test_get_logger_returns_adapter_and_filters_metadata(monkeypatch):
    ctx = BaseContext()

    # Patch merge_ctx_metadata to return a mix of truthy and falsy values
    ctx.merge_ctx_metadata = mock.MagicMock(
        return_value={
            "key1": "value1",
            "key2": None,
            "key3": "",
            "key4": 0,
            "key5": False,
            "key6": "value6",
        }
    )

    # Patch logging.getLogger to return a dummy logger
    dummy_logger = mock.MagicMock(spec=logging.Logger)
    monkeypatch.setattr(logging, "getLogger", mock.MagicMock(return_value=dummy_logger))

    adapter = ctx.get_logger("test.logger")

    # It should return a LoggerAdapter wrapping the dummy_logger
    assert isinstance(adapter, logging.LoggerAdapter)
    assert adapter.logger == dummy_logger

    # extra should include only truthy metadata keys
    expected_extra = {"key1": "value1", "key6": "value6"}
    assert adapter.extra == expected_extra


def test_logger_adapter_injects_extra(monkeypatch, caplog):
    ctx = BaseContext()
    ctx.merge_ctx_metadata = mock.MagicMock(return_value={"foo": "bar", "empty": ""})

    # Use the real logger here
    logger = ctx.get_logger("test.logger")

    with caplog.at_level(logging.INFO):
        logger.info("test message")

    # The log record should have 'foo' in extra and not 'empty'
    record = caplog.records[0]
    assert getattr(record, "foo", None) == "bar"
    assert not hasattr(record, "empty")


def test_get_telemetry_client_returns_client():
    ctx = BaseContext()

    mock_client = mock.MagicMock(name="TelemetryClient")
    mock_runtime = mock.MagicMock()
    mock_runtime.get_client.return_value = mock_client

    mock_test_suite = mock.MagicMock()
    mock_test_suite.get_runtime.return_value = mock_runtime

    ctx.get_suite = mock.MagicMock(return_value=mock_test_suite)

    client = ctx.get_telemetry_client("some_runtime")

    assert client is mock_client
    ctx.get_suite.assert_called_once()
    mock_test_suite.get_runtime.assert_called_once_with("some_runtime")
    mock_runtime.get_client.assert_called_once()


def test_get_telemetry_client_returns_none_if_no_runtime():
    ctx = BaseContext()

    mock_test_suite = mock.MagicMock()
    mock_test_suite.get_runtime.return_value = None

    ctx.get_suite = mock.MagicMock(return_value=mock_test_suite)

    client = ctx.get_telemetry_client("nonexistent_runtime")

    assert client is None
    ctx.get_suite.assert_called_once()
    mock_test_suite.get_runtime.assert_called_once_with("nonexistent_runtime")


def test_to_dict_serializes_correctly():
    ctx = BaseContext()
    ctx.name = "my_context"
    ctx.status = mock.MagicMock(value="PASSED")
    ctx.error = ValueError("something went wrong")
    ctx.start_time = datetime.datetime.fromtimestamp(1000000)
    ctx.end_time = datetime.datetime.fromtimestamp(2000000)
    ctx.metadata = {"foo": "bar"}

    child1 = mock.MagicMock()
    child1.to_dict.return_value = {"name": "child1"}
    child2 = mock.MagicMock()
    child2.to_dict.return_value = {"name": "child2"}
    ctx.child_contexts = [child1, child2]

    result = ctx.to_dict()

    assert result == {
        "name": "my_context",
        "status": "PASSED",
        "error": "something went wrong",
        "start_time": "1970-01-12T05:46:40",
        "end_time": "1970-01-23T19:33:20",
        "duration": 1000000,
        "metadata": {"foo": "bar"},
        "child_contexts": [{"name": "child1"}, {"name": "child2"}],
    }
