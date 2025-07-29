import sys
import os
import pytest
from pydantic import ValidationError

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from loadgen import LoadGenConfig  # noqa: E402


def test_valid_config():
    config = LoadGenConfig(
        body_size=30,
        num_attributes=3,
        attribute_value_size=20,
        batch_size=1000,
        threads=2,
        target_rate=5000,
    )
    assert config.body_size == 30
    assert config.target_rate == 5000


@pytest.mark.parametrize(
    "field, value",
    [
        ("body_size", 0),
        ("num_attributes", -1),
        ("attribute_value_size", 0),
        ("batch_size", 0),
        ("threads", 0),
    ],
)
def test_invalid_config_values(field, value):
    kwargs = {
        "body_size": 25,
        "num_attributes": 2,
        "attribute_value_size": 15,
        "batch_size": 5000,
        "threads": 4,
        "target_rate": 1000,
    }
    kwargs[field] = value
    with pytest.raises(ValidationError):
        LoadGenConfig(**kwargs)
