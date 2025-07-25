import sys
import os

# Add root dir to sys.path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from loadgen import LoadGenerator  # noqa: E402


def test_create_log_record_structure():
    generator = LoadGenerator()
    body_size = 50
    num_attributes = 3
    attribute_value_size = 20

    record = generator.create_log_record(
        body_size=body_size,
        num_attributes=num_attributes,
        attribute_value_size=attribute_value_size,
    )

    # Check that record is a LogRecord
    assert record.body.string_value is not None
    assert len(record.body.string_value) == body_size

    # Check attribute count
    assert len(record.attributes) == num_attributes

    # Check each attribute key and value
    for i, attr in enumerate(record.attributes, start=1):
        assert attr.key == f"attribute.{i}"
        assert attr.value.string_value is not None
        assert len(attr.value.string_value) == attribute_value_size

    # Severity checks
    assert record.severity_text == "INFO"
    assert record.severity_number > 0
