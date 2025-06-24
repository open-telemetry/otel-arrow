import pytest
from lib.core.runtime import Runtime  # adjust import as needed


def test_set_and_get():
    rt = Runtime()
    rt.set("namespace1", 123)
    assert rt.get("namespace1") == 123


def test_get_returns_none_for_missing_namespace():
    rt = Runtime()
    assert rt.get("missing") is None


def test_get_or_create_returns_existing_data():
    rt = Runtime()
    rt.set("ns", "existing")
    result = rt.get_or_create("ns", lambda: "new")
    assert result == "existing"


def test_get_or_create_creates_data_when_missing():
    rt = Runtime()
    factory_called = False

    def factory():
        nonlocal factory_called
        factory_called = True
        return "created"

    result = rt.get_or_create("ns", factory)
    assert result == "created"
    assert factory_called is True
    # Also check the data was stored
    assert rt.get("ns") == "created"
